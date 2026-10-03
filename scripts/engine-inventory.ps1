$ErrorActionPreference='Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
$root=(Resolve-Path src-tauri/binaries).Path
$lock=Get-Content engines.lock.json -Raw | ConvertFrom-Json
$inventory=@(Get-ChildItem -LiteralPath $root -Recurse -File | Where-Object Extension -in @('.exe','.dll','.com','.pyd') | ForEach-Object {
 $relative=$_.FullName.Substring($root.Length+1).Replace('\','/')
 $id=$relative.Split('/')[0]
 $entry=$lock.PSObject.Properties.Value | Where-Object id -eq $id | Select-Object -First 1
 if(!$entry){return}
 [ordered]@{path=$relative;package=$id;packageVersion=$entry.version;fileVersion=$_.VersionInfo.FileVersion;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant();licenseScope=$entry.license;project=$entry.project;source=$entry.source;distribution='downloaded directly in supplier package';loadedBy='external engine process, not Reshape';compiledIntoReshape=$false}
})
ConvertTo-Json -InputObject $inventory -Depth 5 | Set-Content docs/engine-binary-inventory.json -Encoding UTF8
Write-Host ($inventory.Count.ToString()+' real extracted binary records')
