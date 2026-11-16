# Classic graphics verification

Version 0.92.0 on `codex/classic-graphics-overhaul`, based on `2b0df01b22013e4a913a2f363b9963256efefcc7`. Runtime source commit `c767fa0d1360ba55ba93a351b04978306def1193` is verified on GitHub in PR 84. The shipping executable SHA256 is `eb4ccffd5b4284c24faa672440e1f3c752132c856f47404bc01eb2ce128cc9d5`.

## Implementation

The user selected the detailed classic isometric direction from three concept images. This release adds original embedded title art, live title controls, Gloock/Barlow typography, an alpha-tested oak texture, detailed service façades, character accessories, revised ride structures and richer landscaping. The seven-ride demo starts closer with its grid off. All source art and font licenses ship with the game. `assets/art/PROVENANCE.md` records the image prompts and origins.

The concept illustrations remain a target for subsequent art refinement. Gameplay still uses the procedural ride geometry, with the added detail described in `CLASSIC_GRAPHICS_GUIDE.md`. The title artwork and oak sprite are the raster assets integrated in this release.

## Observed checks

- Formatting and strict release Clippy passed on the shipping source.
- All 568 game tests passed, including cached guest geometry equality, ride/terrain depth, selection, construction previews, front-end transitions and the new foliage test. All 464 simulation tests also pass, completing 1,032 tests.
- The production demo writer/verifier passed in separate executable processes. All seven rides completed paid visits, facilities served guests and the exact 18,000-tick continuation matched. Original save bytes were preserved.
- The extracted package passed 161 payload hashes and 141 current guides. Its independent `front` and `demo-park` writer/verifier pairs passed from `C:\Windows`. Title and demo captures were completed and inspected.
- Title screenshots at 1024×680 and 2560×1440 were inspected. The live menu remains visible, with the same available actions and an aspect-preserving park background.
- The demo was inspected at 1920×1080, at close zoom, and in all four rotations. One 1920×1080 rotation capture timed out under local load. Fresh 1280×800 captures for the remaining two rotations succeeded.

Local evidence is in ignored `artifacts/classic-*` folders and logs. The initial tested archive SHA256 is `9be882db98699bb0d5ec84b62964fa5e4038d5db6a272f5dc9f48516af6e5859`. The final documentation refresh keeps the same executable and has its own adjacent `.sha256` file and integrity report.

The first test run exposed a track-draft test that clicked through a new station canopy. The test now clicks an exposed straight draft piece to exercise erase/undo history while the visible-obstruction rule remains active. Subsequent game tests passed.

## Scope and handoff

The local checks establish rendering, simulated play and persistence behavior. Physical player input testing and representative-PC performance acceptance remain open. This development PC is excluded from performance requirements. Save schema/content remains 51/51.

The full GitHub workflow `43869373748` was running at this documentation checkpoint. The previous demo workflow `43857766949` completed successfully. PR 84 is stacked on the demo PR 83.

Existing incomplete Drive exports remain queued. The earlier readback failure has no new verified upload receipt in this graphics session. No manual snapshot or hook was run.
