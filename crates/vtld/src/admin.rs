use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Request, State};
use axum::http::{header, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, post};
use axum::Router;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Notify};
use tracing::info;
use utoipa::OpenApi;

use iscsi_target::SimulationClock;
use iscsi_target::cdb_decode::{decode_cdb, decode_response, CdbBreakdown, ResponseBreakdown};
use iscsi_target::scsi_log::{scsi_status_name, DeviceType, ScsiCommandLog};
use iscsi_target::SessionRegistry;
use smc::{ElementType, MediaChanger};
use ssc::read_media_detail;
use ssc::TapeDrive;

use crate::config::UserConfig;
use crate::error::{AdminError, Error};
use crate::store::Store;

#[derive(Clone)]
pub struct AdminState {
    pub store: Store,
    pub users: Vec<UserConfig>,
    pub jwt_secret: String,
    pub ws_tx: broadcast::Sender<()>,
    pub version: &'static str,
    pub changer: Arc<MediaChanger>,
    pub drives: Vec<Arc<TapeDrive>>,
    pub session_registry: Arc<SessionRegistry>,
    pub changer_log: Arc<ScsiCommandLog>,
    pub drive_logs: Vec<Arc<ScsiCommandLog>>,
    pub data_dir: std::path::PathBuf,
    pub simulation_clock: Arc<SimulationClock>,
    pub config_snapshot: ConfigSnapshot,
}

#[derive(Clone)]
pub struct ConfigSnapshot {
    pub listen_host: String,
    pub listen_admin_port: u16,
    pub store_path: String,
    pub iscsi_port: u16,
    pub iscsi_iqn: String,
    pub library_model: String,
    pub library_serial: String,
    pub library_data_dir: String,
    pub library_drives: usize,
    pub library_slots: usize,
    pub library_media_count: usize,
    pub library_media_barcodes: Vec<String>,
    pub user_count: usize,
    pub initial_simulation_speed: f64,
}

#[derive(Embed)]
#[folder = "frontend/dist"]
struct FrontendAssets;

// --- API Types ---

#[derive(Serialize, utoipa::ToSchema)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct LoginResponse {
    token: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct VtlStatusResponse {
    status: String,
    vendor: String,
    product: String,
    serial: String,
    total_slots: u16,
    used_slots: u16,
    total_drives: u16,
    import_export_slots: u16,
}

#[derive(Serialize, utoipa::ToSchema)]
struct DriveResponse {
    id: usize,
    status: String,
    serial: String,
    barcode: Option<String>,
    position: usize,
    record_count: usize,
}

#[derive(Serialize, utoipa::ToSchema)]
struct SlotResponse {
    address: u16,
    full: bool,
    barcode: Option<String>,
    source_element: u16,
}

#[derive(Serialize, utoipa::ToSchema)]
struct MediaResponse {
    barcode: String,
    location: String,
    location_address: u16,
}

#[derive(Serialize, utoipa::ToSchema)]
struct PartitionDetailResponse {
    index: u8,
    record_count: u64,
    filemark_count: u64,
    filemark_positions: Vec<u64>,
    bytes_written_native: u64,
    bytes_written_compressed: u64,
    bytes_read_native: u64,
}

