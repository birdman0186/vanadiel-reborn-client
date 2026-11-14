[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ($env:OS -ne 'Windows_NT') { throw 'Run this script on Windows.' }
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $repo
$savedEnv = @{}
foreach ($key in @('RUSTUP_TOOLCHAIN', 'CXXFLAGS', 'RUSTFLAGS', 'CARGO_PROFILE_DEV_CODEGEN_BACKEND', 'CARGO_PROFILE_TEST_CODEGEN_BACKEND')) {
    $savedEnv[$key] = [Environment]::GetEnvironmentVariable($key, 'Process')
}
try {
    $pin = (Select-String -Path rust-toolchain.toml -Pattern '^channel = "([^"]+)"').Matches.Groups[1].Value
    if (-not $pin) { throw 'The Rust toolchain pin is missing.' }
    rustup toolchain install $pin --profile minimal
    if ($LASTEXITCODE -ne 0) { throw 'Rust toolchain setup failed.' }
    $env:RUSTUP_TOOLCHAIN = $pin
    $env:CXXFLAGS = ''
    $env:RUSTFLAGS = '-C target-feature=+crt-static'
    $env:CARGO_PROFILE_DEV_CODEGEN_BACKEND = 'llvm'
    $env:CARGO_PROFILE_TEST_CODEGEN_BACKEND = 'llvm'
    git -c core.autocrlf=false -c core.longpaths=true submodule update --init --depth 1 vendor/server vendor/POLUtils
    if ($LASTEXITCODE -ne 0) { throw 'Build input setup failed.' }
    cargo build --release --locked --target x86_64-pc-windows-msvc -p kuluu --no-default-features --features native-window
    if ($LASTEXITCODE -ne 0) { throw 'The Windows build failed.' }
    $targetRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
    $exe = Join-Path $targetRoot 'x86_64-pc-windows-msvc/release/kuluu.exe'
    & $exe --help | Out-Host
    if ($LASTEXITCODE -ne 0) { throw 'The executable startup check failed.' }
    $dist = Join-Path $repo 'dist'
    New-Item -ItemType Directory -Force -Path $dist | Out-Null
    $stage = Join-Path $dist ('vdr-stage-' + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $stage | Out-Null
    Copy-Item $exe $stage
    Copy-Item LICENSE, LEGAL.md $stage
    Copy-Item (Join-Path $PSScriptRoot 'README.md'), (Join-Path $PSScriptRoot 'inspect_lsb.py') $stage
    $commit = (git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot identify the source commit.' }
    [ordered]@{
        source_commit = $commit
        rust_toolchain = $pin
        target = 'x86_64-pc-windows-msvc'
        features = @('native-window')
        sha256 = (Get-FileHash $exe -Algorithm SHA256).Hash.ToLowerInvariant()
        startup_help_verified = $true
        live_server_verified = $false
        gpu_rendering_verified = $false
    } | ConvertTo-Json | Set-Content (Join-Path $stage 'build-info.json') -Encoding UTF8
    $zip = Join-Path $dist 'vanadiel-reborn-windows-x64.zip'
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -Force
    Write-Host "Package: $zip"
} finally {
    foreach ($key in $savedEnv.Keys) {
        [Environment]::SetEnvironmentVariable($key, $savedEnv[$key], 'Process')
    }
    Pop-Location
}
