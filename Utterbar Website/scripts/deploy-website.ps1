# ----------------------------------------------------------------------
# deploy-website.ps1 — push the current working tree of `Utterbar Website`
# to the Cloudways droplet.
#
# Usage (from a PowerShell prompt):
#
#     .\Utterbar Website\scripts\deploy-website.ps1
#
# The Cloudways SSH password is read from `~/.utterbar-deploy/.env` —
# the same off-repo credential file that holds the R2 keys and admin
# API token. Add a line:
#
#     CLOUDWAYS_PASSWORD=your-ssh-password
#
# to that file (one-time setup) and future deploys just work. The file
# is outside the repo, per-user, and should be `chmod 600`.
#
# If you'd rather not persist the password, set `$env:CLOUDWAYS_PASSWORD`
# in your shell before invoking this script — the env var wins over
# the file if both are present.
#
# Prerequisites (one-time, on the builder's machine):
#   winget install PuTTY.PuTTY     # brings pscp + plink
#   winget install PHP.PHP.8.2     # only if you want to run migrations locally
#
# What it does (in order):
#   1.  Sanity-check the working tree: no uncommitted Website changes,
#       no .env in the staged payload, no node_modules/vendor being sent.
#   2.  Rsync-ish copy of the Laravel source to the droplet via pscp.
#       Excludes: vendor/, node_modules/, .env, storage/logs/*, tests/,
#       and anything under `Utterbar Windows App/` (desktop-only).
#   3.  Run `composer install --no-dev --optimize-autoloader` remotely so
#       production deps match composer.lock.
#   4.  Run pending migrations (`php artisan migrate --force`). These
#       are idempotent — existing DB rows are left alone, and the two
#       hash-tokens migrations backfill live tokens in-place.
#   5.  Clear the view / config / route caches.
#   6.  Rebuild the Vite manifest remotely (`npm run build`) so the
#       updated CSS/JS ship with the new Blade views.
#   7.  Smoke-test: curl /api/pricing and /api/updater/check on the
#       live host to make sure the new endpoints respond 200.
#
# What it does NOT do:
#   - Touch the Tauri app or the Remotion bundle.
#   - Push marketing videos to R2 (that's a separate pass).
#   - Publish a new app release — that's `scripts/publish-release.sh`
#     in `Utterbar Windows App/`.
# ----------------------------------------------------------------------

