# ----------------------------------------------------------------------
# analyse-crash.ps1 - read the most recent Utterbar minidump and print
# a symbolicated call stack.
#
# Usage, from a PowerShell prompt:
#     .\Utterbar Windows App\scripts\analyse-crash.ps1
#     .\Utterbar Windows App\scripts\analyse-crash.ps1 -All
#     .\Utterbar Windows App\scripts\analyse-crash.ps1 -Dump C:\path\to\crash.dmp
# ----------------------------------------------------------------------

param(
    [string]$Dump,
    [switch]$All
)

$ErrorActionPreference = 'Stop'

# Locate cdb.exe. Ship-path for Windows SDK debugging tools.
$cdbCandidates = @(
    (Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\Debuggers\x64\cdb.exe'),
    (Join-Path $env:ProgramFiles          'Windows Kits\10\Debuggers\x64\cdb.exe')
)
$cdb = $cdbCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $cdb) {
    Write-Error 'cdb.exe not found. Install Windows SDK Debugging Tools (winget install Microsoft.WindowsSDK or run the SDK installer with the "Debugging Tools for Windows" feature selected).'
    exit 1
}

# Locate the logs directory. All crash dumps are written alongside
# crash.log and utterbar.log.* under %APPDATA%\Utterbar\logs.
$logsDir = Join-Path $env:APPDATA 'Utterbar\logs'
if (-not (Test-Path $logsDir)) {
    Write-Error ('No Utterbar logs directory at ' + $logsDir + ' - has the app ever run on this machine?')
    exit 1
}

# Pick the dump.
if (-not $Dump) {
    $newest = Get-ChildItem $logsDir -Filter 'crash-*.dmp' -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if (-not $newest) {
        Write-Error ('No crash-*.dmp files in ' + $logsDir + '. Has the new SEH-handler build crashed yet?')
        exit 1
    }
    $Dump = $newest.FullName
}

Write-Output ('Analysing: ' + $Dump)

# PDB path: the release build writes utterbar.pdb alongside utterbar.exe.
$pdbDir = Resolve-Path (Join-Path $PSScriptRoot '..\src-tauri\target\release')
if (-not (Test-Path (Join-Path $pdbDir 'utterbar.pdb'))) {
    Write-Warning ('No PDB at ' + $pdbDir + '\utterbar.pdb. Output will be symbol-free.')
}

# Source tree root (used by cdb's .srcpath for rust-src lookups).
$srcRoot = Resolve-Path (Join-Path $PSScriptRoot '..')

# Build the cdb command script. Single-quoted here-string: nothing inside
# is expanded by PowerShell, we concatenate the pdbDir + srcRoot + stack
# command in afterwards. That avoids the variable-reference-before-colon
# parser trap that broke earlier versions of this script.
$stackCmd = if ($All) { '~*kn' } else { 'kn' }

$commands = @'
.sympath+ __PDBDIR__
.sympath+ srv*https://msdl.microsoft.com/download/symbols
.srcpath+ __SRCROOT__
.reload /f
.excr
__STACK__
!analyze -v
q
'@

# Use `.Replace()` (String method, literal) rather than `-replace`
# (regex). Windows paths contain backslashes which `-replace` would
# try to interpret as regex escapes, and `[regex]::Escape` doubles
# backslashes which cdb's parser then treats as escaped spaces on
# directory names. Literal string replace is the only sane option.
$commands = $commands.Replace('__PDBDIR__', $pdbDir.Path)
$commands = $commands.Replace('__SRCROOT__', $srcRoot.Path)
$commands = $commands.Replace('__STACK__', $stackCmd)

$cmdFile = New-TemporaryFile
$commands | Out-File -FilePath $cmdFile -Encoding ascii

Write-Output ''
Write-Output '=========================================='
Write-Output '  cdb output'
Write-Output '=========================================='

# Run cdb. The `Where-Object` filter strips the noisy symbol-loading
# lines so the stack is readable. Limit to 300 lines - anything more
# than that is !analyze -v bloat we don't need.
& $cdb -z $Dump -y $pdbDir -cf $cmdFile -lines 2>&1 |
    Where-Object { $_ -notmatch '^(Loading|Modload|\*|Debug session|ntdll|KERNEL|Reading initial|Executable search|Response|OK|Symbol search|NatVis|The call to|Microsoft|Copyright|Command Line|Press|STATUS_|Launch exception)' } |
    Select-Object -First 300

Remove-Item $cmdFile -ErrorAction SilentlyContinue

Write-Output ''
Write-Output '=========================================='
Write-Output ('  Also in ' + $logsDir + ':')
Write-Output '=========================================='
Write-Output '  - crash.log         (running tally of SEH exceptions)'
Write-Output '  - utterbar.log.*    (session tracing, per-day rolling)'
Write-Output ''
Write-Output '  Send me the contents of the cdb output above plus the'
Write-Output '  last ~50 lines of utterbar.log (today) for diagnosis.'
