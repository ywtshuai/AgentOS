# M0 Test Report

## 测试命令

```powershell
git status --short --branch
.\scripts\check-env.ps1 -Strict
cd rcore\os
make build
```

## 测试结果

- `git status --short --branch`：确认已切换到 `feature/agent-os`。
- `.\scripts\check-env.ps1 -Strict`：失败，发现缺少必需工具。
- `make build`：未执行成功，当前环境找不到 `make`。

## 失败项与修复

当前缺失项：

- `rustup`
- `rustc`
- `cargo`
- `make`
- `qemu-system-riscv64`

建议安装命令：

```powershell
winget install Rustlang.Rustup
winget install GnuWin32.Make
winget install SoftwareFreedomConservancy.QEMU
rustup target add riscv64gc-unknown-none-elf
rustup component add rust-src llvm-tools-preview
cargo install cargo-binutils
```

安装完成后重新运行：

```powershell
.\scripts\check-env.ps1 -Strict
cd rcore\os
make build
make run
```

## 尚未覆盖的风险

- Windows 原生命令行运行 rCore Makefile 可能需要额外 Unix 工具，例如 `sh`、`grep`、`rm`、`cp`。
- QEMU 版本过旧可能无法通过 `rcore/os/scripts/qemu-ver-check.sh`。
- M0 尚未做内核功能变更，Agent-OS 功能测试从 M1 开始覆盖。
