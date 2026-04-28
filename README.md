# Agent-OS

Agent-OS is an rCore-Tutorial v3 based teaching OS extension for the AI Agent OS competition. It treats AI agents as first-class OS processes and adds kernel support for Agent Context memory, structured tool calls, Context Path management, heartbeat/message wakeups, and attribute-based file queries.

The project runs on RISC-V 64 QEMU. User agents are deterministic demo programs that simulate an Agent Loop; no external LLM API is required.

## Repository Layout

- `rcore/`: rCore-Tutorial v3 based kernel, user library, and demo programs.
- `Docs/PLAN.md`: milestone plan and execution protocol.
- `Docs/CompetitionRequirements.md`: original competition task description.
- `Docs/M0` to `Docs/M8`: per-milestone design, implementation log, and test report.
- `Docs/design.md`: overall Agent-OS architecture.
- `Docs/evaluation.md`: functional and performance evaluation.
- `scripts/check-env.ps1`: Windows PowerShell environment checker.

## Environment

Required tools:

- Rust toolchain with `rustup`, `cargo`, and `rustc`.
- `riscv64gc-unknown-none-elf` Rust target.
- `cargo-binutils`, `rust-src`, `llvm-tools-preview`.
- `make`.
- `qemu-system-riscv64`.

On Windows PowerShell:

```powershell
.\scripts\check-env.ps1
```

The verified build path in this repository uses stable Rust with `RUSTC_BOOTSTRAP=1`, matching the upstream rCore nightly feature usage.

## Build

```bash
cd rcore/os
cargo +stable fmt
RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer' cargo +stable check
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer -Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes' make kernel
rustup run stable rust-objcopy target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 make fs-img
```

## Run

```bash
cd rcore/os
make run
```

Useful usershell commands:

```text
agent_m1
agent_m2
agent_m3
agent_m4
agent_m5
agent_m6
agent_demo basic
agent_demo loop
agent_demo fs_query_bench
agent_demo full
```

`agent_demo full` is the main integrated scenario. It demonstrates an Admin-Agent that wakes on heartbeat, queries system state, queries files by attributes, sends a message to a Worker-Agent, and prints the resulting Context Path.

## Agent-OS Syscalls

| ID | Syscall | Purpose |
| --- | --- | --- |
| 500 | `sys_agent_create` | Mark the current process as an Agent and map a 64KB Agent Context region. |
| 501 | `sys_agent_info` | Query Agent metadata for the current process or a target pid. |
| 502 | `sys_tool_call` | Execute a structured kernel tool request and write the result into Agent Context. |
| 503 | `sys_tool_list` | List available kernel tools. |
| 504 | `sys_context_push` | Append a Context Path node. |
| 505 | `sys_context_query` | Query recent Context Path nodes. |
| 506 | `sys_context_rollback` | Roll back the active Context Path node. |
| 507 | `sys_context_clear` | Clear the current Context Path. |
| 508 | `sys_agent_heartbeat_set` | Configure Agent Loop heartbeat wakeups. |
| 509 | `sys_agent_heartbeat_stop` | Stop heartbeat wakeups. |
| 510 | `sys_agent_wait` | Block the Agent until heartbeat or message wakeup. |
| 511 | `sys_file_attr_set` | Bind searchable attributes to a file. |
| 512 | `sys_file_attr_delete` | Delete a file attribute entry. |

## Kernel Tools

| Tool | Purpose |
| --- | --- |
| `get_system_status` | Return process, Agent, memory, and time summaries. |
| `query_process` | Query processes by status and Agent type. |
| `send_message` | Send a structured Agent message and wake the target Agent. |
| `query_file` | Query files by `type`, `owner`, `tag`, and `priority` attributes. |

## Documentation

Start with [Docs/design.md](Docs/design.md) for architecture and [Docs/evaluation.md](Docs/evaluation.md) for test results. Each milestone also has its own `DESIGN.md`, `IMPLEMENTATION_LOG.md`, and `TEST_REPORT.md` under `Docs/Mx/`.
