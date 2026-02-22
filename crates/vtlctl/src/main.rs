use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;

/// CLI for the QuantumVTL admin API.
#[derive(Parser)]
#[command(name = "vtlctl")]
struct Cli {
    /// Admin API base URL.
    #[arg(long, default_value = "http://127.0.0.1:8081", global = true)]
    url: String,

    /// Auth token (or set VTLCTL_TOKEN).
    #[arg(long, global = true, env = "VTLCTL_TOKEN")]
    token: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check vtld health.
    Health,
    /// Login and print a JWT token.
    Login {
        /// Username.
        #[arg(long)]
        username: String,
        /// Password.
        #[arg(long)]
        password: String,
    },
    /// Show stored config entries.
    #[command(subcommand)]
    Config(ConfigCmd),
    /// Show library status.
    Status,
    /// Manage drives.
    #[command(subcommand)]
    Drive(DriveCmd),
    /// Manage media.
    #[command(subcommand)]
    Media(MediaCmd),
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Show all config entries.
    Show,
}

#[derive(Subcommand)]
enum DriveCmd {
    /// List drives.
    List,
}

#[derive(Subcommand)]
enum MediaCmd {
    /// List media inventory.
    List,
}

// --- Response types ---

#[derive(Deserialize)]
struct ErrorBody {
    error: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
}

fn check_error(status: StatusCode, body: &str) -> Result<()> {
    if status.is_client_error() || status.is_server_error() {
        if let Ok(err) = serde_json::from_str::<ErrorBody>(body) {
            bail!("{}", err.error);
        }
        bail!("request failed ({}): {}", status, body);
    }
    Ok(())
}

fn auth_header(client: &Client, url: &str, token: &Option<String>) -> reqwest::blocking::RequestBuilder {
    let builder = client.get(url);
    if let Some(t) = token {
        builder.bearer_auth(t)
    } else {
        builder
    }
}

// --- Command handlers ---

fn cmd_health(client: &Client, base: &str) -> Result<()> {
    let resp = client
        .get(format!("{base}/api/health"))
        .send()
        .context("failed to reach admin API")?;
    let body = resp.text()?;
    let v: serde_json::Value = serde_json::from_str(&body)?;
    println!(
        "status: {}  version: {}",
        v["status"].as_str().unwrap_or("unknown"),
        v["version"].as_str().unwrap_or("unknown"),
    );
    Ok(())
}

fn cmd_login(client: &Client, base: &str, username: &str, password: &str) -> Result<()> {
    let resp = client
        .post(format!("{base}/api/auth/login"))
        .json(&serde_json::json!({ "username": username, "password": password }))
        .send()
        .context("failed to reach admin API")?;
    let status = resp.status();
    let body = resp.text()?;
    check_error(status, &body)?;
    let login: LoginResponse = serde_json::from_str(&body)?;
    println!("{}", login.token);
    Ok(())
}

fn cmd_status(client: &Client, base: &str, token: &Option<String>) -> Result<()> {
    let resp = auth_header(client, &format!("{base}/api/vtl/status"), token)
        .send()
        .context("failed to reach admin API")?;
    let status = resp.status();
    let body = resp.text()?;
    check_error(status, &body)?;
    let v: serde_json::Value = serde_json::from_str(&body)?;
    println!("library status: {}", v["status"].as_str().unwrap_or("unknown"));
    Ok(())
}

fn cmd_drive_list(client: &Client, base: &str, token: &Option<String>) -> Result<()> {
    let resp = auth_header(client, &format!("{base}/api/vtl/drives"), token)
        .send()
        .context("failed to reach admin API")?;
    let status = resp.status();
    let body = resp.text()?;
    check_error(status, &body)?;
    let drives: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    if drives.is_empty() {
        println!("No drives.");
        return Ok(());
    }
    println!("{:<4}  STATUS", "ID");
    for d in &drives {
        println!(
            "{:<4}  {}",
            d["id"].as_u64().unwrap_or(0),
            d["status"].as_str().unwrap_or("unknown"),
        );
    }
    Ok(())
}

fn cmd_media_list(client: &Client, base: &str, token: &Option<String>) -> Result<()> {
    let resp = auth_header(client, &format!("{base}/api/vtl/media"), token)
        .send()
        .context("failed to reach admin API")?;
    let status = resp.status();
    let body = resp.text()?;
    check_error(status, &body)?;
    let media: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    if media.is_empty() {
        println!("No media.");
        return Ok(());
    }
    println!("{:<10}  STATUS", "BARCODE");
    for m in &media {
        println!(
            "{:<10}  {}",
            m["barcode"].as_str().unwrap_or("?"),
            m["status"].as_str().unwrap_or("unknown"),
        );
    }
    Ok(())
}

fn cmd_config_show(client: &Client, base: &str, token: &Option<String>) -> Result<()> {
    let resp = auth_header(client, &format!("{base}/api/config"), token)
        .send()
        .context("failed to reach admin API")?;
    let status = resp.status();
    let body = resp.text()?;
    check_error(status, &body)?;
    let entries: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    if entries.is_empty() {
        println!("No config entries.");
        return Ok(());
    }
    let kw = entries
        .iter()
        .map(|e| e["key"].as_str().unwrap_or("").len())
        .max()
        .unwrap()
        .max(3);
    println!("{:<kw$}  VALUE", "KEY");
    for e in &entries {
        println!(
            "{:<kw$}  {}",
            e["key"].as_str().unwrap_or(""),
            e["value"].as_str().unwrap_or(""),
        );
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let base = cli.url.trim_end_matches('/');
    let client = Client::new();

    match cli.command {
        Command::Health => cmd_health(&client, base),
        Command::Login { username, password } => cmd_login(&client, base, &username, &password),
        Command::Status => cmd_status(&client, base, &cli.token),
        Command::Config(sub) => match sub {
            ConfigCmd::Show => cmd_config_show(&client, base, &cli.token),
        },
        Command::Drive(sub) => match sub {
            DriveCmd::List => cmd_drive_list(&client, base, &cli.token),
        },
        Command::Media(sub) => match sub {
            MediaCmd::List => cmd_media_list(&client, base, &cli.token),
        },
    }
}