#[derive(Serialize, utoipa::ToSchema)]
struct MediaDetailResponse {
    barcode: String,
    generation: String,
    write_protected: bool,
    worm: bool,
    medium_type: String,
    location: String,
    location_type: String,
    in_drive: Option<usize>,
    partition_count: u8,
    total_records: u64,
    total_filemarks: u64,
    native_bytes_written: u64,
    compressed_bytes_written: u64,
    native_capacity_bytes: u64,
    capacity_used_pct: f64,
    approximate_remaining_mb: u64,
    compression_enabled: bool,
    compression_ratio: f64,
    total_loads: u32,
    optimization_done: bool,
    partitions: Vec<PartitionDetailResponse>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct LibrarySnapshot {
    status: VtlStatusResponse,
    drives: Vec<DriveResponse>,
    slots: Vec<SlotResponse>,
    media: Vec<MediaResponse>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ConfigEntry {
    key: String,
    value: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct FullConfigResponse {
    sections: Vec<ConfigSection>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ConfigSection {
    name: String,
    description: String,
    settings: Vec<ConfigSetting>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ConfigSetting {
    key: String,
    description: String,
    value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_value: Option<serde_json::Value>,
    required: bool,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ChangerDetailResponse {
    vendor: String,
    product: String,
    serial: String,
    firmware_version: String,
    state: String,
    temperature_c: u8,
    humidity_pct: u8,
    total_moves: u64,
    picker_position: u16,
    active_alerts: Vec<u16>,
    prevent_medium_removal: bool,
    num_drives: u16,
    num_slots: u16,
    num_import_export: u16,
    elements: Vec<ElementDetailResponse>,
    robot_operation: Option<RobotOperationResponse>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct RobotOperationResponse {
    kind: String,
    source: Option<u16>,
    dest: Option<u16>,
    started_at_ms: i64,
    estimated_secs: f64,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ElementDetailResponse {
    address: u16,
    element_type: String,
    full: bool,
    barcode: Option<String>,
    source_element: u16,
    access: bool,
    except: bool,
    disabled: bool,
    asc_ascq: Option<[u8; 2]>,
    medium_type: String,
    import_export: bool,
    operator_intervention: bool,
}

#[derive(Serialize, utoipa::ToSchema)]
struct SpeedTimeEntryResponse {
    speed_index: u8,
    rate_mbps: f64,
    time_secs: f64,
    time_pct: f64,
}

#[derive(Serialize, utoipa::ToSchema)]
struct DriveDetailResponse {
    id: usize,
    serial: String,
    generation: String,
    loaded: bool,
    barcode: Option<String>,
    write_protected: bool,
    worm: bool,
    partition: u8,
    block_number: u64,
    file_number: u64,
    at_bop: bool,
    at_eod: bool,
    current_wrap: Option<u32>,
    total_wraps: Option<u32>,
    position_in_wrap_pct: Option<f64>,
    write_buffer_pct: f64,
    read_cache_pct: f64,
    objects_in_buffer: u32,
    buffer_state: String,
    drive_state: String,
    tape_speed: Option<u8>,
    operation_progress_pct: Option<f64>,
    instantaneous_rate_bytes_sec: Option<u64>,
    compression_ratio: Option<f64>,
    backhitch_count_this_mount: u32,
    capacity_used_pct: Option<f64>,
    native_bytes_written: u64,
    compressed_bytes_written: u64,
    approximate_remaining_mb: Option<u64>,
    total_loads: u32,
    motion_hours: f64,
    // Buffer detail
    buffer_capacity_bytes: usize,
    buffer_used_bytes: usize,
    read_cache_bytes: usize,
    tape_velocity_pct: Option<f64>,
    host_rate_bytes_sec: Option<u64>,
    tape_rate_bytes_sec: Option<u64>,
    speed_change_count: u32,
    buffer_backhitch_count: u32,
    high_water_mark_pct: f64,
    stall_time_secs: f64,
    speed_time_distribution: Option<Vec<SpeedTimeEntryResponse>>,
    tape_efficiency_pct: Option<f64>,
    write_cycle_count: u32,
    read_cycle_count: u32,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ConnectionResponse {
    cid: u16,
    peer_addr: String,
    connected_since: String,
    rx_commands: u64,
    rx_bytes: u64,
    tx_commands: u64,
    tx_bytes: u64,
    active_commands: u32,
    scsi_log: Vec<ScsiLogSummaryEntry>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct SessionResponse {
    initiator_name: String,
    tsih: u16,
    connections: Vec<ConnectionResponse>,
    active_commands: u32,
}

// --- SCSI Log API Types ---

#[derive(Serialize, utoipa::ToSchema)]
struct ScsiLogSummaryEntry {
    seq: u64,
    timestamp: String,
    duration_us: u64,
    opcode: u8,
    opcode_name: String,
    status: u8,
    status_name: String,
    data_out_len: usize,
    data_in_len: usize,
    has_sense: bool,
    completed: bool,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ScsiLogResponse {
    device_type: String,
    device_id: usize,
    entries: Vec<ScsiLogSummaryEntry>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ScsiCommandDetailResponse {
    seq: u64,
    timestamp: String,
    duration_us: u64,
    opcode: u8,
    opcode_name: String,
    cdb_hex: String,
    data_out_hex: Option<String>,
    data_out_len: usize,
    status: u8,
    status_name: String,
    data_in_hex: Option<String>,
    data_in_len: usize,
    sense_hex: String,
    cdb_breakdown: CdbBreakdown,
    response_breakdown: ResponseBreakdown,
    completed: bool,
}

#[derive(Deserialize)]
struct ScsiLogQuery {
    limit: Option<usize>,
}

// --- Handlers ---

#[utoipa::path(
    get,
    path = "/api/health",
    tag = "System",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
async fn health(State(state): State<AdminState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: state.version.into(),
    })
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
async fn login(
    State(state): State<AdminState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AdminError> {
    let valid = state
        .users
        .iter()
        .any(|u| u.username == body.username && u.password == body.password);
    if !valid {
        return Err(Error::Unauthorized("invalid credentials".into()).into());
    }
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;
    let claims = Claims {
        sub: body.username,
        exp,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| Error::Other(format!("token creation failed: {e}")))?;
    Ok(Json(LoginResponse { token }))
}

#[utoipa::path(
    get,
    path = "/api/vtl/status",
    tag = "VTL",
    responses(
        (status = 200, description = "Library status summary", body = VtlStatusResponse)
    )
)]
async fn vtl_status(State(state): State<AdminState>) -> Json<VtlStatusResponse> {
    let snap = state.changer.snapshot();
    let used_slots = snap
        .elements
        .iter()
        .filter(|e| e.element_type == ElementType::Storage && e.full)
        .count() as u16;
    Json(VtlStatusResponse {
        status: "online".into(),
        vendor: snap.vendor,
        product: snap.product,
        serial: snap.serial,
        total_slots: snap.num_slots,
        used_slots,
        total_drives: snap.num_drives,
        import_export_slots: snap.num_import_export,
    })
}

#[utoipa::path(
    get,
    path = "/api/vtl/drives",
    tag = "VTL",
    responses(
        (status = 200, description = "List drives", body = Vec<DriveResponse>)
    )
)]
async fn vtl_drives(State(state): State<AdminState>) -> Json<Vec<DriveResponse>> {
    let drives: Vec<DriveResponse> = state
        .drives
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let snap = d.snapshot();
            DriveResponse {
                id: i,
                status: if snap.loaded {
                    "loaded".into()
                } else {
                    "empty".into()
                },
                serial: snap.serial,
                barcode: snap.barcode,
                position: snap.position,
                record_count: snap.record_count,
            }
        })
        .collect();
    Json(drives)
}

#[utoipa::path(
    get,
    path = "/api/vtl/media",
    tag = "VTL",
    responses(
        (status = 200, description = "Media inventory", body = Vec<MediaResponse>)
    )
)]
async fn vtl_media(State(state): State<AdminState>) -> Json<Vec<MediaResponse>> {
    let snap = state.changer.snapshot();
    let media: Vec<MediaResponse> = snap
        .elements
        .iter()
        .filter(|e| e.full && e.barcode.is_some())
        .map(|e| {
            let location = match e.element_type {
                ElementType::DataTransfer => format!("drive:{}", e.address),
                ElementType::Storage => format!("slot:{}", e.address),
                ElementType::ImportExport => format!("import_export:{}", e.address),
                ElementType::Transport => format!("transport:{}", e.address),
            };
            MediaResponse {
                barcode: e.barcode.clone().unwrap(),
                location,
                location_address: e.address,
            }
        })
        .collect();
    Json(media)
}

#[utoipa::path(
    get,
    path = "/api/vtl/media/{barcode}",
    tag = "VTL",
    params(
        ("barcode" = String, Path, description = "Media barcode")
    ),
    responses(
        (status = 200, description = "Media detail", body = MediaDetailResponse),
        (status = 404, description = "Media not found")
    )
)]
async fn vtl_media_detail(
    State(state): State<AdminState>,
    axum::extract::Path(barcode): axum::extract::Path<String>,
) -> Result<Json<MediaDetailResponse>, StatusCode> {
    let snap = state.changer.snapshot();

    // Find this barcode in the changer elements
    let element = snap
        .elements
        .iter()
        .find(|e| e.barcode.as_deref() == Some(&barcode))
        .ok_or(StatusCode::NOT_FOUND)?;

    let (location, location_type, in_drive) = match element.element_type {
        ElementType::DataTransfer => {
            // Find which drive index this address corresponds to
            let drive_idx = snap
                .elements
                .iter()
                .filter(|e| e.element_type == ElementType::DataTransfer)
                .position(|e| e.address == element.address);
            (
                format!("Drive {}", drive_idx.unwrap_or(0)),
                "data_transfer".to_string(),
                drive_idx,
            )
        }
        ElementType::Storage => (
            format!("Slot 0x{:04X}", element.address),
            "storage".to_string(),
            None,
        ),
        ElementType::ImportExport => (
            format!("I/E Port 0x{:04X}", element.address),
            "import_export".to_string(),
            None,
        ),
        ElementType::Transport => (
            format!("Transport 0x{:04X}", element.address),
            "transport".to_string(),
            None,
        ),
    };

    let medium_type = format!("{:?}", element.medium_type);

    // If media is loaded in a drive, read live in-memory state (partition stats
    // are only flushed to the .redb store on unload, so the on-disk data is stale
    // while the drive is actively reading/writing).
    let detail = in_drive
        .and_then(|idx| state.drives.get(idx))
        .and_then(|drive| drive.media_detail())
        .or_else(|| read_media_detail(&state.data_dir, &barcode));

    let to_partition_responses = |partitions: Vec<ssc::PartitionDetail>| -> Vec<PartitionDetailResponse> {
        partitions
            .into_iter()
            .map(|p| PartitionDetailResponse {
                index: p.index,
                record_count: p.record_count,
                filemark_count: p.filemark_count,
                filemark_positions: p.filemark_positions,
                bytes_written_native: p.bytes_written_native,
                bytes_written_compressed: p.bytes_written_compressed,
                bytes_read_native: p.bytes_read_native,
            })
            .collect()
    };

    match detail {
        Some(d) => Ok(Json(MediaDetailResponse {
            barcode: d.barcode,
            generation: format!("{:?}", d.generation),
            write_protected: d.write_protected,
            worm: d.worm,
            medium_type,
            location,
            location_type,
            in_drive,
            partition_count: d.partition_count,
            total_records: d.total_records,
            total_filemarks: d.total_filemarks,
            native_bytes_written: d.native_bytes_written,
            compressed_bytes_written: d.compressed_bytes_written,
            native_capacity_bytes: d.native_capacity_bytes,
            capacity_used_pct: d.capacity_used_pct,
            approximate_remaining_mb: d.approximate_remaining_mb,
            compression_enabled: d.compression_enabled,
            compression_ratio: d.compression_ratio,
            total_loads: d.total_loads,
            optimization_done: d.optimization_done,
            partitions: to_partition_responses(d.partitions),
        })),
        None => {
            // Media is in the changer but has no .redb file yet (never loaded)
            Ok(Json(MediaDetailResponse {
                barcode: barcode.clone(),
                generation: "Unknown".to_string(),
                write_protected: false,
                worm: false,
                medium_type,
                location,
                location_type,
                in_drive,
                partition_count: 0,
                total_records: 0,
                total_filemarks: 0,
                native_bytes_written: 0,
                compressed_bytes_written: 0,
                native_capacity_bytes: 0,
                capacity_used_pct: 0.0,
                approximate_remaining_mb: 0,
                compression_enabled: false,
                compression_ratio: 0.0,
                total_loads: 0,
                optimization_done: false,
                partitions: Vec::new(),
            }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/vtl/snapshot",
    tag = "VTL",
    responses(
        (status = 200, description = "Complete library snapshot", body = LibrarySnapshot)
    )
)]
async fn vtl_snapshot(State(state): State<AdminState>) -> Json<LibrarySnapshot> {
    let changer_snap = state.changer.snapshot();

    let used_slots = changer_snap
        .elements
        .iter()
        .filter(|e| e.element_type == ElementType::Storage && e.full)
        .count() as u16;

    let status = VtlStatusResponse {
        status: "online".into(),
        vendor: changer_snap.vendor.clone(),
        product: changer_snap.product.clone(),
        serial: changer_snap.serial.clone(),
        total_slots: changer_snap.num_slots,
        used_slots,
        total_drives: changer_snap.num_drives,
        import_export_slots: changer_snap.num_import_export,
    };

    let drives: Vec<DriveResponse> = state
        .drives
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let ds = d.snapshot();
            DriveResponse {
                id: i,
                status: if ds.loaded {
                    "loaded".into()
                } else {
                    "empty".into()
                },
                serial: ds.serial,
                barcode: ds.barcode,
                position: ds.position,
                record_count: ds.record_count,
            }
        })
        .collect();

    let slots: Vec<SlotResponse> = changer_snap
        .elements
        .iter()
        .filter(|e| e.element_type == ElementType::Storage)
        .map(|e| SlotResponse {
            address: e.address,
            full: e.full,
            barcode: e.barcode.clone(),
            source_element: e.source_element,
        })
        .collect();

    let media: Vec<MediaResponse> = changer_snap
        .elements
        .iter()
        .filter(|e| e.full && e.barcode.is_some())
        .map(|e| {
            let location = match e.element_type {
                ElementType::DataTransfer => format!("drive:{}", e.address),
                ElementType::Storage => format!("slot:{}", e.address),
                ElementType::ImportExport => format!("import_export:{}", e.address),
                ElementType::Transport => format!("transport:{}", e.address),
            };
            MediaResponse {
                barcode: e.barcode.clone().unwrap(),
                location,
                location_address: e.address,
            }
        })
        .collect();

    Json(LibrarySnapshot {
        status,
        drives,
        slots,
        media,
    })
}

#[utoipa::path(
    get,
    path = "/api/config",
    tag = "Config",
    responses(
        (status = 200, description = "Show stored config entries", body = Vec<ConfigEntry>)
    )
)]
async fn config_show(
    State(state): State<AdminState>,
) -> Result<Json<Vec<ConfigEntry>>, AdminError> {
    let entries = state.store.config_list().await?;
    let resp: Vec<ConfigEntry> = entries
        .into_iter()
        .map(|(key, value)| ConfigEntry {
            key,
            value: String::from_utf8_lossy(&value).into_owned(),
        })
        .collect();
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/config/full",
    tag = "Config",
    responses(
        (status = 200, description = "Full configuration with descriptions and defaults", body = FullConfigResponse)
    )
)]
async fn config_full(
    State(state): State<AdminState>,
) -> Result<Json<FullConfigResponse>, AdminError> {
    let snap = &state.config_snapshot;

    // Check live simulation speed
    let live_speed = state.simulation_clock.speed_factor();
    let sim_current = if (live_speed - snap.initial_simulation_speed).abs() > f64::EPSILON {
        Some(serde_json::json!(live_speed))
    } else {
        None
    };

    let media_value = if snap.library_media_count > 0 {
        serde_json::json!({
            "count": snap.library_media_count,
            "barcodes": snap.library_media_barcodes,
        })
    } else {
        serde_json::json!({ "count": 0 })
    };

    let sections = vec![
        ConfigSection {
            name: "Listen".into(),
            description: "Network bind addresses for the admin UI and API".into(),
            settings: vec![
                ConfigSetting {
                    key: "listen.host".into(),
                    description: "IP address to bind the server to".into(),
                    value: serde_json::json!(snap.listen_host),
                    default_value: Some(serde_json::json!("0.0.0.0")),
                    current_value: None,
                    required: false,
                },
                ConfigSetting {
                    key: "listen.admin_port".into(),
                    description: "TCP port for the admin web UI and REST API".into(),
                    value: serde_json::json!(snap.listen_admin_port),
                    default_value: Some(serde_json::json!(8081)),
                    current_value: None,
                    required: false,
                },
            ],
        },
        ConfigSection {
            name: "Store".into(),
            description: "Embedded database storage".into(),
            settings: vec![
                ConfigSetting {
                    key: "store.path".into(),
                    description: "Path to the redb database file".into(),
                    value: serde_json::json!(snap.store_path),
                    default_value: Some(serde_json::json!("vtld.redb")),
                    current_value: None,
                    required: false,
                },
            ],
        },
        ConfigSection {
            name: "iSCSI".into(),
            description: "iSCSI target configuration".into(),
            settings: vec![
                ConfigSetting {
                    key: "iscsi.port".into(),
                    description: "TCP port the iSCSI target listens on".into(),
                    value: serde_json::json!(snap.iscsi_port),
                    default_value: Some(serde_json::json!(3260)),
                    current_value: None,
                    required: false,
                },
                ConfigSetting {
                    key: "iscsi.iqn".into(),
                    description: "iSCSI Qualified Name identifying this target".into(),
                    value: serde_json::json!(snap.iscsi_iqn),
                    default_value: Some(serde_json::json!("iqn.2024-01.com.quantumvtl:vtl")),
                    current_value: None,
                    required: false,
                },
            ],
        },
        ConfigSection {
            name: "Library".into(),
            description: "Virtual tape library hardware emulation".into(),
            settings: vec![
                ConfigSetting {
                    key: "library.model".into(),
                    description: "Model name reported in SCSI INQUIRY data".into(),
                    value: serde_json::json!(snap.library_model),
                    default_value: None,
                    current_value: None,
                    required: true,
                },
                ConfigSetting {
                    key: "library.serial".into(),
                    description: "Serial number reported in SCSI INQUIRY and VPD pages".into(),
                    value: serde_json::json!(snap.library_serial),
                    default_value: None,
                    current_value: None,
                    required: true,
                },
                ConfigSetting {
                    key: "library.data_dir".into(),
                    description: "Directory where virtual tape media data files are stored".into(),
                    value: serde_json::json!(snap.library_data_dir),
                    default_value: None,
                    current_value: None,
                    required: true,
                },
                ConfigSetting {
                    key: "library.drives".into(),
                    description: "Number of tape drives in the library".into(),
                    value: serde_json::json!(snap.library_drives),
                    default_value: None,
                    current_value: None,
                    required: true,
                },
                ConfigSetting {
                    key: "library.slots".into(),
                    description: "Number of storage slots for tape media".into(),
                    value: serde_json::json!(snap.library_slots),
                    default_value: None,
                    current_value: None,
                    required: true,
                },
                ConfigSetting {
                    key: "library.media".into(),
                    description: "Pre-loaded media cartridges".into(),
                    value: media_value,
                    default_value: Some(serde_json::json!({ "count": 0 })),
                    current_value: None,
                    required: false,
                },
            ],
        },
        ConfigSection {
            name: "Users".into(),
            description: "Authentication accounts (passwords hidden)".into(),
            settings: vec![
                ConfigSetting {
                    key: "users".into(),
                    description: "Number of configured user accounts".into(),
                    value: serde_json::json!(snap.user_count),
                    default_value: Some(serde_json::json!(0)),
                    current_value: None,
                    required: false,
                },
            ],
        },
        ConfigSection {
            name: "Simulation".into(),
            description: "Tape operation timing simulation".into(),
            settings: vec![
                ConfigSetting {
                    key: "simulation_speed".into(),
                    description: "Speed multiplier for tape operations (1.0 = real-time)".into(),
                    value: serde_json::json!(snap.initial_simulation_speed),
                    default_value: Some(serde_json::json!(1.0)),
                    current_value: sim_current,
                    required: false,
                },
            ],
        },
    ];

    Ok(Json(FullConfigResponse { sections }))
}

