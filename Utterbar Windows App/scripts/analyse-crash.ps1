# ----------------------------------------------------------------------
# analyse-crash.ps1 — read the most recent Utterbar minidump and print a
# symbolicated call stack from every thread. Drop this into a chat
# message when a crash happens and we'll know exactly which line of Rust
# (or which native call in sherpa-onnx / DirectML / Tauri) was running
# at the moment of death.
#
# Usage, from a PowerShell prompt:
#     .\Utterbar Windows App\scripts\analyse-crash.ps1
#
# Optional:
#     -Dump <path>   specific dump file instead of the newest
#     -All           print stacks for ALL threads (default: crashing thread only)
# ----------------------------------------------------------------------

param(
    [string]$Dump,
    [switch]$All
)

$ErrorActionPreference = 'Stop'

$cdbCandidates = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\Debuggers\x64\cdb.exe",
    "${env:ProgramFiles}\Windows Kits\10\Debuggers\x64\cdb.exe"
)
$cdb = $cdbCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $cdb) {
    Write-Error "cdb.exe not found. Install Windows SDK debugging tools:"
    Write-Error "  Download https://go.microsoft.com/fwlink/?linkid=2286561 and run it with /features OptionId.WindowsDesktopDebuggers"
    exit 1
}

$logsDir = "$env:APPDATA\Utterbar\logs"
if (-not (Test-Path $logsDir)) {
    Write-Error "No Utterbar logs directory at $logsDir — has the app ever run on this machine?"
    exit 1
}

if (-not $Dump) {
    $newest = Get-ChildItem $logsDir -Filter "crash-*.dmp" -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if (-not $newest) {
        Write-Error "No crash-*.dmp files in $logsDir. Has the new SEH-handler build crashed yet?"
        exit 1
    }
    $Dump = $newest.FullName
}
Write-Output "Analysing: $Dump"

# PDB path: the release build writes utterbar.pdb alongside utterbar.exe.
$pdbDir = Resolve-Path (Join-Path $PSScriptRoot "..\src-tauri\target\release")
if (-not (Test-Path "$pdbDir\utterbar.pdb")) {
    Write-Warning "No PDB at $pdbDir\utterbar.pdb. Output will be symbol-free."
}

# Build the cdb command script.
$stackCmd = if ($All) { '~*kn' } else { 'kn' }
$commands = @"
.sympath+ $pdbDir
.srcpath+ $(Resolve-Path (Join-Path $PSScriptRoot ".."))
.reload /f
.excr
$stackCmd
!analyze -v
q
"@
$cmdFile = New-TemporaryFile
$commands | Out-File -FilePath $cmdFile -Encoding ascii

Write-Output ""
Write-Output "=========================================="
Write-Output "  cdb output"
Write-Output "=========================================="
& $cdb -z $Dump -y $pdbDir -cf $cmdFile -lines 2>&1 |
    Where-Object { $_ -notmatch "^(Loading|Modload|\*|Debug session|ntdll|KERNEL|Reading initial|Executable search|Response|OK|Symbol search)" } |
    Select-Object -First 200

Remove-Item $cmdFile

Write-Output ""
Write-Output "=========================================="
Write-Output "  Also in $logsDir:"
Write-Output "=========================================="
Write-Output "  - crash.log         (running tally of SEH exceptions)"
Write-Output "  - utterbar.log.*    (session tracing, per-day rolling)"
Write-Output ""
Write-Output "  Send me the contents of the cdb output above plus the"
Write-Output "  last ~50 lines of utterbar.log.<today> for diagnosis."
