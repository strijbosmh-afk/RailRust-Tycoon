# Controlled large-map rendering and GPU diagnostics — 0.9.0

Date: 20 October 2026. This checkpoint adds an isolated rendering preview for the existing diagnostic parks and opt-in instrumented GPU/CPU render spans. Implementation is in progress on `codex/large-map-gpu-diagnostics`, based on the 0.8.0 checkpoint `318e729`. Runtime verification, source/review identity, CI, and packaging evidence are pending below.

**No reference PC is selected.** The user explicitly excluded this poorly performing development PC as a performance reference. Any timings here are local diagnostics. They cannot establish minimum hardware, representative FPS, a portable speedup, or acceptance/failure of the proposed 60 FPS target. The full 256 × 256 / 5,000-guest / 50-ride gate PRF-05 remains in progress.

## Scope and preserved behavior

Normal launches still create the editable 32 × 32 demonstration, and normal loading still accepts only 32 × 32 parks. Save schema 1/content 1, limits, user-data directory, backup handling, deterministic simulation, cached paths, and retained guest rendering remain unchanged. The new preview does not write or load player saves.

`--render-case baseline|save-scale|stress` chooses an isolated fixture. The optional `--render-camera overview|crowd|ride`, `--render-rotation 0|1|2|3`, and `--render-underground` flags require it. The default is overview, rotation 0, Surface. The fixtures are the existing seeded production simulation:

| Case | Map | Guests | Paths | Objects | Circuit/train | Codec save capacity |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | 32 × 32 | 12 | 37 | 92 | One 46-piece circuit / one train | Supported |
| save-scale | 256 × 256 | 256 | 1,024 | 1,070 | One 46-piece circuit / one train | Supported |
| stress | 256 × 256 | 5,000 | 1,024 | 1,070 | One 46-piece circuit / one train | Unsupported: exceeds 256-guest bound |

Every case uses seed 20261020. Codec capacity is distinct from preview controls: **saving and loading are disabled in every preview**, including save-scale. Large cases use the connected grid at tiles 80..111 and the separate circuit near tiles 7..24/4..10. Neither fabricates fifty rides or mixed-park realism.

Standalone previews permit camera movement, inspection, pause/speed, grid, Underground, and Find train. Terrain changes, path removal/restoration, train Open/Close, and persistence are disabled. The interface labels the preview. Automated profiles and captures additionally suppress ordinary input so their workload cannot change mid-run.

Overview fits the entire scene; Crowd focuses the path network; Ride focuses the circuit. Larger-map zoom and pan limits apply only to diagnostic views. Shared `scene_bounds` conservatively includes terrain, decoration, paths, track, cars, and high stranded guests; `path_focus` derives the projected path-deck bounds. Rendering and picking retain square-map assumptions; this does not introduce rectangular-map play or a general large-park editor.

## CLI and output safety

Preview without automatic output:

```powershell
.\target\release\parkbound.exe --render-case stress --render-camera crowd
```

A controlled capture uses only the `overview` capture scene; the shared render flags select camera, rotation, and visibility. `--capture-map flat|terraced`, `--capture-small-window`, and `--capture-ticks 0..1800` are allowed. The output must be a new absolute `.png` path. Do not combine large fixtures with historical named path/train/save capture scenes, which assume the small demonstration's locations.

```powershell
.\target\release\parkbound.exe --render-case save-scale --render-camera overview --render-rotation 2 --capture-frame 'C:\Parkbound-QA\large-overview.png' --capture-scene overview --capture-ticks 600
```

Existing `--profile-frames <absolute-new.json>` remains the baseline main-loop profile. Add common render flags for a controlled large fixture, and `--profile-gpu` to request instrumented render diagnostics. GPU opt-in without a render case profiles the ordinary 32 × 32 baseline. `--profile-samples` accepts 120..3600, default 600; `--profile-no-vsync` requests AutoNoVsync instead of AutoVsync. The profile requests a visible physical 1920 × 1080 window and records actual size/present-mode/focus/visibility/occlusion. Warmup requires at least 120 frames and two seconds; the runtime deadline is 120 seconds.

```powershell
.\target\release\parkbound.exe --profile-frames 'C:\Parkbound-QA\large-gpu.json' --profile-samples 120 --profile-gpu --render-case stress --render-camera crowd
```

Use an existing parent folder and a fresh output path. Mixed save/CPU-benchmark/render modes, duplicate options, unsupported values, and orphan flags fail rather than being silently ignored. Frame JSON uses exclusive creation; no prior report may be replaced. A deadline/window-loss outcome retains partial evidence and exits unsuccessfully.

The packaged/source wrapper adds `-SkipCpu`, `-RenderCase`, `-RenderCamera`, `-RenderRotation`, `-RenderUnderground`, and `-GpuDiagnostics`. The defaults preserve the previous three-case CPU batch and ordinary baseline frame profiles without GPU instrumentation. Frame filenames remain `frames-vsync.json` and `frames-uncapped.json`; the summary records the actual configuration. This short frame-only run avoids repeating CPU fixtures:

```powershell
pwsh -NoProfile -File scripts/measure-performance.ps1 -OutputDirectory 'C:\Parkbound-QA\new-large-run' -SkipCpu -RenderCase stress -RenderCamera crowd -GpuDiagnostics -ProfileSamples 120
```

`-SkipCpu -SkipFrames` is rejected. Explicit frame options with `-SkipFrames` are rejected. The wrapper validates actual fixture counts/configuration and the GPU method/counts; honest unsupported or unavailable GPU timestamps remain valid diagnostic outcomes. It limits report reads to 64 MiB, runs children sequentially with a 150-second frame timeout, and stops only its owned processes. It retains allowlisted hardware metadata, executable/source identity, and 200 ms sampled process memory without account/machine names or absolute user paths. No performance threshold is applied.

