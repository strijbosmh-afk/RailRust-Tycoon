# 0.8.0 local optimization diagnostics

Collected 20 October 2026. **No reference PC is selected.** The user explicitly rejected this poorly performing development PC as a reference. These results investigate implementation work on that local host; they do not establish minimum hardware, portable speedups, representative FPS, or pass/fail the proposed 60 FPS target.

The six JSON files are exact copies of completed production diagnostics. `SHA256SUMS.txt` covers those raw files; do not rewrite their numbers or metadata. This README is interpretation, not part of the original runtime reports. The [optimization report](../../../docs/OPTIMIZATION_REPORT.md) records source checks, image comparisons, save compatibility, and package evidence.

| Files | Work |
| --- | --- |
| `cpu-baseline-1.json`, `cpu-save-scale-1.json`, `cpu-stress-1.json` | One sequential run per case, 60 untimed simulation warmup ticks then 300 measured ticks |
| `summary.json` | Wrapper metadata, status, 200 ms sampled process memory, executable/source identity, and selected hardware fields for those three processes |
| `extended/cpu-stress-1.json` | One stress run, 60 untimed simulation warmup ticks then 1,800 measured ticks |
| `extended/summary.json` | Metadata and sampled memory for that separate process |

All four CPU processes completed successfully. Neither batch ran frame profiles. Both recorded clean checkout `3f1ea886fea965a80bc5d3438ae80ecc4de4543d`; it includes a PowerShell-only validator fix after runtime implementation `9460a90180f94a1d8f73f5a35d3480caf43f0b14`. All used the unchanged 45,947,392-byte executable with SHA-256 `f907dbfc3effdcd460ab5e84ef15bf58cf623f9321231f89dd4a2c2f99ba6c24`.

The host was Windows 11 Enterprise 10.0.26200, i5-9400T, Intel UHD 630 driver 31.0.101.2115, and 8,453,365,760 bytes of physical RAM. Background load, thermal state, and power state were not controlled.

| Case | Guests / paths / map | Simulation mean / p99 ms | Retained moving mean ms | Legacy moving mean ms |
| --- | --- | --- | --- | --- |
| Baseline, 300 ticks | 12 / 37 / 32 × 32 | 0.00050 / 0.00780 | 0.02138 | 0.07452 |
| Save-scale, 300 ticks | 256 / 1,024 / 256 × 256 | 0.04579 / 0.66890 | 0.47380 | 3.10717 |
| Stress, 300 ticks | 5,000 / 1,024 / 256 × 256 | 0.99691 / 3.51510 | 14.80183 | 214.01023 |
| Stress, 1,800 ticks | 5,000 / 1,024 / 256 × 256 | 2.46539 / 6.54630 | 20.53611 | 104.42832 |

Each fixture uses seed 20261020 and one 46-piece circuit/train. The larger cases have a connected 1,024-node grid with 1,984 undirected edges. They are synthetic CPU workloads, not fifty rides or proof of interactive large-map presentation. Stress serialization remains unsupported under the unchanged 256-guest save bound.

Each animated renderer pair uses identical cloned states, sixty untimed moving warmups, then sixty timed updates. One complete simulation tick advances before each rendering timer. All four reported `matched_moving_workloads: true` with equal final authoritative state and emitted vertex totals. Cold rendering is separately measured over five samples including preparation, update, and destruction. `geometry.guests` retains the old frozen constructor, whose path queries now benefit from the shared simulation cache; do not call a historical comparison an isolated renderer speedup.

All simulation intervals added zero graph builds and exactly 300/1,800 reuses. Retained renderer updates advanced from 60 to 120 with templates fixed at 192, decks at 1, and vector growths at 5. Those operation counts, together with separate invalidation/geometry tests, are stronger evidence of removed repeated work than noisy single-run timings.

The short and extended stress runs reach different animated states and were executed at different times. Their retained-render p99 values were 28.57470 and 95.68950 ms. Fixed phase order and active-host noise limit timing conclusions; no speedup multiplier is claimed. No GPU timestamps or presented-frame timing were collected.

Historical 0.7.0 raw data remains unchanged under `../2026-10-20-reference/`. Its historical directory name does not designate reference hardware. PRF-05 remains open until supported target workloads and an agreed representative PC are available.