#[utoipa::path(
    get,
    path = "/api/vtl/changer",
    tag = "VTL",
    responses(
        (status = 200, description = "Detailed changer status", body = ChangerDetailResponse)
    )
)]
async fn vtl_changer(State(state): State<AdminState>) -> Json<ChangerDetailResponse> {
    let snap = state.changer.snapshot();
    let elements: Vec<ElementDetailResponse> = snap
        .elements
        .iter()
        .map(|e| {
            let element_type = match e.element_type {
                ElementType::Transport => "transport",
                ElementType::Storage => "storage",
                ElementType::ImportExport => "import_export",
                ElementType::DataTransfer => "data_transfer",
            }
            .to_string();
            ElementDetailResponse {
                address: e.address,
                element_type,
                full: e.full,
                barcode: e.barcode.clone(),
                source_element: e.source_element,
                access: e.access,
                except: e.except,
                disabled: e.disabled,
                asc_ascq: e.asc_ascq.map(|(a, b)| [a, b]),
                medium_type: format!("{:?}", e.medium_type),
                import_export: e.import_export,
                operator_intervention: e.operator_intervention,
            }
        })
        .collect();
    let state_str = match &snap.state {
        smc::LibraryState::Initializing => "Initializing".to_string(),
        smc::LibraryState::Ready => "Ready".to_string(),
        smc::LibraryState::NotReady(reason) => format!("NotReady: {reason}"),
        smc::LibraryState::Moving { .. } => "Moving".to_string(),
        smc::LibraryState::Scanning => "Scanning".to_string(),
    };

    let robot_operation = snap.robot_started_at_ms.zip(snap.robot_estimated_secs).map(
        |(started_at_ms, estimated_secs)| {
            let (kind, source, dest) = match &snap.state {
                smc::LibraryState::Moving { source, dest } => {
                    ("moving".to_string(), Some(*source), Some(*dest))
                }
                smc::LibraryState::Scanning => ("scanning".to_string(), None, None),
                _ => ("unknown".to_string(), None, None),
            };
            RobotOperationResponse {
                kind,
                source,
                dest,
                started_at_ms,
                estimated_secs,
            }
        },
    );

    Json(ChangerDetailResponse {
        vendor: snap.vendor,
        product: snap.product,
        serial: snap.serial,
        firmware_version: snap.firmware_version,
        state: state_str,
        temperature_c: snap.temperature_c,
        humidity_pct: snap.humidity_pct,
        total_moves: snap.total_moves,
        picker_position: snap.picker_position,
        active_alerts: snap.active_alerts,
        prevent_medium_removal: snap.prevent_medium_removal,
        num_drives: snap.num_drives,
        num_slots: snap.num_slots,
        num_import_export: snap.num_import_export,
        elements,
        robot_operation,
    })
}

