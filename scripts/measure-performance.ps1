#Requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $OutputDirectory,
    [string] $Executable,
    [ValidateSet('baseline', 'save-scale', 'stress')]
    [string[]] $Cases = @('baseline', 'save-scale', 'stress'),
    [ValidateRange(1, 5)]
    [int] $Repetitions = 3,
    [ValidateRange(30, 1800)]
    [int] $BenchmarkTicks = 300,
    [ValidateRange(120, 3600)]
    [int] $ProfileSamples = 600,
    [switch] $SkipFrames
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$workspaceDirectory = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$sampleIntervalMilliseconds = 200
$benchmarkTimeoutSeconds = 180
$profileTimeoutSeconds = 150

function Write-NewJson([string] $Path, [object] $Value) {
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes(($Value | ConvertTo-Json -Depth 40) + "`n")
    $file = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try {
        $file.Write($bytes, 0, $bytes.Length)
        $file.Flush($true)
    }
    finally {
        $file.Dispose()
    }
}

function Get-HardwareRecord {
    # Explicit allowlists prevent CIM objects from leaking account, computer,
    # serial, device-path, or installation-directory properties into reports.
    $record = [ordered]@{
        operating_system = $null
        processors = @()
        graphics = @()
        total_physical_memory_bytes = $null
        unavailable = [Collections.Generic.List[string]]::new()
    }
    try {
        $operatingSystem = Get-CimInstance -ClassName Win32_OperatingSystem
        $record.operating_system = [ordered]@{
            caption = $operatingSystem.Caption
            version = $operatingSystem.Version
            build_number = $operatingSystem.BuildNumber
            architecture = $operatingSystem.OSArchitecture
        }
    }
    catch { $record.unavailable.Add('operating_system') }
    try {
        $record.processors = @(Get-CimInstance -ClassName Win32_Processor | ForEach-Object {
            [ordered]@{
                name = $_.Name
                cores = $_.NumberOfCores
                logical_processors = $_.NumberOfLogicalProcessors
                maximum_clock_mhz = $_.MaxClockSpeed
            }
        })
    }
    catch { $record.unavailable.Add('processors') }
    try {
        $record.graphics = @(Get-CimInstance -ClassName Win32_VideoController | ForEach-Object {
            [ordered]@{ name = $_.Name; driver_version = $_.DriverVersion }
        })
    }
    catch { $record.unavailable.Add('graphics') }
    try {
        $record.total_physical_memory_bytes = (Get-CimInstance -ClassName Win32_ComputerSystem).TotalPhysicalMemory
    }
    catch { $record.unavailable.Add('total_physical_memory') }
    return $record
}

function Get-SourceRecord {
    $commit = $null
    $dirty = $null
    $manifestPath = Join-Path $workspaceDirectory 'Cargo.toml'
    $hasWorkspaceManifest = Test-Path -LiteralPath $manifestPath -PathType Leaf
    if ($hasWorkspaceManifest -and (Get-Command git -ErrorAction SilentlyContinue)) {
        $candidate = & git -C $workspaceDirectory rev-parse HEAD 2>$null
        if ($LASTEXITCODE -eq 0 -and "$candidate" -match '^[0-9a-fA-F]{40}$') {
            $commit = "$candidate".ToLowerInvariant()
            $changes = @(& git -C $workspaceDirectory status --porcelain --untracked-files=normal 2>$null)
            if ($LASTEXITCODE -eq 0) { $dirty = $changes.Count -ne 0 }
        }
    }
    $declaredVersion = $null
    if ($hasWorkspaceManifest) {
        $manifest = [IO.File]::ReadAllText($manifestPath)
        if ($manifest -match '(?ms)^\[workspace\.package\]\s*(?:(?!^\[).)*?^version\s*=\s*"([^"]+)"') {
            $declaredVersion = $Matches[1]
        }
    }
    return [ordered]@{
        checkout_commit = $commit
        checkout_dirty = $dirty
        workspace_declared_version = $declaredVersion
        note = 'Checkout metadata identifies the source directory, not proof of how an arbitrary executable was built. Runtime report versions and executable SHA-256 identify the measured binary.'
    }
}

