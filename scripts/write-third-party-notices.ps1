[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [object] $Metadata,
    [Parameter(Mandatory = $true)]
    [string] $OutputDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$supplementalDirectory = Join-Path $PSScriptRoot 'notices'
$supplementalByPackage = @{}
foreach ($entry in (Get-Content -Raw -LiteralPath (Join-Path $supplementalDirectory 'sources.json') | ConvertFrom-Json)) {
    $supplementalByPackage["$($entry.package)@$($entry.version)"] = $entry
}

$packagesById = @{}
foreach ($package in $Metadata.packages) {
    $packagesById[$package.id] = $package
}
$nodesById = @{}
foreach ($node in $Metadata.resolve.nodes) {
    $nodesById[$node.id] = $node
}
$game = @($Metadata.packages | Where-Object { $_.name -eq 'park_game' })
if ($game.Count -ne 1) {
    throw 'Expected exactly one park_game package in Cargo metadata.'
}

# Metadata is filtered to Windows by the caller. Keep every reachable dependency,
# including build tools, so this inventory errs on the side of including notices.
$visited = [System.Collections.Generic.HashSet[string]]::new()
$pending = [System.Collections.Generic.Stack[string]]::new()
$pending.Push($game[0].id)
while ($pending.Count -gt 0) {
    $id = $pending.Pop()
    if (-not $visited.Add($id)) { continue }
    foreach ($dependencyId in $nodesById[$id].dependencies) {
        $pending.Push($dependencyId)
    }
}

$thirdPartyPackages = @(
    $visited | ForEach-Object { $packagesById[$_] } |
        Where-Object { $_.source -ne $null } | Sort-Object name, version
)
$notices = [System.Text.StringBuilder]::new()
[void] $notices.AppendLine('PARKBOUND - THIRD-PARTY NOTICES')
[void] $notices.AppendLine('Generated from Cargo.lock and the source packages used for this Windows build.')
[void] $notices.AppendLine('Each component retains its own license and copyright notices below.')
[void] $notices.AppendLine('The inventory includes transitive dependencies and build-time dependencies.')
[void] $notices.AppendLine('The default embedded Bevy Fira Mono font is covered by its separate font license below.')
[void] $notices.AppendLine()

$inventory = [System.Collections.Generic.List[object]]::new()
$missingTexts = [System.Collections.Generic.List[string]]::new()
$foundFontLicense = $false
foreach ($package in $thirdPartyPackages) {
    $sourceDirectory = Split-Path -Parent $package.manifest_path
    $licenseFiles = @(
        Get-ChildItem -LiteralPath $sourceDirectory -File -Recurse |
            Where-Object {
                ($_.Name -match '(?i)(licen[cs]e|copying|copyright|notice|^ofl(?:[._-]|$)|^unlicense$)') -and
                ($_.Extension -notin @('.rs', '.py', '.png', '.ttf', '.woff', '.woff2', '.svg'))
            } | Select-Object -ExpandProperty FullName
    )
    if ($package.license_file) {
        $declaredFile = Join-Path $sourceDirectory $package.license_file
        if (-not (Test-Path -LiteralPath $declaredFile -PathType Leaf)) {
            throw "Declared license file is missing for $($package.name): $declaredFile"
        }
        $licenseFiles += $declaredFile
    }
    $noticeFiles = @(
        $licenseFiles | Sort-Object -Unique | ForEach-Object {
            [pscustomobject]@{
                path = $_
                label = $_.Substring($sourceDirectory.Length).TrimStart([char[]] '\/').Replace('\', '/')
                source = 'Included in the published Cargo source package.'
            }
        }
    )
    $supplement = $supplementalByPackage["$($package.name)@$($package.version)"]
    if ($supplement) {
        $vcs = Get-Content -Raw -LiteralPath (Join-Path $sourceDirectory '.cargo_vcs_info.json') | ConvertFrom-Json
        if ($vcs.git.sha1 -ne $supplement.source_commit) {
            throw "Source commit changed for $($package.name); review its supplemental license notices."
        }
        foreach ($file in $supplement.files) {
            $path = [System.IO.Path]::GetFullPath((Join-Path $supplementalDirectory $file.file))
            $prefix = [System.IO.Path]::GetFullPath($supplementalDirectory) + [System.IO.Path]::DirectorySeparatorChar
            if (-not $path.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
                throw 'A supplemental license path leaves the notices directory.'
            }
            if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $file.sha256) {
                throw "Supplemental notice hash mismatch: $($file.file)"
            }
            $noticeFiles += [pscustomobject]@{ path = $path; label = $file.file; source = $file.url }
        }
    }
    if ($noticeFiles.Count -eq 0) {
        $missingTexts.Add("$($package.name) $($package.version) [$($package.license)]")
    }

    [void] $notices.AppendLine(('=' * 78))
    [void] $notices.AppendLine("$($package.name) $($package.version)")
    [void] $notices.AppendLine("Declared license: $($package.license)")
    [void] $notices.AppendLine("Repository: $($package.repository)")
    [void] $notices.AppendLine("Authors: $($package.authors -join ', ')")
    if ($supplement) { [void] $notices.AppendLine($supplement.context) }
    [void] $notices.AppendLine()
    $relativeFiles = [System.Collections.Generic.List[string]]::new()
    foreach ($file in $noticeFiles) {
        $relativeFiles.Add($file.label)
        [void] $notices.AppendLine("--- $($file.label) ---")
        [void] $notices.AppendLine("Source: $($file.source)")
        [void] $notices.AppendLine([System.IO.File]::ReadAllText($file.path))
        [void] $notices.AppendLine()
        if ($package.name -eq 'bevy_text' -and $file.label -match '(?i)fira.*licen[cs]e') {
            $foundFontLicense = $true
        }
    }
    $inventory.Add([ordered]@{
        name = $package.name
        version = $package.version
        license = $package.license
        repository = $package.repository
        authors = @($package.authors)
        included_notice_files = @($relativeFiles)
    })
}

# Never produce a distribution with a known omission. A dependency update that
# changes notice locations must be inspected before this guard is adjusted.
if ($missingTexts.Count -gt 0) {
    throw "Source license texts were not found for: $($missingTexts -join '; '). Add reviewed upstream notices before packaging."
}
if (-not $foundFontLicense) {
    throw 'The embedded Fira Mono font license was not found in bevy_text. Inspect the font and its license before packaging.'
}

$utf8 = [System.Text.UTF8Encoding]::new($false)
[System.IO.File]::WriteAllText((Join-Path $OutputDirectory 'THIRD_PARTY_NOTICES.txt'), $notices.ToString(), $utf8)
[System.IO.File]::WriteAllText(
    (Join-Path $OutputDirectory 'DEPENDENCIES.json'),
    (ConvertTo-Json -InputObject $inventory.ToArray() -Depth 8),
    $utf8
)
Write-Host "Included license and copyright notices for $($thirdPartyPackages.Count) dependencies and the default font."
