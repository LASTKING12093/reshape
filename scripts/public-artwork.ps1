$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
Add-Type -AssemblyName System.Drawing
$assets = Join-Path (Get-Location) '.github\assets'
$mark = [Drawing.Image]::FromFile((Join-Path (Get-Location) 'src-tauri\icons\setup-mark.png'))
$shot = [Drawing.Image]::FromFile((Join-Path $assets 'empty-state.png'))
$bitmap = New-Object Drawing.Bitmap(1280,640)
$graphics = [Drawing.Graphics]::FromImage($bitmap)
$graphics.Clear([Drawing.Color]::Black)
$graphics.SmoothingMode = [Drawing.Drawing2D.SmoothingMode]::AntiAlias
$graphics.InterpolationMode = [Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$graphics.TextRenderingHint = [Drawing.Text.TextRenderingHint]::AntiAliasGridFit
$white = New-Object Drawing.SolidBrush([Drawing.Color]::FromArgb(245,245,245))
$gray = New-Object Drawing.SolidBrush([Drawing.Color]::FromArgb(164,164,164))
$coral = New-Object Drawing.SolidBrush([Drawing.Color]::FromArgb(255,154,146))
$title = New-Object Drawing.Font('Segoe UI',56,[Drawing.FontStyle]::Bold,[Drawing.GraphicsUnit]::Pixel)
$subtitle = New-Object Drawing.Font('Segoe UI',28,[Drawing.FontStyle]::Regular,[Drawing.GraphicsUnit]::Pixel)
$small = New-Object Drawing.Font('Segoe UI',15,[Drawing.FontStyle]::Regular,[Drawing.GraphicsUnit]::Pixel)
$graphics.DrawImage($mark,60,100,84,84)
$graphics.DrawString('reshape',$title,$white,56,201)
$graphics.DrawString("Convert it.`nKeep the good stuff.",$subtitle,$white,60,293)
$graphics.DrawString('CONVERT / COMPRESS / CREATE',$small,$coral,62,408)
$graphics.DrawString('Your files. On your computer.',$small,$gray,62,453)
$graphics.DrawImage($shot,510,48,740,534)
$bitmap.Save((Join-Path $assets 'social-preview.png'),[Drawing.Imaging.ImageFormat]::Png)
$bitmap.Save((Join-Path $assets 'hero.png'),[Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose(); $bitmap.Dispose(); $mark.Dispose(); $shot.Dispose()
foreach($resource in @($white,$gray,$coral,$title,$subtitle,$small)) { $resource.Dispose() }
