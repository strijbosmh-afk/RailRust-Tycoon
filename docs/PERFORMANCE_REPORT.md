# Performance measurement report — 0.7.0

Date: 20 October 2026. Acceptance ID: **PRF-05**, **in progress for the full proposed target**. This checkpoint adds repeatable diagnostics. All 147 release tests and formatting pass; final timing, strict lint/build, package, and hosted-CI evidence is pending. The [save report](SAVE_REPORT.md) preserves accepted 0.6.0 continuation evidence; earlier reports remain historical.

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

## Evidence and architecture decision

| Evidence | Current result |
| --- | --- |
| Source branch / base | `codex/performance-measurement`, based on save documentation checkpoint `d634dda` |
| Runtime commit / review / hosted CI | Pending |
| Source checks / regression count | 147 release tests passed: 60 simulation and 87 game. Formatting passed; strict Clippy and final release build pending. New coverage includes four fixture, two benchmark, three timing-summary, and seven frame-profile tests. |
| Wrapper parsing and file-diff check | Passed during implementation; runtime validation pending |
| Recorded hardware and power/display conditions | Pending measurement; historical hardware is not a fresh reading |
| CPU baseline/save-scale/stress timings | Pending |
| Frame profiles and actual window/adapter metadata | Pending |
| Process memory | Pending |
| Executable identity / package / relocation | Pending |
| Resulting architectural change or revised target | Pending measured evidence |

Reviewed reference outputs are intended for `benchmarks/results/2026-10-20-reference/`; collection and verification are pending. Keep large raw results outside `docs` so packaging includes this readable report without duplicating every timing sample.

The current production graph rebuilds connectivity per tick and uses BFS per new route. Large crowds may expose those costs. Measure them before approving graph caching, spatial indexing, route reuse, or other changes; preserve deterministic ordering, removal/recovery, and save continuation in any optimization. Report an unsupported or cap-limited fixture honestly and retain its completed work.

PRF-05 remains in progress until the full benchmark is measured or the target/gate is deliberately revised with recorded evidence. The measurement tooling can be complete while this broader target remains open. The next implementation should follow the measured bottleneck or missing scale support; do not begin a large content catalogue on the assumption that a one-train benchmark establishes 50-ride capacity.

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
