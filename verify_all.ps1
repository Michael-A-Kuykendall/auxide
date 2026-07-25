# Integration gate for the auxide stack (native Windows / PowerShell).
#
# Runs build + test + clippy (warning-clean, including tests/examples via
# --all-targets) across all four crates, which live as sibling directories.
# Exits non-zero on the first crate that fails. On CI/Linux use verify_all.sh.

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Resolve-Path (Join-Path $ScriptDir '..')
$Crates = @('auxide', 'auxide-dsp', 'auxide-io', 'auxide-midi')

foreach ($c in $Crates) {
    $Dir = Join-Path $Root $c
    Write-Host "=== $c ==="
    Push-Location $Dir
    try {
        cargo build; if ($LASTEXITCODE -ne 0) { exit 1 }
        cargo test;  if ($LASTEXITCODE -ne 0) { exit 1 }
        cargo clippy --all-targets -- -D warnings; if ($LASTEXITCODE -ne 0) { exit 1 }
    } finally {
        Pop-Location
    }
}

Write-Host "ALL CRATES GREEN"