function Assert-Timing([object] $Timing, [object[]] $Samples) {
    if ($null -eq $Timing) {
        if ($Samples.Count -ne 0) { throw 'Missing timing summary' }
        return
    }
    if ($Timing -isnot [Collections.IDictionary] -or $Timing.samples -ne $Samples.Count -or $Samples.Count -eq 0) {
        throw 'Invalid timing sample count'
    }
    foreach ($field in @('total_ms', 'min_ms', 'mean_ms', 'p50_ms', 'p95_ms', 'p99_ms', 'max_ms')) {
        if (-not $Timing.Contains($field) -or $Timing[$field] -isnot [ValueType] -or
            -not [double]::IsFinite([double] $Timing[$field]) -or [double] $Timing[$field] -lt 0) {
            throw 'Invalid timing statistic'
        }
    }
    foreach ($sample in $Samples) {
        if ($sample -isnot [ValueType] -or -not [double]::IsFinite([double] $sample) -or [double] $sample -lt 0) {
            throw 'Invalid raw timing sample'
        }
    }
}

function Assert-Phase([object] $Phase) {
    if ($Phase -isnot [Collections.IDictionary] -or $Phase.status -notin @('complete', 'budget_exceeded') -or
        $Phase.samples_ms -isnot [array] -or $Phase.requested_samples -lt 1 -or
        $Phase.samples_ms.Count -gt $Phase.requested_samples) {
        throw 'Invalid phase'
    }
    Assert-Timing -Timing $Phase.timing -Samples $Phase.samples_ms
    if ($Phase.status -eq 'complete' -and $Phase.samples_ms.Count -ne $Phase.requested_samples) {
        throw 'Incomplete phase marked complete'
    }
}

