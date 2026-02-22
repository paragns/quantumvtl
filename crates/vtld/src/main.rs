use std::sync::Arc;

use rand::Rng;
use tokio::sync::{Notify, broadcast};
use tracing::{error, info};

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

    let admin_state = AdminState {
        store,
        users: config.users,
        jwt_secret,
        ws_tx,
        version: VERSION,
    };

    let admin_addr = format!("{}:{}", config.listen.host, config.listen.admin_port);
    if let Err(e) = run_admin_server(&admin_addr, admin_state, shutdown).await {
        error!("admin server error: {e}");
    }

    info!("server stopped");
    Ok(())
}
