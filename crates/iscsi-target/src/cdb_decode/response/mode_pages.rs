//! Individual mode page field decoders.

use crate::cdb_decode::names::*;
use crate::cdb_decode::{data_field, DataField};

/// Decode page-specific fields. `page_data` includes the page header;
/// `base_offset` is the byte offset within the overall response.
pub fn decode_mode_page(page_code: u8, page_data: &[u8], base_offset: usize) -> Vec<DataField> {
    // Skip the 2-byte header (page code + page length)
    let body = if page_data.len() > 2 { &page_data[2..] } else { return vec![]; };
    let off = base_offset + 2; // body starts after header
    match page_code {
        0x01 => decode_rw_error_recovery(body, off),
        0x02 => decode_disconnect_reconnect(body, off),
        0x0A => decode_control(body, off),
        0x0F => decode_data_compression(body, off),
        0x10 => decode_device_configuration(body, off),
        0x11 => decode_medium_partition(body, off),
        0x1C => decode_informational_exceptions(body, off),
        0x1D => decode_element_address_assignment(body, off),
        0x1E => decode_transport_geometry(body, off),
        0x1F => decode_device_capabilities(body, off),
        _ => vec![],
    }
}

fn decode_rw_error_recovery(body: &[u8], off: usize) -> Vec<DataField> {
    let mut f = Vec::new();
    if body.is_empty() { return f; }
    let b0 = body[0];
    f.push(data_field("AWRE", off, Some("7"), format!("{}", (b0 >> 7) & 1), if b0 & 0x80 != 0 { "Auto write reallocation" } else { "Disabled" }));
    f.push(data_field("ARRE", off, Some("6"), format!("{}", (b0 >> 6) & 1), if b0 & 0x40 != 0 { "Auto read reallocation" } else { "Disabled" }));
    f.push(data_field("TB", off, Some("5"), format!("{}", (b0 >> 5) & 1), if b0 & 0x20 != 0 { "Transfer block on error" } else { "No transfer" }));
    f.push(data_field("RC", off, Some("4"), format!("{}", (b0 >> 4) & 1), if b0 & 0x10 != 0 { "Read continuous" } else { "Normal" }));
    f.push(data_field("EER", off, Some("3"), format!("{}", (b0 >> 3) & 1), if b0 & 0x08 != 0 { "Enable early recovery" } else { "Disabled" }));
    f.push(data_field("PER", off, Some("2"), format!("{}", (b0 >> 2) & 1), if b0 & 0x04 != 0 { "Post error" } else { "No post" }));
    f.push(data_field("DTE", off, Some("1"), format!("{}", (b0 >> 1) & 1), if b0 & 0x02 != 0 { "Disable transfer on error" } else { "Normal" }));
    f.push(data_field("DCR", off, Some("0"), format!("{}", b0 & 1), if b0 & 0x01 != 0 { "Disable correction" } else { "Correction enabled" }));

    if body.len() >= 2 {
        f.push(data_field("Read Retry Count", off + 1, None, format!("{:02X}", body[1]), format!("{}", body[1])));
    }
    if body.len() >= 9 {
        f.push(data_field("Write Retry Count", off + 8, None, format!("{:02X}", body[8]), format!("{}", body[8])));
    }
    f
}

fn decode_disconnect_reconnect(body: &[u8], off: usize) -> Vec<DataField> {
    let mut f = Vec::new();
    if body.len() >= 1 {
        f.push(data_field("Buffer Full Ratio", off, None, format!("{:02X}", body[0]), format!("{}", body[0])));
    }
    if body.len() >= 2 {
        f.push(data_field("Buffer Empty Ratio", off + 1, None, format!("{:02X}", body[1]), format!("{}", body[1])));
    }
    if body.len() >= 4 {
        let bil = u16::from_be_bytes([body[2], body[3]]);
        f.push(data_field("Bus Inactivity Limit", off + 2, None, format!("{:04X}", bil), format!("{}", bil)));
    }
    f
}

