//! LTO generation constants and tape geometry.

use serde::{Deserialize, Serialize};

/// LTO tape generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LtoGeneration {
    Lto5,
    Lto6,
    Lto7,
    Lto8,
    Lto8M, // Type M cartridge (L7 media formatted for LTO-8)
    Lto9,
}

impl LtoGeneration {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Lto5 => "LTO-5",
            Self::Lto6 => "LTO-6",
            Self::Lto7 => "LTO-7",
            Self::Lto8 => "LTO-8",
            Self::Lto8M => "LTO-8 Type M",
            Self::Lto9 => "LTO-9",
        }
    }

    /// SCSI product identification suffix (e.g., "TD5" for LTO-5)
    pub fn product_suffix(&self) -> &'static str {
        match self {
            Self::Lto5 => "TD5",
            Self::Lto6 => "TD6",
            Self::Lto7 => "TD7",
            Self::Lto8 | Self::Lto8M => "TD8",
            Self::Lto9 => "TD9",
        }
    }

    pub fn geometry(&self) -> &'static TapeGeometry {
        match self {
            Self::Lto5 => &LTO5_GEOMETRY,
            Self::Lto6 => &LTO6_GEOMETRY,
            Self::Lto7 => &LTO7_GEOMETRY,
            Self::Lto8 => &LTO8_GEOMETRY,
            Self::Lto8M => &LTO8M_GEOMETRY,
            Self::Lto9 => &LTO9_GEOMETRY,
        }
    }
}

/// Physical tape geometry for an LTO generation.
#[derive(Debug, Clone)]
pub struct TapeGeometry {
    pub generation: LtoGeneration,
    /// Number of wraps (serpentine passes across the tape).
    pub num_wraps: u32,
    /// Number of data bands.
    pub num_bands: u32,
    /// Parallel write channels per wrap direction.
    pub tracks_per_wrap: u32,
    /// Meters of tape per wrap.
    pub wrap_length_m: f64,
    /// Total tape length in meters.
    pub total_tape_length_m: f64,
    /// Native (uncompressed) capacity in bytes.
    pub native_capacity_bytes: u64,
    /// Maximum logical block size in bytes.
    pub max_logical_block_bytes: u32,
    /// Maximum logical block size when encrypted.
    pub max_logical_block_encrypted_bytes: u32,
    /// Minimum logical block size in bytes.
    pub min_logical_block_bytes: u32,
    /// Sustained native transfer rate in bytes/sec.
    pub sustained_rate_bytes_sec: u64,
    /// Internal buffer size in bytes.
    pub buffer_size_bytes: usize,
    /// Number of digital speed matching levels.
    pub num_speeds: u8,
    /// SCSI density code for this generation.
    pub density_code: u8,
    /// Maximum number of partitions.
    pub max_partitions: u8,
    /// Whether media optimization is required on first load (LTO-9).
    pub requires_media_optimization: bool,
}

// 16 MiB - 1
const MAX_BLOCK: u32 = 16_777_215;
// 8 MiB
const MAX_BLOCK_ENC: u32 = 8_388_608;

pub static LTO5_GEOMETRY: TapeGeometry = TapeGeometry {
    generation: LtoGeneration::Lto5,
    num_wraps: 80,
    num_bands: 4,
    tracks_per_wrap: 16,
    wrap_length_m: 846.0,
    total_tape_length_m: 846.0,
    native_capacity_bytes: 1_500_000_000_000, // 1.5 TB
    max_logical_block_bytes: MAX_BLOCK,
    max_logical_block_encrypted_bytes: MAX_BLOCK_ENC,
    min_logical_block_bytes: 1,
    sustained_rate_bytes_sec: 140_000_000, // 140 MB/s
    buffer_size_bytes: 256 * 1024 * 1024,  // 256 MB
    num_speeds: 14,
    density_code: 0x58,
    max_partitions: 2,
    requires_media_optimization: false,
};

