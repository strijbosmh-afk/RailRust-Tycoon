# Sloped track — 0.25.0

Updated 24 October 2026. Work on `codex/sloped-track` starts from 0.24 checkpoint `796d962c9e930f3f2c012a435ecf8a201017288e` on `codex/track-construction` / [draft PR #22](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/22). That base passed [push CI 39949541606](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39949541606) and [PR CI 39949549735](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39949549735). Those results do not verify the new source. The [0.24 report](TRACK_CONSTRUCTION_REPORT.md) preserves its original runtime, package and proof identities.

Parkbound remains an original Rust theme park game intended for free Windows distribution. No representative performance PC is selected; development machines remain excluded from performance acceptance.

## Player contract

The saved coaster editor supports eight piece choices. The selected height always means the incoming endpoint, and C rotates the incoming direction of travel in world coordinates.

| Editor choice | Height change | Allowed entry height |
| --- | --- | --- |
| Station | Level | 0–30 |
| Straight | Level | 0–30 |
| Left curve | Level quarter turn | 0–30 |
| Right curve | Level quarter turn | 0–30 |
| Chain lift | +1 | 0–29 |
| Drop -1 | −1 | 1–30 |
| Drop -2 | −2 | 2–30 |
| Brakes | Level | 0–30 |

Changing piece type clamps the entry height to that type's range. Height controls retain the complete grade rather than flattening a piece at a limit. The API rejects unsupported or out-of-range geometry. Clearance spans the lowest endpoint through two steps above the highest endpoint, covering the full slope and protecting elevated paths and terrain intersections.

Open the editor with Z or Rides → Edit coaster track. Click to place or replace a draft piece; C rotates, Page Up/Down or height buttons change entry height, and X erases the visible draft at its sampled height. Clear discards the plan; Done/Escape retains it. Draft rails, previews and direction arrows use the same sampled centerline as trains. Lift chains and brake markings identify operating equipment.

The live coaster continues to use its installed circuit during planning. Apply requires exactly one station, opposite matching endpoints, equal heights at every connection, one directed closed loop containing every piece, and current world clearance. Incomplete or height-mismatched plans remain editable and saveable. Apply also requires a closed, stopped coaster with no active visitors. Failure leaves the complete authoritative park unchanged.

Successful replacement preserves ride identity, ticket price, guests, earned revenue, reviews, laps, finances and unrelated objects. Only owned rails and initial train placement change; changed rails receive fresh IDs, and an identical reapplication consumes none. Existing portals remain, so moving the station may require reconnecting access. Track construction is still free.

The existing deterministic motion rules now apply to player-built equipment: a lift holds 24 distance units per tick, a drop adds 3 up to a maximum of 96, and brakes subtract 5 down to 20. The train and rider positions follow the sampled grade. These fixed settings remain approximate simulation rules; player tuning, realistic gravity/forces, banking, inversions, block sections and multiple coasters are outside this stage.

`TrackBuildKind::min_height()` / `max_height()` expose the entry bounds, and `spec(pos, height, direction)` creates the directed endpoints. Draft and installed-circuit validation share the supported piece-shape rules. Existing `validate_track_draft_piece`, `set_track_draft_piece`, `validate_track_draft` and `apply_track_draft` remain the preview, edit, traversal and commit boundaries; renderer state cannot bypass simulation validation.

## Persistence

Saves use strict schema 14/content 14. Explicit migrations support versions 1–13, preserving original world, IDs, RNG, passengers, money, paid basis and financial history. Schema-13 level drafts survive unchanged. Versions 1–12 receive an empty draft as before. The historical schema-13 decoder retains its level-only contract, rejecting slopes falsely labelled as old data before migration.

Incomplete and subsequently obstructed drafts remain valid saved planning states; Apply repeats completion and clearance checks. Structural corruption, invalid grades/endpoints, unknown fields, hybrid schemas and oversized collections remain rejected. Normal play remains 32 × 32 with the existing save/object/guest limits and 256 draft pieces. Loading does not rewrite either slot. Version 0.24 rejects schema 14; preserve closed-game legacy copies for rollback.

## Reproduction and verification scope

The existing commands remain `--save-check write-track-construction` and `--save-check verify-track-construction`. Run them in separate processes against one new absolute isolated save directory. The writer creates a 14-piece partial-draft backup and a paid primary midway along a constructed chain lift. The complete plan has 28 pieces, both curve directions, three lifts from height 8 to 11, one-step and two-step descents, and brakes. It preserves the original station and public access, including a bridge above the southern rail.

The 7,200-tick continuation checks arbitrary placement order, whole-state refusal for mismatched heights and occupied construction, and a lift that collides with a bridge where a level piece still fits. It closes safely after a paid lap and public return, changes the descending profile and brake location, applies fresh rail IDs while preserving earned history, then requires another paid completed journey. Both layouts must demonstrate lift speed, downhill acceleration and braking. Mid-lift/drop/brake round-trips, fresh-process future equality, paused partial-draft restoration and unchanged original save-slot bytes are asserted.

The writer also provides `native-missing-station.park.json`. In an isolated QA slot, add the missing station at player tile 9,11, heading East, entry height 8; then save, Apply, reopen and inspect movement. Check all four new piece buttons, entry-height bounds, direction changes, visible-height erase, refused Apply, Save/Load and Backup through actual mouse/keyboard input. Compare complete saved state where a change is expected. This checklist is pending native acceptance, not evidence of a pass.

Existing capture names remain available:

| Scene | Current fixture |
| --- | --- |
| `track-draft` | Fourteen saved pieces and a valid missing-lift preview at entry height 9 |
| `track-invalid` | A mismatched lift endpoint blocks Apply; a hovered lift intersects the public bridge |
| `track-ready` | A complete graded loop with the two-step drop selected |
| `track-built` | Paying passengers paused midway up the installed chain lift |
| `track-occupied` | The running paid train and atomic Apply refusal |

Review all four rotations, Surface/Underground, Flat/Terraced and both 1280×800 and 1024×680 windows. An exactly edge-on projected steep slope has no visible picking ribbon; rotate the view to expose it. Captures establish presentation separately from native input and continuous-motion acceptance.

## Current verification status

Final runtime source **`d19346c469d864933620468d72301b41c420aa63`** is pushed in [draft PR #23](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/23), stacked on PR #22. Exact-source [Windows PR CI 40072533308](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40072533308) passed all **28 steps**, including **501 release tests (292 game / 209 simulation)**, formatting, strict all-target release Clippy, production build, packaging and all retained process-continuation proofs.

The delivered **final CI executable** is **48,472,064 bytes**, SHA-256 **`4377c734df042fac7a5731853b4ba3feef94337519df64d00d016e5161587e52`**. A fresh extraction under `artifacts/sloped-track-0.25.0/relocated package é`, launched from `C:\Windows`, passed the final graded writer/verifier for **7,200 ticks**. It also rendered `ready-r1-small` byte-identically to the final inspected capture. The duplicate fresh local build and queued checks were deliberately stopped after final CI passed; its cache was retained. The delivered executable comes from CI, and no local full-compilation pass is claimed.

This final executable also passed authentic schema-13 `verify-legacy` in fresh `artifacts/sloped-track-0.25.0/legacy13-final/`, using copies of slots written by the genuine 0.24 executable identified below. It preserved every original raw park field and checked **3,600 ticks per slot**. The earlier migration run remains separate historical evidence.

All **ten final captures** under `artifacts/sloped-track-0.25.0/final-captures/` passed and were directly inspected. They cover all four rotations, Flat/Terraced, Surface/Underground, and 1280×800/1024×680 windows. Initial visual review found clipped compact-panel guidance; the final runtime fixes the notice space and shortens the affected guidance. Final controls, height transitions and valid/refused construction messages fit. Captures establish presentation separately from native input.

| Check | Status |
| --- | --- |
| Formatting | Passed in final exact-source CI |
| Full workspace release tests, strict Clippy and production build | Passed in CI: 501 tests (292 game / 209 simulation) |
| Fresh local compilation | Deliberately stopped after final CI passed; cache retained; delivered binary is from CI |
| Retained production continuation proofs | Passed in final CI; final relocated package passed the graded 7,200-tick proof |
| Authentic schema-13 migration | Passed on the final executable: original fields preserved, 3,600 ticks per slot |
| Old-reader rejection | Passed on initial `06697d8` with unchanged slot hashes; final changes affect only three display files |
| Capture generation and visual inspection | All ten final captures passed and were directly inspected |
| Portable package | 40 payload hashes and 31 Markdown guides verified; relocated proof and byte-identical capture passed |
| Current commit/PR | `d19346c469d864933620468d72301b41c420aa63`, draft PR #23 |
| Exact-source Windows CI | Run 40072533308 passed all 28 steps |
| Native 0.25 mouse/keyboard acceptance | Unverified |
| Clean-PC offline/standard-user compatibility | Unverified |
| Representative performance acceptance | No reference PC selected |

### Initial implementation and migration evidence

Initial runtime **`06697d8ad15288deb8fa4c1fd43b956d797ccbf5`** passed all 28 steps of [CI 40072010130](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40072010130), including 501 release tests and all retained proofs. Its CI executable had SHA-256 **`ca7d9dd83132b1218e4f06614a698548c61846ae3c3ec3659108fa4be0ec3658`** and passed the local graded 7,200-tick proof. Keep that artifact identity separate from the delivered final executable above.

The authentic 0.24 executable from historical CI artifact [39949541606](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39949541606) generated legacy schema-13 primary and backup slots. Its SHA-256 is `54eeb6a0e6adc1567a794460b62d91fade347038a092b6862674fea552dc735b`, distinct from the local 0.24 executable in its historical report. The initial 0.25 executable passed `verify-legacy` against those slots, preserving every original raw park field and checking **3,600 ticks per slot** with unchanged original files. The old reader rejected new schema-14 saves with `Save version is unsupported`; both slot hashes remained unchanged. Final runtime `d19346c` changes only three display files; the save codec and simulation are unchanged. Final CI reran retained proofs, and the final executable separately passed authentic migration and relocated graded continuation as recorded above.

### Package and continuation on this PC

The current checkout is `C:\Users\strij\OneDrive\Documenten\ChatGPT\RailRust Tycoon`. Rust 1.93.1 is installed. Visual Studio 2022 Community 17.14 is available at `C:\Program Files\Microsoft Visual Studio\2022\Community`, and VS 18 Build Tools is also installed. Build with the intended VS 2022 environment; the unchanged packaging helper discovers the VS 2022 runtime. Its discovery query finds the Community installation. Do not treat the presence of VS 18 alone as a package failure or as equivalent runtime provenance.

The available portable package is `dist\Parkbound-0.25.0-windows-x64.zip`. It retains the final CI executable, notices and manifests unchanged and includes refreshed current guides. **40 payload hashes and 31 Markdown guides** verify. The adjacent `.zip.sha256` identifies the archive and must be regenerated with the payload manifest after subsequent guide refreshes; no self-referential ZIP hash is embedded here. Receipts remain under ignored `artifacts/sloped-track-0.25.0/`. Same-PC extraction does not establish clean-machine compatibility. Preserve third-party/Rust notices, dependency manifests and runtime provenance. The [resume guide](../RESUME_DEVELOPMENT.md) retains all three checkout paths and handoff rules.

## Next stage

Add track purchase/refund rules with immutable quotes, full validation before any charge, saved paid basis and attributable ledger history. Define replacement and demolition refunds explicitly, preserve free legacy track treatment and prevent profitable rebuild cycles. Further geometry, more coaster instances, broader ride families, realistic physics and full ratings remain later work. RID-02/RID-03 and general construction/management families remain in progress.