fn decode_control(body: &[u8], off: usize) -> Vec<DataField> {
    let mut f = Vec::new();
    if body.is_empty() { return f; }
    let b0 = body[0];
    let tst = (b0 >> 5) & 0x07;
    let tmf_only = (b0 >> 4) & 1;
    let dpicz = (b0 >> 3) & 1;
    let d_sense = (b0 >> 2) & 1;
    let gltsd = (b0 >> 1) & 1;
    let rlec = b0 & 1;
    f.push(data_field("TST", off, Some("7:5"), format!("{}", tst), match tst { 0 => "Single task set".to_string(), 1 => "Separate task set per I_T nexus".to_string(), _ => format!("TST {}", tst) }));
    f.push(data_field("TMF_ONLY", off, Some("4"), format!("{}", tmf_only), if tmf_only != 0 { "Only accept TMFs" } else { "Normal" }));
    f.push(data_field("DPICZ", off, Some("3"), format!("{}", dpicz), if dpicz != 0 { "Disable protection info check on ZBC" } else { "Normal" }));
    f.push(data_field("D_SENSE", off, Some("2"), format!("{}", d_sense), if d_sense != 0 { "Descriptor format sense" } else { "Fixed format sense" }));
    f.push(data_field("GLTSD", off, Some("1"), format!("{}", gltsd), if gltsd != 0 { "Global logging target save disabled" } else { "Normal" }));
    f.push(data_field("RLEC", off, Some("0"), format!("{}", rlec), if rlec != 0 { "Report log exception condition" } else { "No report" }));

    if body.len() >= 2 {
        let qerr = (body[1] >> 1) & 0x03;
        let queue_alg = (body[1] >> 4) & 0x0F;
        f.push(data_field("Queue Algorithm Modifier", off + 1, Some("7:4"), format!("{}", queue_alg), match queue_alg { 0 => "Restricted reordering", 1 => "Unrestricted reordering", _ => "Reserved" }));
        f.push(data_field("QErr", off + 1, Some("2:1"), format!("{}", qerr), match qerr { 0 => "Residual not reported", 1 => "Abort remaining", 3 => "Abort remaining, set UA", _ => "Reserved" }));
    }
    f
}

fn decode_data_compression(body: &[u8], off: usize) -> Vec<DataField> {
    let mut f = Vec::new();
    if body.is_empty() { return f; }
    let dce = (body[0] >> 7) & 1;
    let dcc = (body[0] >> 6) & 1;
    f.push(data_field("DCE", off, Some("7"), format!("{}", dce), if dce != 0 { "Compression enabled" } else { "Compression disabled" }));
    f.push(data_field("DCC", off, Some("6"), format!("{}", dcc), if dcc != 0 { "Compression supported" } else { "Not supported" }));

    if body.len() >= 2 {
        let dde = (body[1] >> 7) & 1;
        let red = (body[1] >> 5) & 0x03;
        f.push(data_field("DDE", off + 1, Some("7"), format!("{}", dde), if dde != 0 { "Decompression enabled" } else { "Decompression disabled" }));
        f.push(data_field("RED", off + 1, Some("6:5"), format!("{}", red), match red { 0 => "No reporting", 1 => "Report exceptions per target", 2 => "Report exceptions per I_T nexus", _ => "Reserved" }));
    }
    if body.len() >= 6 {
        let algo = u32::from_be_bytes([body[2], body[3], body[4], body[5]]);
        f.push(data_field("Compression Algorithm", off + 2, None, format!("{:08X}", algo), compression_algorithm_name(algo)));
    }
    if body.len() >= 10 {
        let dalgo = u32::from_be_bytes([body[6], body[7], body[8], body[9]]);
        f.push(data_field("Decompression Algorithm", off + 6, None, format!("{:08X}", dalgo), compression_algorithm_name(dalgo)));
    }
    f
}

fn decode_device_configuration(body: &[u8], off: usize) -> Vec<DataField> {
    let mut f = Vec::new();
    if body.len() < 2 { return f; }
    f.push(data_field("Active Format", off, None, format!("{:02X}", body[0] & 0x1F), format!("{}", body[0] & 0x1F)));
    f.push(data_field("Active Partition", off + 1, None, format!("{:02X}", body[1]), format!("partition {}", body[1])));

    if body.len() >= 3 {
        f.push(data_field("Write Buffer Full Ratio", off + 2, None, format!("{:02X}", body[2]), format!("{}", body[2])));
    }
    if body.len() >= 4 {
        f.push(data_field("Write Buffer Empty Ratio", off + 3, None, format!("{:02X}", body[3]), format!("{}", body[3])));
    }
    if body.len() >= 6 {
        let gap_size = body[5];
        f.push(data_field("Gap Size", off + 5, None, format!("{:02X}", gap_size), format!("{}", gap_size)));
    }
    if body.len() >= 7 {
        let eod = (body[6] >> 5) & 0x07;
        let bis = (body[6] >> 3) & 1;
        let rew = body[6] & 1;
        f.push(data_field("EOD Defined", off + 6, Some("7:5"), format!("{}", eod), match eod { 0 => "Default".to_string(), 1 => "Format defined EOD".to_string(), _ => format!("EOD type {}", eod) }));
        f.push(data_field("BIS", off + 6, Some("3"), format!("{}", bis), if bis != 0 { "Block IDs supported" } else { "Not supported" }));
        f.push(data_field("REW", off + 6, Some("0"), format!("{}", rew), if rew != 0 { "Rewind on reset" } else { "No rewind" }));
    }
    if body.len() >= 12 {
        let sel_dcc = (body[11] >> 6) & 1;
        let cap = (body[11] >> 5) & 1;
        let caf = (body[11] >> 4) & 1;
        let lois = body[11] & 1;
        f.push(data_field("SEL Data Comp", off + 11, Some("6"), format!("{}", sel_dcc), if sel_dcc != 0 { "Select data compression" } else { "Not selected" }));
        f.push(data_field("CAP", off + 11, Some("5"), format!("{}", cap), if cap != 0 { "Change active partition allowed" } else { "Not allowed" }));
        f.push(data_field("CAF", off + 11, Some("4"), format!("{}", caf), if caf != 0 { "Change active format allowed" } else { "Not allowed" }));
        f.push(data_field("LOIS", off + 11, Some("0"), format!("{}", lois), if lois != 0 { "Logical object identifiers supported" } else { "Not supported" }));
    }
    f
}

