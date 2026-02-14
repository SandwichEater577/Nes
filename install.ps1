# Nes Installer (PowerShell)
# Run: .\install.ps1
# Or:  irm https://raw.githubusercontent.com/SandwichEater577/Nes/main/install.ps1 | iex

$ErrorActionPreference = "Stop"
$InstallDir = "$env:USERPROFILE\.nes"
$Bin = "$InstallDir\nes.exe"
$RepoRoot = $PSScriptRoot

Write-Host ""
Write-Host "  nes — the nestea shell installer" -ForegroundColor Yellow
Write-Host "  =================================" -ForegroundColor Yellow
Write-Host ""

# Step 1: Get the binary
$localExe = Join-Path $RepoRoot "nes.exe"
$found = $false

if (Test-Path $localExe) {
    Write-Host "[1/3] Found pre-built nes.exe" -ForegroundColor Cyan
    $found = $true
}

if (-not $found) {
    # Try to build from source
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "[1/3] Building nes from source..." -ForegroundColor Cyan
        $cargoToml = Join-Path $RepoRoot "Cargo.toml"
        if (Test-Path $cargoToml) {
            Push-Location $RepoRoot
            cargo build --release 2>&1 | Out-Null
            $exitCode = $LASTEXITCODE

            # Find the built binary
            $builtExe = Get-ChildItem -Path (Join-Path $RepoRoot "target") -Filter "nes.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
            Pop-Location

            if ($builtExe) {
                Copy-Item $builtExe.FullName $localExe -Force
                $found = $true
                Write-Host "       Build complete." -ForegroundColor Green
            } elseif ($exitCode -ne 0) {
                Write-Host "ERROR: Build failed." -ForegroundColor Red
                Write-Host "       Make sure Rust is installed: https://rustup.rs" -ForegroundColor Red
                exit 1
            }
        }
    }
}

if (-not $found) {
    # Try downloading from GitHub releases
    Write-Host "[1/3] Downloading nes.exe from GitHub..." -ForegroundColor Cyan
    $url = "https://github.com/SandwichEater577/Nes/releases/latest/download/nes.exe"
    try {
        Invoke-WebRequest -Uri $url -OutFile $localExe -UseBasicParsing
        $found = $true
        Write-Host "       Downloaded." -ForegroundColor Green
    } catch {
        Write-Host "ERROR: Could not download nes.exe" -ForegroundColor Red
        Write-Host "       Install Rust (https://rustup.rs) and run again, or" -ForegroundColor Red
        Write-Host "       place nes.exe next to this script." -ForegroundColor Red
        exit 1
    }
}

# Step 2: Install
Write-Host "[2/3] Installing to $InstallDir" -ForegroundColor Cyan
if (-not (Test-Path $InstallDir)) { New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null }
Copy-Item $localExe $Bin -Force

# Step 3: Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*\.nes*") {
    Write-Host "[3/3] Adding to PATH..." -ForegroundColor Cyan
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$InstallDir", "User")
    $env:PATH = "$env:PATH;$InstallDir"
    Write-Host "       Added. Restart your terminal for it to take effect." -ForegroundColor Yellow
} else {
    Write-Host "[3/3] PATH already configured." -ForegroundColor Green
}

Write-Host ""
Write-Host "  Done! Nes is installed." -ForegroundColor Green
Write-Host "  Open a new terminal and type: nes help" -ForegroundColor White
Write-Host ""
