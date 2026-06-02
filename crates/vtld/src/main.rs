use std::sync::Arc;

use rand::Rng;
use tokio::sync::{broadcast, Notify};
use tracing::{error, info};

use iscsi_target::SimulationClock;
use iscsi_target::scsi_log::{DeviceType, TracedDevice};
use iscsi_target::target::{Target, TargetServer};
use iscsi_target::SessionRegistry;
use smc::MediaChanger;
use smc::timing::RobotTimingModel;
use ssc::TapeDrive;
use ssc::media::dedup::DedupPool;
use ssc::media::geometry::LtoGeneration;
use vtld::admin::{AdminState, ConfigSnapshot, run_admin_server};
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
    let media_barcodes: Vec<String> = config
        .library
        .media
        .iter()
        .map(|m| m.barcode.clone())
        .collect();

    // Create tape drives and collect notification handles for the changer
    let data_dir = std::path::PathBuf::from(&config.library.data_dir);
    // Simulation clock: configurable speed, default realistic (1:1 timing) — adjustable via API
    let simulation_clock = Arc::new(SimulationClock::new(config.simulation_speed));
    info!("simulation speed: {}x", config.simulation_speed);

    // Create shared dedup store if enabled
    let dedup_store = if config.library.dedup {
        info!(
            shards = config.library.dedup_shards,
            cache_mb = config.library.dedup_cache_bytes / 1_048_576,
            flush_workers = config.library.dedup_flush_workers,
            "dedup enabled, opening dedup pool"
        );
        Some(Arc::new(
            DedupPool::open(
                &data_dir,
                config.library.dedup_shards,
                config.library.dedup_cache_bytes,
                config.library.dedup_flush_workers,
            )
            .expect("failed to open dedup pool"),
        ))
    } else {
        None
    };

    let capacity_overrides: std::sync::Arc<std::collections::HashMap<String, u64>> =
        std::sync::Arc::new(
            config
                .library
                .media
                .iter()
                .filter_map(|m| m.capacity_bytes.map(|c| (m.barcode.clone(), c)))
                .collect(),
        );

    let mut drive_notifiers: Vec<Arc<dyn iscsi_target::MediaLoadNotify>> = Vec::new();
    let mut drive_arcs: Vec<Arc<TapeDrive>> = Vec::new();
    let mut drive_serials: Vec<String> = Vec::new();
    for i in 0..config.library.drives {
        let serial = format!("DRIVE{:03}", i);
        let drive = Arc::new(TapeDrive::new(
            &serial,
            LtoGeneration::Lto9,
            data_dir.clone(),
            simulation_clock.clone(),
            dedup_store.clone(),
            capacity_overrides.clone(),
        ));
        drive_notifiers.push(drive.clone());
        drive_arcs.push(drive);
        drive_serials.push(serial);
    }

    // Spawn background buffer ticker per drive (4 Hz)
    for drive in &drive_arcs {
        let drive = drive.clone();
        let ws_tx = ws_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));
            loop {
                interval.tick().await;
                if drive.tick_buffer() {
                    let _ = ws_tx.send(());
                }
            }
        });
    }

    let changer = Arc::new(MediaChanger::new(
        &config.library.model,
        &config.library.serial,
        config.library.drives as u16,
        config.library.slots as u16,
        &media_barcodes,
        drive_notifiers,
        drive_serials,
        RobotTimingModel::scalar_i6(),
        simulation_clock.clone(),
        Some(ws_tx.clone()),
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

    let config_snapshot = ConfigSnapshot {
        listen_host: config.listen.host.clone(),
        listen_admin_port: config.listen.admin_port,
        store_path: config.store.path.clone(),
        iscsi_port: config.iscsi.port,
        iscsi_iqn: config.iscsi.iqn.clone(),
        library_model: config.library.model.clone(),
        library_serial: config.library.serial.clone(),
        library_data_dir: config.library.data_dir.clone(),
        library_drives: config.library.drives,
        library_slots: config.library.slots,
        library_media_count: config.library.media.len(),
        library_media_barcodes: config.library.media.iter().map(|m| m.barcode.clone()).collect(),
        user_count: config.users.len(),
        initial_simulation_speed: config.simulation_speed,
    };

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
        data_dir: data_dir.clone(),
        simulation_clock,
        config_snapshot,
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
