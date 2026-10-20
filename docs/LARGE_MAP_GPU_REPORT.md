# Controlled large-map rendering and GPU diagnostics — 0.9.0

Date: 20 October 2026. This checkpoint adds an isolated rendering preview for the existing diagnostic parks and opt-in instrumented GPU/CPU render spans. Implementation `5d49b7e00ce25692aac3ad2d6dda86939afd0a47` is pushed on `codex/large-map-gpu-diagnostics`, based on the 0.8.0 checkpoint `318e729`, in [draft PR #7](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/7), stacked on PR #6. All 178 release tests, six local GPU profiles, inspected captures, bidirectional continuation, and implementation CI passed. Relocated package execution and all 24 payload hashes passed; the final refresh verified all fifteen current guides and all 24 payload hashes below.

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

## Observed local diagnostic results

Six production profiles completed with exit code 0 and valid wrapper reports: baseline/overview, save-scale/overview, and stress/crowd, each once with AutoVsync and once with AutoNoVsync. All requested 120 main-loop intervals. The three wrapper summaries and six raw profiles are retained byte-for-byte with a manifest in [the local dataset](https://github.com/strijbosmh-afk/RailRust-Tycoon/tree/codex/large-map-gpu-diagnostics/benchmarks/results/2026-10-20-large-map-gpu-local). All three summaries recorded the clean implementation commit above and the unchanged executable.

The diagnostic host was Windows 11 Enterprise 10.0.26200, Intel Core i5-9400T, Intel UHD 630 driver 31.0.101.2115, and 8,453,365,760 bytes of physical RAM. Bevy used the Vulkan backend. This is the explicitly excluded development PC, not reference hardware. Power, temperature, and background load were not controlled.

All six actual windows were 1920 × 1080 at scale 1, visible and focused throughout their measured intervals; size/present-mode configuration did not change. Occlusion was **unknown in all 120 intervals of every run**. Requested Auto modes may select backend fallbacks; these fields do not establish physical presentation timing.

Every report had `samples_received`, enabled timestamp-query/inside-encoder/inside-pass features and pipeline statistics, and 24 diagnostic paths: opaque, transparent, UI, and upscaling passes, each with CPU/GPU elapsed values and four pipeline counters. None reached storage limits or retained invalid values. Baseline uncapped excluded 24 already-delivered pre-window observations; the other reports recorded zero. Source dropped batches remain unknown.

| Fixture / camera | Requested mode | Main-loop intervals | Received GPU samples per named pass | Opaque-pass GPU mean ms | Landscape vertices |
| --- | --- | --- | --- | --- | --- |
| Baseline / overview | AutoVsync | 120 | 87 | 3.71345 | 99,282 |
| Baseline / overview | AutoNoVsync | 120 | 88 | 3.44888 | 99,282 |
| Save-scale / overview | AutoVsync | 120 | 85 | 18.94282 | 2,251,764 |
| Save-scale / overview | AutoNoVsync | 120 | 91 | 20.21746 | 2,251,764 |
| Stress / crowd | AutoVsync | 120 | 120 | 50.79779 | 2,251,764 |
| Stress / crowd | AutoNoVsync | 120 | 120 | 49.07778 | 2,251,764 |

Opaque values are **one instrumented GPU span**, not total GPU/frame time. The other three GPU spans, CPU encoding spans, pipeline counters, raw distributions, frame intervals, and memory samples remain in the dataset. Do not add spans together, subtract CPU values from main-loop values, infer an upload cost, or convert these local observations into a hardware requirement. Even the runs with 120 GPU samples have no exact source-frame correspondence.

The large landscape mesh had 2,251,764 vertices and the same number of indices, representing 117,091,728 logical CPU payload bytes. This full static mesh remains submitted in both overview and focused crowd views; the value is not physical GPU upload traffic. No measured interval rebuilt the static scene. All runs retained 192 guest templates, one deck rebuild, and five guest-buffer growths. Stress final guest meshes contained 606,342–606,552 vertices, depending on the ending simulation state.

## Acceptance and evidence

The bounded checkpoint requires correct controlled rendering, truthful diagnostic attribution, isolation from player persistence, and a relocated distributable build. It does not close PRF-05's hardware/full-workload gate.

| Item | Current evidence |
| --- | --- |
| Source / PR / CI | Implementation `5d49b7e00ce25692aac3ad2d6dda86939afd0a47`; branch `codex/large-map-gpu-diagnostics`; [draft PR #7](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/7) stacked on PR #6. [CI run 39435077589](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39435077589) completed successfully, including formatting, strict lint, tests, build/package, production continuation, CPU-report smoke checks, and artifact upload |
| World/camera correctness | Passing regressions cover 256-map corner/edge picking, scene/depth bounds, all rotations/views, and retained-renderer map replacement |
| Preview/input/persistence isolation | Integrated source tests and four production CLI rejection checks passed without creating a preview save directory |
| GPU reporting | Passing tests cover capability/status, encoder-only timestamps, delivery deduplication, receipt boundaries, invalid values, and bounded storage; six production reports received actual GPU samples |
| PowerShell wrapper | 44 AST-isolated checks passed; parser clean. Covers legacy reports, new fixtures, options, partial/unsupported outcomes, and contradictory GPU claims. No game/Cargo/CIM ran in this test |
| Formatting / release tests / strict lint / build | All 178 release tests passed: 113 game / 65 simulation. Formatting, release Clippy with warnings denied, and release compilation passed |
| Actual local GPU reports | Six completed profiles/wrapper checks, 120 main-loop intervals each, received counts and geometry above; raw data retained |
| Inspected render captures | Six processes exited 0: old/new normal baseline plus four large views. Current images were inspected at 1280 × 800, paused after 600 ticks: full 256-map overview, save-scale Crowd rotation 1, stress Ride rotation 2 with circuit/cars, and stress Crowd rotation 3 Underground/Flat with 5,000 guests. Full overview fits and focused views retain the intended content |
| Save continuation and ordinary 32-map regression | Production 0.8 writes → 0.9 verifies and 0.9 writes → 0.8 verifies both passed 3,600 ticks/commands, full state, backup, and paused loading. Baseline image comparison found 42 differing pixels, all in the version-digit box x32..37/y739..746; zero outside it |
| Executable | 46,109,696 bytes; SHA-256 `28df016814b5a86e0b9366ea0a380c284bcd9797f4c8c79fff29c61d64e17a88` |
| Notices / package hashes / relocated launch | Package contains 24 payloads plus manifest, 291 dependency notices, default-font/Rust notices, and one required MSVC DLL. All 24 extracted payload hashes matched. A fresh temporary path with spaces/é ran from C:\Windows and passed 3,600-tick continuation/commands, explicit backup, paused load, and a stress Ride/rotation-2 capture byte-identical to the original. The executable hash remained unchanged. The final refresh verified all fifteen current guides and all 24 payload hashes |
| Bundled wrapper relocation | The sibling executable was selected correctly. Baseline CPU 30 ticks and both 120-main-interval save-scale Ride/rotation-2/Underground GPU profiles completed with exit 0. Actual windows were 1920 × 1080, with 98/95 received GPU samples per pass for AutoVsync/AutoNoVsync. These are functional checks, not hardware acceptance |
| Representative PC / full target | No reference PC selected; PRF-05 remains in progress |

The bounded runtime matrix exercised full-map overview and focused crowd/ride views, all four rotations across those captures, Surface/Underground, and Flat terrain. Ordinary 32-map/save behavior was checked separately. The fourteen production runtime checks passed; receipts, baseline comparison, and captures are under ignored `artifacts/large-map-gpu-0.9.0/`. The nine reviewed profile/summary JSON files are retained in the linked dataset with their checksum manifest. Preserve unexpected or partial outcomes in future runs rather than replacing them with inferred values.

The next proposed bounded rendering change is static terrain chunk visibility: retain the correct full-scene oracle, partition static geometry, and prove that a focused camera submits only intersecting chunks while overview, edge clipping, depth/picking, all rotations, and Underground remain equivalent. The current full landscape remains submitted even in a focused view; reducing that unnecessary work is a testable architecture change. This is **not a diagnosis of the dominant bottleneck** or a promise of FPS gains. Keep physical upload/presentation and representative-hardware acceptance separate.

Source checks, local GPU observation, package relocation, native input, clean-machine compatibility, and hardware acceptance remain distinct evidence. Native Save/Load/Backup, O/F/buttons, physical Delete, panning/dragging/physical display, and clean-PC offline/standard-user gaps remain open unless directly reverified.

Historical reports and both diagnostic datasets remain unchanged. The current source is in the original shared OneDrive folder:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The recorded second-PC clone is separate:

```text
C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon
```

Push source and handoff records before changing PCs. Use one active editing/build/Git session in the shared folder. User saves need separate transfer. Archive hashes change after guide refreshes; use the adjacent `.zip.sha256` and verify all final payloads/guides rather than embedding the archive hash inside its own documentation.
