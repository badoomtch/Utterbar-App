# ----------------------------------------------------------------------
# rotate-licence-key.ps1 - one-shot Ed25519 keypair rotation.
#
# Usage (from any PowerShell prompt):
#
#     .\scripts\rotate-licence-key.ps1
#
# What it does:
#   1. Generates a new Ed25519 keypair locally via PHP + libsodium.
#   2. Edits src/services/licence/verify.rs to embed the new PUBLIC_KEY_BYTES.
#   3. SSHs to the Cloudways server and rewrites the two LICENCE_*_KEY_B64
#      lines in .env (append if missing, replace if present).
#   4. php artisan config:clear on the server so the new key takes effect.
#   5. php artisan licence:remint --all --non-interactive so every existing
#      licence row has a signed_blob that verifies under the new key.
#   6. Runs `pnpm tauri build` so the desktop installer rebaked with the new
#      public key is produced at src-tauri/target/release/bundle/nsis/.
#
# What it does NOT do:
#   - Publish a release. The runbook requires Jay-tests-first. When you're
#     ready to ship, run scripts/publish-release.sh from Utterbar Windows App/
#     against the new installer.
#   - Install the new app on this machine. You have to run the installer
#     yourself; the old app still has the old public key baked in.
#   - Commit anything. You review the verify.rs diff and commit when happy.
#
# Credential source:
#   ~/.utterbar-deploy/.env  (same file deploy-website.ps1 uses)
#   Needs CLOUDWAYS_MASTER_USER + CLOUDWAYS_MASTER_PASS + CLOUDWAYS_IP
#     + CLOUDWAYS_APP_FOLDER
#   OR CLOUDWAYS_HOST + CLOUDWAYS_PASSWORD
#
# Prerequisites:
#   - winget install PuTTY.PuTTY   (plink)
#   - winget install PHP.PHP.8.2   (php with ext-sodium enabled)
#   - pnpm + cargo + Tauri tooling for the rebuild step.
# ----------------------------------------------------------------------

param(
    [switch]$DryRun,    # Print everything it would do, do nothing.
    [switch]$SkipBuild  # Skip the Tauri rebuild at the end.
)

$ErrorActionPreference = 'Stop'

# ----------------------------------------------------------------------
# 0. Locate repo root + the files we touch.
# ----------------------------------------------------------------------
$repoRoot   = Resolve-Path (Join-Path $PSScriptRoot '..')
$verifyRs   = Join-Path $repoRoot 'Utterbar Windows App\src-tauri\src\services\licence\verify.rs'
$tauriRoot  = Join-Path $repoRoot 'Utterbar Windows App'

if (-not (Test-Path $verifyRs)) {
    Write-Error "verify.rs not found at $verifyRs - is this the Utterbar repo?"
    exit 1
}

Write-Host ""
Write-Host "Utterbar licence key rotation" -ForegroundColor Cyan
Write-Host "=============================" -ForegroundColor Cyan
Write-Host "Repo:     $repoRoot"
Write-Host "verify.rs $verifyRs"
Write-Host ""

