#!/bin/bash
# VM init script (runs as PID 1) — iSCSI initiator test
# DO NOT use set -e here: we are PID 1, any unhandled exit = kernel panic.

export PATH="/usr/sbin:/sbin:/usr/bin:/bin"

# With SLIRP user-mode networking, the host is reachable at the gateway IP.
ISCSI_TARGET_IP="10.0.2.2"
ISCSI_TARGET_PORT="3260"

log() { echo "[init] $*"; }

finish() {
    local code="${1:-1}"
    log "writing exit code $code to /dev/kmsg"
    echo "ISCSI_TEST_EXIT=$code" > /dev/kmsg 2>/dev/null || true
    sync
    sleep 1
    echo o > /proc/sysrq-trigger
    /sbin/poweroff -f 2>/dev/null || true
    while true; do sleep 1; done
}

die() {
    log "FAIL: $*"
    finish 1
}

# Mount essential filesystems (ignore already-mounted errors)
mount -t proc proc /proc 2>/dev/null || true
mount -t sysfs sys /sys 2>/dev/null || true
mount -t devtmpfs dev /dev 2>/dev/null || true
mkdir -p /run /tmp /dev/pts
mount -t tmpfs tmpfs /run 2>/dev/null || true
mount -t tmpfs tmpfs /tmp 2>/dev/null || true
mount -t devpts devpts /dev/pts 2>/dev/null || true

echo 1 > /proc/sys/kernel/sysrq 2>/dev/null || true

# Load modules
modprobe virtio_net 2>/dev/null || true
modprobe iscsi_tcp 2>/dev/null || true
modprobe sg 2>/dev/null || true
modprobe ch 2>/dev/null || true

# Find network interface
NET_IF=""
for iface in eth0 ens3 enp0s3; do
    if ip link show "$iface" >/dev/null 2>&1; then
        NET_IF="$iface"
        break
    fi
done
if [ -z "$NET_IF" ]; then
    NET_IF=$(ip -o link show | awk -F': ' '!/lo/{print $2; exit}')
fi
if [ -z "$NET_IF" ]; then
    log "available interfaces:"
    ip link show 2>&1
    die "no network interface found"
fi

log "configuring network on $NET_IF..."
ip link set lo up
ip link set "$NET_IF" up || die "failed to bring up $NET_IF"

# Use static IP in the SLIRP network (DHCP takes too long to set up)
ip addr add 10.0.2.15/24 dev "$NET_IF" || die "failed to set IP on $NET_IF"
ip route add default via 10.0.2.2 dev "$NET_IF" || die "failed to set default route"

sleep 2

log "network state:"
ip addr show "$NET_IF" 2>&1
ip route show 2>&1

# Verify connectivity
log "pinging target at $ISCSI_TARGET_IP..."
PING_OK=0
for attempt in 1 2 3 4 5; do
    if ping -c 1 -W 2 "$ISCSI_TARGET_IP" >/dev/null 2>&1; then
        PING_OK=1
        break
    fi
    log "ping attempt $attempt failed, retrying..."
    sleep 1
done
if [ "$PING_OK" -ne 1 ]; then
    # SLIRP doesn't respond to ICMP ping — try TCP connect instead
    log "ping failed (SLIRP may not respond to ICMP), trying TCP connect..."
    if timeout 3 bash -c "echo > /dev/tcp/$ISCSI_TARGET_IP/$ISCSI_TARGET_PORT" 2>/dev/null; then
        log "TCP connect to $ISCSI_TARGET_IP:$ISCSI_TARGET_PORT succeeded"
    else
        log "ARP table:"
        cat /proc/net/arp 2>&1
        die "cannot reach iSCSI target at $ISCSI_TARGET_IP:$ISCSI_TARGET_PORT"
    fi
fi
log "target reachable"

# Start iSCSI daemon
log "starting iscsid..."
mkdir -p /etc/iscsi /var/lib/iscsi/ifaces /var/lib/iscsi/isns \
    /var/lib/iscsi/nodes /var/lib/iscsi/send_targets \
    /var/lib/iscsi/slp /var/lib/iscsi/static /var/run/lock/iscsi
echo "InitiatorName=iqn.2024-01.com.quantumvtl:test-initiator" > /etc/iscsi/initiatorname.iscsi

iscsid --no-pid-file &
sleep 2

# Discover targets
log "discovering targets at $ISCSI_TARGET_IP:$ISCSI_TARGET_PORT..."
DISCOVERY_OUTPUT=$(iscsiadm -m discovery -t sendtargets -p "$ISCSI_TARGET_IP:$ISCSI_TARGET_PORT" 2>&1)
DISCOVERY_RC=$?
log "discovery output: $DISCOVERY_OUTPUT (rc=$DISCOVERY_RC)"
if [ "$DISCOVERY_RC" -ne 0 ]; then
    die "iSCSI discovery failed (rc=$DISCOVERY_RC): $DISCOVERY_OUTPUT"
fi

# Login
log "logging in to iSCSI target..."
LOGIN_OUTPUT=$(iscsiadm -m node --login 2>&1)
LOGIN_RC=$?
log "login output: $LOGIN_OUTPUT (rc=$LOGIN_RC)"
if [ "$LOGIN_RC" -ne 0 ]; then
    die "iSCSI login failed (rc=$LOGIN_RC): $LOGIN_OUTPUT"
fi

# Wait for SCSI devices to appear
log "waiting for SCSI devices..."
ATTEMPTS=0
MAX_ATTEMPTS=30
CHANGER_DEV=""
while [ "$ATTEMPTS" -lt "$MAX_ATTEMPTS" ]; do
    LSSCSI_OUT=$(lsscsi -g 2>/dev/null || true)
    CHANGER_DEV=$(echo "$LSSCSI_OUT" | grep "mediumx" | awk '{print $NF}' | head -1)
    if [ -n "$CHANGER_DEV" ]; then
        break
    fi
    sleep 1
    ATTEMPTS=$((ATTEMPTS + 1))
done

if [ -z "$CHANGER_DEV" ]; then
    log "lsscsi output:"
    lsscsi -g 2>/dev/null || true
    die "no medium changer device found after ${MAX_ATTEMPTS}s"
fi

log "found changer device: $CHANGER_DEV"

# Run sg_inq
log "running sg_inq on $CHANGER_DEV..."
SG_INQ_OUTPUT=$(sg_inq "$CHANGER_DEV" 2>&1)
SG_INQ_RC=$?
log "sg_inq output:"
echo "$SG_INQ_OUTPUT"

if [ "$SG_INQ_RC" -ne 0 ]; then
    die "sg_inq failed with rc=$SG_INQ_RC"
fi

# Verify the INQUIRY response contains Quantum
if echo "$SG_INQ_OUTPUT" | grep -qi "QUANTUM"; then
    log "SUCCESS: INQUIRY response contains QUANTUM vendor"
else
    die "INQUIRY response does not contain QUANTUM vendor"
fi

# Logout
log "logging out..."
iscsiadm -m node --logout 2>/dev/null || true

log "test passed!"
finish 0
