$ErrorActionPreference = 'Stop'
cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "Portable candidate: $PWD\target\release\dda-remote.exe"