#[utoipa::path(
    get,
    path = "/api/vtl/drives/{id}",
    tag = "VTL",
    params(
        ("id" = usize, Path, description = "Drive index (0-based)")
    ),
    responses(
        (status = 200, description = "Detailed drive status", body = DriveDetailResponse),
        (status = 404, description = "Drive not found")
    )
)]
async fn vtl_drive_detail(
    State(state): State<AdminState>,
    axum::extract::Path(id): axum::extract::Path<usize>,
) -> Result<Json<DriveDetailResponse>, StatusCode> {
    let drive = state.drives.get(id).ok_or(StatusCode::NOT_FOUND)?;
    let snap = drive.snapshot();
    Ok(Json(DriveDetailResponse {
        id,
        serial: snap.serial,
        generation: format!("{:?}", snap.generation),
        loaded: snap.loaded,
        barcode: snap.barcode,
        write_protected: snap.write_protected,
        worm: snap.worm,
        partition: snap.partition,
        block_number: snap.block_number,
        file_number: snap.file_number,
        at_bop: snap.at_bop,
        at_eod: snap.at_eod,
        current_wrap: snap.current_wrap,
        total_wraps: snap.total_wraps,
        position_in_wrap_pct: snap.position_in_wrap_pct,
        write_buffer_pct: snap.write_buffer_pct,
        read_cache_pct: snap.read_cache_pct,
        objects_in_buffer: snap.objects_in_buffer,
        buffer_state: snap.buffer_state,
        drive_state: snap.drive_state.display_name().to_string(),
        tape_speed: snap.tape_speed,
        operation_progress_pct: snap.operation_progress_pct,
        instantaneous_rate_bytes_sec: snap.instantaneous_rate_bytes_sec,
        compression_ratio: snap.compression_ratio,
        backhitch_count_this_mount: snap.backhitch_count_this_mount,
        capacity_used_pct: snap.capacity_used_pct,
        native_bytes_written: snap.native_bytes_written,
        compressed_bytes_written: snap.compressed_bytes_written,
        approximate_remaining_mb: snap.approximate_remaining_mb,
        total_loads: snap.total_loads,
        motion_hours: snap.motion_hours,
        buffer_capacity_bytes: snap.buffer_capacity_bytes,
        buffer_used_bytes: snap.buffer_used_bytes,
        read_cache_bytes: snap.read_cache_bytes,
        tape_velocity_pct: snap.tape_velocity_pct,
        host_rate_bytes_sec: snap.host_rate_bytes_sec,
        tape_rate_bytes_sec: snap.tape_rate_bytes_sec,
        speed_change_count: snap.speed_change_count,
        buffer_backhitch_count: snap.buffer_backhitch_count,
        high_water_mark_pct: snap.high_water_mark_pct,
        stall_time_secs: snap.stall_time_secs,
        speed_time_distribution: snap.speed_time_distribution.map(|dist| {
            dist.into_iter()
                .map(|e| SpeedTimeEntryResponse {
                    speed_index: e.speed_index,
                    rate_mbps: e.rate_mbps,
                    time_secs: e.time_secs,
                    time_pct: e.time_pct,
                })
                .collect()
        }),
        tape_efficiency_pct: snap.tape_efficiency_pct,
        write_cycle_count: snap.write_cycle_count,
        read_cycle_count: snap.read_cycle_count,
    }))
}

