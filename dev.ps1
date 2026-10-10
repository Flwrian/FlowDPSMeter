param([ValidateSet('dev','build','test','replay')][string]$Mode = 'dev', [string]$Capture)
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot
$bundledToolchain = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../work/toolchain'))
if (-not (Get-Command cargo -ErrorAction SilentlyContinue) -and (Test-Path -LiteralPath (Join-Path $bundledToolchain 'cargo/bin/cargo.exe'))) {
    $env:CARGO_HOME = Join-Path $bundledToolchain 'cargo'
    $env:RUSTUP_HOME = Join-Path $bundledToolchain 'rustup'
    $env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Installer Rust stable MSVC depuis https://rustup.rs.' }
if ($Mode -in @('dev','build')) {
    if (-not (Test-Path -LiteralPath 'node_modules')) { npm ci; if ($LASTEXITCODE -ne 0) {exit $LASTEXITCODE} }
}
switch ($Mode) {
    'dev' { npm run tauri -- dev }
    'build' { npm run tauri -- build --no-bundle }
    'test' {
        cargo test --manifest-path src-tauri/Cargo.toml --no-default-features --lib --test party_meter --test wire_pipeline
        if ($LASTEXITCODE -ne 0) {exit $LASTEXITCODE}
        cargo test --manifest-path src-tauri/Cargo.toml --lib -- --test-threads=1
        npm test
        if ($LASTEXITCODE -ne 0) {exit $LASTEXITCODE}
        npm run typecheck
    }
    'replay' {
        if (-not $Capture) { $Capture = 'examples/groupe-synthetique.apm.jsonl' }
        cargo run --manifest-path src-tauri/Cargo.toml --no-default-features --bin apm-replay -- $Capture
    }
}
exit $LASTEXITCODE
