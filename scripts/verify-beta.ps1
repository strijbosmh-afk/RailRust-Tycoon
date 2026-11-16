#Requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$OutputDirectory,
    [string]$Executable,
    [ValidateSet('starter', 'medium-mixed', 'large-mixed', 'crowd-capacity')]
    [string[]]$Cases = @('starter', 'medium-mixed', 'large-mixed', 'crowd-capacity'),
    [ValidateRange(900, 1296000)][long]$Ticks = 108000,
    [ValidateRange(900, 108000)][long]$CheckpointTicks = 9000,
    [UInt64]$Seed = 20261116,
    [ValidateRange(30, 7200)][int]$ProcessTimeoutSeconds = 1800
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if (-not $Executable) {
    $packaged = Join-Path $PSScriptRoot 'Parkbound.exe'
    $Executable = if (Test-Path -LiteralPath $packaged -PathType Leaf) { $packaged }
        else { Join-Path $PSScriptRoot '../target/release/parkbound.exe' }
}
if (-not [IO.Path]::IsPathFullyQualified($OutputDirectory) -or (Test-Path -LiteralPath $OutputDirectory)) {
    throw 'Use a new absolute output directory.'
}
if (-not (Test-Path -LiteralPath $Executable -PathType Leaf)) { throw 'Build Parkbound or supply its executable.' }
if ([Math]::Ceiling($Ticks / [double]$CheckpointTicks) -gt 128) { throw 'Use at most 128 checkpoint intervals.' }
if (@($Cases | Select-Object -Unique).Count -ne $Cases.Count -or $Cases.Count -eq 0) { throw 'Choose unique beta cases.' }
$binary = [IO.Path]::GetFullPath($Executable)
$output = [IO.Path]::GetFullPath($OutputDirectory)
if (($binary + $output) -match '["\r\n]') { throw 'Paths cannot contain quotes or line breaks.' }
[void](New-Item -ItemType Directory -Path $output)
$runs = [Collections.Generic.List[object]]::new()
$manifest = [ordered]@{
    version = 1
    executable_sha256 = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
    recorded_utc = [DateTime]::UtcNow.ToString('o')
    hardware_role = 'Diagnostics only. Hardware acceptance must use the separately agreed target PC.'
    seed = $Seed
    requested_ticks = $Ticks
    checkpoint_ticks = $CheckpointTicks
    status = 'running'
    runs = $runs
}

function Invoke-BetaProcess([string[]]$Arguments, [string]$LogName) {
    $process = Start-Process -FilePath $binary -ArgumentList $Arguments -WindowStyle Hidden -PassThru `
        -RedirectStandardOutput (Join-Path $output "$LogName.stdout.txt") `
        -RedirectStandardError (Join-Path $output "$LogName.stderr.txt")
    $timer = [Diagnostics.Stopwatch]::StartNew()
    [long]$peakBytes = 0
    try {
        while (-not $process.HasExited) {
            $process.Refresh()
            $peakBytes = [Math]::Max($peakBytes, $process.PrivateMemorySize64)
            if ($timer.Elapsed.TotalSeconds -gt $ProcessTimeoutSeconds) {
                $process.Kill($true)
                $process.WaitForExit()
                throw "Beta process $LogName exceeded its timeout. Evidence is retained."
            }
            Start-Sleep -Milliseconds 200
        }
        $process.WaitForExit()
        $exitCode = $process.ExitCode
        return [ordered]@{ exit_code = $exitCode; wall_seconds = $timer.Elapsed.TotalSeconds; sampled_peak_private_bytes = $peakBytes; sample_interval_ms = 200 }
    } finally { $process.Dispose() }
}

try {
    foreach ($case in $Cases) {
        $directory = Join-Path $output $case
        $run = [ordered]@{ case = $case; writer = $null; replay = $null }
        $runs.Add($run)
        $run.writer = Invoke-BetaProcess -Arguments @('--beta-check', "`"$directory`"", '--beta-case', $case,
            '--beta-ticks', "$Ticks", '--beta-checkpoint', "$CheckpointTicks", '--beta-seed', "$Seed") -LogName "$case-writer"
        if ($run.writer.exit_code -ne 0) { throw "Beta writer failed for $case. Inspect its report and error log." }
        $run.replay = Invoke-BetaProcess -Arguments @('--beta-verify', "`"$directory`"") -LogName "$case-replay"
        if ($run.replay.exit_code -ne 0) { throw "Fresh-process replay failed for $case." }
        $report = Get-Content -LiteralPath (Join-Path $directory 'beta-report.json') -Raw | ConvertFrom-Json
        $replay = Get-Content -LiteralPath (Join-Path $directory 'beta-verification.json') -Raw | ConvertFrom-Json
        if ($report.status -ne 'complete' -or $report.completed_ticks -ne $Ticks -or
            $report.case -ne $case -or -not $replay.exact_replay -or -not $replay.primary_and_backup_unchanged) {
            throw "Incomplete beta evidence for $case."
        }
        Write-Host "$case passed $Ticks ticks and $($report.checkpoints.Count) checkpoints."
    }
    $manifest.status = 'complete'
} catch {
    $manifest.status = 'failed'
    $manifest.error = $_.Exception.Message
    throw
} finally {
    $manifest | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $output 'beta-suite.json') -Encoding utf8
}
