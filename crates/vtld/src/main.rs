use std::sync::Arc;

use rand::Rng;
use tokio::sync::{Notify, broadcast};
use tracing::{error, info};

use iscsi_target::SessionRegistry;
use iscsi_target::scsi_log::{DeviceType, TracedDevice};
use iscsi_target::target::{Target, TargetServer};
use smc::MediaChanger;
use ssc::TapeDrive;
use ssc::media::geometry::LtoGeneration;
use vtld::admin::{AdminState, run_admin_server};
use vtld::config::load_config;
use vtld::store::Store;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config_path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("VTLD_CONFIG").ok())
        .expect("usage: vtld <config.yaml> (or set VTLD_CONFIG)");

    let config = load_config(&config_path)?;

    // Initialize store
    if let Some(parent) = std::path::Path::new(&config.store.path).parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    let store = Store::new(&config.store.path)?;
    info!("opened store at {}", config.store.path);

    info!(
        "library model={} serial={} drives={} slots={} media={}",
        config.library.model,
        config.library.serial,
        config.library.drives,
        config.library.slots,
        config.library.media.len(),
    );

    // Shared shutdown signal
    let shutdown = Arc::new(Notify::new());
    {
        let shutdown = shutdown.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("shutting down...");
            shutdown.notify_waiters();
        });
    }

    // WebSocket broadcast channel
    let (ws_tx, _) = broadcast::channel::<()>(16);

    // JWT secret
    let jwt_secret: String = {
        let mut rng = rand::thread_rng();
        (0..64)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"[idx] as char
            })
            .collect()
    };

    // Start iSCSI target
    let media_barcodes: Vec<String> = config.library.media.iter().map(|m| m.barcode.clone()).collect();

    // Create tape drives and collect notification handles for the changer
    let data_dir = std::path::PathBuf::from(&config.library.data_dir);
    let mut drive_notifiers: Vec<Arc<dyn iscsi_target::MediaLoadNotify>> = Vec::new();
    let mut drive_arcs: Vec<Arc<TapeDrive>> = Vec::new();
    for i in 0..config.library.drives {
        let serial = format!("DRIVE{:03}", i);
        let drive = Arc::new(TapeDrive::new(&serial, LtoGeneration::Lto9, data_dir.clone()));
        drive_notifiers.push(drive.clone());
        drive_arcs.push(drive);
    }

    let changer = Arc::new(MediaChanger::new(
        &config.library.model,
        &config.library.serial,
        config.library.drives as u16,
        config.library.slots as u16,
        &media_barcodes,
        drive_notifiers,
    ));

    let session_registry = Arc::new(SessionRegistry::new());

    // Wrap devices with tracing
    let (traced_changer, changer_log) = TracedDevice::new(
        changer.clone(),
        DeviceType::MediaChanger,
        20,
        Some(ws_tx.clone()),
    );
    let traced_changer = Arc::new(traced_changer);

    let mut drive_logs = Vec::new();
    let mut traced_drives: Vec<Arc<TracedDevice>> = Vec::new();
    for drive in &drive_arcs {
        let (traced, log) = TracedDevice::new(
            drive.clone(),
            DeviceType::TapeDrive,
            20,
            Some(ws_tx.clone()),
        );
        drive_logs.push(log);
        traced_drives.push(Arc::new(traced));
    }

    let admin_state = AdminState {
        store,
        users: config.users,
        jwt_secret,
        ws_tx,
        version: VERSION,
        changer: changer.clone(),
        drives: drive_arcs.clone(),
        session_registry: session_registry.clone(),
        changer_log,
        drive_logs,
    };

    let mut iscsi_target = Target::new(config.iscsi.iqn.clone());
    iscsi_target.add_lun(0, traced_changer);

    for (i, traced) in traced_drives.into_iter().enumerate() {
        iscsi_target.add_lun((i + 1) as u64, traced);
    }

    let iscsi_addr = format!("{}:{}", config.listen.host, config.iscsi.port);
    let iscsi_server = TargetServer::new(iscsi_target, session_registry);
    let iscsi_shutdown = shutdown.clone();
    tokio::spawn(async move {
        if let Err(e) = iscsi_server.run(&iscsi_addr, iscsi_shutdown).await {
            error!("iSCSI target error: {e}");
        }
    });

    let admin_addr = format!("{}:{}", config.listen.host, config.listen.admin_port);
    if let Err(e) = run_admin_server(&admin_addr, admin_state, shutdown).await {
        error!("admin server error: {e}");
    }

    info!("server stopped");
    Ok(())
}
