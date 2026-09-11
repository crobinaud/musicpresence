<#
.SYNOPSIS
    Builds the release binary and compiles the Windows Installer with Inno Setup.
.DESCRIPTION
    Requires Inno Setup 6 (ISCC.exe) and Rust toolchain.
#>

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path "$PSScriptRoot\.."
Set-Location $repoRoot

Write-Host "🔨 [1/2] Compiling MusicPresence in release mode (cargo build --release)..." -ForegroundColor Cyan
cargo build --release

# Locate Inno Setup Compiler (ISCC)
$isccPaths = @(
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe"
)

$iscc = $null
foreach ($path in $isccPaths) {
    if (Test-Path $path) {
        $iscc = $path
        break
    }
}

if (-not $iscc) {
    $cmd = Get-Command iscc.exe -ErrorAction SilentlyContinue
    if ($cmd) { $iscc = $cmd.Source }
}

if (-not $iscc) {
    Write-Error "❌ Inno Setup (ISCC.exe) not found! Please install it via 'winget install JRSoftware.InnoSetup' or visit https://jrsoftware.org/isinfo.php"
    exit 1
}

# Read version from Cargo.toml
$cargoToml = Get-Content "Cargo.toml" -Raw
if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
    $version = $matches[1]
} else {
    $version = "1.0.0"
}

Write-Host "📦 [2/2] Generating Windows Installer (Inno Setup v$version)..." -ForegroundColor Cyan
& $iscc "/DAppVersion=$version" "installer\installer.iss"

if ($LASTEXITCODE -eq 0 -and (Test-Path "MusicPresence-Setup.exe")) {
    Write-Host "✅ Installer created successfully: MusicPresence-Setup.exe" -ForegroundColor Green
    Write-Host "👉 Run '.\MusicPresence-Setup.exe' to install MusicPresence on your computer!" -ForegroundColor Yellow
} else {
    Write-Error "❌ Failed to generate installer."
}
