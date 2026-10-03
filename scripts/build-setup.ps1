param([string]$Payload, [switch]$Preview)
$ErrorActionPreference='Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
$framework = Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319'
$csc = Join-Path $framework 'csc.exe'
$wpf = Join-Path $framework 'WPF'
New-Item -ItemType Directory -Force .cache,release | Out-Null
$out = if($Preview){'.cache\Reshape-setup-design.exe'}else{'release\Reshape-Setup-2.0.4.exe'}
$argsList = @('/nologo','/codepage:65001','/target:winexe','/platform:x64','/optimize+',('/out:'+$out),'/win32icon:src-tauri/icons/reshape-app.ico','/win32manifest:installer/setup.manifest',('/reference:'+$wpf+'\PresentationFramework.dll'),('/reference:'+$wpf+'\PresentationCore.dll'),('/reference:'+$wpf+'\WindowsBase.dll'),'/reference:System.Xaml.dll','/resource:installer/setup.xaml,layout','/resource:src-tauri/icons/256.png,icon','/resource:src-tauri/icons/setup-mark.png,mark')
if(!$Preview) {
 $lock=Get-Content engines.lock.json -Raw | ConvertFrom-Json
 if($lock.'libreoffice.msi'.version -ne '26.2.6'){throw 'Release blocked: update and validate LibreOffice 26.2.6 before packaging.'}
 if(!$Payload -or !(Test-Path -LiteralPath $Payload)){throw 'NSIS payload is required.'}
 $resolved=(Resolve-Path -LiteralPath $Payload).Path
 (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash | Set-Content .cache/setup-payload.sha256
 $argsList += @(('/resource:'+$resolved+',payload'),'/resource:.cache/setup-payload.sha256,payload.sha256')
}
$argsList += @('/reference:System.Web.Extensions.dll','/reference:System.IO.Compression.dll','/reference:System.IO.Compression.FileSystem.dll','/resource:engines.lock.json,engines.lock','/resource:installer/engine-files.json,engine.files')
& $csc @argsList (Join-Path (Get-Location) 'installer\Setup.cs') (Join-Path (Get-Location) 'installer\EngineSetup.cs')
if($LASTEXITCODE -ne 0){throw 'Setup compilation failed'}
if(!$Preview){ $hash=(Get-FileHash -LiteralPath $out -Algorithm SHA256).Hash.ToLower(); "$hash  Reshape-Setup-2.0.4.exe" | Set-Content release/SHA256SUMS.txt }
Get-Item -LiteralPath $out | Select-Object FullName,Length