#[utoipa::path(
    get,
    path = "/api/vtl/sessions",
    tag = "VTL",
    responses(
        (status = 200, description = "Active iSCSI sessions", body = Vec<SessionResponse>)
    )
)]
async fn vtl_sessions(State(state): State<AdminState>) -> Json<Vec<SessionResponse>> {
    let sessions: Vec<SessionResponse> = state
        .session_registry
        .session_snapshots()
        .into_iter()
        .map(|s| {
            let active_commands: u32 = s.connections.iter().map(|c| c.active_commands).sum();
            SessionResponse {
                initiator_name: s.initiator_name,
                tsih: s.tsih,
                connections: s
                    .connections
                    .into_iter()
                    .map(|c| ConnectionResponse {
                        cid: c.cid,
                        peer_addr: c.peer_addr,
                        connected_since: c.connected_since,
                        rx_commands: c.rx_commands,
                        rx_bytes: c.rx_bytes,
                        tx_commands: c.tx_commands,
                        tx_bytes: c.tx_bytes,
                        active_commands: c.active_commands,
                        scsi_log: c
                            .scsi_log
                            .last_n(20)
                            .iter()
                            .map(log_to_summary)
                            .collect(),
                    })
                    .collect(),
                active_commands,
            }
        })
        .collect();
    Json(sessions)
}

