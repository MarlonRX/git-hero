# Git Hero Installer (Windows)
# PowerShell install script for Windows.
#
# Usage:
#   irm https://raw.githubusercontent.com/MarlonRX/git-hero/main/scripts/install.ps1 | iex
#
# Or download and run manually:
#   .\install.ps1

$ErrorActionPreference = "Stop"

$BINARY = "gith"
$REPO = "MarlonRX/git-hero"
$INSTALL_DIR = Join-Path $env:LOCALAPPDATA "GitHero\bin"

# Helper functions
function Write-Info    { param([string]$msg) Write-Host "  $msg" -ForegroundColor Cyan }
function Write-Ok      { param([string]$msg) Write-Host "  $msg" -ForegroundColor Green }
function Write-Warn    { param([string]$msg) Write-Host "  $msg" -ForegroundColor Yellow }
function Write-Fail    { param([string]$msg) Write-Host "  $msg" -ForegroundColor Red }
function Write-Step    { param([string]$step, [string]$msg)
    Write-Host "  $step  " -ForegroundColor White -NoNewline
    Write-Host $msg
}

Write-Host ""
Write-Info "Git Hero installer"
Write-Host "  ----------------------------------------" -ForegroundColor DarkGray
Write-Host ""

# Detect platform
$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
Write-Step "->" "Detected: Windows $arch"

# Create install directory
if (-not (Test-Path $INSTALL_DIR)) {
    New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
}

$dstExe = Join-Path $INSTALL_DIR "$BINARY.exe"
$installed = $false

# ── 1. Try downloading prebuilt binary from GitHub Releases ────────
Write-Step "->" "Trying to download prebuilt binary..."
try {
    # Get latest release version from GitHub API
    $releaseUrl = "https://api.github.com/repos/$REPO/releases/latest"
    $headers = @{ "Accept" = "application/vnd.github.v3+json" }
    $release = Invoke-RestMethod -Uri $releaseUrl -Headers $headers -UseBasicParsing
    $version = $release.tag_name -replace '^v', ''

    # Find the Windows zip asset
    $asset = $release.assets | Where-Object { $_.name -like "*windows*" -and $_.name -like "*.zip" } | Select-Object -First 1

    if ($asset) {
        $zipUrl = $asset.browser_download_url
        $zipPath = Join-Path $env:TEMP "gith-$version.zip"

        Write-Step "->" "Downloading v$version..."
        Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath -UseBasicParsing

        # Extract
        $tempDir = Join-Path $env:TEMP "gith-extract-$([System.IO.Path]::GetRandomFileName())"
        Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

        # Find and copy the exe
        $srcExe = Get-ChildItem -Path $tempDir -Filter "$BINARY.exe" -Recurse | Select-Object -First 1
        if ($srcExe) {
            Copy-Item $srcExe.FullName $dstExe -Force
            Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
            Remove-Item -Force $zipPath -ErrorAction SilentlyContinue
            Write-Ok "OK Downloaded and installed v$version"
            $installed = $true
        } else {
            Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
            Remove-Item -Force $zipPath -ErrorAction SilentlyContinue
            Write-Warn "  Zip did not contain gith.exe"
        }
    } else {
        Write-Warn "  No Windows binary found in latest release"
    }
} catch {
    Write-Warn "  Could not download prebuilt binary: $($_.Exception.Message)"
}

# ── 2. Try cargo install ───────────────────────────────────────────
if (-not $installed) {
    $hasCargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($hasCargo) {
        Write-Step "->" "Installing via cargo..."
        $cargoRoot = Join-Path $env:USERPROFILE ".local"
        try {
            $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
            & cargo install $BINARY --root $cargoRoot 2>$null
            if ($LASTEXITCODE -eq 0) {
                $cargoExe = Join-Path $cargoRoot "bin\$BINARY.exe"
                if (Test-Path $cargoExe) {
                    Copy-Item $cargoExe $dstExe -Force
                    Write-Ok "OK Installed via cargo"
                    $installed = $true
                }
            }
        } catch {
            Write-Warn "  cargo install failed"
        }
    }
}

# ── 3. Build from source ───────────────────────────────────────────
if (-not $installed) {
    $hasCargo = Get-Command cargo -ErrorAction SilentlyContinue
    $hasGit = Get-Command git -ErrorAction SilentlyContinue

    if (-not $hasCargo) {
        Write-Host ""
        Write-Fail "Rust/cargo required. Install it first:"
        Write-Host "     https://rustup.rs/" -ForegroundColor Cyan
        Write-Host ""
        exit 1
    }

    if (-not $hasGit) {
        Write-Fail "git required. Install it first."
        exit 1
    }

    $BUILD_DIR = Join-Path $env:TEMP "git-hero-build-$([System.IO.Path]::GetRandomFileName())"
    Write-Step "->" "Cloning repository..."

    # Redirect stderr to stdout to avoid PowerShell treating it as an error
    $output = & git clone --depth 1 "https://github.com/$REPO.git" (Join-Path $BUILD_DIR $BINARY) 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Fail "Clone failed"
        if (Test-Path $BUILD_DIR) { Remove-Item -Recurse -Force $BUILD_DIR }
        exit 1
    }

    Write-Step "->" "Building from source - this may take a few minutes..."
    $projectDir = Join-Path $BUILD_DIR $BINARY
    Push-Location $projectDir
    $output = & cargo build --release 2>&1
    $BUILD_EXIT = $LASTEXITCODE
    Pop-Location

    if ($BUILD_EXIT -ne 0) {
        Write-Fail "Build failed. Try manually:"
        Write-Host "     git clone https://github.com/$REPO.git"
        Write-Host "     cd gith"
        Write-Host "     cargo build --release"
        Remove-Item -Recurse -Force $BUILD_DIR -ErrorAction SilentlyContinue
        exit 1
    }

    $srcExe = Join-Path $BUILD_DIR "$BINARY\target\release\$BINARY.exe"
    Copy-Item $srcExe $dstExe -Force
    Remove-Item -Recurse -Force $BUILD_DIR -ErrorAction SilentlyContinue
    Write-Ok "OK Built and installed"
    $installed = $true
}

# ── Add to PATH if not already there ───────────────────────────────
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$INSTALL_DIR*") {
    Write-Step "->" "Adding to user PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$INSTALL_DIR;$userPath", "User")
    $env:Path = "$INSTALL_DIR;$env:Path"
    Write-Ok "OK Added to PATH - restart terminal to take effect"
}

# ── Verify ─────────────────────────────────────────────────────────
Write-Host ""
if (Test-Path $dstExe) {
    Write-Ok "OK gith installed successfully!"
    Write-Host ""
    Write-Host "  Get started:" -ForegroundColor White
    Write-Host "    $BINARY" -ForegroundColor Cyan
} else {
    Write-Fail "Installation failed."
    exit 1
}
Write-Host ""
