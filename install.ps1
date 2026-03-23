# geodaddy installer for Windows (PowerShell)
# Usage: irm https://raw.githubusercontent.com/borabiricik/geodaddy-cli/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "borabiricik/geodaddy-cli"
$Binary = "geodaddy"
$InstallDir = "$env:USERPROFILE\.geodaddy\bin"

Write-Host "Fetching latest release..." -ForegroundColor Cyan

# Get latest version
try {
    $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $Release.tag_name
} catch {
    Write-Host "error: Could not fetch latest release. Check https://github.com/$Repo/releases" -ForegroundColor Red
    exit 1
}

# Build download URL
$Target = "x86_64-pc-windows-msvc"
$Archive = "$Binary-$Version-$Target.zip"
$Url = "https://github.com/$Repo/releases/download/$Version/$Archive"

Write-Host "Installing $Binary $Version ($Target)..." -ForegroundColor Cyan

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Download to temp
$TmpDir = New-TemporaryFile | ForEach-Object {
    Remove-Item $_
    New-Item -ItemType Directory -Path "$_.dir"
}
$ZipPath = Join-Path $TmpDir $Archive

Write-Host "Downloading $Url..." -ForegroundColor Cyan
try {
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing
} catch {
    Write-Host "error: Download failed. Is the release published?" -ForegroundColor Red
    Remove-Item -Recurse -Force $TmpDir
    exit 1
}

# Extract
Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

# Install
$BinSrc = Join-Path $TmpDir "$Binary.exe"
$BinDst = Join-Path $InstallDir "$Binary.exe"
Copy-Item -Path $BinSrc -Destination $BinDst -Force

# Cleanup
Remove-Item -Recurse -Force $TmpDir

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
    Write-Host ""
    Write-Host "Added $InstallDir to your PATH." -ForegroundColor Yellow
    Write-Host "Restart your terminal for the PATH change to take effect." -ForegroundColor Yellow
}

# Verify
Write-Host ""
Write-Host "Successfully installed $Binary $Version" -ForegroundColor Green
Write-Host "Location: $BinDst" -ForegroundColor Cyan
Write-Host ""
Write-Host "Run 'geodaddy --help' to get started." -ForegroundColor Cyan