// --- SCSI Log Handlers ---

fn log_to_summary(entry: &iscsi_target::scsi_log::ScsiLogEntry) -> ScsiLogSummaryEntry {
    ScsiLogSummaryEntry {
        seq: entry.seq,
        timestamp: entry.timestamp.clone(),
        duration_us: entry.duration_us,
        opcode: entry.opcode,
        opcode_name: entry.opcode_name.clone(),
        status: entry.status,
        status_name: scsi_status_name(entry.status).to_string(),
        data_out_len: entry.data_out_len,
        data_in_len: entry.data_in_len,
        has_sense: !entry.sense.is_empty(),
        completed: entry.completed,
    }
}

fn hex_string(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

#[utoipa::path(
    get,
    path = "/api/vtl/scsi-log/changer",
    tag = "SCSI Log",
    params(
        ("limit" = Option<usize>, Query, description = "Max entries to return (default 20, max 20)")
    ),
    responses(
        (status = 200, description = "Changer SCSI command log", body = ScsiLogResponse)
    )
)]
async fn scsi_log_changer(
    State(state): State<AdminState>,
    axum::extract::Query(q): axum::extract::Query<ScsiLogQuery>,
) -> Json<ScsiLogResponse> {
    let limit = q.limit.unwrap_or(20).min(20);
    let entries = state.changer_log.last_n(limit);
    Json(ScsiLogResponse {
        device_type: "changer".into(),
        device_id: 0,
        entries: entries.iter().map(log_to_summary).collect(),
    })
}

#[utoipa::path(
    get,
    path = "/api/vtl/scsi-log/drive/{id}",
    tag = "SCSI Log",
    params(
        ("id" = usize, Path, description = "Drive index"),
        ("limit" = Option<usize>, Query, description = "Max entries to return (default 20, max 20)")
    ),
    responses(
        (status = 200, description = "Drive SCSI command log", body = ScsiLogResponse),
        (status = 404, description = "Drive not found")
    )
)]
async fn scsi_log_drive(
    State(state): State<AdminState>,
    axum::extract::Path(id): axum::extract::Path<usize>,
    axum::extract::Query(q): axum::extract::Query<ScsiLogQuery>,
) -> Result<Json<ScsiLogResponse>, StatusCode> {
    let log = state.drive_logs.get(id).ok_or(StatusCode::NOT_FOUND)?;
    let limit = q.limit.unwrap_or(20).min(20);
    let entries = log.last_n(limit);
    Ok(Json(ScsiLogResponse {
        device_type: "drive".into(),
        device_id: id,
        entries: entries.iter().map(log_to_summary).collect(),
    }))
}

#[derive(Deserialize)]
struct ScsiEntryPath {
    device_type: String,
    device_id: usize,
    seq: u64,
}

