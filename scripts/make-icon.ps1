#requires -Version 7.0
[CmdletBinding()]
param(
    [string]$Source = (Join-Path $PSScriptRoot '../assets/art/parkbound-icon.png'),
    [string]$Output = (Join-Path $PSScriptRoot '../assets/art/parkbound.ico')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$sourceImage = [Drawing.Image]::FromFile([IO.Path]::GetFullPath($Source))
$frames = [Collections.Generic.List[byte[]]]::new()
$sizes = @(16, 24, 32, 48, 64, 128, 256)
try {
    foreach ($size in $sizes) {
        $bitmap = [Drawing.Bitmap]::new($size, $size, [Drawing.Imaging.PixelFormat]::Format32bppArgb)
        $graphics = [Drawing.Graphics]::FromImage($bitmap)
        $attributes = [Drawing.Imaging.ImageAttributes]::new()
        $stream = [IO.MemoryStream]::new()
        try {
            $graphics.Clear([Drawing.Color]::Transparent)
            $graphics.CompositingMode = [Drawing.Drawing2D.CompositingMode]::SourceCopy
            $graphics.CompositingQuality = [Drawing.Drawing2D.CompositingQuality]::HighQuality
            $graphics.InterpolationMode = [Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            $graphics.PixelOffsetMode = [Drawing.Drawing2D.PixelOffsetMode]::HighQuality
            $attributes.SetWrapMode([Drawing.Drawing2D.WrapMode]::TileFlipXY)
            $scale = $size / [double][Math]::Max($sourceImage.Width, $sourceImage.Height)
            $width = [int][Math]::Round($sourceImage.Width * $scale)
            $height = [int][Math]::Round($sourceImage.Height * $scale)
            $rectangle = [Drawing.Rectangle]::new(($size - $width) / 2, ($size - $height) / 2, $width, $height)
            $graphics.DrawImage($sourceImage, $rectangle, 0, 0, $sourceImage.Width, $sourceImage.Height, [Drawing.GraphicsUnit]::Pixel, $attributes)
            $bitmap.Save($stream, [Drawing.Imaging.ImageFormat]::Png)
            $frames.Add($stream.ToArray())
        } finally {
            $stream.Dispose()
            $attributes.Dispose()
            $graphics.Dispose()
            $bitmap.Dispose()
        }
    }
} finally { $sourceImage.Dispose() }

$buffer = [IO.MemoryStream]::new()
$writer = [IO.BinaryWriter]::new($buffer)
try {
    $writer.Write([uint16]0)
    $writer.Write([uint16]1)
    $writer.Write([uint16]$sizes.Count)
    $offset = 6 + 16 * $sizes.Count
    for ($i = 0; $i -lt $sizes.Count; $i++) {
        $dimension = if ($sizes[$i] -eq 256) { 0 } else { $sizes[$i] }
        $writer.Write([byte]$dimension)
        $writer.Write([byte]$dimension)
        $writer.Write([uint16]0)
        $writer.Write([uint16]1)
        $writer.Write([uint16]32)
        $writer.Write([uint32]$frames[$i].Length)
        $writer.Write([uint32]$offset)
        $offset += $frames[$i].Length
    }
    foreach ($frame in $frames) { $writer.Write($frame) }
    [IO.File]::WriteAllBytes([IO.Path]::GetFullPath($Output), $buffer.ToArray())
} finally {
    $writer.Dispose()
    $buffer.Dispose()
}
Write-Output "Created icon sizes $($sizes -join ', ') at $Output"
