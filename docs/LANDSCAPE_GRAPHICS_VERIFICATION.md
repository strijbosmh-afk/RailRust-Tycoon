# Landscape graphics verification — 0.94.0

Branch: `codex/landscape-graphics`. Runtime commit: `5488cf973ec5b4222fadd0c7d192036f24db3ac1`. PR: https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/86 (stacked on the ride-graphics branch).

## Completed checks

- Strict release Clippy across the workspace and all targets, formatting and 17 original asset records passed.
- GitHub Windows job 133377462467, run 43889760293, passed formatting, asset validation, strict Clippy and the complete workspace test step against the runtime commit. Its remaining gameplay checks were still running at this checkpoint.
- The first local game suite passed 571 cases and exposed two old path-art expectations. The queue color and shallow detail bias assertions now match the new materials; the remote complete test step passed with those corrections. A local rebuild of that test binary is also being completed.
- All eight targeted landscape tests passed, including material clipping on height/rotation variants, sand and cliffs, static terrain and the production-built landscape fixture save roundtrip.
- The release executable built successfully. Its standalone seven-ride demo write/verify processes passed from outside the checkout: paid visits on all seven rides and exact continuation after loading.
- Inspected actual game captures: all four landscape rotations, landscape detail, demo, sloped paths, underground view, and the demo at 2560x1440. All retained in `artifacts/landscape-graphics/`.
- The extracted Windows package passed all 169 payload hashes and 149 current bundled guides, independent demo write/verify processes and a landscape screenshot. That screenshot was visually inspected. Evidence: `artifacts/landscape-package-retry/verification.txt` and `artifacts/landscape-graphics/package-retry.log`.

Executable SHA256: `e50c082e374ae254de52fc252925901c2dbb16ff7803350f8585b086c0ae6c8f`.
Initial verified ZIP SHA256: `f95d1289561346cccd95f87385e687f55d109880c6fdec77e78901b83884d5bf`.
Final documentation refresh preserves the executable; the adjacent .sha256 file identifies the refreshed ZIP, followed by integrity-only verification.

## Limits

No simulation fields, save formats (51/51), dependencies or external art were changed. Rich turf/cliff detailing follows the existing small-map policy through 64x64; larger maps remain simpler. Native input was not proven by automated captures. This PC provides correctness evidence only; it is explicitly excluded from performance acceptance. Representative hardware and independent player checks remain open.

The first package verification reached its 360-second runtime timeout under heavy local contention. A fresh isolated retry with a longer time limit passed both demo processes and the capture. This is not a performance benchmark.
