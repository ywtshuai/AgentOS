param(
    [switch]$Strict
)

$ErrorActionPreference = "Continue"

function Test-Command {
    param(
        [Parameter(Mandatory=$true)][string]$Name,
        [string[]]$Args = @("--version")
    )

    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    if (-not $cmd) {
        [PSCustomObject]@{
            Name = $Name
            Found = $false
            Detail = "missing"
        }
        return
    }

    $output = ""
    try {
        $output = & $Name @Args 2>&1 | Select-Object -First 1
    } catch {
        $output = $_.Exception.Message
    }

    [PSCustomObject]@{
        Name = $Name
        Found = $true
        Detail = "$output"
    }
}

$checks = @(
    (Test-Command "git"),
    (Test-Command "rustup"),
    (Test-Command "rustc"),
    (Test-Command "cargo"),
    (Test-Command "make"),
    (Test-Command "qemu-system-riscv64")
)

$checks | Format-Table -AutoSize

$missing = @($checks | Where-Object { -not $_.Found })
if ($missing.Count -gt 0) {
    Write-Host ""
    Write-Host "Missing required tools:" -ForegroundColor Yellow
    $missing | ForEach-Object { Write-Host " - $($_.Name)" }
    Write-Host ""
    Write-Host "Suggested Windows setup:"
    Write-Host "  winget install Rustlang.Rustup"
    Write-Host "  winget install GnuWin32.Make"
    Write-Host "  winget install SoftwareFreedomConservancy.QEMU"
    Write-Host "  rustup target add riscv64gc-unknown-none-elf"
    Write-Host "  rustup component add rust-src llvm-tools-preview"
    Write-Host "  cargo install cargo-binutils"

    if ($Strict) {
        exit 1
    }
}

exit 0
