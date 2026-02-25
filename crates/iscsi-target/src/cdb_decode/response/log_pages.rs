//! Individual log page parameter name lookups and value decoders.

use crate::cdb_decode::names::tape_alert_flag_name;

/// Get a human-readable name for a log parameter code on a given page.
pub fn parameter_name(page_code: u8, param_code: u16) -> String {
    match page_code {
        // Write Error Counters (02h)
        0x02 => write_error_param_name(param_code),
        // Read Error Counters (03h)
        0x03 => read_error_param_name(param_code),
        // Non-Medium Error (06h)
        0x06 => match param_code {
            0x0000 => "Non-Medium Error Count".into(),
            _ => format!("Parameter 0x{:04X}", param_code),
        },
        // Sequential-Access Device (0Ch)
        0x0C => seq_access_param_name(param_code),
        // Temperature (0Dh)
        0x0D => match param_code {
            0x0000 => "Temperature".into(),
            0x0001 => "Reference Temperature".into(),
            _ => format!("Parameter 0x{:04X}", param_code),
        },
        // Device Statistics (14h)
        0x14 => device_stats_param_name(param_code),
        // Volume Statistics (17h)
        0x17 => volume_stats_param_name(param_code),
        // Data Compression (1Bh)
        0x1B => data_compression_param_name(param_code),
        // TapeAlert (2Eh)
        0x2E => tape_alert_flag_name(param_code).to_string(),
        // Tape Capacity (31h)
        0x31 => tape_capacity_param_name(param_code),
        _ => format!("Parameter 0x{:04X}", param_code),
    }
}

/// Decode a parameter value into a human-readable string.
pub fn decode_parameter_value(page_code: u8, param_code: u16, data: &[u8]) -> String {
    if data.is_empty() {
        return "empty".into();
    }
    match page_code {
        // TapeAlert — 1-byte flag value
        0x2E => {
            if data.len() >= 1 {
                if data[0] != 0 { "SET".into() } else { "clear".into() }
            } else {
                "?".into()
            }
        }
        // Temperature — 2 bytes, value in degrees C
        0x0D => {
            if data.len() >= 2 {
                let temp = u16::from_be_bytes([data[0], data[1]]);
                if temp == 0xFF { "Not available".into() } else { format!("{} C", temp) }
            } else {
                format_counter(data)
            }
        }
        // Sequential-access (0Ch) — various counters
        0x0C => {
            match param_code {
                0x0000..=0x0005 | 0x0100 => {
                    // These are 8-byte counters (bytes)
                    let val = counter_u64(data);
                    if param_code <= 0x0003 {
                        format_bytes(val)
                    } else {
                        format!("{}", val)
                    }
                }
                0x0008 => {
                    // Cleaning required
                    if !data.is_empty() && data[0] != 0 { "Cleaning required".into() } else { "Clean".into() }
                }
                _ => format_counter(data),
            }
        }
        // Tape capacity (31h) — 4-byte values in MB or native units
        0x31 => {
            let val = counter_u64(data);
            match param_code {
                0x0001 | 0x0003 => format!("{} MB", val),
                0x0002 | 0x0004 => format!("{} MB", val),
                _ => format!("{}", val),
            }
        }
        // Error counters — generic counter
        0x02 | 0x03 => format_counter(data),
        // Data compression (1Bh) — counters
        0x1B => format_counter(data),
        // Default: show as counter
        _ => format_counter(data),
    }
}

fn write_error_param_name(code: u16) -> String {
    match code {
        0x0000 => "Errors Corrected Without Delay".into(),
        0x0001 => "Errors Corrected With Delay".into(),
        0x0002 => "Total Rewrites/Rereads".into(),
        0x0003 => "Total Errors Corrected".into(),
        0x0004 => "Total Times Correction Algorithm Processed".into(),
        0x0005 => "Total Bytes Processed".into(),
        0x0006 => "Total Uncorrected Errors".into(),
        _ => format!("Write Error Param 0x{:04X}", code),
    }
}

fn read_error_param_name(code: u16) -> String {
    match code {
        0x0000 => "Errors Corrected Without Delay".into(),
        0x0001 => "Errors Corrected With Delay".into(),
        0x0002 => "Total Rewrites/Rereads".into(),
        0x0003 => "Total Errors Corrected".into(),
        0x0004 => "Total Times Correction Algorithm Processed".into(),
        0x0005 => "Total Bytes Processed".into(),
        0x0006 => "Total Uncorrected Errors".into(),
        _ => format!("Read Error Param 0x{:04X}", code),
    }
}

