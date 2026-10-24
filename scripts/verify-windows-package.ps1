#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Archive,
    [Parameter(Mandatory)][string]$ExpectedArchiveSha256,
    [Parameter(Mandatory)][string]$ExpectedExecutableSha256,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [string[]]$Proofs = @('access', 'operations', 'design-transfer'),
    [string[]]$CaptureScenes = @('access-needs', 'access-ready', 'access-unavailable'),
    [switch]$SkipRuntime
)

# Verify a specific locally built release, retain evidence in a fresh folder,
# and run only against isolated profiles. Never overwrite an existing profile.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$workspaceDirectory = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
foreach ($value in @($Archive, $OutputDirectory)) {
    if (-not [IO.Path]::IsPathFullyQualified($value)) { throw 'Use absolute archive/output paths.' }
}
foreach ($value in @($ExpectedArchiveSha256, $ExpectedExecutableSha256)) {
    if ($value -notmatch '^[a-fA-F0-9]{64}$') { throw 'Expected SHA256 must contain 64 hexadecimal characters.' }
}
foreach ($value in @($Proofs) + @($CaptureScenes)) {
    if ($value -notmatch '^[a-z][a-z0-9-]*$') { throw 'Invalid proof/capture name.' }
}
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Evidence directory must not already exist.' }
if ((Get-FileHash -LiteralPath $Archive -Algorithm SHA256).Hash -ne $ExpectedArchiveSha256) {
    throw 'Archive hash differs from the expected release.'
}
$packageName = [IO.Path]::GetFileNameWithoutExtension($Archive)
if ($packageName -notmatch '^Parkbound-[0-9A-Za-z.+-]+-windows-x64$') { throw 'Unexpected archive name.' }
$output = [IO.Path]::GetFullPath($OutputDirectory)
$package = Join-Path $output $packageName

