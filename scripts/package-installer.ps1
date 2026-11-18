#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$PackageDirectory,
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '../dist'),
    [string]$IsccPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PackageDirectory = (Resolve-Path -LiteralPath $PackageDirectory).Path
$packageName = Split-Path -Leaf $PackageDirectory
if ($packageName -notmatch '^Parkbound-(\d+\.\d+\.\d+)-windows-x64$') {
    throw 'Select a versioned Parkbound Windows package directory.'
}
$version = $Matches[1]
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
[void](New-Item -ItemType Directory -Path $OutputDirectory -Force)
if (-not $IsccPath) {
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA 'Programs/Inno Setup 6/ISCC.exe'),
        (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6/ISCC.exe')
    )
    $IsccPath = $candidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
}
if (-not $IsccPath -or -not (Test-Path -LiteralPath $IsccPath -PathType Leaf)) {
    throw 'Install Inno Setup 6.7.4 or pass -IsccPath. See INSTALLATIE.md.'
}
$IsccPath = (Resolve-Path -LiteralPath $IsccPath).Path

$manifest = Join-Path $PackageDirectory 'SHA256SUMS.txt'
$files = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
$prefix = $PackageDirectory.TrimEnd('\') + '\'
foreach ($line in Get-Content -LiteralPath $manifest) {
    if ($line -notmatch '^([a-fA-F0-9]{64})  (.+)$') { throw 'Malformed package manifest.' }
    $hash, $relative = $Matches[1], $Matches[2]
    if ($relative -match '(^[/\\]|:|(^|[/\\])\.{1,2}([/\\]|$))') { throw 'Unsafe package path.' }
    $path = [IO.Path]::GetFullPath((Join-Path $PackageDirectory $relative))
    if (-not $path.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase) -or -not $files.Add($path)) {
        throw 'Duplicate or escaping package path.'
    }
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $hash) { throw "Package hash mismatch for $relative" }
}
foreach ($item in Get-ChildItem -LiteralPath $PackageDirectory -Recurse -Force) {
    if ($item.LinkType -in @('SymbolicLink', 'Junction')) { throw 'Package contains a linked path.' }
    if (-not $item.PSIsContainer -and $item.FullName -ne $manifest -and -not $files.Contains($item.FullName)) {
        throw "Unmanifested package file $($item.Name)"
    }
}
foreach ($required in @('Parkbound.exe', 'THIRD_PARTY_NOTICES.txt')) {
    if (-not $files.Contains((Join-Path $PackageDirectory $required))) { throw "Package lacks $required" }
}
if (Test-Path (Join-Path $PackageDirectory 'release.json')) {
    $release = Get-Content (Join-Path $PackageDirectory 'release.json') -Raw | ConvertFrom-Json
    if ($release.version -ne $version) { throw 'Package and release metadata versions differ.' }
}

& $IsccPath '/Qp' "/DPackageDir=$PackageDirectory" "/DReleaseVersion=$version" "/DReleaseOutput=$OutputDirectory" (Join-Path $PSScriptRoot '../distribution/parkbound.iss')
if ($LASTEXITCODE -ne 0) { throw 'Windows installer compilation failed.' }
$installer = Join-Path $OutputDirectory "Parkbound-$version-setup-x64.exe"
$sha256 = (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$installer.sha256", "$sha256  $([IO.Path]::GetFileName($installer))`n", [Text.UTF8Encoding]::new($false))
$record = [ordered]@{
    version = $version
    installer_sha256 = $sha256
    executable_sha256 = (Get-FileHash (Join-Path $PackageDirectory 'Parkbound.exe')).Hash.ToLowerInvariant()
    compiler_file_version = (Get-Item $IsccPath).VersionInfo.FileVersion
    compiler_sha256 = (Get-FileHash $IsccPath).Hash.ToLowerInvariant()
    package_manifest_sha256 = (Get-FileHash $manifest).Hash.ToLowerInvariant()
    unsigned = $true
}
$record | ConvertTo-Json | Set-Content -LiteralPath "$installer.build.json" -Encoding utf8
Write-Output "Installer ready: $installer"
Write-Output "SHA256: $sha256"
