$ErrorActionPreference='Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
& node node_modules/@tauri-apps/cli/tauri.js icon src-tauri/icons/icon.svg --output .cache/refresh-icons
if($LASTEXITCODE -ne 0){throw 'Icon SVG render failed'}
& node node_modules/@tauri-apps/cli/tauri.js icon src-tauri/icons/reshape-mark.svg --output .cache/refresh-mark
if($LASTEXITCODE -ne 0){throw 'Mark SVG render failed'}
Add-Type -AssemblyName System.Drawing
$magick = Join-Path (Get-Location) 'src-tauri\binaries\imagemagick\magick.exe'
foreach($size in @(16,20,24,32,48,64,128,256)) {
  & $magick '.cache/refresh-icons/icon.png' '-resize' "${size}x${size}" "src-tauri/icons/$size.png"
  if($LASTEXITCODE -ne 0){throw 'Icon resize failed'}
}
$frames=@(16,20,24,32,48,64,128,256) | ForEach-Object {"src-tauri/icons/$_.png"}
& $magick @frames 'src-tauri/icons/icon.ico'
if($LASTEXITCODE -ne 0){throw 'ICO generation failed'}
function Color([string]$hex) { [Drawing.ColorTranslator]::FromHtml($hex) }
function Brush([string]$hex) { New-Object Drawing.SolidBrush (Color $hex) }
$logo=[Drawing.Image]::FromFile((Join-Path (Get-Location) '.cache/refresh-mark/icon.png'))
function Banner([int]$w,[int]$h,[string]$path,[bool]$sidebar) {
  $bmp=New-Object Drawing.Bitmap($w,$h)
  $g=[Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode=[Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $g.TextRenderingHint=[Drawing.Text.TextRenderingHint]::AntiAliasGridFit
  $g.Clear((Color '#000000'))
  if($sidebar) {
    $g.DrawImage($logo,44,28,76,76)
    $font=New-Object Drawing.Font('Segoe UI',22,[Drawing.FontStyle]::Bold,[Drawing.GraphicsUnit]::Pixel)
    $textBrush=Brush '#f4f4f4'
    $g.DrawString('reshape',$font,$textBrush,27,109)
    $fontSmall=New-Object Drawing.Font('Segoe UI',11,[Drawing.FontStyle]::Regular,[Drawing.GraphicsUnit]::Pixel)
    $muted=Brush '#a8a8a8'
    $g.DrawString("Your files.`nReimagined.",$fontSmall,$muted,28,146)
    $rectangle=New-Object Drawing.Rectangle(-105,211,340,220)
    $gradient=New-Object Drawing.Drawing2D.LinearGradientBrush($rectangle,(Color '#666666'),(Color '#080808'),65)
    $pen=New-Object Drawing.Pen($gradient,35)
    $g.DrawEllipse($pen,-116,221,272,155)
    $rectangle2=New-Object Drawing.Rectangle(35,236,260,160)
    $warm=New-Object Drawing.Drawing2D.LinearGradientBrush($rectangle2,(Color '#ffac93'),(Color '#803c32'),70)
    $warmPen=New-Object Drawing.Pen($warm,30)
    $g.DrawEllipse($warmPen,39,250,214,119)
    $pen.Dispose();$warmPen.Dispose();$gradient.Dispose();$warm.Dispose();$font.Dispose();$fontSmall.Dispose();$textBrush.Dispose();$muted.Dispose()
  } else {
    $g.DrawImage($logo,8,9,38,38)
    $font=New-Object Drawing.Font('Segoe UI',18,[Drawing.FontStyle]::Bold,[Drawing.GraphicsUnit]::Pixel)
    $textBrush=Brush '#f4f4f4'
    $g.DrawString('reshape',$font,$textBrush,48,15)
    $font.Dispose();$textBrush.Dispose()
  }
  $bmp.Save((Join-Path (Get-Location) $path),[Drawing.Imaging.ImageFormat]::Bmp)
  $g.Dispose();$bmp.Dispose()
}
Banner 164 314 'src-tauri/icons/installer-sidebar.bmp' $true
Banner 150 57 'src-tauri/icons/installer-header.bmp' $false
$logo.Dispose()

Copy-Item src-tauri/icons/icon.ico src-tauri/icons/reshape-app.ico -Force
& $magick src-tauri/icons/256.png -depth 8 RGBA:src-tauri/icons/window-icon.rgba
if($LASTEXITCODE -ne 0){throw "Window icon generation failed"}

Copy-Item .cache/refresh-mark/icon.png src-tauri/icons/setup-mark.png -Force
