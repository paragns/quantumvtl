#!/bin/bash
# setup_quantumvtl.sh — Clone, build, configure, and start QuantumVTL on RHEL/Rocky 8+
#
# Configuration:
#   - 1 library (Quantum Scalar i6)
#   - 4 drives (IBM ULT3580-TD9 / LTO-9)
#   - 100 tapes at 50 MB each (barcodes QVT001L9 – QVT100L9)
#
# Usage:
#   bash setup_quantumvtl.sh [--tape-size-mb N] [--tape-count N]
#   e.g.  bash setup_quantumvtl.sh --tape-size-mb 100 --tape-count 50
#
# Run as root on a clean RHEL/Rocky 8+ VM.

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────
REPO_URL="https://github.com/paragns/quantumvtl.git"
CLONE_DIR="/root/quantumvtl"
DATA_DIR="/opt/quantumvtl/data"
CONF_DIR="/etc/quantumvtl"
BIN_DIR="/usr/local/bin"

TAPE_COUNT=100
TAPE_SIZE_MB=50
TAPE_PREFIX="QVT"
TAPE_SUFFIX="L9"
VTL_DRIVES=4
VTL_SLOTS=110
LIBRARY_MODEL="Quantum Scalar i6"
LIBRARY_SERIAL="AAAABBBB0001"
ISCSI_IQN="iqn.2024-01.com.quantumvtl:vtl"
ADMIN_PORT=8081
ISCSI_PORT=3260
ADMIN_USER="admin"
ADMIN_PASS="admin"

# ── Parse optional overrides ──────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --tape-size-mb)  TAPE_SIZE_MB="$2";  shift 2 ;;
        --tape-count)    TAPE_COUNT="$2";    shift 2 ;;
        --repo-url)      REPO_URL="$2";      shift 2 ;;
        *) echo "Unknown argument: $1" >&2; exit 1 ;;
    esac
done

TAPE_SIZE_BYTES=$(( TAPE_SIZE_MB * 1024 * 1024 ))
VTL_SLOTS=$(( TAPE_COUNT + 10 ))

log() { echo "[$(date '+%H:%M:%S')] $*"; }

# ── 1. Root check ─────────────────────────────────────────────────────────────
if [[ $EUID -ne 0 ]]; then
    echo "ERROR: Run as root." >&2
    exit 1
fi

log "Starting QuantumVTL setup on $(hostname) — $(uname -r)"
log "Config: ${TAPE_COUNT} tapes × ${TAPE_SIZE_MB} MB, ${VTL_DRIVES} drives"

# ── 2. OS packages ────────────────────────────────────────────────────────────
log "Installing OS packages..."
dnf install -y git gcc make pkg-config openssl-devel iscsi-initiator-utils

# Node.js 20 LTS from NodeSource
if ! command -v node &>/dev/null; then
    log "Installing Node.js..."
    curl -fsSL https://rpm.nodesource.com/setup_20.x | bash -
    dnf install -y nodejs
fi

# ── 3. Rust toolchain ─────────────────────────────────────────────────────────
if ! command -v cargo &>/dev/null; then
    log "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --no-modify-path
    source /root/.cargo/env
else
    source /root/.cargo/env 2>/dev/null || true
fi

export PATH="/root/.cargo/bin:$PATH"
rustc --version
cargo --version

# ── 4. Clone repository ───────────────────────────────────────────────────────
log "Cloning QuantumVTL from $REPO_URL..."
if [[ -d "$CLONE_DIR/.git" ]]; then
    log "Already cloned — pulling latest..."
    git -C "$CLONE_DIR" pull
else
    git clone "$REPO_URL" "$CLONE_DIR"
fi

log "Initialising git submodules (iscsi-target)..."
git -C "$CLONE_DIR" submodule update --init --recursive

# ── 5. Build ──────────────────────────────────────────────────────────────────
log "Building vtld and vtlctl (this takes a few minutes)..."
cd "$CLONE_DIR"
cargo build --release -p vtld -p vtlctl

# ── 6. Install binaries ───────────────────────────────────────────────────────
log "Installing binaries to $BIN_DIR..."
install -m 755 target/release/vtld   "$BIN_DIR/vtld"
install -m 755 target/release/vtlctl "$BIN_DIR/vtlctl"

# ── 7. Write config.yaml ──────────────────────────────────────────────────────
log "Writing config to $CONF_DIR/config.yaml..."
mkdir -p "$CONF_DIR" "$DATA_DIR"

{
    cat <<EOF
listen:
  host: "0.0.0.0"
  admin_port: ${ADMIN_PORT}

store:
  path: "/opt/quantumvtl/vtld.redb"

iscsi:
  port: ${ISCSI_PORT}
  iqn: "${ISCSI_IQN}"

library:
  model: "${LIBRARY_MODEL}"
  serial: "${LIBRARY_SERIAL}"
  data_dir: "${DATA_DIR}"
  drives: ${VTL_DRIVES}
  slots: ${VTL_SLOTS}
  media:
EOF

    for i in $(seq -w 1 "$TAPE_COUNT"); do
        echo "    - barcode: \"${TAPE_PREFIX}$(printf '%03d' $((10#$i)))${TAPE_SUFFIX}\""
        echo "      capacity_bytes: ${TAPE_SIZE_BYTES}"
    done

    cat <<EOF

users:
  - username: ${ADMIN_USER}
    password: ${ADMIN_PASS}

simulation_speed: 1000.0
EOF
} > "$CONF_DIR/config.yaml"

log "Config written: $TAPE_COUNT tapes × ${TAPE_SIZE_BYTES} bytes (${TAPE_SIZE_MB} MB)"

# ── 8. Systemd service ────────────────────────────────────────────────────────
log "Creating systemd service..."
cat > /etc/systemd/system/quantumvtl.service <<'UNIT'
[Unit]
Description=QuantumVTL iSCSI Virtual Tape Library
After=network.target
Wants=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/vtld /etc/quantumvtl/config.yaml
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
UNIT

systemctl daemon-reload
systemctl enable --now quantumvtl.service

# ── 9. Wait for health ────────────────────────────────────────────────────────
log "Waiting for vtld to start..."
for i in $(seq 1 20); do
    if curl -sf "http://127.0.0.1:${ADMIN_PORT}/api/health" &>/dev/null; then
        break
    fi
    sleep 1
done

HEALTH=$(curl -sf "http://127.0.0.1:${ADMIN_PORT}/api/health" || echo '{"status":"unreachable"}')
log "Health: $HEALTH"

# ── 10. Verify ────────────────────────────────────────────────────────────────
log ""
log "=== QuantumVTL Status ==="
systemctl status --no-pager quantumvtl.service | head -10
echo ""
log "=== Library ==="
log "  Admin UI:      http://$(hostname -I | awk '{print $1}'):${ADMIN_PORT}"
log "  iSCSI target:  $(hostname -I | awk '{print $1}'):${ISCSI_PORT}"
log "  IQN:           ${ISCSI_IQN}"
log "  Drives:        ${VTL_DRIVES}"
log "  Tapes:         ${TAPE_COUNT} × ${TAPE_SIZE_MB} MB"
log ""
log "=== Connect iSCSI initiator (from StorNext or test host) ==="
log "  iscsiadm -m discovery -t st -p $(hostname -I | awk '{print $1}')"
log "  iscsiadm -m node --login"
log "  lsscsi -g     # verify tape devices appear"
log ""
log "Setup complete."
