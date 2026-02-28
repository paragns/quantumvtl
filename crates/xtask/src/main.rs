use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

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
        if !run_cmd(
            Command::new("npm")
                .arg("install")
                .current_dir(&frontend_dir),
        ) {
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

/// Build vtld and ensure VM image exists. Returns false on failure.
fn prepare_iscsi_test() -> bool {
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

    true
}

/// Run the iSCSI test with dedup enabled or disabled.
fn run_iscsi_test(dedup: bool) -> bool {
    let repo = repo_root();
    let script = repo.join("tests/iscsi/iscsi_test_wrapper.sh");
    let label = if dedup { "dedup ON" } else { "dedup OFF" };
    eprintln!("Running iSCSI integration test ({label})...");
    run_cmd(
        Command::new("bash")
            .arg(script)
            .env("VTLD_DEDUP", if dedup { "true" } else { "false" })
            .current_dir(&repo),
    )
}

fn test_iscsi() -> bool {
    if !prepare_iscsi_test() {
        return false;
    }

    eprintln!("\n========== Phase 1/2: dedup OFF ==========\n");
    if !run_iscsi_test(false) {
        eprintln!("ERROR: iSCSI test FAILED with dedup OFF");
        return false;
    }

    eprintln!("\n========== Phase 2/2: dedup ON ==========\n");
    if !run_iscsi_test(true) {
        eprintln!("ERROR: iSCSI test FAILED with dedup ON");
        return false;
    }

    eprintln!("\nBoth dedup variants passed.");
    true
}

fn test_iscsi_nodedup() -> bool {
    if !prepare_iscsi_test() {
        return false;
    }
    run_iscsi_test(false)
}

fn test_iscsi_dedup() -> bool {
    if !prepare_iscsi_test() {
        return false;
    }
    run_iscsi_test(true)
}

fn print_usage() {
    eprintln!(
        "\
Usage: cargo xtask <COMMAND>

Commands:
  build-frontend      Build the Vue.js admin frontend
  build-vm            Build the iSCSI test VM image
  test-iscsi          Run iSCSI integration tests with dedup OFF then ON (requires KVM)
  test-iscsi-nodedup  Run iSCSI integration tests with dedup OFF only
  test-iscsi-dedup    Run iSCSI integration tests with dedup ON only"
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
        Some("test-iscsi-nodedup") => {
            if test_iscsi_nodedup() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Some("test-iscsi-dedup") => {
            if test_iscsi_dedup() {
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
