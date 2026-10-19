[CmdletBinding()]
param(
    [switch] $SkipBuild
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$workspaceDirectory = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$targetTriple = 'x86_64-pc-windows-msvc'

function Assert-DistChild([string] $Path, [string] $DistDirectory) {
    $absolutePath = [System.IO.Path]::GetFullPath($Path)
    $allowedPrefix = $DistDirectory.TrimEnd([char[]] '\/') + [System.IO.Path]::DirectorySeparatorChar
    if (-not $absolutePath.StartsWith($allowedPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to alter a path outside the distribution directory: $absolutePath"
    }
}

if ($env:OS -ne 'Windows_NT') {
    throw 'Create the Windows distribution on Windows with the MSVC Rust toolchain.'
}
Push-Location -LiteralPath $workspaceDirectory
try {
    $hostInformation = & rustc -vV
    if ($LASTEXITCODE -ne 0 -or $hostInformation -notcontains "host: $targetTriple") {
        throw "This script requires the $targetTriple Rust host toolchain."
    }
    if (-not $SkipBuild) {
        & cargo build --release --locked --package park_game --bin parkbound
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    }

    $metadataJson = (& cargo metadata --locked --format-version 1 --filter-platform $targetTriple | Out-String)
    if ($LASTEXITCODE -ne 0) { throw 'Unable to read locked Cargo metadata.' }
    $metadata = $metadataJson | ConvertFrom-Json
    $gamePackages = @($metadata.packages | Where-Object { $_.name -eq 'park_game' })
    if ($gamePackages.Count -ne 1) { throw 'Expected exactly one park_game package.' }
    $version = $gamePackages[0].version
    if ($version -notmatch '^[0-9A-Za-z.+-]+$') { throw 'Unexpected package version.' }
    $executable = Join-Path $metadata.target_directory 'release/parkbound.exe'
    if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
        throw "Release executable was not found: $executable. Run without -SkipBuild."
    }
    $readme = Join-Path $workspaceDirectory 'README.md'
    if (-not (Test-Path -LiteralPath $readme -PathType Leaf)) { throw 'README.md is required for distribution.' }

    $distributionDirectory = Join-Path $workspaceDirectory 'dist'
    [void] (New-Item -ItemType Directory -Path $distributionDirectory -Force)
    $packageName = "Parkbound-$version-windows-x64"
    $packageDirectory = Join-Path $distributionDirectory $packageName
    $stagingDirectory = Join-Path $distributionDirectory ('.staging-' + [guid]::NewGuid().ToString('N'))
    $stagedPackage = Join-Path $stagingDirectory $packageName
    [void] (New-Item -ItemType Directory -Path $stagedPackage -Force)
    Copy-Item -LiteralPath $executable -Destination (Join-Path $stagedPackage 'Parkbound.exe')
    Copy-Item -LiteralPath $readme -Destination (Join-Path $stagedPackage 'README.md')
    $resumeGuide = Join-Path $workspaceDirectory 'RESUME_DEVELOPMENT.md'
    if (-not (Test-Path -LiteralPath $resumeGuide -PathType Leaf)) {
        throw 'RESUME_DEVELOPMENT.md is required for distribution.'
    }
    Copy-Item -LiteralPath $resumeGuide -Destination (Join-Path $stagedPackage 'RESUME_DEVELOPMENT.md')
    $documents = Join-Path $workspaceDirectory 'docs'
    if (Test-Path -LiteralPath $documents -PathType Container) {
        Copy-Item -LiteralPath $documents -Destination (Join-Path $stagedPackage 'docs') -Recurse
    }
    $developmentPlan = Join-Path $workspaceDirectory 'DEVELOPMENT_PLAN.md'
    if (Test-Path -LiteralPath $developmentPlan -PathType Leaf) {
        Copy-Item -LiteralPath $developmentPlan -Destination $stagedPackage
    }
    & (Join-Path $PSScriptRoot 'write-third-party-notices.ps1') -Metadata $metadata -OutputDirectory $stagedPackage
    & (Join-Path $PSScriptRoot 'copy-windows-runtime.ps1') -Executable $executable -OutputDirectory $stagedPackage

    $rustSysroot = & rustc --print sysroot
    if ($LASTEXITCODE -ne 0) { throw 'Could not locate Rust standard library notices.' }
    $rustNotices = Join-Path $rustSysroot 'share/doc/rust/COPYRIGHT-library.html'
    if (-not (Test-Path -LiteralPath $rustNotices -PathType Leaf)) {
        throw 'Rust standard library notices are missing. Install the rust-docs component for this toolchain.'
    }
    Copy-Item -LiteralPath $rustNotices -Destination (Join-Path $stagedPackage 'RUST_STANDARD_LIBRARY_NOTICES.html')
    [System.IO.File]::AppendAllText(
        (Join-Path $stagedPackage 'THIRD_PARTY_NOTICES.txt'),
        "`nRust standard library: license and copyright texts are in RUST_STANDARD_LIBRARY_NOTICES.html.`n",
        [System.Text.UTF8Encoding]::new($false)
    )

    $hashLines = @(
        Get-ChildItem -LiteralPath $stagedPackage -File -Recurse | Sort-Object FullName | ForEach-Object {
            $relative = $_.FullName.Substring($stagedPackage.Length).TrimStart([char[]] '\/').Replace('\', '/')
            $hash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
            "$hash  $relative"
        }
    )
    $utf8 = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllLines((Join-Path $stagedPackage 'SHA256SUMS.txt'), $hashLines, $utf8)
    $temporaryArchive = Join-Path $stagingDirectory "$packageName.zip"
    Compress-Archive -LiteralPath $stagedPackage -DestinationPath $temporaryArchive -CompressionLevel Optimal

    # Preserve an existing package for comparison instead of recursively deleting it.
    if (Test-Path -LiteralPath $packageDirectory) {
        $previousDirectory = Join-Path $distributionDirectory ($packageName + '.previous-' + [guid]::NewGuid().ToString('N'))
        Assert-DistChild -Path $packageDirectory -DistDirectory $distributionDirectory
        Assert-DistChild -Path $previousDirectory -DistDirectory $distributionDirectory
        Move-Item -LiteralPath $packageDirectory -Destination $previousDirectory
    }
    Assert-DistChild -Path $stagedPackage -DistDirectory $distributionDirectory
    Assert-DistChild -Path $packageDirectory -DistDirectory $distributionDirectory
    Move-Item -LiteralPath $stagedPackage -Destination $packageDirectory
    $archivePath = Join-Path $distributionDirectory "$packageName.zip"
    Move-Item -LiteralPath $temporaryArchive -Destination $archivePath -Force
    # Only the now-empty, freshly-created staging directory is removed.
    Remove-Item -LiteralPath $stagingDirectory
    $archiveHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    [System.IO.File]::WriteAllText("$archivePath.sha256", "$archiveHash  $packageName.zip`n", $utf8)
    Write-Host "Windows package: $packageDirectory"
    Write-Host "ZIP download: $archivePath"
    Write-Host "SHA-256: $archiveHash"
    Write-Host 'This local build is unsigned. No files have been published.'
}
finally {
    Pop-Location
}
