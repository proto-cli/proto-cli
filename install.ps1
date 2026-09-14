# Proto CLI Windows installer
# Usage:
#   iex (iwr -Uri https://raw.githubusercontent.com/proto-cli/proto-cli/master/install.ps1 -UseBasicParsing).Content
#   .\install.ps1 -Prefix C:\tools\bin
#   .\install.ps1 -Version v0.2.2 -Force

[CmdletBinding()]
param(
    [string]$Prefix = "$env:LOCALAPPDATA\Proto\bin",
    [string]$Version = "",
    [switch]$PrintTarget,
    [switch]$DryRun,
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$REPO        = "proto-cli/proto-cli"
$RELEASES_API = "https://api.github.com/repos/$REPO/releases"
$DOWNLOAD_BASE = "https://github.com/$REPO/releases/download"
$BIN_NAME    = "proto"

# ── colours ──────────────────────────────────────────────────────────────────

function Write-Info    { param($Msg) Write-Host "  ◆ " -NoNewline -ForegroundColor Cyan; Write-Host $Msg }
function Write-Success { param($Msg) Write-Host "  ✔ " -NoNewline -ForegroundColor Green; Write-Host $Msg }
function Write-Warn    { param($Msg) Write-Host "  ⚠ " -NoNewline -ForegroundColor Yellow; Write-Host $Msg }
function Write-Err     { param($Msg) Write-Host "  ✗ " -NoNewline -ForegroundColor Red;    Write-Host $Msg }
function Write-Sep     { Write-Host "`n  ──────────────────────────────────────────────`n" -ForegroundColor DarkGray }

# ── banner ───────────────────────────────────────────────────────────────────

function Show-Banner {
    Write-Host ""
    Write-Host "     ⣀⡀     " -ForegroundColor Cyan
    Write-Host "⢠⣤⡀⣾⣿⣿⠀⣤⣤⡄" -ForegroundColor Cyan
    Write-Host "⢿⣿⡇⠘⠛⠁⢸⣿⣿⠃" -ForegroundColor Cyan
    Write-Host "⠈⣉⣤⣾⣿⣿⡆⠉⣴⣶⣶" -ForegroundColor Cyan
    Write-Host "⣾⣿⣿⣿⣿⣿⣿⡀⠻⠟⠃" -ForegroundColor Cyan
    Write-Host "⠙⠛⠻⢿⣿⣿⣿⡇  " -ForegroundColor Cyan
    Write-Host "     ⠈⠙⠋⠁  " -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Proto CLI " -NoNewline -ForegroundColor White
    Write-Host "installer" -ForegroundColor Gray
    Write-Host "  Your friendly protogen CLI companion" -ForegroundColor DarkCyan
}

# ── target detection ─────────────────────────────────────────────────────────

function Get-Target {
    if (-not [System.Environment]::Is64BitOperatingSystem) {
        Write-Err "32-bit Windows is not supported. No prebuilt binary is available."
        exit 1
    }

    # ARM64 Windows (native) is built on PROCESSOR_ARCHITECTURE;
    # ARM64 under x64 emulation reports PROCESSOR_ARCHITEW6432 = "ARM64".
    $procArch = $env:PROCESSOR_ARCHITECTURE
    if ($procArch -eq "ARM64" -or $env:PROCESSOR_ARCHITEW6432 -eq "ARM64") {
        return "aarch64-pc-windows-msvc.exe"
    }
    return "x86_64-pc-windows-msvc.exe"
}

# ── version resolution ───────────────────────────────────────────────────────

function Resolve-Version {
    if ($Version) {
        if (-not $Version.StartsWith("v")) { $Version = "v$Version" }
        return $Version
    }

    try {
        $headers = @{}
        # Use a token if available to avoid rate-limiting
        if ($env:GITHUB_TOKEN) { $headers["Authorization"] = "token $env:GITHUB_TOKEN" }

        $resp = Invoke-RestMethod -Uri $RELEASES_API -Headers $headers -UseBasicParsing -ErrorAction Stop
        $latest = ($resp | Where-Object { -not $_.prerelease } |
                   Sort-Object { [version]($_.tag_name -replace '^v','') } -Descending |
                   Select-Object -First 1)

        if (-not $latest) {
            Write-Err "Could not determine the latest release version."
            Write-Err "Check your network connection, or pass one explicitly with -Version."
            exit 1
        }
        return $latest.tag_name
    } catch {
        Write-Err "Could not reach GitHub API: $($_.Exception.Message)"
        Write-Err "Check your network connection, or pass one explicitly with -Version."
        exit 1
    }
}

# ── SHA-256 verification ─────────────────────────────────────────────────────

function Test-Checksum {
    param([string]$FilePath, [string]$Expected)

    $hash = (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash.ToLower()

    if ($Expected) {
        $Expected = $Expected.ToLower()
        if ($hash -ne $Expected) {
            Write-Err "Checksum verification failed."
            Write-Err "  expected: $Expected"
            Write-Err "  actual:   $hash"
            Write-Err "Installation aborted for security reasons."
            exit 1
        }
        Write-Success "Checksum verified"
    } else {
        Write-Warn "No published checksum found; skipping verification."
    }
}

# ── main install logic ──────────────────────────────────────────────────────

function Install-Proto {
    $target = Get-Target
    $version = Resolve-Version

    Write-Sep
    Write-Info "Latest release: $version"
    Write-Info "Target: $target"
    Write-Info "Install dir: $Prefix"
    Write-Sep

    $asset   = "proto-$target"
    $url     = "$DOWNLOAD_BASE/$version/$asset"
    $tmpDir  = Join-Path $env:TEMP "proto-install-$(Get-Random)"

    try {
        New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null
        $zipFile = Join-Path $tmpDir $asset

        # Download binary
        Write-Info "Downloading $asset ($version)..."
        try {
            $headers = @{}
            if ($env:GITHUB_TOKEN) { $headers["Authorization"] = "token $env:GITHUB_TOKEN" }
            Invoke-WebRequest -Uri $url -OutFile $zipFile -Headers $headers -UseBasicParsing -ErrorAction Stop
        } catch {
            Write-Err "Failed to download $url"
            Write-Err "The binary for your platform may not be published for this release yet."
            exit 1
        }

        # Download and verify checksum
        $shaUrl = "$url.sha256"
        $expected = ""
        try {
            $shaContent = (Invoke-WebRequest -Uri $shaUrl -Headers $headers -UseBasicParsing -ErrorAction Stop).Content
            # Format: "<hash>  <filename>" or just "<hash>"
            $expected = ($shaContent -split '\s+')[0].Trim()
        } catch {
            # No checksum available
        }

        Test-Checksum -FilePath $zipFile -Expected $expected

        # "Install" — copy to prefix
        if ($DryRun) {
            Write-Success "Dry-run: verified binary ready at $zipFile"
            Write-Info "Would install to $Prefix\$BIN_NAME.exe"
            return
        }

        if (-not (Test-Path $Prefix)) {
            New-Item -ItemType Directory -Path $Prefix -Force | Out-Null
        }

        $dest = Join-Path $Prefix "$BIN_NAME.exe"
        Copy-Item -Path $zipFile -Destination $dest -Force

        Write-Success "Installed: $dest"

        # Check if prefix is on PATH
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$Prefix*") {
            Write-Warn "$Prefix is not in your PATH."
            Write-Host ""
            Write-Host "  Add it to your PATH for this session:" -ForegroundColor Gray
            Write-Host ('    $env:PATH += ";{0}"' -f $Prefix) -ForegroundColor Cyan
            Write-Host ""
            Write-Host "  Add it permanently (user-level):" -ForegroundColor Gray
            Write-Host ('    [Environment]::SetEnvironmentVariable("Path", "$env:PATH;{0}", "User")' -f $Prefix) -ForegroundColor Cyan
            Write-Host ""
            Write-Host "  Or re-run this script with -Force to add it automatically." -ForegroundColor Gray
        }

        if ($Force -and ($userPath -notlike "*$Prefix*")) {
            $newPath = if ($userPath) { "$userPath;$Prefix" } else { $Prefix }
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            $env:PATH = "$env:PATH;$Prefix"
            Write-Success "Added $Prefix to user PATH"
        }

    } finally {
        Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# ── entry point ──────────────────────────────────────────────────────────────

if ($PrintTarget) {
    Write-Output (Get-Target)
    exit 0
}

Show-Banner
Install-Proto

Write-Sep
Write-Host "  " -NoNewline
Write-Host "✦ Proto CLI installed successfully ✦" -ForegroundColor Green
Write-Host ""
Write-Host "        proto help             Show all commands"
Write-Host "        proto system           View system information"
Write-Host "        proto completions bash --install   Shell completions"
Write-Host ""
Write-Host "  Reinstall / update anytime:" -ForegroundColor Gray
Write-Host "    " -NoNewline
Write-Host "iex (iwr -Uri https://raw.githubusercontent.com/$REPO/master/install.ps1 -UseBasicParsing).Content" -ForegroundColor Cyan
Write-Host ""