pub static LTO6_GEOMETRY: TapeGeometry = TapeGeometry {
    generation: LtoGeneration::Lto6,
    num_wraps: 136,
    num_bands: 4,
    tracks_per_wrap: 16,
    wrap_length_m: 846.0,
    total_tape_length_m: 846.0,
    native_capacity_bytes: 2_500_000_000_000, // 2.5 TB
    max_logical_block_bytes: MAX_BLOCK,
    max_logical_block_encrypted_bytes: MAX_BLOCK_ENC,
    min_logical_block_bytes: 1,
    sustained_rate_bytes_sec: 160_000_000, // 160 MB/s
    buffer_size_bytes: 512 * 1024 * 1024,  // 512 MB
    num_speeds: 14,
    density_code: 0x5A,
    max_partitions: 4,
    requires_media_optimization: false,
};

pub static LTO7_GEOMETRY: TapeGeometry = TapeGeometry {
    generation: LtoGeneration::Lto7,
    num_wraps: 112,
    num_bands: 4,
    tracks_per_wrap: 32,
    wrap_length_m: 960.0,
    total_tape_length_m: 960.0,
    native_capacity_bytes: 6_000_000_000_000, // 6 TB
    max_logical_block_bytes: MAX_BLOCK,
    max_logical_block_encrypted_bytes: MAX_BLOCK_ENC,
    min_logical_block_bytes: 1,
    sustained_rate_bytes_sec: 300_000_000, // 300 MB/s
    buffer_size_bytes: 512 * 1024 * 1024,
    num_speeds: 14,
    density_code: 0x5C,
    max_partitions: 4,
    requires_media_optimization: false,
};

pub static LTO8_GEOMETRY: TapeGeometry = TapeGeometry {
    generation: LtoGeneration::Lto8,
    num_wraps: 208,
    num_bands: 4,
    tracks_per_wrap: 32,
    wrap_length_m: 960.0,
    total_tape_length_m: 960.0,
    native_capacity_bytes: 12_000_000_000_000, // 12 TB
    max_logical_block_bytes: MAX_BLOCK,
    max_logical_block_encrypted_bytes: MAX_BLOCK_ENC,
    min_logical_block_bytes: 1,
    sustained_rate_bytes_sec: 360_000_000, // 360 MB/s
    buffer_size_bytes: 512 * 1024 * 1024,
    num_speeds: 14,
    density_code: 0x5D,
    max_partitions: 4,
    requires_media_optimization: false,
};

pub static LTO8M_GEOMETRY: TapeGeometry = TapeGeometry {
    generation: LtoGeneration::Lto8M,
    num_wraps: 208,
    num_bands: 4,
    tracks_per_wrap: 32,
    wrap_length_m: 960.0,
    total_tape_length_m: 960.0,
    native_capacity_bytes: 9_000_000_000_000, // 9 TB (Type M)
    max_logical_block_bytes: MAX_BLOCK,
    max_logical_block_encrypted_bytes: MAX_BLOCK_ENC,
    min_logical_block_bytes: 1,
    sustained_rate_bytes_sec: 360_000_000,
    buffer_size_bytes: 512 * 1024 * 1024,
    num_speeds: 14,
    density_code: 0x5E,
    max_partitions: 4,
    requires_media_optimization: false,
};

pub static LTO9_GEOMETRY: TapeGeometry = TapeGeometry {
    generation: LtoGeneration::Lto9,
    num_wraps: 280,
    num_bands: 4,
    tracks_per_wrap: 32,
    wrap_length_m: 1035.0,
    total_tape_length_m: 1035.0,
    native_capacity_bytes: 18_000_000_000_000, // 18 TB
    max_logical_block_bytes: MAX_BLOCK,
    max_logical_block_encrypted_bytes: MAX_BLOCK_ENC,
    min_logical_block_bytes: 1,
    sustained_rate_bytes_sec: 400_000_000, // 400 MB/s
    buffer_size_bytes: 1024 * 1024 * 1024, // 1 GB
    num_speeds: 14,
    density_code: 0x60,
    max_partitions: 4,
    requires_media_optimization: true,
};
