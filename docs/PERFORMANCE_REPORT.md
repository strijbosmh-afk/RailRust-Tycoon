# Performance measurement report — 0.7.0

Date: 20 October 2026. Acceptance ID: **PRF-05**, **in progress for the full proposed target**. Repeatable diagnostics, all 147 release tests, strict source checks, fourteen reference/extended processes, hosted Windows CI, and relocated-package checks passed. Measurements identify substantial crowd geometry costs and a baseline frame-rate deficit. The [save report](SAVE_REPORT.md) preserves accepted 0.6.0 continuation evidence; earlier reports remain historical.

The proposed target is a 256 × 256 park with 5,000 guests and 50 operating rides at 60 FPS and 1080p on a recorded reference PC. The current model supports one circuit/train and the normal UI supports 32 × 32 maps. Measuring a larger crowd with one circuit cannot establish the 50-ride or complete playable-park target. The tooling must report actual supported counts and explicit unsupported cases; a successful benchmark process is not a performance-target pass.

## Repeatable workload

| Case | Map | Guests | Path nodes | Circuit/train | Save status |
| --- | --- | --- | --- | --- | --- |
| `baseline` | 32 × 32 Terraced demonstration | 12 | 37 | One 46-piece circuit / one train | Supported |
| `save-scale` | 256 × 256 Terraced | 256 | 1,024 | One 46-piece circuit / one train | Supported by existing save limits |
| `stress` | 256 × 256 Terraced | 5,000 | 1,024 | One 46-piece circuit / one train | Explicitly unsupported: existing codec cap is 256 guests |

The larger cases use a connected 32 × 32 grid of paths at x/y 80..111, with 1,984 undirected edges and diameter 62. They contain 1,070 objects: 1,024 paths plus 46 circuit pieces. Stable seeded placement and staggered initial waits make the starting work reproducible; the first 31 ticks include an initial routing burst. The graph is the production path graph, and routing probes use its production BFS. Diagnostics do not replace the normal algorithms or silently raise save limits.

## Commands and wrapper

Build the exact release executable first. The wrapper does not build, modify player saves/settings, or publish artifacts:

```powershell
cargo build --release --locked -p park_game --bin parkbound -j 2
pwsh -NoProfile -File scripts/measure-performance.ps1 -OutputDirectory 'C:\existing-qa-parent\Parkbound measurements é'
```

The output directory must be absolute and new; its parent must exist. Use a distinct directory for each batch. The default executable is a sibling `Parkbound.exe` when packaged, otherwise `target\release\parkbound.exe` in the source checkout. `-Executable` can select a specific binary. The ZIP includes the wrapper at its root; with PowerShell 7 installed, run `.\measure-performance.ps1 -Executable .\Parkbound.exe -OutputDirectory <absolute-new-directory>`. Playing the game does not require PowerShell 7. Measurement output stays local unless deliberately reviewed for source publication.

| Wrapper option | Default / bounds |
| --- | --- |
| `-Cases` | `baseline`, `save-scale`, `stress`; select one or more without duplicates |
| `-Repetitions` | 3 CPU runs per case; range 1–5 |
| `-BenchmarkTicks` | 300 measured ticks; range 30–1,800 |
| `-ProfileSamples` | 600 measured frame samples; range 120–3,600 |
| `-SkipFrames` | Omit the two frame-profile processes |

All processes run sequentially. CPU runs have a 180-second wrapper timeout, frame runs 150 seconds. Memory is sampled every 200 ms from only the owned child process. A timed-out child and its descendants are stopped before proceeding; raw output/failure metadata is retained. The script never terminates unrelated applications or clears player state.

Underlying production-executable modes:

```text
parkbound.exe --benchmark <absolute NEW JSON> --benchmark-case baseline|save-scale|stress --benchmark-ticks 300
parkbound.exe --profile-frames <absolute NEW JSON> --profile-samples 600
parkbound.exe --profile-frames <absolute NEW JSON> --profile-samples 600 --profile-no-vsync
```