fn seq_access_param_name(code: u16) -> String {
    match code {
        0x0000 => "Data Bytes Received with WRITE".into(),
        0x0001 => "Data Bytes Written to Medium".into(),
        0x0002 => "Data Bytes Read from Medium".into(),
        0x0003 => "Data Bytes Transferred by READ".into(),
        0x0004 => "Native Capacity from BOP".into(),
        0x0005 => "Native Capacity from EW to EOP".into(),
        0x0008 => "Cleaning Required".into(),
        0x0100 => "Medium Mount / Dismount Count".into(),
        _ => format!("Seq Access Param 0x{:04X}", code),
    }
}

fn device_stats_param_name(code: u16) -> String {
    match code {
        0x0000 => "Lifetime Media Loads".into(),
        0x0001 => "Lifetime Cleaning Operations".into(),
        0x0002 => "Lifetime Power On Hours".into(),
        0x0003 => "Lifetime Media Motion Hours".into(),
        0x0004 => "Lifetime Meters of Tape Processed".into(),
        0x0005 => "Lifetime Media Written (GB)".into(),
        0x0006 => "Lifetime Media Read (GB)".into(),
        _ => format!("Device Stats Param 0x{:04X}", code),
    }
}

fn volume_stats_param_name(code: u16) -> String {
    match code {
        0x0000 => "Volume Mounts".into(),
        0x0001 => "Volume Data Sets Written".into(),
        0x0002 => "Volume Write Retries".into(),
        0x0003 => "Volume Data Sets Read".into(),
        0x0004 => "Volume Read Retries".into(),
        0x0005 => "Volume Recovered Write Errors".into(),
        0x0006 => "Volume Unrecovered Write Errors".into(),
        0x0007 => "Volume Recovered Read Errors".into(),
        0x0008 => "Volume Unrecovered Read Errors".into(),
        0x000C => "Last Mount Unrecovered Write Errors".into(),
        0x000D => "Last Mount Unrecovered Read Errors".into(),
        0x000E => "Last Mount Bytes Written (MB)".into(),
        0x000F => "Last Mount Bytes Read (MB)".into(),
        0x0010 => "Lifetime Bytes Written (MB)".into(),
        0x0011 => "Lifetime Bytes Read (MB)".into(),
        0x0012 => "Last Load Write Compression Ratio".into(),
        0x0013 => "Last Load Read Compression Ratio".into(),
        0x0014 => "Medium Mount Time".into(),
        0x0015 => "Medium Ready Time".into(),
        0x0016 => "Total Native Capacity".into(),
        0x0017 => "Total Used Native Capacity".into(),
        _ => format!("Volume Stats Param 0x{:04X}", code),
    }
}

fn data_compression_param_name(code: u16) -> String {
    match code {
        0x0000 => "Read Compression Ratio".into(),
        0x0001 => "Write Compression Ratio".into(),
        0x0002 => "Megabytes Transferred to Server".into(),
        0x0003 => "Bytes Transferred to Server".into(),
        0x0004 => "Megabytes Read from Tape".into(),
        0x0005 => "Bytes Read from Tape".into(),
        0x0006 => "Megabytes Transferred from Server".into(),
        0x0007 => "Bytes Transferred from Server".into(),
        0x0008 => "Megabytes Written to Tape".into(),
        0x0009 => "Bytes Written to Tape".into(),
        _ => format!("Data Compression Param 0x{:04X}", code),
    }
}

fn tape_capacity_param_name(code: u16) -> String {
    match code {
        0x0001 => "Main Partition Remaining Capacity (MB)".into(),
        0x0002 => "Alternate Partition Remaining Capacity (MB)".into(),
        0x0003 => "Main Partition Maximum Capacity (MB)".into(),
        0x0004 => "Alternate Partition Maximum Capacity (MB)".into(),
        _ => format!("Tape Capacity Param 0x{:04X}", code),
    }
}

fn counter_u64(data: &[u8]) -> u64 {
    let mut val: u64 = 0;
    for &b in data.iter().take(8) {
        val = (val << 8) | (b as u64);
    }
    val
}

fn format_counter(data: &[u8]) -> String {
    let val = counter_u64(data);
    format!("{}", val)
}

fn format_bytes(val: u64) -> String {
    if val >= 1_073_741_824 {
        format!("{:.2} GB", val as f64 / 1_073_741_824.0)
    } else if val >= 1_048_576 {
        format!("{:.2} MB", val as f64 / 1_048_576.0)
    } else if val >= 1024 {
        format!("{:.1} KB", val as f64 / 1024.0)
    } else {
        format!("{} bytes", val)
    }
}
