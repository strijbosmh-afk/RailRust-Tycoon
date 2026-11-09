#requires -Version 7.0
[CmdletBinding()]
param([string]$ManifestPath = (Join-Path $PSScriptRoot '../assets/manifest.json'))
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$path = [IO.Path]::GetFullPath($ManifestPath)
if (-not $path.StartsWith($root + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Asset manifest must be inside the project.'
}
if ((Get-Item -LiteralPath $path).Length -gt 65536) { throw 'Asset manifest exceeds 64 KiB.' }
$assetCatalog = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
if ($assetCatalog.version -ne 1 -or @($assetCatalog.assets).Count -eq 0 -or @($assetCatalog.assets).Count -gt 256) {
    throw 'Asset manifest version or entry count is invalid.'
}
$seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
foreach ($asset in $assetCatalog.assets) {
    if ($asset.id -cnotmatch '^[a-z][a-z0-9.-]{2,63}$' -or -not $seen.Add($asset.id)) {
        throw "Invalid or duplicate asset ID: $($asset.id)"
    }
    if ($asset.type -cnotin @('procedural', 'authored', 'hybrid')) { throw "Unknown asset type: $($asset.id)" }
    if ($asset.rights -ne 'Parkbound original' -and $asset.rights -notmatch '^Licensed: .+') {
        throw "Missing redistribution rights: $($asset.id)"
    }
    if ($asset.source -cnotmatch '^[a-zA-Z0-9_./-]+$' -or $asset.source -match '(^|/)\.\.(/|$)') {
        throw "Unsafe asset source: $($asset.id)"
    }
    $source = [IO.Path]::GetFullPath((Join-Path $root $asset.source))
    if (-not $source.StartsWith($root + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or
        -not (Test-Path -LiteralPath $source -PathType Leaf)) { throw "Missing asset source: $($asset.id)" }
    if (@($asset.variants).Count -eq 0 -or @($asset.variants).Count -gt 32) { throw "Invalid variants: $($asset.id)" }
    if ($asset.type -ne 'procedural') {
        if ($asset.sha256 -notmatch '^[a-fA-F0-9]{64}$' -or $asset.width -lt 1 -or $asset.height -lt 1) {
            throw "Authored asset needs a hash and positive dimensions: $($asset.id)"
        }
        if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -ne $asset.sha256) {
            throw "Authored asset hash mismatch: $($asset.id)"
        }
    }
}
Write-Output "PASS: $($seen.Count) original asset families have unique IDs, sources and redistribution records."
