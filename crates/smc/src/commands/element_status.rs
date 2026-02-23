//! READ ELEMENT STATUS (B8h) command handler.

use crate::sense::{self, SenseBuilder};
use crate::state::{ChangerState, ELEM_DTE, ELEM_IEE, ELEM_MTE, ELEM_STE};
use iscsi_target::ScsiResult;

/// Handle READ ELEMENT STATUS (B8h).
pub fn handle_read_element_status(cdb: &[u8], st: &ChangerState) -> ScsiResult {
    let voltag = cdb[1] & 0x10 != 0;
    let _dvcid = cdb[1] & 0x01 != 0;
    let type_filter = cdb[1] & 0x0F;
    let start_addr = ((cdb[2] as u16) << 8) | cdb[3] as u16;
    let num_elements = ((cdb[4] as u16) << 8) | cdb[5] as u16;
    let _curdata = cdb[6] & 0x01 != 0;
    let alloc_len =
        ((cdb[7] as usize) << 16) | ((cdb[8] as usize) << 8) | (cdb[9] as usize);

    if alloc_len == 0 {
        return SenseBuilder::invalid_field_in_cdb().to_check_condition();
    }

    // Determine which element types to report
    let types_to_report: Vec<u8> = if type_filter == 0 {
        vec![ELEM_MTE, ELEM_STE, ELEM_IEE, ELEM_DTE]
    } else {
        vec![type_filter]
    };

    // Descriptor length depends on voltag
    // Base: 12 bytes; with voltag: +40 = 52 bytes
    let desc_len: u16 = if voltag { 52 } else { 16 };

    // Collect ALL elements with addr >= start_addr matching the type filter,
    // then limit to num_elements total.  The SCSI spec says num_elements is
    // a maximum count, NOT an address range.
    let mut all_matching: Vec<(u16, u8)> = Vec::new();
    for (&addr, elem) in &st.elements {
        if addr < start_addr {
            continue;
        }
        let dominated = type_filter != 0 && elem.element_type != type_filter;
        if dominated {
            continue;
        }
        all_matching.push((addr, elem.element_type));
    }
    // BTreeMap iterates in address order, so all_matching is already sorted.
    all_matching.truncate(num_elements as usize);

    // Build per-type pages
    let mut report_data = Vec::new();
    let mut total_elements_reported: u16 = 0;

    for &etype in &types_to_report {
        // Collect elements of this type from the count-limited set
        let descriptors: Vec<_> = all_matching
            .iter()
            .filter(|&&(_, et)| et == etype)
            .filter_map(|&(addr, _)| {
                st.elements.get(&addr).map(|elem| (addr, elem))
            })
            .collect();

        if descriptors.is_empty() {
            continue;
        }

        // Page header (8 bytes)
        let num_desc = descriptors.len() as u16;
        let desc_bytes = num_desc as u32 * desc_len as u32;

        let mut page_header = vec![0u8; 8];
        page_header[0] = etype;
        page_header[1] = if voltag { 0x80 } else { 0x00 }; // PVolTag
        page_header[2] = (desc_len >> 8) as u8;
        page_header[3] = desc_len as u8;
        // Byte 4: reserved
        page_header[5] = ((desc_bytes >> 16) & 0xFF) as u8;
        page_header[6] = ((desc_bytes >> 8) & 0xFF) as u8;
        page_header[7] = (desc_bytes & 0xFF) as u8;

        report_data.extend_from_slice(&page_header);

        for (addr, elem) in &descriptors {
            let mut desc = vec![0u8; desc_len as usize];

            // Bytes 0-1: Element address
            desc[0] = (addr >> 8) as u8;
            desc[1] = *addr as u8;

            // Byte 2: Status flags
            let mut flags: u8 = 0x00;
            if elem.full {
                flags |= 0x01; // FULL
            }
            if elem.except {
                flags |= 0x08; // EXCEPT
            }
            if elem.disabled {
                // ED is in byte 9 bit 4, not here
            }

            // Type-specific flags in byte 2
            match etype {
                ELEM_STE => {
                    if elem.access {
                        flags |= 0x10; // ACCESS
                    }
                }
                ELEM_IEE => {
                    flags |= 0x20; // InEnab (always enabled)
                    if elem.access {
                        flags |= 0x10; // ACCESS
                    }
                    if elem.operator_intervention {
                        flags |= 0x80; // OIR
                    }
                    // CMC=0 (bit 6)
                    if elem.import_export {
                        flags |= 0x04; // ImpExp (placed by operator)
                    }
                }
                ELEM_DTE => {
                    if elem.access {
                        flags |= 0x10; // ACCESS
                    }
                }
                ELEM_MTE => {
                    // No special flags
                }
                _ => {}
            }
            desc[2] = flags;

            // Bytes 4-5: ASC/ASCQ (if Except=1)
            if let Some((asc, ascq)) = elem.asc_ascq {
                desc[4] = asc;
                desc[5] = ascq;
            }

            // Byte 9: SValid | Invert | ED | Medium Type
            let mut byte9: u8 = 0x00;
            if elem.full && elem.source_element != 0 {
                byte9 |= 0x80; // SValid
            }
            if elem.disabled {
                byte9 |= 0x10; // ED (Element Disabled)
            }
            byte9 |= elem.medium_type.to_bits() & 0x07;
            desc[9] = byte9;

            // Bytes 10-11: Source element address
            desc[10] = (elem.source_element >> 8) as u8;
            desc[11] = elem.source_element as u8;

            // Volume tag (if voltag)
            if voltag {
                // Bytes 12-43: Primary Volume Tag (32 bytes, space-padded)
                if let Some(ref bc) = elem.barcode {
                    let bc_bytes = bc.as_bytes();
                    let copy_len = bc_bytes.len().min(32);
                    desc[12..12 + copy_len].copy_from_slice(&bc_bytes[..copy_len]);
                    for b in &mut desc[12 + copy_len..44] {
                        *b = b' ';
                    }
                } else {
                    for b in &mut desc[12..44] {
                        *b = b' ';
                    }
                }
                // Bytes 44-47: Reserved (already 0)
                // Bytes 48-51: Reserved (already 0)
            }

            report_data.extend_from_slice(&desc);
            total_elements_reported += 1;
        }
    }

    // Build 8-byte main header
    let byte_count = report_data.len() as u32;
    let mut response = Vec::with_capacity(8 + report_data.len());

    // Bytes 0-1: First element address reported
    response.push((start_addr >> 8) as u8);
    response.push(start_addr as u8);
    // Bytes 2-3: Number of elements available
    response.push((total_elements_reported >> 8) as u8);
    response.push(total_elements_reported as u8);
    // Byte 4: Reserved
    response.push(0x00);
    // Bytes 5-7: Byte count of report available (3 bytes)
    response.push(((byte_count >> 16) & 0xFF) as u8);
    response.push(((byte_count >> 8) & 0xFF) as u8);
    response.push((byte_count & 0xFF) as u8);

    response.extend_from_slice(&report_data);

    response.truncate(alloc_len);
    sense::good_with_data(response)
}
