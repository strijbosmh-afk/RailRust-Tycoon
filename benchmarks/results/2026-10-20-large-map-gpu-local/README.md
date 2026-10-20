# 0.9.0 controlled-render GPU diagnostics

Collected 20 October 2026. **No reference PC is selected.** The user explicitly excluded this poorly performing development PC as a reference. These instrumented local observations establish that the diagnostic path renders its fixtures and receives actual GPU timestamp samples; they do not establish hardware requirements, representative FPS, a portable speedup, or a bottleneck diagnosis.

The nine JSON files are exact copies of production output. `SHA256SUMS.txt` covers their original bytes. Each `baseline/`, `save-scale/`, and `stress/` folder contains `frames-vsync.json`, `frames-uncapped.json`, and the wrapper `summary.json`. The [milestone report](../../../docs/LARGE_MAP_GPU_REPORT.md) records source/runtime/package evidence and the full measurement contract.

All three wrappers used `-SkipCpu -GpuDiagnostics -ProfileSamples 120` plus an explicit render case/camera. CPU benchmark settings retained in summary configuration were inactive. Each wrapper ran the two frame modes sequentially, with no repetitions. Every process exited 0, completed all 120 requested main-loop intervals, and passed fixture/method validation.

All summaries recorded clean source `5d49b7e00ce25692aac3ad2d6dda86939afd0a47`, version 0.9.0, and an unchanged 46,109,696-byte executable with SHA-256 `28df016814b5a86e0b9366ea0a380c284bcd9797f4c8c79fff29c61d64e17a88`. Source identity does not independently prove how an arbitrary executable was built; the release identity is recorded in the report.

The host was Windows 11 Enterprise 10.0.26200, i5-9400T, Intel UHD 630 driver 31.0.101.2115, and 8,453,365,760 bytes of RAM. Bevy used Vulkan. Thermal/power/background conditions were not controlled. All measured windows were actually 1920 × 1080, scale 1, visible/focused, with no configuration changes. **Occlusion was unknown in all 120 intervals of each run.**

| Fixture | Camera / rotation / view | Guests / paths / map | GPU samples per pass, vsync / uncapped | Landscape vertices |
| --- | --- | --- | --- | --- |
| Baseline | Overview / 0 / Surface | 12 / 37 / 32 × 32 | 87 / 88 | 99,282 |
| Save-scale | Overview / 0 / Surface | 256 / 1,024 / 256 × 256 | 85 / 91 | 2,251,764 |
| Stress | Crowd / 0 / Surface | 5,000 / 1,024 / 256 × 256 | 120 / 120 | 2,251,764 |

Every fixture uses seed 20261020 and one 46-piece circuit/train. Saving/editing was disabled. Stress exceeds the unchanged 256-guest save bound; save-scale's codec capacity does not enable preview saving. Each report has `samples_received` and 24 paths: opaque, transparent, UI, and upscaling passes × GPU elapsed, CPU encoding elapsed, and four pipeline counters. Enabled device features included timestamp-query/inside-encoder/inside-pass and pipeline-statistics support. These flags are enabled features, not a separate query of all adapter capabilities.

No values were rejected as invalid or omitted for sample limits. Baseline uncapped excluded 24 already-delivered pre-window observations; other runs excluded zero. The asynchronous bridge is latest-only: missing/overwritten source batches and source-frame IDs are unknown. Late warmup results can enter the receipt window and final in-flight results can be absent. Even a count of 120 cannot prove one-to-one correspondence with the 120 main-loop intervals.

Opaque-pass GPU means were 3.71345/3.44888 ms for baseline, 18.94282/20.21746 ms for save-scale, and 50.79779/49.07778 ms for stress (vsync/uncapped). These are actual device timestamp spans already converted to milliseconds by Bevy. They are not whole-frame GPU time or display presentation time. Other named spans, raw distributions, main-loop intervals, CPU hooks, and sampled memory remain in the JSON; nested spans must never be summed.

The 256-map landscape retained 2,251,764 indices and vertices, or 117,091,728 logical CPU payload bytes, in both overview and focused crowd views. Static rebuild count was zero during every measured interval. Guest caches retained 192 templates, one deck rebuild, and five vector growths. These are CPU geometry/work counters, not physical GPU allocation/upload or visible-object counts. Physical upload bytes/time and presentation timing remain null/unmeasured.

The full landscape being submitted in focused views motivates a bounded static-chunk visibility experiment with image/depth/picking equivalence checks. It does not prove that terrain is the dominant bottleneck or promise FPS gains. Historical 0.7/0.8 raw datasets, manifests, and path names remain unchanged. PRF-05's fifty-ride and representative-hardware targets remain open.
