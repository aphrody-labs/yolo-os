#Requires -Version 7
<#
.SYNOPSIS
  Install (idempotently) the YOLO Arch WSL distro from the official Arch Linux
  WSL image, then provision it with setup-in-distro.sh.

.DESCRIPTION
  1. Checks WSL >= 2.4.4 (needed for .wsl images and --install --from-file).
  2. Downloads archlinux-<Version>.wsl and its .SHA256 from the Arch mirror into
     -Cache (skipped if the cached file already matches) and verifies SHA-256.
  3. Installs it as -Name in -Location with --no-launch, unless -Name exists.
     Never touches any other distro.
  4. Runs setup-in-distro.sh as root inside the distro, restarts only this
     distro (wsl --terminate <Name>) so /etc/wsl.conf applies, and optionally
     writes receipts.

.EXAMPLE
  pwsh -File distro/wsl/install-yolo-arch.ps1 -Receipts docs/receipts/wsl/receipts.txt
#>
[CmdletBinding()]
param(
  [string]$Name = 'yolo-arch',
  [string]$Location = 'C:\wsl\yolo-arch',
  [string]$Cache = 'C:\wsl\dl',
  [string]$Version = '2026.11.01.182233',
  [string]$Mirror = 'https://geo.mirror.pkgbuild.com/wsl',
  # Cross-check against an independently published value (Microsoft
  # DistributionInfo.json lists the same image). Empty to skip.
  [string]$ExpectedSha256 = 'f8b934f77d0afcbe6e3730d6e4f378e8ee3c90d51479395004e1d92fe3b3c9b1',
  [string]$Receipts = '',
  [switch]$SkipSetup
)
$ErrorActionPreference = 'Stop'
$env:WSL_UTF8 = '1'
$here = $PSScriptRoot

function Invoke-Wsl { & wsl.exe @args; if ($LASTEXITCODE -ne 0) { throw "wsl $($args -join ' ') failed ($LASTEXITCODE)" } }

# 1. WSL version.
$wslLine = (& wsl.exe --version | Select-Object -First 1)
if ($wslLine -notmatch '(\d+)\.(\d+)\.(\d+)') { throw "cannot parse wsl --version: $wslLine" }
$wslVer = [version]"$($Matches[1]).$($Matches[2]).$($Matches[3])"
if ($wslVer -lt [version]'2.4.4') { throw "WSL $wslVer < 2.4.4" }
Write-Host "WSL $wslVer"

$distros = (& wsl.exe --list --quiet) | ForEach-Object { $_.Trim() } | Where-Object { $_ }
if ($distros -contains $Name) {
  Write-Host "distro '$Name' already registered; skipping download and install"
} else {
  # 2. Download and verify.
  New-Item -ItemType Directory -Force $Cache | Out-Null
  $file = "archlinux-$Version.wsl"
  $img = Join-Path $Cache $file
  $sumFile = "$img.SHA256"
  & curl.exe -fsSL -o $sumFile "$Mirror/$Version/$file.SHA256"
  if ($LASTEXITCODE -ne 0) { throw "checksum download failed" }
  $published = ((Get-Content $sumFile -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
  if ($ExpectedSha256 -and $published -ne $ExpectedSha256.ToLowerInvariant()) {
    throw "mirror checksum $published differs from expected $ExpectedSha256"
  }
  $have = if (Test-Path $img) { (Get-FileHash $img -Algorithm SHA256).Hash.ToLowerInvariant() } else { '' }
  if ($have -ne $published) {
    & curl.exe -fL -o $img "$Mirror/$Version/$file"
    if ($LASTEXITCODE -ne 0) { throw "image download failed" }
    $have = (Get-FileHash $img -Algorithm SHA256).Hash.ToLowerInvariant()
  }
  if ($have -ne $published) { throw "SHA-256 mismatch: got $have, published $published" }
  Write-Host "SHA-256 OK $have $file"

  # 3. Install this distro only.
  New-Item -ItemType Directory -Force $Location | Out-Null
  Invoke-Wsl --install --from-file $img --name $Name --location $Location --no-launch
}

if ($SkipSetup) { return }

# 4. Provision. The scripts are read through /mnt/c (drvfs); nothing is copied
#    except yolo-uutils-shim, which setup installs into /usr/local/sbin.
$setup = (& wsl.exe -d $Name -u root --cd / -- wslpath -a ($here -replace '\\', '/')).Trim()
Invoke-Wsl -d $Name -u root --cd / -- bash "$setup/setup-in-distro.sh"
Invoke-Wsl --terminate $Name
if ($Receipts) {
  $out = & wsl.exe -d $Name --cd ~ -- bash -l "$setup/receipts.sh"
  $out | Set-Content -Encoding utf8NoBOM $Receipts
  Write-Host "receipts written to $Receipts"
}
