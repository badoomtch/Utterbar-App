# Utterbar v0.15.6+ DirectML BUNDLE-REBUILD orchestration script.
#
# DEVELOPER-MAINTENANCE script. Not part of normal app builds.
# Production app builds use the prebuilt bundle vendored at
# `Utterbar Windows App/src-tauri/vendor/sherpa-onnx/lib/`,
# linked through SHERPA_LIB_PATH (see .cargo/config.toml). Normal
# `cargo build` and `cargo tauri build` invocations do NOT compile
# sherpa-onnx from source.
#
# This script is for the rare manual rebuild of that bundle, e.g.:
#   - sherpa-rs / sherpa-rs-sys version bump
#   - ORT security patch landing in a new sherpa-onnx release
#   - DirectML version refresh
#
# Two-pass workflow because sherpa-rs-sys's build.rs extracts the
# sherpa-onnx source on first cargo invocation, but the extracted
# `cmake/show-info.cmake` uses `wmic` which is removed on Windows 11
# 24H2+. We patch between extraction and CMake configure:
#
#   Pass 1: cargo build (expected to fail at show-info.cmake)
#   Patch overlay: copy vendor/sherpa-onnx-patches/show-info.cmake
#                  over the extracted file
#   Cache clear:   delete CMakeCache.txt + CMakeFiles/ so cmake-rs
#                  re-runs configure with the patched script
#   Pass 2: cargo build (now configure succeeds; build proceeds)
#
# Pre-requisites:
#   - CMake 4.x on PATH (4.3 used to produce the v0.15.6 bundle)
#   - Visual Studio 2022 Build Tools with VC.Tools.x86.x64 + Win10 SDK
#   - Rust toolchain stable-x86_64-pc-windows-msvc
#   - C:\ut-target writable (avoids MSBuild FileTracker MAX_PATH=260)
#
# Required env (pre-set this in the calling shell, not inside this script):
#   - In Cargo.toml: sherpa-rs = { version = "0.6", default-features = false, features = ["directml"] }
#     (production keeps `default-features = false` only, no `directml` feature)
#   - In .cargo/config.toml: comment out the `[env] SHERPA_LIB_PATH = ...` stanza
#     (production keeps the SHERPA_LIB_PATH stanza enabled)
#
# After the build succeeds:
#   - Copy *.dll + *.lib from
#     C:\ut-target\release\build\sherpa-rs-sys-*\out\lib\
#     into Utterbar Windows App/src-tauri/vendor/sherpa-onnx/lib/
#   - Mirror DLLs (no .lib) into Utterbar Windows App/src-tauri/runtime/
#   - Update BUNDLE_MANIFEST.txt with new SHAs + ORT/DirectML versions
#   - Restore Cargo.toml + .cargo/config.toml to production state
#
# See the "Bundle rebuild" section of RELEASE_RUNBOOK.md for the full
# documented procedure.
#
# Usage:
#   pwsh -File "Utterbar Windows App/scripts/build-directml-bundle.ps1"
#
# Logs land in $env:LOCALAPPDATA\Temp\utterbar-step1-pass{1,2}.log

$ErrorActionPreference = "Stop"

# Resolve script + repo paths from $PSScriptRoot so the script doesn't
# depend on the caller's CWD.
$ScriptDir = $PSScriptRoot
$AppDir = Split-Path -Parent $ScriptDir
$SrcTauriDir = Join-Path $AppDir "src-tauri"
$PatchSrc = Join-Path $SrcTauriDir "vendor\sherpa-onnx-patches\show-info.cmake"
$LogDir = Join-Path $env:LOCALAPPDATA "Temp"
$Pass1Log = Join-Path $LogDir "utterbar-step1-pass1.log"
$Pass2Log = Join-Path $LogDir "utterbar-step1-pass2.log"

if (-not (Test-Path $PatchSrc)) {
    throw "Patch file missing: $PatchSrc -- was the vendor/sherpa-onnx-patches/show-info.cmake created?"
}
if (-not (Test-Path "C:\ut-target")) {
    throw "C:\ut-target does not exist -- create it before running"
}

$env:CARGO_TARGET_DIR = "C:\ut-target"
$env:SHERPA_BUILD_DEBUG = "1"

Write-Output "===== STEP 1: cargo build pass 1 (trigger sherpa-onnx extraction) ====="
Write-Output "CARGO_TARGET_DIR = $env:CARGO_TARGET_DIR"
Write-Output "Working dir      = $SrcTauriDir"
Write-Output "Pass 1 log       = $Pass1Log"
Write-Output "Pass 2 log       = $Pass2Log"
Write-Output ""

Set-Location $SrcTauriDir

