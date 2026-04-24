# Stages sherpa-onnx runtime DLLs produced by cargo build.
# - Copies them into src-tauri/runtime/ (referenced by tauri.conf.json bundle.resources)
# - Also drops them next to the built exe at src-tauri/target/release/ so running the
#   exe directly from the build output works without reinstalling.
#
# Runs as tauri.conf.json beforeBundleCommand.

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$deps     = Join-Path $repoRoot 'src-tauri\target\release\deps'
$stage    = Join-Path $repoRoot 'src-tauri\runtime'
$release  = Join-Path $repoRoot 'src-tauri\target\release'

if (-not (Test-Path $deps)) {
    Write-Error "expected $deps to exist after cargo build"
}

New-Item -ItemType Directory -Path $stage -Force | Out-Null

$names = @(
    'sherpa-onnx-c-api.dll',
    'sherpa-onnx-cxx-api.dll',
    'onnxruntime.dll',
    'onnxruntime_providers_shared.dll',
    'cargs.dll'
)

foreach ($n in $names) {
    $src = Join-Path $deps $n
    if (-not (Test-Path $src)) {
        Write-Warning ("missing: " + $n)
        continue
    }
    Copy-Item -Path $src -Destination (Join-Path $stage $n) -Force
    # Best-effort copy next to the built exe for local runs; tolerate the file being
    # locked when the exe is open (the installer still picks them up from $stage).
    try {
        Copy-Item -Path $src -Destination (Join-Path $release $n) -Force -ErrorAction Stop
    } catch {
        Write-Warning ("could not refresh " + $n + " in target/release (file locked) - installer still gets the fresh copy from runtime/")
    }
    Write-Host ("staged " + $n)
}
