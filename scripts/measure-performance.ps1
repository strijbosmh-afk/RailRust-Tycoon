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
    [switch] $SkipFrames,
    [switch] $SkipCpu,
    [ValidateSet('baseline', 'save-scale', 'stress')]
    [string] $RenderCase = 'baseline',
    [ValidateSet('overview', 'crowd', 'ride')]
    [string] $RenderCamera = 'overview',
    [ValidateRange(0, 3)]
    [int] $RenderRotation = 0,
    [switch] $RenderUnderground,
    [switch] $GpuDiagnostics
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$workspaceDirectory = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$sampleIntervalMilliseconds = 200
$benchmarkTimeoutSeconds = 180
$profileTimeoutSeconds = 150

function Assert-RunOptions(
    [bool] $NoCpu,
    [bool] $NoFrames,
    [Collections.IDictionary] $BoundOptions
) {
    if ($NoCpu -and $NoFrames) { throw 'SkipCpu and SkipFrames would leave no measurements.' }
    $frameOptions = @('ProfileSamples', 'RenderCase', 'RenderCamera', 'RenderRotation', 'RenderUnderground', 'GpuDiagnostics')
    if ($NoFrames -and @($frameOptions | Where-Object { $BoundOptions.Keys -contains $_ }).Count -ne 0) {
        throw 'Frame options cannot be used with SkipFrames.'
    }
}

function Get-FrameArguments(
    [string] $ReportPath,
    [int] $Samples,
    [bool] $Uncapped,
    [bool] $ControlledRender,
    [string] $Fixture,
    [string] $Camera,
    [int] $Rotation,
    [bool] $Underground,
    [bool] $Gpu
) {
    $arguments = @('--profile-frames', $ReportPath, '--profile-samples', "$Samples")
    if ($Uncapped) { $arguments += '--profile-no-vsync' }
    if ($ControlledRender) {
        $arguments += @('--render-case', $Fixture, '--render-camera', $Camera, '--render-rotation', "$Rotation")
        if ($Underground) { $arguments += '--render-underground' }
    }
    if ($Gpu) { $arguments += '--profile-gpu' }
    return $arguments
}

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

function Assert-RenderScene(
    [object] $Scene,
    [bool] $ControlledRender,
    [string] $Fixture,
    [string] $Camera,
    [int] $Rotation,
    [bool] $Underground
) {
    $expectedMode = if ($ControlledRender) { 'render_preview' } else { 'normal_demo' }
    $expectedView = if ($Underground) { 'underground' } else { 'surface' }
    $mapSide = if ($Fixture -eq 'baseline') { 32 } else { 256 }
    $guests = switch ($Fixture) { 'baseline' { 12 }; 'save-scale' { 256 }; 'stress' { 5000 }; default { throw 'Unknown render fixture' } }
    $paths = if ($Fixture -eq 'baseline') { 37 } else { 1024 }
    $objects = if ($Fixture -eq 'baseline') { 92 } else { 1070 }
    if ($Scene -isnot [Collections.IDictionary] -or $Scene.mode -ne $expectedMode -or
        $Scene.camera -ne $Camera -or $Scene.rotation -ne $Rotation -or $Scene.view -ne $expectedView -or
        $Scene.edits_and_saves_enabled -isnot [bool] -or $Scene.edits_and_saves_enabled -ne $false) {
        throw 'Unexpected controlled render configuration'
    }
    $actual = $Scene.fixture
    if ($actual -isnot [Collections.IDictionary] -or $actual.case -ne $Fixture -or
        $actual.seed -ne 20261020 -or $actual.map_width -ne $mapSide -or $actual.map_height -ne $mapSide -or
        $actual.tiles -ne ($mapSide * $mapSide) -or $actual.guests -ne $guests -or
        $actual.paths -ne $paths -or $actual.objects -ne $objects -or
        $actual.rides -ne 1 -or $actual.trains -ne 1 -or $actual.track_pieces -ne 46 -or
        $actual.save_supported -isnot [bool] -or $actual.save_supported -ne ($Fixture -ne 'stress')) {
        throw 'Unexpected render fixture counts'
    }
    if ($Scene.zoom_relative_to_map_fit -isnot [ValueType] -or
        -not [double]::IsFinite([double] $Scene.zoom_relative_to_map_fit) -or $Scene.zoom_relative_to_map_fit -le 0 -or
        $Scene.pan_world -isnot [array] -or $Scene.pan_world.Count -ne 2) {
        throw 'Invalid render camera geometry'
    }
    foreach ($coordinate in $Scene.pan_world) {
        if ($coordinate -isnot [ValueType] -or -not [double]::IsFinite([double] $coordinate)) {
            throw 'Invalid render camera coordinate'
        }
    }
}

