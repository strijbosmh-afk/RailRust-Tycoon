# Phase 3 visual audit — 0.76.0

This is a reproducible screenshot review of the 0.76.0 Windows executable, not Phase 3 acceptance. The final executable SHA-256 is `f1b8e9a64acc24909f41c3cdb2c71332521be7f80d1af530303437f989d4330d`. The development PC is not a performance reference.

## Capture matrix

The same `overview` park was captured at rotations 0, 1, 2 and 3 for each size below. All 16 processes exited successfully and wrote PNG files. The local captures and 20-file SHA-256 list are under `artifacts/phase-3-5-0.76.0/` in the shared OneDrive checkout.

| Window | UI scale | Rotations | Result |
| --- | --- | --- | --- |
| 1024×680 | 100% | 0–3 | All four captured and directly inspected. |
| 1920×1080 | 150% | 0–3 | All four captured; rotation 0 directly inspected. |
| 2560×1440 | 150% | 0–3 | All four captured; rotation 1 directly inspected. |
| 3840×2160 | 200% | 0–3 | All four captured; rotations 0 and 2 directly inspected. |

Additional near-view captures were `scenery` at 1024×680/100%/rotation 0 and 1920×1080/150%/rotation 2, `rest-facility-map` at 1920×1080/150%/rotation 1, and `visitor-looks` at 1920×1080/150%/rotation 3. All four exited successfully and were directly inspected.

## Observations and open acceptance work

- The sampled rotations keep the same land, water, path and track arrangement; no obvious foreground/background reversal or ground clipping appears in these stills. Simulation and renderer tests separately cover tunnel picking at all four rotations. These screenshots cannot establish live pointer accuracy or animation behavior.
- Vector artwork stays crisp in the sampled HD/QHD/4K captures. The busy park fits but appears small at the minimum window, leaving little visible guest and service detail. The near views reveal the current procedural guest bodies, service booth and coaster supports clearly; they remain much simpler than the supplied detailed park reference.
- The right Ride/Build panels cover part of the near-view scene. At 1024×680, lower Build controls require scrolling. The first underground capture exposed a slope-button label spilling across its neighboring buttons; the final build uses short labels and was recaptured. The existing display guide covers scrolling, but this audit does not exercise native mouse or mixed-DPI monitor movement.
- Original asset provenance remains covered by `assets/manifest.json` and `scripts/validate-assets.ps1`. Night/weather treatment, richer ride/service silhouettes at both zoom extremes, a more representative crowded multi-ride scene, and native picking/animation inspection remain required by the Phase 3 gate.

To reproduce one capture, run the built executable with `--capture-frame <absolute-output.png> --capture-scene overview --capture-window-size 1920x1080 --capture-ui-scale 150 --capture-rotation 0`. Use a fresh output path for each rotation/size. The capture helper exits after writing the frame; when launching from PowerShell, wait for the process to finish before checking the file.
