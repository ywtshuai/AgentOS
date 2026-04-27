# Agent-OS

Agent-OS is an rCore-Tutorial v3 based teaching OS extension for the AI Agent OS competition. The implementation target is RISC-V 64 on QEMU, with deterministic mock user agents instead of any real external LLM API.

## Repository Layout

- `rcore/`: imported rCore-Tutorial v3 `ch6` baseline. This gives us processes, syscalls, virtual memory, and a simple file system for later Agent-OS milestones.
- `Docs/`: project plan, competition requirements, and per-milestone deliverables.
- `scripts/check-env.ps1`: local toolchain checker for Rust, Make, QEMU, and RISC-V utilities.

## Environment Check

On Windows PowerShell:

```powershell
.\scripts\check-env.ps1
```

Required tools:

- Rust toolchain with `rustup`, `cargo`, and `rustc`
- `riscv64gc-unknown-none-elf` target
- `cargo-binutils`, `rust-src`, `llvm-tools-preview`
- `make`
- `qemu-system-riscv64`

## Build And Run

After installing the toolchain:

```powershell
cd rcore\os
make build
make run
```

The current M0 baseline is a clean rCore import plus environment validation. Agent-specific syscalls and user demos start in M1.
