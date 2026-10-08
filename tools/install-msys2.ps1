# SPDX-License-Identifier: GPL-3.0-or-later
# Copyright (C) 2026 legluondunet
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$root = $env:Z3_MSYS2_ROOT
if (-not $root -or (Test-Path -LiteralPath $root)) { throw 'MSYS2 destination is missing or already exists.' }
$temporary = Join-Path ([IO.Path]::GetTempPath()) ('Z3-MSYS2-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $temporary | Out-Null
try {
    $release = Invoke-RestMethod -Uri 'https://api.github.com/repos/msys2/msys2-installer/releases/latest' -Headers @{ 'User-Agent' = 'Z3-Launcher' } -TimeoutSec 120
    $installer = @($release.assets | Where-Object { $_.name -match '^msys2-x86_64-[0-9]+\.exe$' })
    if ($installer.Count -ne 1) { throw 'Official MSYS2 installer asset not found.' }
    $installer = $installer[0]
    $checksum = @($release.assets | Where-Object { $_.name -eq ($installer.name + '.sha256') })
    if ($checksum.Count -ne 1) { throw 'Official MSYS2 checksum not found.' }
    foreach ($asset in @($installer, $checksum[0])) {
        if (-not $asset.browser_download_url.StartsWith('https://github.com/msys2/msys2-installer/releases/download/')) {
            throw 'Unexpected MSYS2 download address.'
        }
        Write-Output ('Downloading ' + $asset.name)
        Invoke-WebRequest -UseBasicParsing -Uri $asset.browser_download_url -OutFile (Join-Path $temporary $asset.name) -TimeoutSec 300
    }
    $file = Join-Path $temporary $installer.name
    $hashText = Get-Content -LiteralPath (Join-Path $temporary $checksum[0].name) -Raw
    if ($hashText -notmatch '^\s*([a-fA-F0-9]{64})(?:\s|$)') { throw 'Invalid SHA-256 checksum file.' }
    $expected = $Matches[1]
    if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $expected) { throw 'MSYS2 SHA-256 verification failed.' }
    Write-Output 'MSYS2 SHA-256 verified. Installing...'
    & $file in --confirm-command --accept-messages --root $root | Out-Default
    if ($LASTEXITCODE -ne 0) { throw ('MSYS2 installer failed: ' + $LASTEXITCODE) }
    if (-not (Test-Path -LiteralPath (Join-Path $root 'usr/bin/bash.exe'))) { throw 'MSYS2 installation is incomplete.' }
    # Initialize the first login before invoking pacman directly from the launcher.
    & (Join-Path $root 'usr/bin/bash.exe') --login -c 'true'
    if ($LASTEXITCODE -ne 0) { throw 'MSYS2 initial login failed.' }
} finally {
    Remove-Item -LiteralPath $temporary -Recurse -Force -ErrorAction SilentlyContinue
}
