param([switch]$SkipTests)
$ErrorActionPreference='Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
$lock=Get-Content engines.lock.json -Raw | ConvertFrom-Json
if($lock.'libreoffice.msi'.version -ne '26.2.6'){throw 'Release blocked: known LibreOffice document-import vulnerabilities. Update and validate the engine before packaging.'}
$env:CARGO_HOME=Join-Path (Get-Location) '.cache\cargo'
# Avoid embedding the developer's workspace or home paths in distributed binaries.
$remaps = @(('--remap-path-prefix=' + (Get-Location).Path + '=C:\build\reshape'), ('--remap-path-prefix=' + $env:USERPROFILE + '=C:\build\user'))
$env:CARGO_ENCODED_RUSTFLAGS = $remaps -join [char]31
foreach($tool in @('node','npm.cmd','cargo','rustc')) { if(!(Get-Command $tool -ErrorAction SilentlyContinue)){throw "Missing development tool: $tool"} }
function Run-Step([string]$Program,[string[]]$Arguments){ & $Program @Arguments; if($LASTEXITCODE -ne 0){throw "$Program failed with $LASTEXITCODE"} }
if(!(Test-Path node_modules)){Run-Step 'npm.cmd' @('ci')}
foreach($engine in @('ffmpeg\ffmpeg.exe','imagemagick\magick.exe','libreoffice\program\soffice.com','pandoc\pandoc.exe','7zip\7z.exe','qpdf\bin\qpdf.exe','poppler\Library\bin\pdftoppm.exe')){if(!(Test-Path (Join-Path 'src-tauri\binaries' $engine))){throw "Engine resource missing: $engine. Run scripts/prepare-engines.ps1."}}
Run-Step 'node' @('scripts/generate-registry.mjs')
Run-Step 'node' @('scripts/harden-engines.mjs')
Run-Step 'node' @('scripts/manifest.mjs')
New-Item -ItemType Directory -Force src-tauri\legal | Out-Null
Copy-Item LICENSE,THIRD_PARTY_LICENSES.md,THIRD_PARTY_NOTICES.md,engines.lock.json src-tauri\legal -Force
Run-Step 'powershell' @('-ExecutionPolicy','Bypass','-File','scripts/test-setup.ps1')
Run-Step 'node' @('node_modules/typescript/bin/tsc','--noEmit')
if(!$SkipTests){Run-Step 'node' @('scripts/fixtures.mjs');Run-Step 'node' @('--test','scripts/registry.test.mjs');Run-Step 'cargo' @('test','--offline','--manifest-path','src-tauri/Cargo.toml')}
Run-Step 'node' @('node_modules/@tauri-apps/cli/tauri.js','build')
Run-Step 'node' @('scripts/audit-binary.mjs','src-tauri/target/release/reshape.exe')
New-Item -ItemType Directory -Force release | Out-Null
$installer=Get-ChildItem src-tauri\target\release\bundle\nsis\*_x64-setup.exe | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if(!$installer){throw 'Installer not generated'}
Run-Step 'powershell' @('-ExecutionPolicy','Bypass','-File','scripts/build-setup.ps1','-Payload',$installer.FullName)
$hash=Get-FileHash release\Reshape-Setup-2.0.4.exe -Algorithm SHA256
"$($hash.Hash.ToLower())  Reshape-Setup-2.0.4.exe" | Set-Content release\SHA256SUMS.txt
Write-Host "Installer: $((Resolve-Path release\Reshape-Setup-2.0.4.exe).Path)"
Write-Host "SHA-256: $($hash.Hash)"
