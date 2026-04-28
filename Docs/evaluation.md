# Agent-OS Evaluation

## Environment

The implementation is evaluated on RISC-V 64 QEMU under the rCore-Tutorial v3 build flow. The verified commands use stable Rust with `RUSTC_BOOTSTRAP=1`.

## Build Checks

Executed checks:

```bash
cargo +stable fmt
RUSTC_BOOTSTRAP=1 cargo +stable check
RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer' cargo +stable check
git diff --check
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Afunction-casts-as-integer -Clink-arg=-Tsrc/linker.ld -Cforce-frame-pointers=yes' make kernel
rustup run stable rust-objcopy target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin
RUSTUP_TOOLCHAIN=stable RUSTC_BOOTSTRAP=1 make fs-img
```

Result: all build checks passed in the M7 verification run.

## Functional Tests

Regression programs:

| Program | Coverage | Result |
| --- | --- | --- |
| `agent_m1` | ordinary processes are not Agents; `agent_info` error paths | Passed |
| `agent_m2` | Agent creation, Agent Context mapping, direct Context read/write | Passed |
| `agent_m3` | tool list, system status, process query, message tool, tool errors | Passed |
| `agent_m4` | Context Path push, query, rollback, clear, FIFO quota behavior | Passed |
| `agent_m5` | heartbeat wait, message wakeup, blocked Agent scheduling | Passed |
| `agent_m6` | file attributes, multi-condition file query, delete behavior | Passed |

Integrated demo programs:

| Program | Coverage | Result |
| --- | --- | --- |
| `agent_demo basic` | Agent creation, tool calls, result reads, Context Path writes | Passed |
| `agent_demo loop` | heartbeat wakeup, Worker-Agent message wakeup, Context Path | Passed |
| `agent_demo fs_query_bench` | file attributes and query access-count comparison | Passed |
| `agent_demo full` | end-to-end Admin-Agent scenario across M1-M6 mechanisms | Passed |

Key M7 QEMU output:

```text
agent_demo basic: passed
agent_demo loop: passed
agent_demo fs_query_bench: passed
agent_demo full: passed
agent_m6 passed
agent_m5 passed
agent_m4 passed
agent_m3 passed
agent_m2 passed
agent_m1 passed
```

The automated QEMU command is terminated by `timeout` after usershell returns to waiting for input. Exit code `124` is expected in that harness once all pass lines have been printed.

## File Query Performance

The M6/M7 benchmark creates four files and attaches attributes. The query:

```text
type=memory AND owner=worker AND tag=social
```

returns one matching file:

```text
query_file matches=1 traversal=4 indexed=2 first=m7_b
full file_query matches=1 traversal=4 indexed=2
```

Interpretation:

- Full traversal checks 4 attribute entries.
- The simplified index model checks 2 candidate entries selected by the first query condition.
- Candidate query visits are reduced by 50 percent in the demo dataset.

## Agent Context Access

Agent Context is mapped into user space, so the Agent can directly read tool results and Context Path summaries after the syscall has written them. This avoids a second syscall for every result read and demonstrates the mechanism/policy split required by the competition statement:

- Kernel enforces identity, quota, and metadata updates.
- User-space policy reads and interprets cached context bytes directly.

## Stability Notes

The integrated scenario repeatedly exercises Agent creation, wakeup, file query, Context Path, and shell command mapping without kernel panic in the recorded M7 QEMU run. Known implementation limits are documented in the milestone reports:

- File attributes are memory-only and reset after reboot.
- Context Path FIFO is coarse-grained when the write cursor wraps.
- Heartbeat scanning is linear over the process tree, suitable for the teaching demo scale.