CPU diagnostics run without a renderer. They request 60 warmup ticks before measured simulation ticks, excluding the initial routing burst when warmup completes. A phase has a 30-second soft budget and the run has a 150-second soft budget, checked between complete operations. A slow operation can exceed those soft limits; the wrapper's 180-second timeout bounds the process. Read actual completed work, status, and units before comparing runs.

Measured simulation samples each advance one complete production tick. Graph probes include production graph construction plus the diagnostic stable-ID list, allocation, and destruction. Route probes use sixteen warmups and 128 deterministic endpoint pairs. CPU mesh probes use one warmup each and five static-scene, thirty guest, and thirty train samples; `geometry.terrain` includes the entire static scene with paths, tracks, and decorations. These are geometry-generation costs without GPU upload/rendering. Supported codec probes check equality before thirty encode/decode samples. Filesystem probes measure five production saves and loads in isolated temporary storage, including codec, sync, and backup work with a warm OS cache; they are not cold-disk or OneDrive throughput.

Frame profiles cover the baseline UI only. They request physical 1920 × 1080 and report the actual logical/physical size and scale. Warmup requires at least 120 frames and two seconds; the internal deadline is 120 seconds. The two runs request `AutoVsync` and `AutoNoVsync`. Keep the game window visible and unobscured; actual focus, visibility, and occlusion metadata must accompany interpretation. Occlusion unknown is reported as unknown, not assumed false.

## Meaning of the measurements

CPU simulation, graph building, routing, serialization/loading, frame pacing, and process memory are separate measurements. Retain fixture/seed/work counts, warmup, completed samples, elapsed time, and any unsupported or incomplete outcomes with the values. Compare distributions and variation across repetitions, not only a best run.

Frame intervals measure CPU main-loop pacing. They include scheduling and presentation backpressure; they are neither GPU execution times nor proof of physically presented frames. Vsync can cap measured FPS, and an occluded/minimized or differently sized window can make comparisons invalid. Do not compare synthetic fixed-update equivalence tests to hardware frame-rate measurements.

`summary.json` records the executable SHA-256/size, runtime-reported version, source checkout commit/dirty state where a source manifest exists, selected metrics, and relative run filenames. Packaged runs without a source manifest leave checkout metadata null. Source metadata identifies the checkout; it does not prove that an arbitrary `-Executable` was built from that checkout. The executable hash is checked again after the batch to detect replacement during measurement.

Raw CPU reports use `kind: parkbound-cpu-benchmark`, schema 1, and `status: complete|partial`. Phases preserve requested versus actual samples and null timing when no work completed; persistence may be supported, unsupported, or skipped after the global budget. Frame reports use `kind: interactive_frame_profile`, schema 1, and `status: completed|deadline_exceeded|window_unavailable`. Partial frame reports remain useful evidence even though their process exits unsuccessfully. The wrapper validates report identity and sample/tick consistency, keeps raw reports, and applies no FPS threshold.

The wrapper records only selected OS version/build/architecture, CPU name/core counts/maximum clock, GPU name/driver, and physical RAM. It omits account names, computer names, serials, and absolute executable/output paths from its summary. Missing hardware fields remain unavailable. Memory maxima from 200 ms working/private-byte samples can miss short spikes; the observed OS peak-working-set counter is reported separately. Neither value measures GPU memory or whole-system consumption.

The existing 8 MiB save format, 256-guest limit, one-circuit model, and 32 × 32 player presentation remain unchanged. A `stress` save rejection is an expected, explicit limitation; it must not be replaced by timing a smaller park and labelling that the stress result.

## Reference measurements

The main batch ran on 20 October 2026, 12:48–12:50 UTC, using release implementation `e93eb99ddb4ac84a8ea1bc6f8a324424e5d27559`. The wrapper recorded a clean checkout at launch and the same executable hash before and after all eleven processes. All nine CPU reports completed every requested phase, and both frame runs completed 600 samples with exit code 0. The source and executable identities below tie these numbers to this implementation.

