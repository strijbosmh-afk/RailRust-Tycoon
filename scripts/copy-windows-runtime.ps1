[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string] $Executable,
    [Parameter(Mandatory = $true)]
    [string] $OutputDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) {
    throw 'Visual Studio Installer/vswhere.exe is required to locate the licensed MSVC redistributable files.'
}
$installation = & $vswhere -latest -version '[17.0,18.0)' -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $installation) {
    throw 'Visual Studio 2022 with the MSVC C++ tools is required for this packaging script.'
}
$toolsVersion = Get-ChildItem -LiteralPath (Join-Path $installation 'VC/Tools/MSVC') -Directory |
    Where-Object { $_.Name -match '^\d+\.\d+\.\d+$' } | Sort-Object { [version] $_.Name } -Descending | Select-Object -First 1
$redistVersion = Get-ChildItem -LiteralPath (Join-Path $installation 'VC/Redist/MSVC') -Directory |
    Where-Object { $_.Name -match '^\d+\.\d+\.\d+$' } | Sort-Object { [version] $_.Name } -Descending | Select-Object -First 1
if (-not $toolsVersion -or -not $redistVersion) { throw 'MSVC tools or redistributable directory is missing.' }
$dumpbin = Join-Path $toolsVersion.FullName 'bin/Hostx64/x64/dumpbin.exe'
$crtDirectory = Join-Path $redistVersion.FullName 'x64/Microsoft.VC143.CRT'

function Get-ImportedDlls([string] $BinaryPath) {
    $output = & $dumpbin /nologo /dependents $BinaryPath
    if ($LASTEXITCODE -ne 0) { throw "Cannot inspect imported DLLs: $BinaryPath" }
    @($output | Where-Object { $_ -match '^\s+[A-Za-z0-9_.-]+\.dll\s*$' } |
        ForEach-Object { $_.Trim() } | Sort-Object -Unique)
}

$pending = [System.Collections.Generic.Queue[string]]::new()
$pending.Enqueue($Executable)
$copied = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
$inspected = [System.Collections.Generic.List[object]]::new()
$runtimeFiles = [System.Collections.Generic.List[object]]::new()
while ($pending.Count -gt 0) {
    $binary = $pending.Dequeue()
    $imports = @(Get-ImportedDlls -BinaryPath $binary)
    $inspected.Add([ordered]@{ file = [System.IO.Path]::GetFileName($binary); imported_dlls = $imports })
    foreach ($name in $imports) {
        if ($name -notmatch '^(?i)(vcruntime|msvcp|concrt)140(?:_[a-z0-9]+)*\.dll$') { continue }
        if (-not $copied.Add($name)) { continue }
        $source = Join-Path $crtDirectory $name
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
            throw "Required runtime $name is missing from the MSVC x64 redistributable directory."
        }
        $destination = Join-Path $OutputDirectory $name
        Copy-Item -LiteralPath $source -Destination $destination
        $fileInfo = (Get-Item -LiteralPath $source).VersionInfo
        $runtimeFiles.Add([ordered]@{
            file = $name
            version = $fileInfo.FileVersion
            copyright = $fileInfo.LegalCopyright
            source = "Visual Studio 2022/VC/Redist/MSVC/$($redistVersion.Name)/x64/Microsoft.VC143.CRT/$name"
            sha256 = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
        })
        $pending.Enqueue($destination)
    }
}

if ($runtimeFiles.Count -gt 0) {
    $licenseDirectory = Join-Path $OutputDirectory 'licenses/MSVC'
    [void] (New-Item -ItemType Directory -Path $licenseDirectory -Force)
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'notices/msvc/LICENSE.txt') -Destination $licenseDirectory
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'notices/msvc/Visual-C-Runtime-2015-2022-License.docx') -Destination $licenseDirectory
    $runtimeNotice = "`n" + ('=' * 78) + "`nMicrosoft Visual C++ runtime`n" +
        "Unmodified Microsoft runtime files supplied beside Parkbound.exe: $($copied -join ', ').`n" +
        "Copyright Microsoft Corporation. See licenses/MSVC/LICENSE.txt and the original license DOCX.`n" +
        "Redistribution source: https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution`n"
    [System.IO.File]::AppendAllText((Join-Path $OutputDirectory 'THIRD_PARTY_NOTICES.txt'), $runtimeNotice, [System.Text.UTF8Encoding]::new($false))
}
$manifest = [ordered]@{
    inspected_binaries = $inspected.ToArray()
    bundled_microsoft_runtime = $runtimeFiles.ToArray()
    operating_system = 'Windows 11 x64 supplies the Windows system and Universal CRT DLLs.'
    verification = 'Import inspection only; test the extracted package on a clean Windows machine before release.'
}
[System.IO.File]::WriteAllText(
    (Join-Path $OutputDirectory 'NATIVE_DEPENDENCIES.json'),
    (ConvertTo-Json -InputObject $manifest -Depth 8),
    [System.Text.UTF8Encoding]::new($false)
)
Write-Host "Bundled $($runtimeFiles.Count) imported MSVC runtime DLLs from the Visual Studio redistributable directory."
