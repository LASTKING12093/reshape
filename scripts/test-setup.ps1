param([switch]$PrepareOnly)
$ErrorActionPreference='Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
New-Item -ItemType Directory -Force .cache | Out-Null
$compiler=Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\csc.exe'
$entry=if($PrepareOnly){'PrepareEngines'}else{'EngineSetupTests'}
& $compiler /nologo /target:exe /optimize+ "/out:.cache/$entry.exe" /reference:System.Web.Extensions.dll /reference:System.IO.Compression.dll /reference:System.IO.Compression.FileSystem.dll (Join-Path (Get-Location) 'installer\EngineSetup.cs') (Join-Path (Get-Location) "installer\$entry.cs")
if($LASTEXITCODE -ne 0){throw 'Engine tool compilation failed'}
if(!$PrepareOnly){& ".cache/$entry.exe";if($LASTEXITCODE -ne 0){throw 'Installer tests failed'}}