| Recorded reference hardware | Value |
| --- | --- |
| OS | Windows 11 Enterprise, 10.0.26200, build 26200, 64-bit |
| CPU | Intel Core i5-9400T, six cores / six logical processors; CIM reports 1,800 MHz maximum-clock field |
| Memory | 8,453,365,760 physical bytes, approximately 7.87 GiB |
| GPU / driver | Intel UHD Graphics 630 / 31.0.101.2115 |
| Frame adapter | Vulkan, integrated Intel UHD Graphics 630 |

Power plan, temperature, live CPU clock, and background-process load were not captured. This is a measurement of the current desktop session, not a controlled hardware certification. Raw report values and run-to-run variation remain part of the evidence.

The following values are **ranges of the three individual run means**, in milliseconds. Simulation p95 is likewise the range of each run's p95, not a percentile of pooled data. Every simulation run measured 300 complete ticks after 60 complete warmup ticks.

| CPU case | Simulation mean / tick | Simulation p95 / tick | Graph-build mean | Route mean |
| --- | --- | --- | --- | --- |
| Baseline | 0.0154–0.0216 | 0.0224–0.0241 | 0.0148–0.0168 | 0.00341–0.00349 |
| Save-scale | 7.45–9.22 | 10.40–15.33 | 7.75–9.42 | 0.370–0.411 |
| Stress | 7.79–8.32 | 9.59–10.54 | 6.82–7.33 | 0.311–0.334 |

| CPU case | Static-scene mesh mean | Guest mesh mean | Train mesh mean |
| --- | --- | --- | --- |
| Baseline | 9.34–14.89 | 0.150–0.240 | 0.0275–0.0302 |
| Save-scale | 325.20–509.58 | 4.71–7.71 | 0.0290–0.0382 |
| Stress | 312.89–580.86 | 114.61–135.45 | 0.0271–0.0344 |

Static meshes rebuild on changes, so their cost is not automatically a per-frame cost. Dynamic guest geometry is the largest measured CPU rendering concern at 5,000 guests: each standalone rebuild takes over 100 ms in this fixture. These are CPU generation probes; they do not establish the frame rate of an unsupported large-map renderer. The baseline simulation's highest single sample was 1.3754 ms; save-scale's was 53.709 ms, so means alone do not describe all stalls.

Three additional stress runs measured **1,800 ticks each**, advancing tick 60 → 1,860 after complete warmup. All phases completed. Their simulation means were 10.76–11.67 ms/tick, p95 18.17–20.41 ms, p99 29.93–39.29 ms, and per-run maxima 61.65–332.11 ms. Longer observation includes more destination arrivals and route requests than the default ten simulated seconds; the higher tails show why a short, recently initialized sample is insufficient. Background host scheduling can also contribute to individual outliers, so a maximum alone does not establish an algorithmic cause. Neither batch establishes long-session behavior, 50-ride capacity, or an absence of stalls.

| Save case | Encoded bytes | Encode mean | Decode mean | Store save mean | Store load mean |
| --- | --- | --- | --- | --- | --- |
| Baseline | 47,682 | 0.150–0.217 ms | 0.399–0.991 ms | 24.66–57.48 ms | 2.13–2.59 ms |
| Save-scale | 2,179,271 | 7.35–12.28 ms | 15.64–22.79 ms | 278.63–479.32 ms | 132.78–234.67 ms |
| Stress | Unsupported | — | — | — | — |

Store costs include production validation, codec, filesystem reads/writes, synchronization, and backup rotation with a warm OS cache. They should not be subtracted from codec probes to infer pure storage latency. Stress rejection is expected under the unchanged 256-guest save limit.

