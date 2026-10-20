# Static visibility local diagnostics — 20 October 2026

**This PC is explicitly excluded as performance reference hardware. No reference PC is selected.** These records verify diagnostic behavior and static-geometry selection on the development host. They do not establish minimum specifications, a portable speedup, representative FPS, or acceptance of the proposed 60 FPS target.

The ten JSON files are byte-for-byte copies of six production profiles, three wrapper summaries, and the eight-pair image comparison record, totaling 4,977,504 bytes. `SHA256SUMS.txt` records their hashes. Do not rewrite raw records to change their interpretation; add a new dated dataset for subsequent runs.

All three summaries recorded clean implementation `ff2857e68bfa8217fb85fe3bd107312f385d490e`, version 0.10.0, and the unchanged 46,168,064-byte executable with SHA-256 `b3a63267dc450f1deab1a8a48aa1f93e93d0c4d6c9ba1e05bc2009f12bdb6c52`. Build, review, CI, save compatibility, and package evidence are recorded separately in [STATIC_VISIBILITY_REPORT.md](../../../docs/STATIC_VISIBILITY_REPORT.md).

Each case ran once in each requested frame mode, AutoVsync and AutoNoVsync, using `-SkipCpu -GpuDiagnostics -ProfileSamples 120` with explicit `-RenderCase` and `-RenderCamera`. Baseline and save-scale used Overview; stress used Crowd. All used rotation 0 and Surface. CPU benchmarks were disabled; retained CPU default settings in the summary do not mean those benchmarks or default repetitions ran.

All six profiles completed with exit code 0, 120 main-loop intervals, and `samples_received`. Actual windows were 1920 × 1080, visible/focused, with no changed configuration, unfocused intervals, or hidden intervals. **Occlusion remained unknown.** Power, temperature, and background desktop activity were not controlled.

The final static snapshots match between the two modes:

| Case / camera | Selected / total chunks | Generated vertices | Selected vertices | Selected logical CPU bytes |
| --- | --- | --- | --- | --- |
| baseline / overview | 1 / 1 | 99,282 | 99,282 | 5,162,664 |
| save-scale / overview | 212 / 212 | 2,251,764 | 2,251,764 | 117,091,728 |
| stress / crowd | 13 / 212 | 2,251,764 | 149,556 | 7,776,912 |

Indices equal vertex counts for these fixtures. Every run recorded one geometry build and one visibility evaluation since initialization. Stress hid 199 chunks and recorded 199 visibility toggles; baseline/save-scale hid zero. These are cumulative counters and a final CPU-eligibility snapshot, not interval averages, physical GPU draws, exact visible pixels, or GPU upload traffic. Dynamic guests/train are outside the static counts.

Received GPU samples per named pass were 88/82 for baseline, 106/99 for save-scale, and 120/120 for stress (AutoVsync/AutoNoVsync). Four named GPU passes received those counts. Readback is asynchronous and latest-only; even 120 samples cannot establish correspondence with the 120 main-loop intervals. Late warmup arrivals and missing final/in-between batches remain possible. CPU encoding, GPU spans, pipeline counters, main-loop intervals, and sampled process memory retain their distinct meanings; do not sum spans or infer presentation/upload duration.

The image comparison covers eight views against 0.9.0: normal baseline, minimum-window large Ride rotation 2, large overview rotation 0, Crowd rotation 1, stress Ride rotation 2, stress Crowd rotation 3 Underground/Flat, Ride rotation 0 Underground/Terraced, and Ride rotation 3 Surface. Each pair differs only in 709 version-line pixels; world and remaining UI pixels match exactly. Eight current captures, three fresh prior captures, and five retained prior captures form these comparisons.

This demonstrates reduced selected static geometry in a focused large-map view while preserving the tested output. It does not identify a dominant bottleneck or justify expanding map/save/ride limits. Historical datasets and notice files remain unchanged. The recommended next gameplay slice is interactive path placement and demolition in the existing 32 × 32 park, with continuation tests; further performance acceptance requires an agreed representative PC.
