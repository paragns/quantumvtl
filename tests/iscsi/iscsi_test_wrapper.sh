#!/bin/bash
# Run the iSCSI integration test:
# 1. Create a network namespace with TAP device
# 2. Start vtld inside the namespace
# 3. Boot a QEMU VM inside the namespace
# 4. Parse serial output for pass/fail
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VM_DIR="$REPO_ROOT/target/iscsi-vm"
VTLD_BIN="$REPO_ROOT/target/release/vtld"
SERIAL_LOG="$VM_DIR/serial.log"
TEST_CONFIG="$SCRIPT_DIR/test_config.yaml"

NS_NAME="vtl-test-$$"
VTLD_DATA_DIR=""

cleanup() {
    echo "==> Cleaning up..."
    if [ -n "${VTLD_PID:-}" ]; then
        kill "$VTLD_PID" 2>/dev/null || true
        wait "$VTLD_PID" 2>/dev/null || true
    fi
    ip netns del "$NS_NAME" 2>/dev/null || true
    if [ -n "$VTLD_DATA_DIR" ] && [ -d "$VTLD_DATA_DIR" ]; then
        rm -rf "$VTLD_DATA_DIR"
    fi
}
trap cleanup EXIT

# Verify prerequisites
for f in "$VM_DIR/vmlinuz" "$VM_DIR/initrd.img" "$VM_DIR/rootfs.qcow2"; do
    if [ ! -f "$f" ]; then
        echo "ERROR: $f not found. Run 'cargo xtask build-vm' first."
        exit 1
    fi
done

if [ ! -f "$VTLD_BIN" ]; then
    echo "ERROR: $VTLD_BIN not found. Run 'cargo build --release' first."
    exit 1
fi

echo "==> Creating network namespace: $NS_NAME"
ip netns add "$NS_NAME"

# Create a veth pair: one end in default NS, one in test NS.
# host-side gets 10.0.0.1, VM-side is the QEMU TAP.
# Actually, simplest reliable approach: veth pair acts as a point-to-point
# link between the namespace's local stack and QEMU.
#
# However, QEMU needs a TAP device. We'll use a veth pair where:
#   - veth-host (in NS) gets IP 10.0.0.1/24 — vtld binds here
#   - veth-vm (in NS) is used as the QEMU NIC via macvtap... nope.
#
# Simplest approach: use QEMU user-mode networking (SLIRP).
# The VM reaches the host (namespace local stack) via the SLIRP gateway.

# For SLIRP: the VM gets 10.0.2.0/24 with gateway at 10.0.2.2.
# SLIRP proxies TCP connections from guest to the host. But it proxies
# to the host's actual network, not the namespace. Since QEMU runs IN
# the namespace, SLIRP will connect from the namespace context.
# So vtld listening on 127.0.0.1:3260 in the namespace will be reachable
# from the VM at 10.0.2.2:3260.

ip netns exec "$NS_NAME" ip link set lo up

echo "==> Starting vtld in network namespace..."
VTLD_DATA_DIR=$(mktemp -d)
mkdir -p "$VTLD_DATA_DIR/data"

# Create a test config that binds to loopback (SLIRP will reach it)
TEST_CONFIG_TMP="$VM_DIR/test_config_run.yaml"
cat > "$TEST_CONFIG_TMP" <<EOF
listen:
  host: "0.0.0.0"
  admin_port: 8081

store:
  path: "$VTLD_DATA_DIR/vtld.redb"

iscsi:
  port: 3260
  iqn: "iqn.2024-01.com.quantumvtl:vtl"

library:
  model: "Scalar i6"
  serial: "AAAABBBB0001"
  data_dir: "$VTLD_DATA_DIR/data"
  drives: 2
  slots: 8
  media:
    - barcode: "TEST01L9"
      capacity_bytes: 18000000000000
    - barcode: "TEST02L9"
      capacity_bytes: 18000000000000
    - barcode: "TEST03L9"
      capacity_bytes: 18000000000000
    - barcode: "TEST04L9"
      capacity_bytes: 18000000000000

users:
  - username: admin
    password: admin
EOF

ip netns exec "$NS_NAME" "$VTLD_BIN" "$TEST_CONFIG_TMP" &
VTLD_PID=$!
sleep 2

if ! kill -0 "$VTLD_PID" 2>/dev/null; then
    echo "ERROR: vtld failed to start"
    exit 1
fi
echo "==> vtld is running (pid=$VTLD_PID)"

# Verify vtld is listening
ip netns exec "$NS_NAME" bash -c "echo | timeout 2 bash -c 'cat < /dev/tcp/127.0.0.1/3260' 2>/dev/null" && echo "==> iSCSI port verified" || echo "==> WARNING: could not verify iSCSI port"

echo "==> Booting QEMU VM in namespace..."
OVERLAY="$VM_DIR/overlay.qcow2"
qemu-img create -f qcow2 -b "$VM_DIR/rootfs.qcow2" -F qcow2 "$OVERLAY" 2>/dev/null
rm -f "$SERIAL_LOG"

# Use SLIRP user-mode networking. VM reaches host at 10.0.2.2.
timeout 180 ip netns exec "$NS_NAME" qemu-system-x86_64 \
    -enable-kvm \
    -m 512 \
    -nographic \
    -no-reboot \
    -kernel "$VM_DIR/vmlinuz" \
    -initrd "$VM_DIR/initrd.img" \
    -append "root=/dev/vda rw console=ttyS0 init=/init.sh panic=1 net.ifnames=0 biosdevname=0" \
    -drive "file=$OVERLAY,format=qcow2,if=virtio" \
    -netdev "user,id=net0,net=10.0.2.0/24,host=10.0.2.2,restrict=off" \
    -device "virtio-net-pci,netdev=net0" \
    -serial "file:$SERIAL_LOG" \
    -monitor none \
    2>&1 || true

rm -f "$OVERLAY"

echo "==> Checking test result..."
if [ ! -f "$SERIAL_LOG" ]; then
    echo "ERROR: serial log not found"
    exit 1
fi

echo "--- Init output ---"
grep "^\[init\]" "$SERIAL_LOG" || true
echo "---"

if grep -q "ISCSI_TEST_EXIT=0" "$SERIAL_LOG"; then
    echo "==> TEST PASSED"
    exit 0
else
    echo "==> TEST FAILED"
    echo "--- Last 40 lines of serial log ---"
    tail -40 "$SERIAL_LOG"
    echo "---"
    exit 1
fi