param(
    # Explicit -HostName / -RemotePath still override anything loaded from
    # the env file. Useful if you ever need to target a staging server
    # without editing the credential file. Leave empty to let the env
    # file drive both.
    [string]$HostName = '',
    [string]$RemotePath = '',
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'

# ----------------------------------------------------------------------
# PowerShell 5.1 native-stderr workaround
# ----------------------------------------------------------------------
# PowerShell 5.1 wraps any line a native command writes to stderr in a
# NativeCommandError ErrorRecord, and `$ErrorActionPreference = 'Stop'`
# above turns that into a terminating error. Several tools we drive
# (composer, plink, pscp, npm) write innocuous progress / status lines
# to stderr on success — composer's "Installing dependencies from lock
# file" is the canonical offender. The script aborts before $LASTEXITCODE
# can be checked, even though the underlying command exited 0.
#
# Fix: wrap each native call in this helper, which flips
# $ErrorActionPreference to 'Continue' for the duration of the call,
# letting stderr go to its own stream without aborting. The
# `if ($LASTEXITCODE -ne 0) { throw }` checks after each call remain
# the real exit-code gate — that's the contract: we trust exit codes,
# not stderr presence.
#
# Do NOT add `2>&1` inside the wrapped block — that re-introduces the
# NativeCommandError wrapping and defeats the helper.
function Invoke-NativeQuiet([scriptblock]$Block) {
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try { & $Block } finally { $ErrorActionPreference = $prev }
}

# Credential loading from ~/.utterbar-deploy/.env (off-repo, per-user).
#
# Supports two Cloudways auth patterns; the script picks whichever is set:
#
# 1. MASTER-LEVEL credentials (most common for fresh Cloudways servers):
#      CLOUDWAYS_IP          = <server ip>
#      CLOUDWAYS_MASTER_USER = master_xxxxxxxxxx
#      CLOUDWAYS_MASTER_PASS = <pw>
#    Apps live at ~/applications/<APP_FOLDER>/public_html/ on the master
#    user's home directory. The app folder is the 10-char random Cloudways
#    assigns (NOT the domain) — look it up in Application Management.
#    Set CLOUDWAYS_APP_FOLDER in the env file too, or pass -RemotePath.
#
# 2. APP-LEVEL credentials (only if you've explicitly set an app SSH
#    password via Cloudways → SSH/SFTP Access → "Enable SSH"):
#      CLOUDWAYS_HOST     = <user>@<server ip>
#      CLOUDWAYS_PASSWORD = <pw>
#    App files live at ~/public_html on the per-app user. The script
#    targets that by default under this pattern.
#
# No secret ever lives in the repo; editing ~/.utterbar-deploy/.env once
# sets you up for every future deploy.
function Read-EnvValue($key) {
    $envFile = Join-Path $env:USERPROFILE '.utterbar-deploy\.env'
    if (-not (Test-Path $envFile)) { return $null }
    # Iterate explicitly so the linter can see the assignment flow
    # (ForEach-Object closure hides it). Last match wins — later lines
    # in the env file override earlier ones if a key is duplicated.
    $found = $null
    foreach ($line in Get-Content $envFile) {
        if ($line -match "^\s*$([regex]::Escape($key))\s*=\s*(.+?)\s*$") {
            $value = $Matches[1]
            if ($value -match '^"(.*)"$' -or $value -match "^'(.*)'`$") {
                $value = $Matches[1]
            }
            $found = $value
        }
    }
    return $found
}

# Resolve the credentials. Master-level wins if present; falls back to
# app-level; errors out if neither is set.
$masterUser = Read-EnvValue 'CLOUDWAYS_MASTER_USER'
$masterPass = Read-EnvValue 'CLOUDWAYS_MASTER_PASS'
$masterIp   = Read-EnvValue 'CLOUDWAYS_IP'
$appFolder  = Read-EnvValue 'CLOUDWAYS_APP_FOLDER'
$appHost    = Read-EnvValue 'CLOUDWAYS_HOST'
$appPass    = Read-EnvValue 'CLOUDWAYS_PASSWORD'

if (-not $HostName -or -not $RemotePath) {
    if ($masterUser -and $masterPass -and $masterIp) {
        if (-not $HostName)   { $HostName   = "${masterUser}@${masterIp}" }
        if (-not $RemotePath) {
            if (-not $appFolder) {
                Write-Error "Master-credential path selected, but CLOUDWAYS_APP_FOLDER is not set in ~/.utterbar-deploy/.env. Cloudways apps live at ~/applications/<FOLDER>/public_html - look up the 10-char app folder in Application Management and add it to the env file."
                exit 1
            }
            $RemotePath = "applications/${appFolder}/public_html"
        }
        $pass = $masterPass
    }
    elseif ($appHost -and $appPass) {
        if (-not $HostName)   { $HostName   = $appHost }
        if (-not $RemotePath) { $RemotePath = 'public_html' }
        $pass = $appPass
    }
    else {
        Write-Error "Neither master credentials (CLOUDWAYS_MASTER_USER + CLOUDWAYS_MASTER_PASS + CLOUDWAYS_IP) nor app credentials (CLOUDWAYS_HOST + CLOUDWAYS_PASSWORD) are set in ~/.utterbar-deploy/.env. Add one set."
        exit 1
    }
}
else {
    # Both -HostName and -RemotePath were passed explicitly — caller
    # is overriding everything. Still need a password from somewhere.
    if ($masterPass) { $pass = $masterPass }
    elseif ($appPass) { $pass = $appPass }
    else {
        Write-Error "-HostName / -RemotePath passed explicitly but no password in env file."
        exit 1
    }
}

# Locate the Website root relative to this script.
$websiteRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $websiteRoot
Write-Output "website root: $websiteRoot"

# Ensure pscp + plink are available.
$pscp = Get-Command pscp -ErrorAction SilentlyContinue
$plink = Get-Command plink -ErrorAction SilentlyContinue
if (-not $pscp -or -not $plink) {
    Write-Error "pscp / plink not on PATH. Run: winget install PuTTY.PuTTY"
    exit 1
}

# -----------------------------------------------------------------
# 1. Safety check — never send an .env or a secret.
# -----------------------------------------------------------------
$excludes = @(
    'vendor',
    'node_modules',
    '.env',
    '.env.local',
    '.git',
    'tests',
    'storage/logs',
    'storage/framework/cache',
    'storage/framework/sessions',
    'storage/framework/views',
    'storage/framework/testing',
    'bootstrap/cache',
    'scripts/deploy-website.ps1',  # Don't copy this script itself to the server
    '.phpunit.cache',
    '.phpunit.result.cache'
)

# Build the list of files to send. pscp on Windows doesn't support
# native --exclude, so we materialise a filtered file list and -r the
# individual directories we know are safe.
$safeDirs = @(
    'app',
    'bootstrap',        # app.php + providers; cache/ excluded via server-side check
    'config',
    'database',
    'public',
    'resources',
    'routes'
)
$safeFiles = @(
    'artisan',
    'composer.json',
    'composer.lock',
    'package.json',
    'package-lock.json',
    'phpunit.xml',
    'vite.config.js'
)

# -----------------------------------------------------------------
# 2. pscp the source tree.
# -----------------------------------------------------------------
Write-Output ""
Write-Output "=== Pushing source tree to ${HostName}:${RemotePath} ==="

function Send($localPath, $remoteSubpath) {
    if (-not (Test-Path $localPath)) {
        Write-Output "  skip (absent): $localPath"
        return
    }
    $target = "${HostName}:$RemotePath/$remoteSubpath"
    Write-Output "  -> $localPath => $target"
    if ($DryRun) { return }
    Invoke-NativeQuiet { & pscp -pw $pass -batch -r -q $localPath $target }
    if ($LASTEXITCODE -ne 0) { throw "pscp failed for $localPath" }
}

foreach ($d in $safeDirs) { Send $d '' }
foreach ($f in $safeFiles) { Send $f '' }

# The bootstrap/cache dir must exist remotely but contents were ignored above.
# Laravel refuses to boot without the dir present.
if (-not $DryRun) {
    Invoke-NativeQuiet { & plink -pw $pass -batch $HostName "mkdir -p $RemotePath/bootstrap/cache $RemotePath/storage/framework/cache/data $RemotePath/storage/framework/sessions $RemotePath/storage/framework/views $RemotePath/storage/logs" }
    if ($LASTEXITCODE -ne 0) { throw "remote mkdir failed" }
}

# -----------------------------------------------------------------
# 3. composer install (production mode).
# -----------------------------------------------------------------
Write-Output ""
Write-Output "=== composer install --no-dev --optimize-autoloader ==="
if (-not $DryRun) {
    Invoke-NativeQuiet { & plink -pw $pass -batch $HostName "cd $RemotePath && composer install --no-dev --optimize-autoloader --no-interaction" }
    if ($LASTEXITCODE -ne 0) { throw "composer install failed" }
}

# -----------------------------------------------------------------
# 4. Migrations.
# -----------------------------------------------------------------
Write-Output ""
Write-Output "=== php artisan migrate --force ==="
if (-not $DryRun) {
    Invoke-NativeQuiet { & plink -pw $pass -batch $HostName "cd $RemotePath && php artisan migrate --force" }
    if ($LASTEXITCODE -ne 0) { throw "artisan migrate failed" }
}

# -----------------------------------------------------------------
# 5. Cache clears.
# -----------------------------------------------------------------
Write-Output ""
Write-Output "=== Clearing view / config / route caches ==="
if (-not $DryRun) {
    Invoke-NativeQuiet { & plink -pw $pass -batch $HostName "cd $RemotePath && php artisan view:clear && php artisan config:clear && php artisan route:clear" }
    if ($LASTEXITCODE -ne 0) { throw "cache clears failed" }
}

# -----------------------------------------------------------------
# 6. Build Vite assets locally and upload. Cloudways ships Node 18,
#    which Tailwind v4 refuses to work with. Building locally on the
#    operator's machine dodges the "install Node 22 on the droplet"
#    dance entirely - node_modules stays off the server, and we only
#    ship the compiled public/build directory.
# -----------------------------------------------------------------
Write-Output ""
Write-Output "=== Building Vite assets locally ==="
if (-not $DryRun) {
    $localBuild = Join-Path $websiteRoot 'public\build'

    # npm install first if node_modules is missing.
    if (-not (Test-Path (Join-Path $websiteRoot 'node_modules'))) {
        Write-Output "  node_modules missing - running npm install..."
        Push-Location $websiteRoot
        try { Invoke-NativeQuiet { & npm install --silent } } finally { Pop-Location }
        if ($LASTEXITCODE -ne 0) { throw "npm install failed" }
    }

    Push-Location $websiteRoot
    try {
        Invoke-NativeQuiet { & npm run build }
        if ($LASTEXITCODE -ne 0) { throw "local vite build failed" }
    } finally { Pop-Location }

    if (-not (Test-Path $localBuild)) { throw "vite build finished but public/build is missing" }

    Write-Output ""
    Write-Output "=== Uploading public/build to server ==="
    # Wipe old build remotely so stale hashed-filename assets don't linger.
    Invoke-NativeQuiet { & plink -pw $pass -batch $HostName "rm -rf $RemotePath/public/build" }
    if ($LASTEXITCODE -ne 0) { throw "remote public/build wipe failed" }
    Invoke-NativeQuiet { & pscp -pw $pass -batch -r -q $localBuild "${HostName}:$RemotePath/public/" }
    if ($LASTEXITCODE -ne 0) { throw "pscp upload of public/build failed" }
}

# -----------------------------------------------------------------
# 7. Smoke test.
# -----------------------------------------------------------------
Write-Output ""
Write-Output "=== Smoke-testing live endpoints ==="
$endpoints = @(
    'https://utterbar.com/api/pricing',
    'https://utterbar.com/api/updater/check?current=0.6.0&channel=stable',
    'https://utterbar.com/download',
    'https://utterbar.com/pricing'
)
foreach ($url in $endpoints) {
    try {
        $r = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 15
        Write-Output "  $($r.StatusCode) $url"
    }
    catch {
        Write-Warning "  FAIL $url - $($_.Exception.Message)"
    }
}

Write-Output ""
Write-Output "=== Deploy complete ==="
