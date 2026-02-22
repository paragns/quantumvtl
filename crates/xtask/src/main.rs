use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::env;

fn repo_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn run_cmd(cmd: &mut Command) -> bool {
    eprintln!("+ {:?}", cmd);
    match cmd.status() {
        Ok(s) => s.success(),
        Err(e) => {
            eprintln!("failed to run {:?}: {e}", cmd);
            false
        }
    }
}

fn build_frontend() -> bool {
    let repo = repo_root();
    let frontend_dir = repo.join("crates/vtld/frontend");

    if !frontend_dir.join("node_modules").exists() {
        eprintln!("Installing frontend dependencies...");
        if !run_cmd(Command::new("npm").arg("install").current_dir(&frontend_dir)) {
            eprintln!("ERROR: npm install failed");
            return false;
        }
    }

    eprintln!("Building frontend...");
    if !run_cmd(
        Command::new("npm")
            .args(["run", "build"])
            .current_dir(&frontend_dir),
    ) {
        eprintln!("ERROR: frontend build failed");
        return false;
    }

    eprintln!("Frontend built successfully");
    true
}

fn print_usage() {
    eprintln!(
        "\
Usage: cargo xtask <COMMAND>

Commands:
  build-frontend    Build the Vue.js admin frontend"
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        Some("build-frontend") => {
            if build_frontend() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        _ => {
            print_usage();
            if args.first().map(String::as_str) == Some("--help") {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
