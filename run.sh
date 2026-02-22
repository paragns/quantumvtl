#!/usr/bin/env bash
set -euo pipefail

DELETE_DATA=false
while getopts "d" opt; do
    case $opt in
        d) DELETE_DATA=true ;;
        *) echo "Usage: $0 [-d]" >&2; exit 1 ;;
    esac
done
shift $((OPTIND - 1))

ROOT="$(cd "$(dirname "$0")" && pwd)"
DATADIR="${TMPDIR:-/tmp}/quantumvtl-dev"

if $DELETE_DATA; then
    echo "==> Deleting existing data..."
    rm -rf "$DATADIR"
fi

# --- Build frontend ---
echo "==> Building frontend..."
(cd "$ROOT/crates/vtld/frontend" && npm install --silent && npm run build)

# --- Build backend ---
echo "==> Building vtld..."
cargo build --release -p vtld

# --- Generate temp config ---
mkdir -p "$DATADIR/data"

MEDIA_YAML=""
for i in $(seq -w 1 40); do
    MEDIA_YAML="${MEDIA_YAML}
    - barcode: QVT${i}L10
      capacity_bytes: 48000000000000"
done

cat > "$DATADIR/config.yaml" <<EOF
listen:
  host: "0.0.0.0"
  admin_port: 8000

store:
  path: "$DATADIR/vtld.redb"

iscsi:
  port: 3260
  iqn: "iqn.2024-01.com.quantumvtl:vtl"

library:
  model: "Quantum Scalar i6"
  serial: "AAAABBBB0001"
  data_dir: "$DATADIR/data"
  drives: 8
  slots: 100
  media:${MEDIA_YAML}

users:
  - username: admin
    password: admin
EOF

echo ""
echo "  iSCSI target:  0.0.0.0:3260"
echo "  Admin UI:      http://127.0.0.1:8000"
echo "  Data dir:      $DATADIR"
echo ""

# --- Run ---
exec "$ROOT/target/release/vtld" "$DATADIR/config.yaml"