#[utoipa::path(
    get,
    path = "/api/vtl/scsi-log/entry/{device_type}/{device_id}/{seq}",
    tag = "SCSI Log",
    params(
        ("device_type" = String, Path, description = "\"changer\" or \"drive\""),
        ("device_id" = usize, Path, description = "Device index (0 for changer)"),
        ("seq" = u64, Path, description = "Sequence number")
    ),
    responses(
        (status = 200, description = "Full command detail with CDB/response breakdown", body = ScsiCommandDetailResponse),
        (status = 404, description = "Entry not found")
    )
)]
async fn scsi_log_entry(
    State(state): State<AdminState>,
    axum::extract::Path(path): axum::extract::Path<ScsiEntryPath>,
) -> Result<Json<ScsiCommandDetailResponse>, StatusCode> {
    let (log, dt) = match path.device_type.as_str() {
        "changer" => (state.changer_log.clone(), DeviceType::MediaChanger),
        "drive" => {
            let l = state
                .drive_logs
                .get(path.device_id)
                .ok_or(StatusCode::NOT_FOUND)?;
            (l.clone(), DeviceType::TapeDrive)
        }
        _ => return Err(StatusCode::NOT_FOUND),
    };

    let entry = log.get_by_seq(path.seq).ok_or(StatusCode::NOT_FOUND)?;
    let cdb_breakdown = decode_cdb(&entry.cdb, dt);
    let response_breakdown = decode_response(&entry);

    Ok(Json(ScsiCommandDetailResponse {
        seq: entry.seq,
        timestamp: entry.timestamp.clone(),
        duration_us: entry.duration_us,
        opcode: entry.opcode,
        opcode_name: entry.opcode_name.clone(),
        cdb_hex: hex_string(&entry.cdb),
        data_out_hex: entry.data_out.as_ref().map(|d| hex_string(d)),
        data_out_len: entry.data_out_len,
        status: entry.status,
        status_name: scsi_status_name(entry.status).to_string(),
        data_in_hex: entry.data_in.as_ref().map(|d| hex_string(d)),
        data_in_len: entry.data_in_len,
        sense_hex: hex_string(&entry.sense),
        cdb_breakdown,
        response_breakdown,
        completed: entry.completed,
    }))
}

#[derive(Deserialize)]
struct SessionScsiEntryPath {
    tsih: u16,
    seq: u64,
}

#[utoipa::path(
    get,
    path = "/api/vtl/sessions/{tsih}/scsi-log/{seq}",
    tag = "SCSI Log",
    params(
        ("tsih" = u16, Path, description = "Session TSIH"),
        ("seq" = u64, Path, description = "Sequence number")
    ),
    responses(
        (status = 200, description = "Full command detail", body = ScsiCommandDetailResponse),
        (status = 404, description = "Entry not found")
    )
)]
async fn session_scsi_log_entry(
    State(state): State<AdminState>,
    axum::extract::Path(path): axum::extract::Path<SessionScsiEntryPath>,
) -> Result<Json<ScsiCommandDetailResponse>, StatusCode> {
    let entry = state
        .session_registry
        .find_scsi_log_entry(path.tsih, path.seq)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Session connections handle tape drive commands
    let dt = DeviceType::TapeDrive;
    let cdb_breakdown = decode_cdb(&entry.cdb, dt);
    let response_breakdown = decode_response(&entry);

    Ok(Json(ScsiCommandDetailResponse {
        seq: entry.seq,
        timestamp: entry.timestamp.clone(),
        duration_us: entry.duration_us,
        opcode: entry.opcode,
        opcode_name: entry.opcode_name.clone(),
        cdb_hex: hex_string(&entry.cdb),
        data_out_hex: entry.data_out.as_ref().map(|d| hex_string(d)),
        data_out_len: entry.data_out_len,
        status: entry.status,
        status_name: scsi_status_name(entry.status).to_string(),
        data_in_hex: entry.data_in.as_ref().map(|d| hex_string(d)),
        data_in_len: entry.data_in_len,
        sense_hex: hex_string(&entry.sense),
        cdb_breakdown,
        response_breakdown,
        completed: entry.completed,
    }))
}

// --- OpenAPI ---

#[derive(OpenApi)]
#[openapi(
    info(
        title = "QuantumVTL API",
        description = "REST API for managing the QuantumVTL virtual tape library.",
        version = "0.1.0"
    ),
    paths(
        health,
        login,
        vtl_status,
        vtl_drives,
        vtl_media,
        vtl_snapshot,
        vtl_changer,
        vtl_drive_detail,
        vtl_sessions,
        config_show,
        config_full,
        scsi_log_changer,
        scsi_log_drive,
        scsi_log_entry,
    ),
    components(schemas(
        HealthResponse,
        LoginRequest,
        LoginResponse,
        VtlStatusResponse,
        DriveResponse,
        SlotResponse,
        MediaResponse,
        LibrarySnapshot,
        ChangerDetailResponse,
        RobotOperationResponse,
        ElementDetailResponse,
        DriveDetailResponse,
        SessionResponse,
        ConfigEntry,
        FullConfigResponse,
        ConfigSection,
        ConfigSetting,
        ScsiLogSummaryEntry,
        ScsiLogResponse,
        ScsiCommandDetailResponse,
        iscsi_target::cdb_decode::CdbBreakdown,
        iscsi_target::cdb_decode::CdbField,
        iscsi_target::cdb_decode::DataField,
        iscsi_target::cdb_decode::ResponseBreakdown,
        iscsi_target::cdb_decode::SenseBreakdown,
    )),
    tags(
        (name = "System", description = "Health and system status"),
        (name = "Auth", description = "Authentication"),
        (name = "VTL", description = "Virtual tape library operations"),
        (name = "Config", description = "Configuration store"),
        (name = "SCSI Log", description = "SCSI command/response tracing"),
    )
)]
struct ApiDoc;

async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    // Cache the spec and build it on a thread with sufficient stack space.
    // The utoipa-generated `ApiDoc::openapi()` is deeply recursive and can
    // overflow the default 2 MB tokio worker-thread stack.
    static SPEC: std::sync::OnceLock<utoipa::openapi::OpenApi> = std::sync::OnceLock::new();
    let spec = SPEC.get_or_init(|| {
        std::thread::Builder::new()
            .name("openapi-init".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(ApiDoc::openapi)
            .expect("failed to spawn openapi-init thread")
            .join()
            .expect("openapi-init thread panicked")
    });
    Json(spec.clone())
}