# ----------------------------------------------------------------------
# 1. Credential loading (same pattern as deploy-website.ps1).
# ----------------------------------------------------------------------
function Read-EnvValue($key) {
    $envFile = Join-Path $env:USERPROFILE '.utterbar-deploy\.env'
    if (-not (Test-Path $envFile)) { return $null }
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

$masterUser = Read-EnvValue 'CLOUDWAYS_MASTER_USER'
$masterPass = Read-EnvValue 'CLOUDWAYS_MASTER_PASS'
$masterIp   = Read-EnvValue 'CLOUDWAYS_IP'
$appFolder  = Read-EnvValue 'CLOUDWAYS_APP_FOLDER'
$appHost    = Read-EnvValue 'CLOUDWAYS_HOST'
$appPass    = Read-EnvValue 'CLOUDWAYS_PASSWORD'

if ($masterUser -and $masterPass -and $masterIp -and $appFolder) {
    $sshHost    = "${masterUser}@${masterIp}"
    $remotePath = "applications/${appFolder}/public_html"
    $pass       = $masterPass
} elseif ($appHost -and $appPass) {
    $sshHost    = $appHost
    $remotePath = 'public_html'
    $pass       = $appPass
} else {
    Write-Error "No Cloudways credentials in ~/.utterbar-deploy/.env. See deploy-website.ps1 for the expected keys."
    exit 1
}

Write-Host "SSH host: $sshHost"
Write-Host "Remote:   $remotePath"
Write-Host ""

# ----------------------------------------------------------------------
# 2. Ensure plink is on PATH. Fail fast.
# ----------------------------------------------------------------------
$plink = Get-Command plink -ErrorAction SilentlyContinue
if (-not $plink) {
    Write-Error "plink not on PATH. Run: winget install PuTTY.PuTTY"
    exit 1
}

# ----------------------------------------------------------------------
# 3. Generate a new Ed25519 keypair locally via PHP.
# ----------------------------------------------------------------------
$php = Get-Command php -ErrorAction SilentlyContinue
if (-not $php) {
    Write-Error "php not on PATH. Run: winget install PHP.PHP.8.2 (and enable ext-sodium in php.ini)"
    exit 1
}

Write-Host "1. Generating Ed25519 keypair locally..." -ForegroundColor Yellow

# Write the PHP to a temp file and execute with `php <file>`. Running
# multi-line PHP via `php -r` mangles on Windows because cmd/PS quoting
# collapses the linebreaks into a single line that PHP then mis-parses.
$phpSnippet = @'
<?php
if (!function_exists('sodium_crypto_sign_keypair')) { fwrite(STDERR, "libsodium missing"); exit(2); }
$kp = sodium_crypto_sign_keypair();
$sk = sodium_crypto_sign_secretkey($kp);
$pk = sodium_crypto_sign_publickey($kp);
echo base64_encode($sk) . "\t" . base64_encode($pk) . "\t" . bin2hex($pk);
sodium_memzero($sk);
'@

$phpTmp = Join-Path $env:TEMP ("utterbar-keygen-{0}.php" -f [guid]::NewGuid())
try {
    [System.IO.File]::WriteAllText($phpTmp, $phpSnippet, [System.Text.UTF8Encoding]::new($false))
    $phpOut = & php $phpTmp 2>&1
    $phpExit = $LASTEXITCODE
} finally {
    Remove-Item $phpTmp -Force -ErrorAction SilentlyContinue
}

if ($phpExit -ne 0) {
    Write-Error "PHP keygen failed: $phpOut"
    exit 1
}

$parts = $phpOut -split "`t"
if ($parts.Count -ne 3) {
    Write-Error "Unexpected PHP output shape: $phpOut"
    exit 1
}
$skB64 = $parts[0]
$pkB64 = $parts[1]
$pkHex = $parts[2]

if ($skB64.Length -lt 60 -or $pkB64.Length -lt 40 -or $pkHex.Length -ne 64) {
    Write-Error "Generated key has wrong length. sk_b64=$($skB64.Length), pk_b64=$($pkB64.Length), pk_hex=$($pkHex.Length)"
    exit 1
}

Write-Host "   New public key (hex):    $pkHex" -ForegroundColor Green
Write-Host "   New public key (base64): $pkB64" -ForegroundColor Green
Write-Host "   (Private key staying in memory. Not echoed.)"
Write-Host ""

# ----------------------------------------------------------------------
# 4. Rewrite verify.rs with the new public key bytes.
#    Looks for the `pub const PUBLIC_KEY_BYTES: [u8; 32] = [ ... ];` block
#    and replaces the 32 hex bytes inside.
# ----------------------------------------------------------------------
Write-Host "2. Patching verify.rs..." -ForegroundColor Yellow

$verifySrc = Get-Content $verifyRs -Raw

# Convert hex string to Rust array literal: 0xff, 0x56, ..., 16 bytes per line.
$bytes = for ($i = 0; $i -lt 64; $i += 2) { '0x' + $pkHex.Substring($i, 2) }
$line1 = ($bytes[0..15] -join ', ')
$line2 = ($bytes[16..31] -join ', ')
$rustArray = "    $line1,`n    $line2,"

$pattern = '(?s)(pub const PUBLIC_KEY_BYTES: \[u8; 32\] = \[)\s*.*?\s*(\];)'
if ($verifySrc -notmatch $pattern) {
    Write-Error "Could not find PUBLIC_KEY_BYTES block in verify.rs. Abort."
    exit 1
}
$newSrc = [regex]::Replace($verifySrc, $pattern, "`$1`n$rustArray`n`$2")

if ($DryRun) {
    Write-Host "   [dry-run] would write new verify.rs"
} else {
    # UTF-8 no BOM to match what the file already is.
    [System.IO.File]::WriteAllText($verifyRs, $newSrc, [System.Text.UTF8Encoding]::new($false))
    Write-Host "   verify.rs updated." -ForegroundColor Green
}
Write-Host ""

# ----------------------------------------------------------------------
# 5. SSH to the server and update .env. Detect-and-append behaviour:
#    - If the line is there, replace it in-place.
#    - If missing, append it.
#    Uses a heredoc of shell commands, piped to plink.
# ----------------------------------------------------------------------
Write-Host "3. Updating server .env via SSH..." -ForegroundColor Yellow

# Build a remote shell script. The shell body is single-quoted here-string
# so PowerShell doesn't touch $KV / $KEY / $VAL / $ESC - bash gets them
# verbatim. The two base64 values and the remote path are injected by
# .Replace() afterwards using sentinels that would never appear in base64.
$remoteTemplate = @'
set -e
cd '__REMOTE_PATH__'
touch .env
for KV in 'LICENCE_PRIVATE_KEY_B64=__SK_B64__' 'LICENCE_PUBLIC_KEY_B64=__PK_B64__'; do
    KEY=${KV%%=*}
    VAL=${KV#*=}
    if grep -q -E "^${KEY}=" .env; then
        # Line exists - replace. Use | as sed delimiter so / in base64 isn't a problem.
        # VAL can still contain | or & or \ - escape those for sed's RHS.
        ESC=$(printf '%s' "${VAL}" | sed -e 's/[|&\\]/\\&/g')
        sed -i "s|^${KEY}=.*|${KEY}=${ESC}|" .env
    else
        echo "${KV}" >> .env
    fi
done
php artisan config:clear >/dev/null
echo ENV_UPDATED
'@

$remoteScript = $remoteTemplate.Replace('__REMOTE_PATH__', $remotePath).Replace('__SK_B64__', $skB64).Replace('__PK_B64__', $pkB64)

# Normalise to LF only - bash on Linux chokes on trailing \r in variable expansions.
$remoteScript = $remoteScript -replace "`r`n", "`n"

if ($DryRun) {
    Write-Host "   [dry-run] would run over SSH:"
    Write-Host $remoteScript
} else {
    # Upload the shell script to the server via pscp, then execute it via
    # plink. Piping the script directly on PowerShell's stdin prepends a
    # UTF-8 BOM that bash reads as the first 3 chars of 'set -e', which
    # made the earlier version fail loudly (but only on line 1).
    $pscp = Get-Command pscp -ErrorAction SilentlyContinue
    if (-not $pscp) {
        Write-Error "pscp not on PATH. Run: winget install PuTTY.PuTTY"
        exit 1
    }
    $scriptTmpLocal  = Join-Path $env:TEMP ("utterbar-rotate-{0}.sh" -f [guid]::NewGuid())
    $scriptTmpRemote = "/tmp/utterbar-rotate-{0}.sh" -f [guid]::NewGuid()
    try {
        # UTF-8 no BOM. Line endings are already LF (normalised above).
        [System.IO.File]::WriteAllText($scriptTmpLocal, $remoteScript, (New-Object System.Text.UTF8Encoding $false))

        & pscp -pw $pass -batch -q $scriptTmpLocal "${sshHost}:${scriptTmpRemote}"
        if ($LASTEXITCODE -ne 0) {
            Write-Error "pscp upload of shell script failed."
            exit 1
        }
        $remoteOut = & plink -pw $pass -batch $sshHost "bash $scriptTmpRemote; rm -f $scriptTmpRemote" 2>&1
    } finally {
        Remove-Item $scriptTmpLocal -Force -ErrorAction SilentlyContinue
    }
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Server .env update failed. plink output: $remoteOut"
        exit 1
    }
    if ($remoteOut -notmatch 'ENV_UPDATED') {
        Write-Error "Server did not report ENV_UPDATED. Output: $remoteOut"
        exit 1
    }
    Write-Host "   .env patched + config cache cleared." -ForegroundColor Green
}
Write-Host ""

# ----------------------------------------------------------------------
# 6. Re-mint every licence blob against the new key.
# ----------------------------------------------------------------------
Write-Host "4. Re-minting all licences on the server..." -ForegroundColor Yellow

if ($DryRun) {
    Write-Host "   [dry-run] would run: php artisan licence:remint --all --non-interactive"
} else {
    $remintOut = & plink -pw $pass -batch $sshHost "cd '$remotePath' && php artisan licence:remint --all --non-interactive"
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Re-mint failed. Output: $remintOut"
        exit 1
    }
    Write-Host $remintOut
    Write-Host "   Licences re-minted." -ForegroundColor Green
}
Write-Host ""