function Assert-Child([string]$Path, [string]$Parent) {
    $absolute = [IO.Path]::GetFullPath($Path)
    $prefix = [IO.Path]::GetFullPath($Parent).TrimEnd([char[]]'\/') + [IO.Path]::DirectorySeparatorChar
    if (-not $absolute.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { throw 'Path escapes its intended directory.' }
    return $absolute
}

$zip = [IO.Compression.ZipFile]::OpenRead($Archive)
try {
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    [long]$expandedBytes = 0
    foreach ($entry in $zip.Entries) {
        $relative = $entry.FullName.Replace('\', '/')
        if (-not $relative.StartsWith($packageName + '/', [StringComparison]::Ordinal) -or
            $relative -match '(:|(^|/)\.{1,2}(/|$))' -or -not $seen.Add($relative) -or
            (($entry.ExternalAttributes -shr 16) -band 0xf000) -eq 0xa000 -or
            ($entry.ExternalAttributes -band 0x400)) { throw "Unsafe/duplicate archive entry: $relative" }
        [void](Assert-Child (Join-Path $output $relative) $output)
        $expandedBytes += $entry.Length
    }
    if ($seen.Count -gt 512 -or $expandedBytes -gt 512MB) { throw 'Unexpected archive expansion size.' }
} finally { $zip.Dispose() }
Expand-Archive -LiteralPath $Archive -DestinationPath $output
$receipt = Join-Path $output 'verification.txt'
function Record([string]$Message) {
    [IO.File]::AppendAllText($receipt, $Message + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))
    Write-Output $Message
}

Record "Archive SHA256: $($ExpectedArchiveSha256.ToLowerInvariant())"
$manifest = Join-Path $package 'SHA256SUMS.txt'
$payloads = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
foreach ($line in Get-Content -LiteralPath $manifest) {
    if ($line -notmatch '^([a-fA-F0-9]{64})  (.+)$') { throw 'Malformed payload manifest.' }
    $expectedHash = $Matches[1]
    $relative = $Matches[2]
    if ($relative -match '(^[/\\]|:|(^|[/\\])\.{1,2}([/\\]|$))') { throw 'Unsafe manifest path.' }
    $path = Assert-Child (Join-Path $package $relative) $package
    if (-not $payloads.Add($path) -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $expectedHash) {
        throw "Duplicate or mismatched payload: $relative"
    }
}
$files = @(Get-ChildItem -LiteralPath $package -Recurse -Force -File)
if ($files.Count -ne $payloads.Count + 1) { throw 'Unexpected payload count.' }
foreach ($file in $files) {
    if ($file.FullName -ne $manifest -and -not $payloads.Contains($file.FullName)) { throw 'Unmanifested payload.' }
}
$guides = @('README.md', 'RESUME_DEVELOPMENT.md', 'DEVELOPMENT_PLAN.md') +
    @(Get-ChildItem -LiteralPath (Join-Path $workspaceDirectory 'docs') -Filter '*.md' -File | ForEach-Object { 'docs/' + $_.Name })
if (@($files | Where-Object Extension -eq '.md').Count -ne $guides.Count) { throw 'Unexpected guide inventory.' }
foreach ($guide in $guides) {
    if ((Get-FileHash -LiteralPath (Join-Path $package $guide)).Hash -ne
        (Get-FileHash -LiteralPath (Join-Path $workspaceDirectory $guide)).Hash) { throw "Stale bundled guide: $guide" }
}
$executable = Join-Path $package 'Parkbound.exe'
foreach ($name in @('pocket-oval.parktrack.json','meadow-winder.parktrack.json','meadow-climb.parktrack.json')) {
    $relative = 'designs/' + $name
    if ((Get-FileHash -LiteralPath (Join-Path $package $relative)).Hash -ne
        (Get-FileHash -LiteralPath (Join-Path $workspaceDirectory $relative)).Hash) { throw "Missing or stale starter: $name" }
}
if ((Get-FileHash -LiteralPath $executable).Hash -ne $ExpectedExecutableSha256) { throw 'Unexpected executable.' }
Record "PASS: $($payloads.Count) payload hashes, no extra files, $($guides.Count) current guides."
Record "Executable SHA256: $($ExpectedExecutableSha256.ToLowerInvariant())"

function Run-ProofProcess([string]$Label, [string]$ProcessArguments) {
    $process = Start-Process -FilePath $executable -ArgumentList $ProcessArguments -WorkingDirectory $env:WINDIR -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output ($Label + '.stdout.txt')) -RedirectStandardError (Join-Path $output ($Label + '.stderr.txt'))
    $timer = [Diagnostics.Stopwatch]::StartNew()
    while (-not $process.WaitForExit(10000)) {
        if ($timer.Elapsed.TotalSeconds -ge 180) {
            Stop-Process -Id $process.Id -Force
            throw "$Label exceeded 180 seconds; logs retained."
        }
        Write-Output "$Label running"
    }
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) { throw "$Label returned $($process.ExitCode); inspect retained stderr." }
    Record "$Label PASS"
}
if (-not $SkipRuntime) {
    foreach ($proof in $Proofs) {
        $profileDirectory = Join-Path $output ($proof + ' profile é')
        foreach ($mode in @('write-', 'verify-')) {
            Run-ProofProcess ($mode + $proof) ('--save-check {0}{1} --save-dir "{2}"' -f $mode, $proof, $profileDirectory)
        }
    }
    foreach ($scene in $CaptureScenes) {
        $capturePath = Join-Path $output ($scene + '.png')
        Run-ProofProcess $scene ('--capture-frame "{0}" --capture-scene {1} --capture-small-window' -f $capturePath, $scene)
        Record "Capture $scene SHA256: $((Get-FileHash -LiteralPath $capturePath).Hash.ToLowerInvariant())"
    }
    Record 'Runtime proofs passed from the extracted package. Inspect captures separately; native input is not established.'
} else {
    Record 'Runtime skipped for this guide-only integrity refresh.'
}
if ((Get-FileHash -LiteralPath $executable).Hash -ne $ExpectedExecutableSha256) { throw 'Executable changed during verification.' }