fn decode_medium_partition(body: &[u8], off: usize) -> Vec<DataField> {
    let mut f = Vec::new();
    if body.len() < 2 { return f; }
    f.push(data_field("Maximum Additional Partitions", off, None, format!("{:02X}", body[0]), format!("{}", body[0])));
    f.push(data_field("Additional Partitions Defined", off + 1, None, format!("{:02X}", body[1]), format!("{}", body[1])));

    if body.len() >= 3 {
        let fdp = (body[2] >> 7) & 1;
        let sdp = (body[2] >> 6) & 1;
        let idp = (body[2] >> 5) & 1;
        let psum = (body[2] >> 3) & 0x03;
        f.push(data_field("FDP", off + 2, Some("7"), format!("{}", fdp), if fdp != 0 { "Fixed data partitions" } else { "Not fixed" }));
        f.push(data_field("SDP", off + 2, Some("6"), format!("{}", sdp), if sdp != 0 { "Select data partitions" } else { "Not selected" }));
        f.push(data_field("IDP", off + 2, Some("5"), format!("{}", idp), if idp != 0 { "Initiator-defined partitions" } else { "Not defined" }));
        f.push(data_field("PSUM", off + 2, Some("4:3"), format!("{}", psum), match psum { 0 => "Bytes", 1 => "Kilobytes", 2 => "Megabytes", 3 => "10^tenths", _ => "Unknown" }));
    }
    if body.len() >= 4 {
        f.push(data_field("Medium Format Recognition", off + 3, None, format!("{:02X}", body[3]), match body[3] { 0 => "Incapable", 1 => "Format recognition only", 2 => "Partition recognition only", 3 => "Format and partition", _ => "Unknown" }));
    }
    // Partition size descriptors (2 bytes each starting at body[6])
    if body.len() > 6 {
        let mut poff = 6;
        let mut pnum = 0;
        while poff + 2 <= body.len() {
            let size = u16::from_be_bytes([body[poff], body[poff + 1]]);
            f.push(data_field(format!("Partition {} Size", pnum), off + poff, None, format!("{:04X}", size), format!("{}", size)));
            poff += 2;
            pnum += 1;
        }
    }
    f
}