# ----------------------------------------------------------------------
# 7. Rebuild the desktop installer.
# ----------------------------------------------------------------------
if ($SkipBuild) {
    Write-Host "5. SKIPPING desktop build (per -SkipBuild)." -ForegroundColor Yellow
    Write-Host ""
} elseif ($DryRun) {
    Write-Host "5. [dry-run] would run: pnpm tauri build in $tauriRoot"
    Write-Host ""
} else {
    Write-Host "5. Rebuilding desktop installer (pnpm tauri build)..." -ForegroundColor Yellow
    Write-Host "   This takes ~7 minutes."

    Push-Location $tauriRoot
    try {
        & pnpm tauri build
        if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }
    } finally {
        Pop-Location
    }

    $installer = Get-ChildItem (Join-Path $tauriRoot 'src-tauri\target\release\bundle\nsis') -Filter '*.exe' -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($installer) {
        Write-Host "   Installer: $($installer.FullName)" -ForegroundColor Green
    } else {
        Write-Warning "   Build succeeded but no installer found in src-tauri\target\release\bundle\nsis\. Check the output above."
    }
    Write-Host ""
}

# ----------------------------------------------------------------------
# 8. Summary + next steps.
# ----------------------------------------------------------------------
Write-Host "DONE." -ForegroundColor Cyan
Write-Host ""
Write-Host "Public key baked into verify.rs:"
Write-Host "  $pkHex"
Write-Host ""
Write-Host "Review + commit:"
Write-Host "  git diff 'Utterbar Windows App/src-tauri/src/services/licence/verify.rs'"
Write-Host ""
Write-Host "Your existing Utterbar install still trusts the OLD key."
Write-Host "Run the installer above to pick up the new public key, then"
Write-Host "re-activate your licence (the app will prompt)."
Write-Host ""
Write-Host "When ready to release, bump version + run:"
Write-Host "  cd 'Utterbar Windows App'"
Write-Host "  ./scripts/publish-release.sh <new-version> <installer-path> releases/<new-version>.md"