function Assert-NaturalNumber([object] $Value) {
    if ($Value -isnot [ValueType] -or $Value -is [bool] -or
        -not [double]::IsFinite([double] $Value) -or [double] $Value -lt 0 -or
        [Math]::Truncate([double] $Value) -ne [double] $Value) {
        throw 'Invalid nonnegative integer'
    }
}

function Assert-GpuDiagnostics([object] $Report, [bool] $Collected) {
    if ($Report -isnot [Collections.IDictionary] -or
        $Report.status -notin @('samples_received', 'unsupported', 'no_samples_received', 'no_render_app', 'device_unavailable') -or
        $Report.gpu_timing_collected -isnot [bool] -or $Report.gpu_timing_collected -ne $Collected -or
        $Report.method -ne 'bevy_render_diagnostics_async_readback' -or $Report.time_unit -ne 'ms' -or
        $Report.source_frame_correlation -ne 'unavailable_receipt_window_only' -or
        $Report.spans -isnot [array]) {
        throw 'Invalid GPU diagnostic method'
    }
    if ($Report.capabilities.device_features_known -isnot [bool]) { throw 'Invalid device capability availability' }
    foreach ($field in @('timestamp_query', 'timestamp_inside_encoders', 'timestamp_inside_passes',
        'pipeline_statistics_query', 'pass_timestamps_enabled')) {
        if (($Report.capabilities.device_features_known -and $Report.capabilities[$field] -isnot [bool]) -or
            (-not $Report.capabilities.device_features_known -and $null -ne $Report.capabilities[$field])) {
            throw 'Invalid GPU capability metadata'
        }
    }
    if ($null -ne $Report.capabilities.adapter_supported_features -or
        $null -ne $Report.bounded_samples.source_dropped_batches -or
        $Report.upload.status -ne 'unmeasured' -or $null -ne $Report.upload.bytes -or $null -ne $Report.upload.duration_ms) {
        throw 'Unexpected GPU attribution claim'
    }
    foreach ($field in @('max_paths', 'max_path_bytes', 'max_samples_per_path', 'max_total_samples', 'retained',
        'omitted_limit', 'invalid_values', 'pre_window_latest_observations')) {
        Assert-NaturalNumber $Report.bounded_samples[$field]
    }
    $bounds = $Report.bounded_samples
    if ($bounds.max_paths -lt 1 -or $bounds.max_paths -gt 128 -or
        $bounds.max_path_bytes -lt 1 -or $bounds.max_path_bytes -gt 512 -or
        $bounds.max_samples_per_path -lt 1 -or $bounds.max_samples_per_path -gt 4096 -or
        $bounds.max_total_samples -lt 1 -or $bounds.max_total_samples -gt 131072 -or
        $Report.spans.Count -gt $bounds.max_paths -or $bounds.retained -gt $bounds.max_total_samples) {
        throw 'Invalid GPU diagnostic bounds'
    }
    $pathsSeen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $retained = 0
    $gpuSamples = 0
    foreach ($span in $Report.spans) {
        if ($span.path -isnot [string] -or [string]::IsNullOrWhiteSpace($span.path) -or
            [Text.Encoding]::UTF8.GetByteCount($span.path) -gt $bounds.max_path_bytes -or -not $pathsSeen.Add($span.path) -or
            $span.kind -notin @('cpu_elapsed', 'gpu_elapsed', 'pipeline_counter') -or
            $span.samples -isnot [array] -or $span.samples.Count -gt $bounds.max_samples_per_path) {
            throw 'Invalid GPU diagnostic span'
        }
        $expectedUnit = if ($span.kind -eq 'pipeline_counter') { 'count' } else { 'ms' }
        if ($span.unit -ne $expectedUnit) { throw 'Invalid GPU diagnostic unit' }
        foreach ($sample in $span.samples) {
            Assert-NaturalNumber $sample.observed_main_frame
            foreach ($value in @($sample.receipt_offset_ms, $sample.value)) {
                if ($value -isnot [ValueType] -or $value -is [bool] -or
                    -not [double]::IsFinite([double] $value) -or [double] $value -lt 0) {
                    throw 'Invalid GPU diagnostic sample'
                }
            }
        }
        if ($span.samples.Count -eq 0) {
            if ($null -ne $span.summary) { throw 'Unexpected empty GPU summary' }
        }
        else {
            if ($span.summary.samples -ne $span.samples.Count) { throw 'Invalid GPU summary count' }
            foreach ($field in @('min', 'mean', 'p50', 'p95', 'p99', 'max')) {
                $value = $span.summary[$field]
                if ($value -isnot [ValueType] -or $value -is [bool] -or
                    -not [double]::IsFinite([double] $value) -or [double] $value -lt 0) {
                    throw 'Invalid GPU summary statistic'
                }
            }
        }
        $retained += $span.samples.Count
        if ($span.kind -eq 'gpu_elapsed') { $gpuSamples += $span.samples.Count }
    }
    if ($retained -ne $bounds.retained -or $Collected -ne ($gpuSamples -gt 0) -or
        ($Report.status -eq 'samples_received') -ne $Collected -or
        ($Collected -and (-not $Report.capabilities.device_features_known -or
            -not $Report.capabilities.timestamp_query -or -not $Report.capabilities.timestamp_inside_encoders))) {
        throw 'Contradictory GPU measurement status'
    }
}

