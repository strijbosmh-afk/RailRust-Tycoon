# Ride graphics verification — 0.93.0

Branch `codex/ride-graphics-overhaul`, based on `69044094a68bad6d1f00cd07db4bcac1352efafa`. Executable SHA256: `6370d3c86fd77fb6a18d5a0dc4e628e0cd1de4123a09c0034dc8403a317d2b3b`.

## Completed checks

- Strict release Clippy across the workspace and all targets passed.
- All 570 game tests passed. These include all ride/terrain depth, preview, picking, paint, train pose, chunk and save tests. The new tests bound the rounded ride geometry in all construction orientations and camera rotations, and exercise triangular/collinear selection outlines.
- The five carousel/flat-ride tests passed after fixing selection outlines for repeated triangle vertices and collinear corners.
- The standalone production executable passed both demo-park and catalogue write/verify pairs from outside the checkout. All seven demo rides completed paid visits. The fresh-process replay matched and retained original save bytes.
- Fourteen 1280×800 catalogue captures were generated and inspected, one for every model. The wheel was also inspected in all four camera rotations.
- The Windows release build completed. Save schema/content remains 51/51; no simulation fields, dependency versions or external image assets were changed.

The initial render test run found non-finite construction-preview outlines on triangular caps. The fix handles repeated vertices and collinear projected corners without drawing beyond the visible face. The final game test suite includes the regression.

## Evidence and limits

Local logs and actual game screenshots are retained in `artifacts/ride-graphics/`. These are executable captures, not generated concept images. Capture scenes use a paused diagnostic park; the ordinary demo remains playable.

All 464 simulation tests also passed, completing 1,034 tests. The extracted package passed 166 payload hashes and 146 current guides, plus independent front/demo write/verify processes and title/demo captures from `C:\Windows`. The first verified archive SHA256 is `02e0dfb36665ea4a455a52f25a30dfa6d5f8a0ccfcab2ce4563f0f32862f2b53`. The final documentation refresh retains the same executable and receives a fresh adjacent `.sha256` file and integrity-only verification.

The development PC is excluded from performance acceptance. No reference frame-rate claim is made. Independent player testing and representative hardware checks remain open. The change improves the current procedural isometric artwork; it does not claim photorealistic assets.

No Drive upload was attempted during development. Three prior native exports remain queued, with 124 of 147 files verified for the oldest. Local files and Git publication are separate from verified Drive storage.

Runtime source `dabbd2e6f99a6518a24e71ef2a78878103e375a3` is verified on GitHub in PR 85, stacked on PR 84. The remote Windows workflow `43878898362` was in progress at this checkpoint.
