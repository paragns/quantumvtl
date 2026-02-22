#!/bin/bash
# Build the KVM test VM image from the Dockerfile.
# Produces: kernel (vmlinuz), initrd, and rootfs.qcow2
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$REPO_ROOT/target/iscsi-vm"
IMAGE_NAME="quantumvtl-iscsi-test"

mkdir -p "$OUTPUT_DIR"

echo "==> Building Docker image..."
docker build -t "$IMAGE_NAME" -f "$SCRIPT_DIR/Dockerfile.iscsi" "$SCRIPT_DIR"

echo "==> Extracting kernel and initrd..."
CONTAINER_ID=$(docker create "$IMAGE_NAME")
trap 'docker rm -f "$CONTAINER_ID" 2>/dev/null' EXIT

# Find kernel and initrd inside the container
VMLINUZ=$(docker run --rm "$IMAGE_NAME" bash -c 'ls /boot/vmlinuz-* | tail -1')
INITRD=$(docker run --rm "$IMAGE_NAME" bash -c 'ls /boot/initrd.img-* | tail -1')

docker cp "$CONTAINER_ID:$VMLINUZ" "$OUTPUT_DIR/vmlinuz"
docker cp "$CONTAINER_ID:$INITRD" "$OUTPUT_DIR/initrd.img"

echo "==> Creating rootfs..."
# Export container filesystem to a raw image, then convert to qcow2
ROOTFS_RAW="$OUTPUT_DIR/rootfs.raw"
ROOTFS_QCOW2="$OUTPUT_DIR/rootfs.qcow2"

# Create a 2GB raw disk image
truncate -s 2G "$ROOTFS_RAW"
mkfs.ext4 -F -q "$ROOTFS_RAW"

# Mount and populate
MOUNT_DIR=$(mktemp -d)
mount -o loop "$ROOTFS_RAW" "$MOUNT_DIR"

docker export "$CONTAINER_ID" | tar -x -C "$MOUNT_DIR"

# Copy init script to rootfs
cp "$SCRIPT_DIR/init.sh" "$MOUNT_DIR/init.sh"
chmod +x "$MOUNT_DIR/init.sh"

umount "$MOUNT_DIR"
rmdir "$MOUNT_DIR"

# Convert to qcow2
qemu-img convert -f raw -O qcow2 "$ROOTFS_RAW" "$ROOTFS_QCOW2"
rm -f "$ROOTFS_RAW"

echo "==> VM image built successfully:"
echo "    Kernel:  $OUTPUT_DIR/vmlinuz"
echo "    Initrd:  $OUTPUT_DIR/initrd.img"
echo "    Rootfs:  $OUTPUT_DIR/rootfs.qcow2"
