# QuantumVTL

Pure Rust iSCSI virtual tape library (VTL) that emulates Quantum-branded LTO tape
libraries/robots and IBM LTO tape drives.

## Project Overview

QuantumVTL is an iSCSI target that presents virtual tape library hardware to
initiators over the network. It implements:

- **SMC (SCSI Media Changer)**: Emulates modern Quantum tape library robots
- **SSC (SCSI Stream Commands)**: Emulates IBM LTO tape drives (Ultrium family)
- **iSCSI target**: Native Rust iSCSI target implementation (no kernel dependencies)

### Storage Architecture

- **State**: `redb` embedded key-value store for library state, media inventory,
  drive status, and element maps
- **Media data**: Flat `.data` files named by media barcode (e.g., `ABC001L9.data`),
  one file per virtual tape cartridge

### Primary Consumer

The sister project **tape-manager** is a lightweight S3-compatible object store that
persists data to tape. It is developing a Rust-native iSCSI initiator to connect to
this VTL. QuantumVTL replaces the current mhvtl-in-KVM approach tape-manager uses
for virtual tape testing.

## Reference Materials (`ref/` directory)

The `ref/` directory is in `.gitignore` — it is reference material only, not part of
this project's source.

| Path | Description |
|------|-------------|
| `ref/tapemanager/` | Full tape-manager source. Borrow build/test infrastructure patterns, especially xtask, KVM test wrapper, and the virtual tape file format. Its `redb_store.rs` is a good model for our state store. |
| `ref/tapemanager/crates/quantum-tape-sys/` | XDI FFI bindings to Quantum's Storage Manager changer control library. The `c/xdi/` subtree has well-tested client-side code for interacting with Quantum libraries — useful as a reference for what commands/responses a Quantum library must handle. |
| `ref/tapemanager/crates/quantum-tape/` | Safe Rust wrappers over quantum-tape-sys. Shows the Rust API surface for changer and drive operations. |
| `ref/mhvtl/` | mhvtl — existing open-source VTL in C. Key references: `usr/ssc.c` (tape SCSI commands), `usr/smc.c` (changer SCSI commands), `usr/spc.c` (common SCSI), personality modules (`*_pm.c`), and `usr/vtllib.h` / `usr/vtltape.h` for data structures. |
| `ref/scst/` | SCST (SCSI target framework) source including `iscsi-scst/` — reference for iSCSI target protocol implementation. |
| `ref/lto-ref.pdf` | IBM LTO SCSI specification. Definitive reference for tape drive command behavior, mode pages, sense data, and media handling. |

## Testing Strategy

### KVM Integration Testing

We use KVM in the **inverse** direction from tape-manager: QuantumVTL runs on the
host as the iSCSI target, and a Linux guest VM acts as an iSCSI initiator. This
validates compatibility with standard Linux tape tools:

- `mtx` — media changer control (move/load/unload/status)
- `mt` — tape drive control (fsf, bsf, rewind, status, etc.)
- Standard `open-iscsi` initiator in the guest

This ensures QuantumVTL is standards-compliant enough for real-world use from day one.

### Build Infrastructure

Borrow patterns from tape-manager's `xtask` crate and KVM test wrappers
(`ref/tapemanager/ref/chimera/kvm/`) for automated integration testing.

## Key Design Decisions

- **Pure Rust**: No kernel modules, no C dependencies. The iSCSI target, SCSI
  command processing, and all emulation logic are implemented in Rust.
- **Async (Tokio)**: Following tape-manager's async-throughout pattern.
- **redb for state**: Embedded, crash-safe, single-file database. No external
  database dependency.
- **File-per-tape storage**: Simple, debuggable, backup-friendly. Each virtual
  cartridge is one file named by its barcode.

## Device Emulation Targets

- **Library/Robot**: Modern Quantum Scalar series (SMC-3 compliant media changer)
- **Drives**: IBM Ultrium LTO family (SSC-3/SSC-4 compliant, LTO-7 through LTO-9)

## Development Environment

- Devcontainer based on Ubuntu 24.04
- Rust toolchain via rustup
- QEMU/KVM available for integration testing
- Privileged container for KVM access