| Baseline frame mode | Main-loop FPS | Mean interval | p95 | p99 | Maximum |
| --- | --- | --- | --- | --- | --- |
| AutoVsync | 40.15 | 24.91 ms | 47.71 ms | 50.40 ms | 54.57 ms |
| AutoNoVsync | 37.94 | 26.35 ms | 51.53 ms | 56.07 ms | 72.85 ms |

Both runs measured physical/logical 1920 × 1080 at scale 1.0, with no configuration or size changes and all 600 intervals marked focused and visible. Occlusion remained **unknown for all 600 intervals**, so these reports do not prove an unobscured display. Auto presentation modes may select backend fallbacks that the probe cannot inspect. Each mode has one run; the small difference between them is not enough to conclude that disabling vsync makes the game slower.

The baseline did not reach the proposed 60 FPS in these measurements. Mean scene-update CPU time was only 0.335–0.367 ms, with zero static rebuilds; mean simulation-update CPU time was 0.092–0.093 ms. These limited main-world timings do not explain the 25–26 ms main-loop intervals or identify a GPU bottleneck. Render-thread, upload, GPU, presentation, and scheduling measurements are required before attributing that deficit.

The measured windows advanced 448 and 475 complete ticks. Neither run recorded a real-time interval above the 250 ms virtual-time cap, and real/virtual elapsed time matched. Endpoint differences include fixed-step overstep; `dropped_ticks` intentionally remains null rather than inventing a dropped-tick count.

The main batch's memory observations are below, in MiB (1,048,576 bytes). CPU rows show the range of three process maxima; frame rows have one run each.

| Process | Sampled working-set maximum | Sampled private-byte maximum | Observed OS peak working set |
| --- | --- | --- | --- |
| Baseline CPU | 9.69–10.06 | 2.66–2.92 | 14.36–14.61 |
| Save-scale CPU | 83.34–94.16 | 156.79–174.03 | 106.22–108.24 |
| Stress CPU | 93.18–97.65 | 141.11–159.02 | 107.69–107.76 |
| Baseline AutoVsync | 400.32 | 329.63 | 401.32 |
| Baseline AutoNoVsync | 399.93 | 327.93 | 400.93 |

The short baseline CPU processes had only two or three 200 ms samples, demonstrating the limits of sampled peaks. The OS peak counter was observed during polling, not guaranteed to have been read after the final allocation. Private bytes and working set measure different things and need not have the same order. These values exclude GPU memory and are not a minimum-RAM specification.

## Evidence and architecture decision