function Read-MeasurementReport([string] $Path, [string] $Mode, [string] $CaseName) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return [ordered]@{ valid = $false; failure = 'report_missing'; version = $null; kind = $null; status = $null; metrics = $null }
    }
    try {
        $report = [IO.File]::ReadAllText($Path) | ConvertFrom-Json -AsHashtable
        if ($report -isnot [Collections.IDictionary]) { throw 'Expected JSON object' }
        foreach ($field in @('kind', 'version', 'status')) {
            if (-not $report.Contains($field) -or $report[$field] -isnot [string] -or [string]::IsNullOrWhiteSpace($report[$field])) {
                throw 'Missing report metadata'
            }
        }
        if ($report.schema_version -ne 1) { throw 'Unsupported measurement schema' }
        if ($Mode -eq 'benchmark') {
            if ($report.kind -ne 'parkbound-cpu-benchmark' -or $report.status -notin @('complete', 'partial') -or
                $report.fixture.case -ne $CaseName -or $report.simulation.requested_samples -ne $BenchmarkTicks) {
                throw 'Unexpected CPU workload'
            }
            foreach ($phase in @($report.warmup, $report.simulation, $report.navigation.build, $report.navigation.route,
                $report.geometry.terrain, $report.geometry.guests, $report.geometry.train)) {
                Assert-Phase $phase
            }
            if ($report.simulation_ticks.advanced -ne $report.simulation.samples_ms.Count -or
                ($report.simulation_ticks.after - $report.simulation_ticks.before) -ne $report.simulation_ticks.advanced) {
                throw 'Inconsistent simulation tick count'
            }
            if ($report.persistence.status -notin @('supported', 'unsupported', 'budget_exceeded')) {
                throw 'Unexpected persistence outcome'
            }
            if ($report.persistence.status -eq 'supported') {
                foreach ($phase in @($report.persistence.encode, $report.persistence.decode,
                    $report.persistence.filesystem.save, $report.persistence.filesystem.load)) {
                    Assert-Phase $phase
                }
            }
            $metrics = [ordered]@{
                fixture = $report.fixture
                simulation_ticks = $report.simulation_ticks
                simulation_timing = $report.simulation.timing
                path_graph = $report.navigation.graph
                persistence_status = $report.persistence.status
            }
        }
        elseif ($Mode -in @('frames-vsync', 'frames-uncapped')) {
            if ($report.kind -ne 'interactive_frame_profile' -or
                $report.status -notin @('completed', 'deadline_exceeded', 'window_unavailable') -or
                $report.completed -ne ($report.status -eq 'completed') -or
                $report.requested_samples -ne $ProfileSamples -or
                $report.frame_samples_ms -isnot [array] -or
                $report.measured_samples -ne $report.frame_samples_ms.Count -or
                $report.measured_samples -gt $report.requested_samples -or
                ($report.completed -and $report.measured_samples -ne $report.requested_samples)) {
                throw 'Unexpected frame measurement'
            }
            Assert-Timing -Timing $report.frame_interval_ms -Samples $report.frame_samples_ms
            $expectedPresentMode = if ($Mode -eq 'frames-uncapped') { 'AutoNoVsync' } else { 'AutoVsync' }
            if ($report.window.requested_present_mode -ne $expectedPresentMode -or $report.gpu_timing_collected -ne $false) {
                throw 'Unexpected frame method'
            }
            $metrics = [ordered]@{
                measured_samples = $report.measured_samples
                frame_interval_ms = $report.frame_interval_ms
                mean_main_loop_fps = $report.mean_main_loop_fps
                window = $report.window
                adapter = $report.adapter
                simulation = $report.simulation
            }
        }
        else { throw 'Unknown measurement mode' }
        return [ordered]@{
            valid = $true
            failure = $null
            version = $report.version
            kind = $report.kind
            status = $report.status
            metrics = $metrics
        }
    }
    catch {
        # Exception messages can contain absolute paths; keep private paths out
        # of the persisted wrapper summary. Retain the raw report for diagnosis.
        return [ordered]@{ valid = $false; failure = 'report_invalid'; version = $null; kind = $null; status = $null; metrics = $null }
    }
}

