use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

fn main() {
    // Re-run if the skip flag changes.
    println!("cargo:rerun-if-env-changed=SKIP_FRONTEND");

    // Watch source files so cargo re-runs this script when they change.
    for path in &[
        "frontend/package.json",
        "frontend/vite.config.ts",
        "frontend/tsconfig.json",
        "frontend/index.html",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    rerun_if_changed_recursive(Path::new("frontend/src"));

    let skip = std::env::var("SKIP_FRONTEND").is_ok();

    if skip {
        if !Path::new("frontend/dist").exists() {
            println!(
                "cargo:warning=SKIP_FRONTEND is set but frontend/dist/ doesn't exist — \
                 rust-embed will fail at compile time"
            );
        }
    } else {
        if needs_rebuild() {
            build_frontend();
        }
    }

    // Always watch dist so rust-embed picks up changes.
    let dist = Path::new("frontend/dist");
    if dist.is_dir() {
        rerun_if_changed_recursive(dist);
    }
    println!("cargo:rerun-if-changed=frontend/dist");
}

fn needs_rebuild() -> bool {
    let dist_marker = Path::new("frontend/dist/index.html");
    if !dist_marker.exists() {
        return true;
    }
    let dist_time = dist_marker
        .metadata()
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let sources = &[
        "frontend/package.json",
        "frontend/vite.config.ts",
        "frontend/tsconfig.json",
        "frontend/index.html",
    ];
    for src in sources {
        let path = Path::new(src);
        if path.exists() {
            if let Ok(t) = path.metadata().and_then(|m| m.modified()) {
                if t > dist_time {
                    return true;
                }
            }
        }
    }

    if let Some(t) = newest_mtime(Path::new("frontend/src")) {
        if t > dist_time {
            return true;
        }
    }

    false
}

fn newest_mtime(dir: &Path) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return None,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(t) = newest_mtime(&path) {
                newest = Some(newest.map_or(t, |cur| cur.max(t)));
            }
        } else if let Ok(t) = path.metadata().and_then(|m| m.modified()) {
            newest = Some(newest.map_or(t, |cur| cur.max(t)));
        }
    }
    newest
}

fn build_frontend() {
    let frontend_dir = Path::new("frontend");

    // Run npm install if node_modules doesn't exist.
    if !frontend_dir.join("node_modules").exists() {
        let status = Command::new("npm")
            .arg("install")
            .current_dir(frontend_dir)
            .status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => panic!("npm install failed with {s}"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                panic!("npm not found — install Node.js or set SKIP_FRONTEND=1 to skip frontend build");
            }
            Err(e) => panic!("failed to run npm install: {e}"),
        }
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(frontend_dir)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => panic!("npm run build failed with {s}"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            panic!("npm not found — install Node.js or set SKIP_FRONTEND=1 to skip frontend build");
        }
        Err(e) => panic!("failed to run npm run build: {e}"),
    }
}

fn rerun_if_changed_recursive(dir: &Path) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rerun_if_changed_recursive(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