# Pass 1 is EXPECTED to fail at the unpatched show-info.cmake. We
# tolerate the failure and inspect what got extracted.
#
# Why `cmd /c` instead of native PowerShell pipe: in Windows
# PowerShell 5.1, `2>&1` on a native exe wraps every stderr line
# in an ErrorRecord (NativeCommandError) which trips
# $ErrorActionPreference = "Stop" the moment cargo prints its
# first compile-progress line (cargo writes to stderr by default).
# `cmd /c` keeps the stream merging inside cmd, so PowerShell only
# sees the final exit code.
$pass1Start = Get-Date
cmd /c "cargo build > `"$Pass1Log`" 2>&1"
$pass1Exit = $LASTEXITCODE
$pass1Duration = (Get-Date) - $pass1Start
Write-Output "Pass 1 finished. Exit=$pass1Exit, duration=$($pass1Duration.TotalSeconds)s"

# Locate the extracted show-info.cmake. Sherpa-rs-sys hashes its OUT_DIR
# from the cargo target, so the directory name varies by target.
$extractedSearch = Join-Path $env:CARGO_TARGET_DIR "debug\build\sherpa-rs-sys-*\out\sherpa-onnx\cmake\show-info.cmake"
$extracted = Get-ChildItem -Path $extractedSearch -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $extracted) {
    Write-Output ""
    Write-Output "FATAL: pass 1 did not extract sherpa-onnx source -- show-info.cmake not found under $env:CARGO_TARGET_DIR\debug\build\sherpa-rs-sys-*"
    Write-Output "Tail of pass 1 log:"
    Get-Content $Pass1Log -Tail 30
    exit 1
}

Write-Output ""
Write-Output "===== Patch overlay ====="
Write-Output "Source: $PatchSrc"
Write-Output "Target: $($extracted.FullName)"
Copy-Item $PatchSrc $extracted.FullName -Force
$patchedHash = (Get-FileHash $extracted.FullName -Algorithm SHA256).Hash
Write-Output "Patched file SHA-256: $patchedHash"
Write-Output ""

# Pass 1 left a half-configured CMake state on disk. Pass 2's
# cmake-rs Rust crate detects that and SKIPS configure -- but the
# state is invalid (configure errored at show-info.cmake before
# install.vcxproj could be generated), so MSBuild fires
# `MSB1009: Project file does not exist. Switch: install.vcxproj`
# almost immediately.
#
# Fix: delete CMakeCache.txt (and CMakeFiles/ for safety) so pass 2
# fully reconfigures with the patched show-info.cmake. The
# `_deps/` directory inside `out/build/` is preserved, which keeps
# the ~600 MB of fetched ORT + DirectML + sherpa-onnx + kaldi etc.
# downloads from being re-fetched.
$buildDir = Join-Path (Split-Path -Parent $extracted.FullName) "..\..\build" | Resolve-Path -ErrorAction SilentlyContinue
if ($buildDir) {
    $cmakeCache = Join-Path $buildDir "CMakeCache.txt"
    $cmakeFilesDir = Join-Path $buildDir "CMakeFiles"
    if (Test-Path $cmakeCache) {
        Remove-Item $cmakeCache -Force
        Write-Output "Deleted: $cmakeCache"
    }
    if (Test-Path $cmakeFilesDir) {
        Remove-Item $cmakeFilesDir -Recurse -Force
        Write-Output "Deleted: $cmakeFilesDir"
    }
} else {
    Write-Output "Note: build dir not found at $($extracted.FullName | Split-Path -Parent)\..\..\build -- pass 2 may reuse stale CMake state"
}
Write-Output ""

Write-Output "===== STEP 1: cargo build pass 2 (configure + compile + link) ====="
Write-Output "Pass 2 will take a while (sherpa-onnx C++ compile is large)."
Write-Output ""

$pass2Start = Get-Date
cmd /c "cargo build > `"$Pass2Log`" 2>&1"
$pass2Exit = $LASTEXITCODE
$pass2Duration = (Get-Date) - $pass2Start
Write-Output ""
Write-Output "Pass 2 finished. Exit=$pass2Exit, duration=$($pass2Duration.TotalSeconds)s"

if ($pass2Exit -eq 0) {
    Write-Output ""
    Write-Output "===== SUCCESS ====="
    Write-Output "Build completed. Inspect artefacts under:"
    Write-Output "  $env:CARGO_TARGET_DIR\debug\build\sherpa-rs-sys-*\out\"
} else {
    Write-Output ""
    Write-Output "===== FAILED ====="
    Write-Output "Pass 2 exit code: $pass2Exit"
    Write-Output "Tail of pass 2 log:"
    Get-Content $Pass2Log -Tail 40
}

exit $pass2Exit
