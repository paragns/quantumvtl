use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Request, State};
use axum::http::{StatusCode, Uri, header};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Json, Response};
use axum::routing::{get, post};
use axum::Router;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::{Notify, broadcast};
use tracing::info;
use utoipa::OpenApi;

use smc::{ElementType, MediaChanger};
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
        config_show,
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
        ConfigEntry,
    )),
    tags(
        (name = "System", description = "Health and system status"),
        (name = "Auth", description = "Authentication"),
        (name = "VTL", description = "Virtual tape library operations"),
        (name = "Config", description = "Configuration store"),
    )
)]
struct ApiDoc;

async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

// --- WebSocket ---

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AdminState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AdminState) {
    let mut rx = state.ws_tx.subscribe();
    let _ = socket.send(Message::Text("refresh".into())).await;
    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(()) => {
                        if socket.send(Message::Text("refresh".into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
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

    match FrontendAssets::get("index.html") {
        Some(file) => Html(file.data).into_response(),
        None => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

// --- Router ---

pub fn admin_router(state: AdminState) -> Router {
    let protected = Router::new()
        .route("/api/vtl/status", get(vtl_status))
        .route("/api/vtl/drives", get(vtl_drives))
        .route("/api/vtl/media", get(vtl_media))
        .route("/api/vtl/snapshot", get(vtl_snapshot))
        .route("/api/config", get(config_show))
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