function Invoke-Measurement(
    [string] $Name,
    [string[]] $Arguments,
    [int] $TimeoutSeconds,
    [string] $Mode,
    [string] $CaseName,
    [int] $Repetition
) {
    $reportPath = Join-Path $outputRoot "$Name.json"
    if (Test-Path -LiteralPath $reportPath) { throw 'Measurement report already exists' }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = [Diagnostics.ProcessStartInfo]::new()
    $process.StartInfo.FileName = $executablePath
    $process.StartInfo.WorkingDirectory = [Environment]::SystemDirectory
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.CreateNoWindow = $true
    $process.StartInfo.WindowStyle = if ($Mode.StartsWith('frames-')) {
        [Diagnostics.ProcessWindowStyle]::Normal
    } else { [Diagnostics.ProcessWindowStyle]::Hidden }
    foreach ($argument in $Arguments) {
        # ArgumentList preserves Unicode and spaces without shell interpolation.
        $process.StartInfo.ArgumentList.Add($argument)
    }
    $watch = [Diagnostics.Stopwatch]::new()
    $samples = 0
    $peakWorking = $null
    $peakPrivate = $null
    $reportedPeakWorking = $null
    $observedCpuMilliseconds = $null
    $exitCode = $null
    $failure = $null
    $started = $false
    $cleanupFailed = $false
    try {
        $watch.Start()
        $started = $process.Start()
        if (-not $started) { throw 'Process did not start' }
        while (-not $process.HasExited) {
            try {
                $process.Refresh()
                $working = $process.WorkingSet64
                $private = $process.PrivateMemorySize64
                $osPeak = $process.PeakWorkingSet64
                $observedCpuMilliseconds = $process.TotalProcessorTime.TotalMilliseconds
                $samples++
                if ($null -eq $peakWorking -or $working -gt $peakWorking) { $peakWorking = $working }
                if ($null -eq $peakPrivate -or $private -gt $peakPrivate) { $peakPrivate = $private }
                if ($null -eq $reportedPeakWorking -or $osPeak -gt $reportedPeakWorking) { $reportedPeakWorking = $osPeak }
            }
            catch {
                if (-not $process.HasExited) { $failure = 'process_sampling_failed' }
            }
            if ($watch.Elapsed.TotalSeconds -ge $TimeoutSeconds) {
                $failure = 'process_timeout'
                break
            }
            [void] $process.WaitForExit($sampleIntervalMilliseconds)
        }
        if ($process.HasExited) {
            $exitCode = $process.ExitCode
            if ($exitCode -ne 0) { $failure = 'process_nonzero_exit' }
        }
    }
    catch {
        if ($null -eq $failure) { $failure = 'process_launch_or_monitor_failed' }
    }
    finally {
        if ($started -and -not $process.HasExited) {
            try {
                # Scope termination to this owned process and its children.
                $process.Kill($true)
                if (-not $process.WaitForExit(5000)) { $cleanupFailed = $true }
            }
            catch { $cleanupFailed = $true }
        }
        $watch.Stop()
        $process.Dispose()
    }
    if ($cleanupFailed) { throw 'Unable to stop an owned measurement process; further runs cancelled' }
    $validation = Read-MeasurementReport -Path $reportPath -Mode $Mode -CaseName $CaseName
    if ($null -eq $failure -and -not $validation.valid) { $failure = $validation.failure }
    return [ordered]@{
        name = $Name
        mode = $Mode
        case = $CaseName
        repetition = $Repetition
        report_file = "$Name.json"
        wrapper_status = $(if ($null -eq $failure) { 'completed' } else { 'failed' })
        failure = $failure
        exit_code = $exitCode
        runtime_kind = $validation.kind
        runtime_status = $validation.status
        runtime_version = $validation.version
        runtime_metrics = $validation.metrics
        elapsed_milliseconds = [Math]::Round($watch.Elapsed.TotalMilliseconds, 3)
        timeout_seconds = $TimeoutSeconds
        memory = [ordered]@{
            sampling_interval_milliseconds = $sampleIntervalMilliseconds
            sample_count = $samples
            sampled_peak_working_set_bytes = $peakWorking
            sampled_peak_private_bytes = $peakPrivate
            observed_os_peak_working_set_bytes = $reportedPeakWorking
            observed_cpu_milliseconds = $observedCpuMilliseconds
            note = 'Working/private values are maxima of 200 ms samples, not guaranteed instantaneous peaks. OS peak working set is a separate process counter. No GPU-memory or whole-system-memory claim.'
        }
    }
}

