//! READ(6) command handler.

use crate::buffer::DriveBuffer;
use crate::media::dedup::DEDUP_BLOCK_SIZE;
use crate::media::tape::{DriveMediaState, RecordDescriptor};
use crate::sense::{self, SenseBuilder};
use iscsi_target::SimulationClock;
use crate::ScsiResult;
use tracing::{trace, warn};

/// Handle READ(6) — CDB[1] bit 0=FIXED, CDB[2-4]=transfer length.
pub fn handle_read_6(
    cdb: &[u8],
    media_state: &mut DriveMediaState,
    buffer: &mut Option<DriveBuffer>,
    clock: &SimulationClock,
) -> ScsiResult {
    let _sili = cdb[1] & 0x02 != 0;
    let fixed = cdb[1] & 0x01 != 0;
    let transfer_length = ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);

    if transfer_length == 0 {
        return sense::good();
    }

    let pos = media_state.position.block_number as usize;
    let record_count = media_state.current_partition().records.len();

    trace!(
        partition = media_state.position.partition,
        position = pos,
        record_count,
        "READ(6)"
    );

    // Check for EOD (blank check)
    if pos >= record_count {
        return SenseBuilder::end_of_data().to_check_condition();
    }

    // Check for filemark at current position
    if media_state.current_partition().records[pos].is_filemark() {
        media_state.position.block_number += 1;
        media_state.position.file_number += 1;
        return SenseBuilder::filemark_detected().to_check_condition();
    }

    // Start read-ahead on first read
    if let Some(ref mut buf) = buffer {
        buf.begin_read_ahead();
    }

    if fixed {
        handle_read_fixed(transfer_length, media_state, buffer, clock)
    } else {
        handle_read_variable(transfer_length, media_state, buffer, clock)
    }
}

/// Read record data from the store via its descriptor, decompressing if needed.
fn read_record_data(
    ms: &mut DriveMediaState,
    desc: &RecordDescriptor,
) -> Result<Vec<u8>, ScsiResult> {
    match desc {
        RecordDescriptor::Data { offset, length } => {
            ms.io_handle.read_sync(*offset, *length).map_err(|e| {
                warn!(error = %e, "failed to read data from tape store");
                SenseBuilder::medium_error().to_check_condition()
            })
        }
        RecordDescriptor::CompressedData {
            offset,
            compressed_length,
            ..
        } => {
            let compressed = ms.io_handle.read_sync(*offset, *compressed_length).map_err(|e| {
                warn!(error = %e, "failed to read compressed data from tape store");
                SenseBuilder::medium_error().to_check_condition()
            })?;
            zstd::decode_all(compressed.as_slice()).map_err(|e| {
                warn!(error = %e, "failed to decompress tape record");
                SenseBuilder::medium_error().to_check_condition()
            })
        }
        RecordDescriptor::DedupData {
            offsets_offset,
            num_chunks,
            remainder,
            native_length: _,
        } => {
            let ds = ms.dedup_store.as_ref().ok_or_else(|| {
                warn!("DedupData record but no dedup store available");
                SenseBuilder::medium_error().to_check_condition()
            })?;

            // Read packed u64 offsets from per-tape .data file
            let packed = ms
                .io_handle
                .read_sync(*offsets_offset, *num_chunks * 8)
                .map_err(|e| {
                    warn!(error = %e, "failed to read dedup offsets from tape store");
                    SenseBuilder::medium_error().to_check_condition()
                })?;

            // Reassemble blocks from dedup store
            let total_size = if *remainder > 0 {
                (*num_chunks as usize - 1) * DEDUP_BLOCK_SIZE + *remainder as usize
            } else {
                *num_chunks as usize * DEDUP_BLOCK_SIZE
            };
            let mut data = Vec::with_capacity(total_size);

            for i in 0..*num_chunks as usize {
                let offset_bytes: [u8; 8] = packed[i * 8..(i + 1) * 8]
                    .try_into()
                    .expect("packed offset slice must be 8 bytes");
                let block_offset = u64::from_le_bytes(offset_bytes);
                let block = ds.read_block(block_offset).map_err(|e| {
                    warn!(error = %e, block_offset, "failed to read dedup block");
                    SenseBuilder::medium_error().to_check_condition()
                })?;

                if i == *num_chunks as usize - 1 && *remainder > 0 {
                    // Last chunk: only take the actual bytes
                    data.extend_from_slice(&block[..*remainder as usize]);
                } else {
                    data.extend_from_slice(&block);
                }
            }

            Ok(data)
        }
        RecordDescriptor::Filemark => unreachable!(),
    }
}

fn handle_read_fixed(
    block_count: u32,
    ms: &mut DriveMediaState,
    buffer: &mut Option<DriveBuffer>,
    clock: &SimulationClock,
) -> ScsiResult {
    // Get the native block size from the first block we'll read
    let first_pos = ms.position.block_number as usize;
    let block_size = ms.current_partition().records[first_pos].native_byte_len() as usize;

    let mut data = Vec::with_capacity(block_size * block_count as usize);
    let mut blocks_read: u32 = 0;

    for _ in 0..block_count {
        let pos = ms.position.block_number as usize;
        let record_count = ms.current_partition().records.len();

        if pos >= record_count {
            break;
        }

        let is_filemark = ms.current_partition().records[pos].is_filemark();
        if is_filemark {
            break;
        }

        // Clone the descriptor to avoid borrow conflict
        let desc = ms.current_partition().records[pos].clone();
        let on_disk_bytes = desc.byte_len() as u64;
        let block_data = match read_record_data(ms, &desc) {
            Ok(d) => d,
            Err(scsi_err) => return scsi_err,
        };

        let native_bytes = block_data.len() as u64;
        data.extend_from_slice(&block_data);
        let part = ms.current_partition_mut();
        part.bytes_read_native += native_bytes;
        part.bytes_read_compressed += on_disk_bytes;
        ms.position.block_number += 1;
        blocks_read += 1;

        // Buffer simulation: account for this read
        if let Some(ref mut buf) = buffer {
            let stall = buf.record_read(native_bytes as usize, clock);
            if !stall.is_zero() {
                clock.sleep_sync(stall);
                buf.tick(clock);
            }
        }
    }

    if blocks_read < block_count {
        let residual = block_count - blocks_read;
        return SenseBuilder::filemark_detected()
            .with_information(residual)
            .to_check_condition_with_data(data);
    }

    sense::good_with_data(data)
}

fn handle_read_variable(
    max_bytes: u32,
    ms: &mut DriveMediaState,
    buffer: &mut Option<DriveBuffer>,
    clock: &SimulationClock,
) -> ScsiResult {
    let pos = ms.position.block_number as usize;

    // Clone the descriptor to avoid borrow conflict
    let desc = ms.current_partition().records[pos].clone();
    let on_disk_bytes = desc.byte_len() as u64;
    let block_data = match read_record_data(ms, &desc) {
        Ok(d) => d,
        Err(scsi_err) => return scsi_err,
    };

    let native_bytes = block_data.len() as u64;
    let part = ms.current_partition_mut();
    part.bytes_read_native += native_bytes;
    part.bytes_read_compressed += on_disk_bytes;
    ms.position.block_number += 1;

    // Buffer simulation: account for this read
    if let Some(ref mut buf) = buffer {
        let stall = buf.record_read(native_bytes as usize, clock);
        if !stall.is_zero() {
            clock.sleep_sync(stall);
            buf.tick(clock);
        }
    }

    let mut result_data = block_data;
    if result_data.len() > max_bytes as usize {
        result_data.truncate(max_bytes as usize);
    }

    sense::good_with_data(result_data)
}
