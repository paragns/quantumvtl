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

fn build_vm() -> bool {
    let repo = repo_root();
    let script = repo.join("tests/iscsi/build_vm_image.sh");

    eprintln!("Building iSCSI test VM image...");
    run_cmd(Command::new("bash").arg(script).current_dir(&repo))
}

fn test_iscsi() -> bool {
    let repo = repo_root();

    // Build vtld in release mode first
    eprintln!("Building vtld (release)...");
    if !run_cmd(
        Command::new("cargo")
            .args(["build", "--release", "-p", "vtld"])
            .current_dir(&repo),
    ) {
        eprintln!("ERROR: failed to build vtld");
        return false;
    }

    // Check if VM image exists
    let vm_dir = repo.join("target/iscsi-vm");
    if !vm_dir.join("vmlinuz").exists() {
        eprintln!("VM image not found. Building...");
        if !build_vm() {
            return false;
        }
    }

    let script = repo.join("tests/iscsi/iscsi_test_wrapper.sh");
    eprintln!("Running iSCSI integration test...");
    run_cmd(Command::new("bash").arg(script).current_dir(&repo))
}

fn print_usage() {
    eprintln!(
        "\
Usage: cargo xtask <COMMAND>

Commands:
  build-frontend    Build the Vue.js admin frontend
  build-vm          Build the iSCSI test VM image
  test-iscsi        Run iSCSI integration tests (requires KVM)"
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
        Some("build-vm") => {
            if build_vm() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Some("test-iscsi") => {
            if test_iscsi() {
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