if (-not $IsWindows) { throw 'Performance measurement requires Windows.' }
if (-not [IO.Path]::IsPathFullyQualified($OutputDirectory)) {
    throw 'OutputDirectory must be an absolute new directory.'
}
$outputRoot = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $outputRoot) { throw 'OutputDirectory already exists; choose a new directory.' }
$outputParent = [IO.Path]::GetDirectoryName($outputRoot)
if (-not (Test-Path -LiteralPath $outputParent -PathType Container)) {
    throw 'The output parent directory must already exist.'
}
$Cases = @($Cases | ForEach-Object { $_.ToLowerInvariant() })
if ($Cases.Count -eq 0 -or @($Cases | Select-Object -Unique).Count -ne $Cases.Count) {
    throw 'Select at least one benchmark case without duplicates.'
}
if ([string]::IsNullOrWhiteSpace($Executable)) {
    $packagedExecutable = Join-Path $PSScriptRoot 'Parkbound.exe'
    $Executable = if (Test-Path -LiteralPath $packagedExecutable -PathType Leaf) {
        $packagedExecutable
    } else { Join-Path $PSScriptRoot '../target/release/parkbound.exe' }
}
$executablePath = (Resolve-Path -LiteralPath $Executable).Path
if (-not (Test-Path -LiteralPath $executablePath -PathType Leaf)) { throw 'Executable is not a file.' }
$binaryHash = (Get-FileHash -LiteralPath $executablePath -Algorithm SHA256).Hash.ToLowerInvariant()
$binaryBytes = (Get-Item -LiteralPath $executablePath).Length
$source = Get-SourceRecord
$hardware = Get-HardwareRecord
# No -Force: never merge results into a previous measurement directory.
[void] (New-Item -ItemType Directory -Path $outputRoot)
$runs = [Collections.Generic.List[object]]::new()
$summary = [ordered]@{
    kind = 'parkbound-measurement-wrapper'
    schema_version = 1
    started_utc = [DateTime]::UtcNow.ToString('o')
    finished_utc = $null
    status = 'running'
    executable_sha256 = $binaryHash
    executable_bytes = $binaryBytes
    executable_unchanged_after_runs = $null
    source = $source
    hardware = $hardware
    configuration = [ordered]@{
        cases = $Cases
        repetitions = $Repetitions
        benchmark_ticks = $BenchmarkTicks
        profile_samples = $ProfileSamples
        frame_profiles_enabled = -not $SkipFrames.IsPresent
        execution = 'Sequential child processes; no normal-player save or settings commands.'
    }
    runs = $runs
    limitations = @(
        'No performance threshold is imposed by this wrapper; preserve incomplete or unsupported runtime outcomes as measured evidence.',
        'Frame profiles cover only the current baseline UI. Main-loop intervals are not GPU execution or physically presented-frame timings.',
        'The full 256x256/5000-guest/50-ride target is unsupported by the current single-circuit model.',
        'Absolute executable/output paths, account names, machine names, and serial numbers are omitted from this summary.'
    )
}
try {
    foreach ($case in $Cases) {
        for ($repetition = 1; $repetition -le $Repetitions; $repetition++) {
            $name = "cpu-$case-$repetition"
            Write-Host "Measuring $name"
            $reportPath = Join-Path $outputRoot "$name.json"
            $runs.Add((Invoke-Measurement -Name $name -Mode 'benchmark' -CaseName $case -Repetition $repetition `
                -Arguments @('--benchmark', $reportPath, '--benchmark-case', $case, '--benchmark-ticks', "$BenchmarkTicks") `
                -TimeoutSeconds $benchmarkTimeoutSeconds))
        }
    }
    if (-not $SkipFrames) {
        foreach ($presentMode in @('vsync', 'uncapped')) {
            $name = "frames-$presentMode"
            Write-Host "Measuring $name; keep the game's window visible and unobscured."
            $reportPath = Join-Path $outputRoot "$name.json"
            $arguments = @('--profile-frames', $reportPath, '--profile-samples', "$ProfileSamples")
            if ($presentMode -eq 'uncapped') { $arguments += '--profile-no-vsync' }
            $runs.Add((Invoke-Measurement -Name $name -Mode "frames-$presentMode" -CaseName 'baseline' -Repetition 1 `
                -Arguments $arguments -TimeoutSeconds $profileTimeoutSeconds))
        }
    }
    $summary.executable_unchanged_after_runs = (Get-FileHash -LiteralPath $executablePath -Algorithm SHA256).Hash.ToLowerInvariant() -eq $binaryHash
    $summary.status = if ($summary.executable_unchanged_after_runs -and @($runs | Where-Object { $_.wrapper_status -ne 'completed' }).Count -eq 0) {
        'completed'
    } else { 'failed' }
}
catch {
    $summary.status = 'failed'
    throw
}
finally {
    $summary.finished_utc = [DateTime]::UtcNow.ToString('o')
    Write-NewJson -Path (Join-Path $outputRoot 'summary.json') -Value $summary
}
if ($summary.status -ne 'completed') { throw 'One or more measurements failed; inspect summary.json and retained run reports.' }
Write-Host 'Measurement complete. Inspect summary.json and each run report; no target performance is implied.'