| Evidence | Current result |
| --- | --- |
| Source branch / base | `codex/performance-measurement`, based on save documentation checkpoint `d634dda` |
| Runtime commit / review / hosted CI | Implementation `e93eb99ddb4ac84a8ea1bc6f8a324424e5d27559`; [draft PR #5](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/5), stacked on save PR #4. [Windows run 39425219126](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39425219126) passed for that implementation. Later documentation-only heads have their own CI status. |
| Source checks / regression count | 147 release tests passed: 60 simulation and 87 game. Formatting, strict Clippy with warnings denied, and release build passed. New coverage includes four fixture, two benchmark, three timing-summary, and seven frame-profile tests. |
| Wrapper checks | Parser and seven report-helper assertions passed. All fourteen production processes across both batches completed, report validation passed, and executable identity remained unchanged. |
| Recorded hardware and power/display conditions | Recorded above; power/temperature/background load not captured; occlusion unknown |
| CPU baseline/save-scale/stress timings | Three complete 300-tick runs per case, plus three complete 1,800-tick stress runs |
| Frame profiles and actual window/adapter metadata | Both 600-sample modes completed at actual 1920 × 1080; baseline 37.94–40.15 main-loop FPS |
| Process memory | Recorded at 200 ms intervals; table and limitations above |
| Executable identity / package / relocation | Release executable: 45,868,032 bytes, SHA-256 `06c10a106dd3afe8d51c604c8238b10b5f5266f594a660b3e7e7b919260f742b`. All 22 payload hashes and thirteen guides matched on extraction; relocated save continuation and packaged wrapper checks passed. |
| CLI failure behavior | Production checks passed for preserving an existing report, rejecting zero benchmark ticks without output, and rejecting mixed frame/save-check modes without output or save-directory creation. |
| Resulting architectural change or revised target | Retain PRF-05 as incomplete. Next work targets reusable guest geometry and cached path connectivity, with separate render/presentation investigation. No optimization is included in this checkpoint. |

Privacy-reviewed [reference results](https://github.com/strijbosmh-afk/RailRust-Tycoon/tree/codex/performance-measurement/benchmarks/results/2026-10-20-reference) are retained in `benchmarks/results/2026-10-20-reference/`: twelve main-batch JSON files, four in `extended/`, a README, and a SHA-256 manifest. All copied JSON bytes were checked against the original reports. The main batch recorded a clean source checkout; the extended batch recorded documentation-only changes with the same executable hash. Raw data stays outside `docs`, so the ZIP includes this readable report without duplicating every timing sample. These files are included in the final evidence checkpoint alongside the updated guides.

The next bounded performance work should first remove repeated guest mesh allocation/reconstruction through reusable or batched geometry, then cache path connectivity instead of rebuilding it on unchanged ticks. Use explicit invalidation on path edits, restoration, and loading; derive caches from authoritative state without changing stable IDs, RNG order, full ticks, removal/recovery, or save continuation. Graph probe and simulation measurements are separate workloads, so their similar means support investigation rather than an exact claim about what fraction of each tick is graph construction.

Measure the same fixtures before and after each change, retaining correctness and geometry/picking regressions. Investigate the baseline frame deficit separately with render-thread/GPU/presentation evidence. Raising the save cap, supporting 50 rides, and rendering a playable large map remain explicit architecture tasks; none follows automatically from these timings.

PRF-05 remains in progress until the full benchmark is measured or the target/gate is deliberately revised with recorded evidence. The measurement tooling can be complete while this broader target remains open. The next implementation should follow the measured bottleneck or missing scale support; do not begin a large content catalogue on the assumption that a one-train benchmark establishes 50-ride capacity.

## Package and relocation verification

The Windows package contains the unchanged measured executable, the measurement wrapper, thirteen Markdown guides, 291 dependency notices with default-font and Rust standard-library attribution, and one required MSVC redistributable DLL. All 22 payload hashes and all thirteen guide copies matched after extraction into a fresh temporary directory with spaces and `é`. Keep every required file together when moving the application.

From `C:\Windows`, separate extracted `--save-check write` and `--save-check verify` processes passed 3,600-tick continuation with commands, primary/backup checks, and paused loading. The extracted wrapper was then run without `-Executable`: it correctly selected its sibling `Parkbound.exe`, left checkout metadata null, and completed one 30-tick baseline CPU run plus both 120-sample frame modes with exit code 0. Its executable hash remained unchanged. These shorter runs verify relocation and wrapper behavior; the main and extended reference batches above remain the performance comparison data.

The archive is regenerated after final guide updates, so its byte count and ZIP hash are not embedded here. The adjacent `.zip.sha256` and packaged `SHA256SUMS.txt` identify the actual delivered snapshot; verify every payload and guide after refreshing it. This proves relocation on the development PC, not a separate clean Windows installation without development tools.

## Retained gaps and handoff

Native Save/Load/Backup, O/F/Open/Close/Find, physical Delete, FND-02 panning/dragging/physical display, and clean-PC offline/standard-user testing remain separate open checks. No performance or instrumentation result implicitly closes them. Source/public title, licensing, and release decisions remain as recorded in the [product decisions](PRODUCT_DECISIONS.md).

Continue using the original shared OneDrive workspace:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The separate historical second-PC checkout is:

```text
C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon
```

Preserve the single-active-session workflow and push source/docs before moving PCs. Local measurement outputs and `%LOCALAPPDATA%\Parkbound\saves` are separate from source synchronization. Do not reuse another PC's numbers as this machine's measurements.
