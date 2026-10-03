$ErrorActionPreference='Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
# Use the same verified extraction code as setup; no MSI system installation.
& powershell -ExecutionPolicy Bypass -File scripts/test-setup.ps1 -PrepareOnly
if($LASTEXITCODE -ne 0){throw 'Engine preparation tool compilation failed'}
$workspace=(Get-Location).Path
$staging=Join-Path $workspace ('.cache\prepare-'+[guid]::NewGuid().ToString('N'))
$target=Join-Path $workspace 'src-tauri\binaries'
$backup=Join-Path $staging 'previous-binaries'
try {
 & .cache\PrepareEngines.exe engines.lock.json installer/engine-files.json $target $staging
 if($LASTEXITCODE -ne 0){throw 'Verified engine preparation failed; existing development engines were preserved'}
 if(Test-Path -LiteralPath $target){Move-Item -LiteralPath $target -Destination $backup}
 try {Move-Item -LiteralPath (Join-Path $staging 'binaries') -Destination $target}
 catch {if(Test-Path -LiteralPath $backup){Move-Item -LiteralPath $backup -Destination $target};throw}
 Write-Host 'Verified processing engines ready.'
} finally {
 if(Test-Path -LiteralPath $staging){
  $resolved=(Resolve-Path -LiteralPath $staging).Path
  if(!$resolved.StartsWith((Join-Path $workspace '.cache\prepare-'),[StringComparison]::OrdinalIgnoreCase)){throw 'Unsafe staging path'}
  Remove-Item -LiteralPath $resolved -Recurse -Force
 }
}