function Read-MeasurementReport(
    [string] $Path,
    [string] $Mode,
    [string] $CaseName,
    [bool] $ControlledRender = $false,
    [string] $Camera = 'overview',
    [int] $Rotation = 0,
    [bool] $Underground = $false,
    [bool] $Gpu = $false
) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return [ordered]@{ valid = $false; failure = 'report_missing'; version = $null; kind = $null; status = $null; metrics = $null }
    }
    try {
        if ((Get-Item -LiteralPath $Path).Length -gt 64MB) { throw 'Measurement report exceeds the bounded input size' }
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
            if ($report.geometry.Contains('guest_renderer')) {
                $render = $report.geometry.guest_renderer
                foreach ($phase in @($render.cold, $render.moving_update, $render.legacy_moving)) {
                    Assert-Phase $phase
                }
                if (($render.ticks_after - $render.ticks_before) -ne $render.moving_update.samples_ms.Count -or
                    ($render.stats_after.updates - $render.stats_before.updates) -ne $render.moving_update.samples_ms.Count) {
                    throw 'Inconsistent moving render workload'
                }
                $bothMovingComplete = $render.moving_update.status -eq 'complete' -and $render.legacy_moving.status -eq 'complete'
                if ($render.matched_moving_workloads -isnot [bool] -or $render.matched_moving_workloads -ne $bothMovingComplete) {
                    throw 'Inconsistent matched moving workload claim'
                }
                if ($bothMovingComplete -and $render.moving_update.work_units -ne $render.legacy_moving.work_units) {
                    throw 'Mismatched optimized and legacy render workloads'
                }
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
            if ($report.window.requested_present_mode -ne $expectedPresentMode -or $report.gpu_timing_collected -isnot [bool]) {
                throw 'Unexpected frame method'
            }
            if ($report.Contains('render_scene')) {
                Assert-RenderScene -Scene $report.render_scene -ControlledRender $ControlledRender -Fixture $CaseName `
                    -Camera $Camera -Rotation $Rotation -Underground $Underground
            }
            elseif ($ControlledRender -or $Gpu -or $report.version -match '^0\.(?:9|[1-9][0-9]+)\.') {
                throw 'Missing render fixture metadata'
            }
            if ($Gpu) {
                Assert-GpuDiagnostics -Report $report.gpu_diagnostics -Collected $report.gpu_timing_collected
            }
            elseif ($report.gpu_timing_collected -or ($report.Contains('gpu_diagnostics') -and $null -ne $report.gpu_diagnostics)) {
                throw 'Unexpected GPU measurement opt-in'
            }
            $metrics = [ordered]@{
                measured_samples = $report.measured_samples
                frame_interval_ms = $report.frame_interval_ms
                mean_main_loop_fps = $report.mean_main_loop_fps
                window = $report.window
                adapter = $report.adapter
                simulation = $report.simulation
                render_scene = $(if ($report.Contains('render_scene')) { $report.render_scene } else { $null })
                gpu_diagnostics = $(if ($Gpu) { $report.gpu_diagnostics } else { $null })
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
    $validation = Read-MeasurementReport -Path $reportPath -Mode $Mode -CaseName $CaseName `
        -ControlledRender $renderOptionsSpecified -Camera $RenderCamera -Rotation $RenderRotation `
        -Underground $RenderUnderground.IsPresent -Gpu $GpuDiagnostics.IsPresent
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
Assert-RunOptions -NoCpu $SkipCpu.IsPresent -NoFrames $SkipFrames.IsPresent -BoundOptions $PSBoundParameters
$renderOptionsSpecified = @('RenderCase', 'RenderCamera', 'RenderRotation', 'RenderUnderground' |
    Where-Object { $PSBoundParameters.ContainsKey($_) }).Count -ne 0
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
$RenderCase = $RenderCase.ToLowerInvariant()
$RenderCamera = $RenderCamera.ToLowerInvariant()
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
    hardware_role = 'Local diagnostic host only; no reference PC has been selected. These measurements do not establish target hardware requirements.'
    configuration = [ordered]@{
        cases = $Cases
        repetitions = $Repetitions
        benchmark_ticks = $BenchmarkTicks
        profile_samples = $ProfileSamples
        cpu_benchmarks_enabled = -not $SkipCpu.IsPresent
        frame_profiles_enabled = -not $SkipFrames.IsPresent
        controlled_render_enabled = $renderOptionsSpecified
        render_case = $RenderCase
        render_camera = $RenderCamera
        render_rotation = $RenderRotation
        render_view = $(if ($RenderUnderground) { 'underground' } else { 'surface' })
        gpu_diagnostics_enabled = $GpuDiagnostics.IsPresent
        execution = 'Sequential child processes; no normal-player save or settings commands.'
    }
    runs = $runs
    limitations = @(
        'No performance threshold is imposed by this wrapper; preserve incomplete or unsupported runtime outcomes as measured evidence.',
        'Frame profiles use the baseline UI unless an explicit controlled render fixture is selected. Main-loop intervals are not GPU execution or physically presented-frame timings.',
        'Optional GPU diagnostics report instrumented timestamp spans when supported; asynchronous receipt, nested spans, and unmeasured presentation prevent a total frame-time inference.',
        'The full 256x256/5000-guest/50-ride target is unsupported by the current single-circuit model.',
        'Absolute executable/output paths, account names, machine names, and serial numbers are omitted from this summary.'
    )
}
try {
    if (-not $SkipCpu) {
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
    }
    if (-not $SkipFrames) {
        foreach ($presentMode in @('vsync', 'uncapped')) {
            $name = "frames-$presentMode"
            Write-Host "Measuring $name; keep the game's window visible and unobscured."
            $reportPath = Join-Path $outputRoot "$name.json"
            $arguments = @(Get-FrameArguments -ReportPath $reportPath -Samples $ProfileSamples `
                -Uncapped ($presentMode -eq 'uncapped') -ControlledRender $renderOptionsSpecified `
                -Fixture $RenderCase -Camera $RenderCamera -Rotation $RenderRotation `
                -Underground $RenderUnderground.IsPresent -Gpu $GpuDiagnostics.IsPresent)
            $runs.Add((Invoke-Measurement -Name $name -Mode "frames-$presentMode" -CaseName $RenderCase -Repetition 1 `
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
