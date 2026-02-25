//! Log Sense response decoder: header + per-parameter dispatch.

use crate::cdb_decode::names::log_page_name;
use crate::cdb_decode::{data_field, data_field_parent, hex_string, DataField};
use super::log_pages;

pub fn decode(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 4 { return fields; }

    let page_code = data[0] & 0x3F;
    let spf = (data[0] >> 6) & 1;
    let ds = (data[0] >> 7) & 1;
    let subpage = if spf != 0 && data.len() > 1 { data[1] } else { 0 };
    let page_length = u16::from_be_bytes([data[2], data[3]]) as usize;

    fields.push(data_field("DS", 0, Some("7"), format!("{}", ds), if ds != 0 { "Disable saving" } else { "Saving allowed" }));
    fields.push(data_field("SPF", 0, Some("6"), format!("{}", spf), if spf != 0 { "Subpage format" } else { "Page_0 format" }));
    fields.push(data_field("Page Code", 0, Some("5:0"), format!("{:02X}", page_code), log_page_name(page_code)));
    if spf != 0 {
        fields.push(data_field("Subpage Code", 1, None, format!("{:02X}", subpage), format!("0x{:02X}", subpage)));
    }
    fields.push(data_field("Page Length", 2, None, format!("{:04X}", page_length), format!("{} bytes", page_length)));

    // Parse individual log parameters
    let header_len = 4;
    let page_end = (header_len + page_length).min(data.len());
    let mut offset = header_len;
    let mut param_num = 1;

    while offset + 4 <= page_end {
        let param_code = u16::from_be_bytes([data[offset], data[offset + 1]]);
        let control = data[offset + 2];
        let param_len = data[offset + 3] as usize;
        let param_end = (offset + 4 + param_len).min(page_end);
        let param_data = &data[offset + 4..param_end];

        let du = (control >> 7) & 1;
        let tsd = (control >> 5) & 1;
        let etc = (control >> 4) & 1;
        let tmc = (control >> 2) & 0x03;
        let lbin = (control >> 1) & 1;
        let lp = control & 1;

        let param_name = log_pages::parameter_name(page_code, param_code);
        let param_decoded = log_pages::decode_parameter_value(page_code, param_code, param_data);

        let mut children = vec![
            data_field("Parameter Code", offset, None, format!("{:04X}", param_code), &param_name),
            data_field("DU", offset + 2, Some("7"), format!("{}", du), if du != 0 { "Disable update" } else { "Update enabled" }),
            data_field("TSD", offset + 2, Some("5"), format!("{}", tsd), if tsd != 0 { "Target save disable" } else { "Target may save" }),
            data_field("ETC", offset + 2, Some("4"), format!("{}", etc), if etc != 0 { "Enable threshold comparison" } else { "Disabled" }),
            data_field("TMC", offset + 2, Some("3:2"), format!("{}", tmc), match tmc { 0 => "Every update", 1 => "Equal", 2 => "Not equal", 3 => "Greater than", _ => "?" }),
            data_field("LBIN", offset + 2, Some("1"), format!("{}", lbin), if lbin != 0 { "List format: binary" } else { "List format: ASCII" }),
            data_field("LP", offset + 2, Some("0"), format!("{}", lp), if lp != 0 { "List parameter" } else { "Data counter" }),
            data_field("Parameter Length", offset + 3, None, format!("{:02X}", param_len), format!("{} bytes", param_len)),
        ];
        if !param_data.is_empty() {
            children.push(data_field("Value", offset + 4, None, hex_string(param_data), &param_decoded));
        }

        fields.push(data_field_parent(
            format!("Parameter {}: {}", param_num, param_name),
            offset, None,
            hex_string(&data[offset..param_end]),
            param_decoded,
            children,
        ));

        offset = param_end;
        param_num += 1;
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_log_page() {
        // Sequential-access device page (0x0C) with one parameter
        let data = [
            0x0C, 0x00, // page code, subpage
            0x00, 0x08, // page length = 8
            // Parameter: code 0x0000, control, length 4, value
            0x00, 0x00, 0x00, 0x04,
            0x00, 0x00, 0x10, 0x00, // 4096
        ];
        let fields = decode(&data);
        assert!(fields.iter().any(|f| f.name == "Page Code" && f.decoded.contains("Sequential")));
        assert!(fields.iter().any(|f| f.name.contains("Parameter 1")));
    }
}