## What each timing means

| Evidence | Measures | Does not measure |
| --- | --- | --- |
| Main-loop frame intervals | Consecutive main-world `Last` schedule wall-clock intervals | GPU execution or actual display presentation |
| Simulation/scene CPU hooks | Existing authoritative tick and mesh-update system wall time | Complete render-thread work or GPU transfers |
| Render-span `cpu_elapsed` | CPU wall time inside Bevy's instrumented render-pass encoding spans | Total render-thread CPU, extraction, asset preparation, upload, presentation, or GPU execution |
| Render-span `gpu_elapsed` | Actual device timestamp differences, converted by Bevy to milliseconds | Whole-frame GPU time, CPU work, or display latency |
| Pipeline counters | Available instrumented device counters | Timing or a complete count of visible objects |
| `render_work` | Current generated mesh vertices/indices, logical CPU payload bytes, guest-cache counters | GPU allocation/upload bytes/time, buffer capacities, or visible-object counts |

GPU instrumentation uses the existing Bevy 0.17.3 `RenderDiagnosticsPlugin`; it does not force unsupported device features. Capabilities describe enabled `RenderDevice` features, not a separate hardware support query. Encoder-only GPU spans require timestamp-query and inside-encoder features; per-pass spans additionally require inside-pass support. Actual retained encoder-only samples are valid even when the per-pass capability is false. `gpu_timing_collected` is true only when actual retained GPU samples exist.

The `gpu_diagnostics` report distinguishes `samples_received`, `unsupported`, `no_samples_received`, `no_render_app`, and `device_unavailable`. Unknown capabilities remain null. Opt-out produces a null diagnostic object and false collection flag. No CPU-derived number is substituted for missing GPU measurements.

Readback is asynchronous and Bevy's transfer to the main world retains only the latest batch. Measurement timestamps identify delivery, and `observed_main_frame` identifies observation. Source frame IDs, overwritten-batch counts, and exact CPU/GPU frame joins are unavailable. The probe deduplicates deliveries and excludes already-delivered pre-window samples, but late warmup results can arrive inside the receipt window and final in-flight results can be missing. It performs no blocking drain. Counts therefore need not equal main-loop samples.

Reports retain up to 128 paths of at most 512 UTF-8 bytes, 4,096 samples per path, and 131,072 total samples, with omitted/invalid/pre-window counts. Negative and nonfinite values are rejected. Summaries retain untrimmed nearest-rank percentiles. Nested/overlapping spans must not be added into a total; exact receipt-window limitations remain alongside the raw data.

Physical GPU upload bytes/time and presentation timing remain explicitly unmeasured. A large pass timestamp alone cannot identify an upload, driver, or presentation bottleneck. Instrumentation itself adds work; these runs are diagnostic observations, not an uninstrumented FPS comparison.

## Acceptance and evidence

The bounded checkpoint requires correct controlled rendering, truthful diagnostic attribution, isolation from player persistence, and a relocated distributable build. It does not close PRF-05's hardware/full-workload gate.

| Item | Current evidence |
| --- | --- |
| Source / PR / CI | Branch `codex/large-map-gpu-diagnostics`, base `318e729`; implementation/review/CI pending |
| World/camera correctness | New tests cover 256-map corner/edge picking, scene/depth bounds, all rotations/views, and retained-renderer map replacement; full run pending |
| Preview/input/persistence isolation | Implementation present; integration tests/runtime proof pending |
| GPU reporting | Capability/status, delivery deduplication, window boundaries, invalid samples, and bounded storage tests implemented; full run pending |
| PowerShell wrapper | 44 AST-isolated checks passed; parser clean. Covers legacy reports, new fixtures, options, partial/unsupported outcomes, and contradictory GPU claims. No game/Cargo/CIM ran in this test |
| Formatting / release tests / strict lint / build | Initial release suite passed 172 tests (107 game / 65 simulation); additional integration regressions and final source/build checks pending |
| Inspected render captures and actual local GPU reports | Pending; do not infer timestamp support or performance before running |
| Save continuation and ordinary 32-map regression | Pending for 0.9.0; historical 0.8.0 evidence remains in its report |
| Executable / notices / package hashes / relocated launch | Pending |
| Representative PC / full target | No reference PC selected; PRF-05 remains in progress |

Verify a small, bounded matrix: full-map overview and focused moving crowd, all four rotations, Surface/Underground, and a ride view. Exercise ordinary 32-map/save behavior separately. Inspect the rendered image; successful PNG creation alone is insufficient. Profiles should report actual mesh counts, window state, and received GPU spans or explicit unavailability. Preserve unexpected or partial outcomes.

After results exist, record whether they justify a specific next rendering change or further instrumentation. Keep source checks, local GPU observation, package relocation, native input, clean-machine compatibility, and hardware acceptance as distinct evidence. Native Save/Load/Backup, O/F/buttons, physical Delete, panning/dragging/physical display, and clean-PC offline/standard-user gaps remain open unless directly reverified.

Historical reports and both diagnostic datasets remain unchanged. The current source is in the original shared OneDrive folder:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The recorded second-PC clone is separate:

```text
C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon
```

Push source and handoff records before changing PCs. Use one active editing/build/Git session in the shared folder. User saves need separate transfer. Archive hashes change after guide refreshes; use the adjacent `.zip.sha256` and verify all final payloads/guides rather than embedding the archive hash inside its own documentation.