// --- WebSocket ---

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AdminState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AdminState) {
    let mut rx = state.ws_tx.subscribe();
    let _ = socket.send(Message::Text("refresh".into())).await;

    // Debounce: coalesce rapid-fire notifications into one message per interval.
    // Without this, hundreds of SCSI commands/sec flood the browser with WS messages,
    // saturating the TCP receive buffer and causing the browser to hang.
    let debounce = tokio::time::Duration::from_millis(200);
    let mut pending = false;
    let mut timer = std::pin::pin!(tokio::time::sleep(debounce));

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(()) => {
                        if !pending {
                            pending = true;
                            timer.as_mut().reset(tokio::time::Instant::now() + debounce);
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Missed some messages — trigger a refresh anyway
                        pending = true;
                        timer.as_mut().reset(tokio::time::Instant::now() + debounce);
                    }
                    Err(_) => break,
                }
            }
            () = &mut timer, if pending => {
                pending = false;
                if socket.send(Message::Text("refresh".into())).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {}
                    _ => break,
                }
            }
        }
    }
}

// --- Auth middleware ---

async fn auth_middleware(
    State(state): State<AdminState>,
    req: Request,
    next: Next,
) -> Result<Response, Response> {
    if state.users.is_empty() {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(v) if v.starts_with("Bearer ") => &v[7..],
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "missing or invalid authorization header" })),
            )
                .into_response());
        }
    };

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(_) => Ok(next.run(req).await),
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "invalid or expired token" })),
        )
            .into_response()),
    }
}

// --- Static file serving ---

async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if !path.is_empty() {
        if let Some(file) = FrontendAssets::get(path) {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime.as_ref())],
                file.data,
            )
                .into_response();
        }
    }

    // SPA fallback: serve index.html with no-cache so the browser always gets
    // the latest entry point (JS/CSS assets have content hashes and are safe to cache).
    match FrontendAssets::get("index.html") {
        Some(file) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            file.data,
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

// --- Router ---

// --- Simulation Speed ---

#[derive(Serialize, Deserialize)]
struct SimulationSpeedResponse {
    speed_factor: f64,
    label: String,
}

#[derive(Deserialize)]
struct SimulationSpeedRequest {
    speed_factor: f64,
}

fn speed_label(factor: f64) -> String {
    if factor >= 1.0 && (factor - factor.round()).abs() < 0.05 {
        format!("{:.0}x", factor.round())
    } else {
        // Sub-1 values: show enough decimals (0.125x, 0.25x, 0.5x)
        format!("{factor}x")
    }
}

async fn get_simulation_speed(
    State(state): State<AdminState>,
) -> Json<SimulationSpeedResponse> {
    let factor = state.simulation_clock.speed_factor();
    // Clamp to valid range in case the clock was initialised with a legacy value
    let factor = factor.clamp(0.125, 16.0);
    Json(SimulationSpeedResponse {
        speed_factor: factor,
        label: speed_label(factor),
    })
}

async fn set_simulation_speed(
    State(state): State<AdminState>,
    Json(req): Json<SimulationSpeedRequest>,
) -> Json<SimulationSpeedResponse> {
    let factor = req.speed_factor.clamp(0.125, 16.0);
    state.simulation_clock.set_speed_factor(factor);
    let _ = state.ws_tx.send(());
    Json(SimulationSpeedResponse {
        speed_factor: factor,
        label: speed_label(factor),
    })
}

pub fn admin_router(state: AdminState) -> Router {
    let protected = Router::new()
        .route("/api/vtl/status", get(vtl_status))
        .route("/api/vtl/drives", get(vtl_drives))
        .route("/api/vtl/drives/{id}", get(vtl_drive_detail))
        .route("/api/vtl/media", get(vtl_media))
        .route("/api/vtl/media/{barcode}", get(vtl_media_detail))
        .route("/api/vtl/snapshot", get(vtl_snapshot))
        .route("/api/vtl/changer", get(vtl_changer))
        .route("/api/vtl/sessions", get(vtl_sessions))
        .route("/api/config", get(config_show))
        .route("/api/config/full", get(config_full))
        .route("/api/vtl/scsi-log/changer", get(scsi_log_changer))
        .route("/api/vtl/scsi-log/drive/{id}", get(scsi_log_drive))
        .route(
            "/api/vtl/scsi-log/entry/{device_type}/{device_id}/{seq}",
            get(scsi_log_entry),
        )
        .route(
            "/api/vtl/sessions/{tsih}/scsi-log/{seq}",
            get(session_scsi_log_entry),
        )
        .route("/api/vtl/simulation-speed", get(get_simulation_speed).post(set_simulation_speed))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let public = Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/login", post(login))
        .route("/api/openapi.json", get(openapi_spec))
        .route("/api/ws", get(ws_handler));

    Router::new()
        .merge(protected)
        .merge(public)
        .with_state(state)
        .fallback(static_handler)
}

pub async fn run_admin_server(
    addr: &str,
    state: AdminState,
    shutdown: Arc<Notify>,
) -> anyhow::Result<()> {
    let app = admin_router(state);
    let listener = TcpListener::bind(addr).await?;
    info!("admin server listening on {addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown.notified().await;
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prove that generating the OpenAPI spec does not overflow the stack.
    ///
    /// `#[tokio::test]` creates a runtime with the default 2 MB worker-thread
    /// stack.  The handler must cope via the `OnceLock` + dedicated-thread
    /// approach — if it tried to call `ApiDoc::openapi()` inline, this test
    /// would crash with a stack overflow.
    #[tokio::test]
    async fn openapi_spec_does_not_overflow_stack() {
        let resp = openapi_spec().await;
        let json = serde_json::to_value(&resp.0).unwrap();
        assert!(json.get("openapi").is_some(), "missing 'openapi' key");
        assert!(json.get("paths").is_some(), "missing 'paths' key");
        assert!(json.get("components").is_some(), "missing 'components' key");
    }
}