fn decode_informational_exceptions(body: &[u8], off: usize) -> Vec<DataField> {
    let mut f = Vec::new();
    if body.is_empty() { return f; }
    let perf = (body[0] >> 7) & 1;
    let ebf = (body[0] >> 5) & 1;
    let ewasc = (body[0] >> 4) & 1;
    let dexcpt = (body[0] >> 3) & 1;
    let test = (body[0] >> 2) & 1;
    let logerr = (body[0] >> 0) & 1;
    f.push(data_field("PERF", off, Some("7"), format!("{}", perf), if perf != 0 { "No performance impact" } else { "Performance acceptable" }));
    f.push(data_field("EBF", off, Some("5"), format!("{}", ebf), if ebf != 0 { "Enable background functions" } else { "Disabled" }));
    f.push(data_field("EWASC", off, Some("4"), format!("{}", ewasc), if ewasc != 0 { "Warning reporting enabled" } else { "Disabled" }));
    f.push(data_field("DEXCPT", off, Some("3"), format!("{}", dexcpt), if dexcpt != 0 { "Exception control disabled" } else { "Enabled" }));
    f.push(data_field("TEST", off, Some("2"), format!("{}", test), if test != 0 { "Test mode" } else { "Normal" }));
    f.push(data_field("LOGERR", off, Some("0"), format!("{}", logerr), if logerr != 0 { "Log errors" } else { "No logging" }));

    if body.len() >= 2 {
        let mrie = body[1] & 0x0F;
        f.push(data_field("MRIE", off + 1, Some("3:0"), format!("{}", mrie), match mrie { 0 => "No reporting", 1 => "Asynchronous event", 2 => "Unit attention", 3 => "Conditionally generate RE", 4 => "Unconditionally generate RE", 5 => "Generate no sense", 6 => "Only report on request", _ => "Reserved" }));
    }
    if body.len() >= 6 {
        let interval = u32::from_be_bytes([body[2], body[3], body[4], body[5]]);
        f.push(data_field("Interval Timer", off + 2, None, format!("{:08X}", interval), format!("{} (100ms units)", interval)));
    }
    if body.len() >= 10 {
        let report_count = u32::from_be_bytes([body[6], body[7], body[8], body[9]]);
        f.push(data_field("Report Count", off + 6, None, format!("{:08X}", report_count), format!("{}", report_count)));
    }
    f
}

fn decode_element_address_assignment(body: &[u8], off: usize) -> Vec<DataField> {
    let mut f = Vec::new();
    if body.len() < 18 { return f; }
    let mt_first = u16::from_be_bytes([body[0], body[1]]);
    let mt_count = u16::from_be_bytes([body[2], body[3]]);
    let st_first = u16::from_be_bytes([body[4], body[5]]);
    let st_count = u16::from_be_bytes([body[6], body[7]]);
    let ie_first = u16::from_be_bytes([body[8], body[9]]);
    let ie_count = u16::from_be_bytes([body[10], body[11]]);
    let dt_first = u16::from_be_bytes([body[12], body[13]]);
    let dt_count = u16::from_be_bytes([body[14], body[15]]);
    f.push(data_field("Medium Transport First Address", off, None, format!("{:04X}", mt_first), format!("{}", mt_first)));
    f.push(data_field("Medium Transport Count", off + 2, None, format!("{:04X}", mt_count), format!("{}", mt_count)));
    f.push(data_field("Storage First Address", off + 4, None, format!("{:04X}", st_first), format!("{}", st_first)));
    f.push(data_field("Storage Count", off + 6, None, format!("{:04X}", st_count), format!("{}", st_count)));
    f.push(data_field("Import/Export First Address", off + 8, None, format!("{:04X}", ie_first), format!("{}", ie_first)));
    f.push(data_field("Import/Export Count", off + 10, None, format!("{:04X}", ie_count), format!("{}", ie_count)));
    f.push(data_field("Data Transfer First Address", off + 12, None, format!("{:04X}", dt_first), format!("{}", dt_first)));
    f.push(data_field("Data Transfer Count", off + 14, None, format!("{:04X}", dt_count), format!("{}", dt_count)));
    f
}

fn decode_transport_geometry(body: &[u8], off: usize) -> Vec<DataField> {
    let mut f = Vec::new();
    if body.is_empty() { return f; }
    let rotational = (body[0] >> 0) & 1;
    f.push(data_field("Rotate", off, Some("0"), format!("{}", rotational), if rotational != 0 { "Rotational" } else { "Not rotational" }));
    if body.len() >= 2 {
        f.push(data_field("Member Number", off + 1, None, format!("{:02X}", body[1]), format!("{}", body[1])));
    }
    f
}

fn decode_device_capabilities(body: &[u8], off: usize) -> Vec<DataField> {
    let mut f = Vec::new();
    if body.len() < 12 { return f; }
    // Byte 0: STORDT, STOREI/E, STORST, STORMT
    f.push(data_field("STOR DT->DT", off, Some("3"), format!("{}", (body[0] >> 3) & 1), if body[0] & 0x08 != 0 { "Can store to DT" } else { "Cannot" }));
    f.push(data_field("STOR I/E->DT", off, Some("2"), format!("{}", (body[0] >> 2) & 1), if body[0] & 0x04 != 0 { "Can store I/E to DT" } else { "Cannot" }));
    f.push(data_field("STOR ST->DT", off, Some("1"), format!("{}", (body[0] >> 1) & 1), if body[0] & 0x02 != 0 { "Can store ST to DT" } else { "Cannot" }));
    f.push(data_field("STOR MT->DT", off, Some("0"), format!("{}", body[0] & 1), if body[0] & 0x01 != 0 { "Can store MT to DT" } else { "Cannot" }));
    f
}
