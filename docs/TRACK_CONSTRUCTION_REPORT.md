# Track construction — 0.24.0

Updated 23 October 2026. This stage starts from `2760c715d7289536e8a55d6656e56103b7304900` on `codex/service-construction-economy` / [PR #21](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/21). Work is on `codex/track-construction`. Parkbound remains an original Rust game intended for free Windows distribution. This PC is excluded as a performance reference.

## Player contract

The first coaster editor builds a saved draft separately from the live circuit. It supports a level station, straight track and left/right quarter turns, with a world heading and whole-step height. Draft editing does not move the running train or charge cash. The existing coaster continues using its installed route until a complete replacement is applied.

Pieces may be entered in any order. A complete layout needs exactly one station, matching opposite tile-edge endpoints at identical heights, and one directed closed loop containing every piece. Missing links, multiple stations, isolated pieces, blocked space and invalid geometry produce feedback rather than partially installing a ride. The draft is bounded to 256 pieces. Level pieces may use different heights while planning, but only matching heights can connect; lifts, drops and brakes remain later editor work.

Apply revalidates the complete layout and the current park. It requires the coaster to be closed and stopped, with no active coaster visitors. A successful replacement changes only its owned rails and train position, retaining ride identity, lap history, prices, guest records, financial history and unrelated park objects. The train starts closed at the new station. Existing entrance and exit objects remain; access is recomputed from the new station. Moving a station may therefore require repairing its gates and paths. A refused replacement leaves the entire authoritative state unchanged.

The retained draft can be edited again. Removing a draft piece affects the plan; ordinary live-track demolition rules remain separate. Clear discards the draft without removing the installed circuit. This stage edits the single coaster and does not add a second coaster, track purchase costs, realistic coaster physics or ride safety ratings.

## Controls and reproduction

Open Rides and select Edit coaster track, or press Z. Choose Station, Straight, Left curve or Right curve. C rotates the world heading; the height buttons and Page Up/Down select height 0–30. Click to place or replace a draft tile. Use Erase [X] for mistakes, Clear draft to start again, Apply track for a validated replacement, and Done [Escape] to return to the park. Save/Load retain unfinished construction independently of transient pointer previews. Selecting a different ride leaves the editor while retaining its saved draft.

Build a small level rectangular loop with its station on a straight side. Add four directed corners and straight sides, all at the same height. Rotate the camera to verify that piece headings remain world-relative. Try an incomplete loop and an extra isolated piece; Apply must refuse each without changing the current circuit. Close the coaster, let passengers and transfers finish, apply the completed loop, then reopen and observe a real lap. Reconnect entrance/exit paths if the station moved.

For a fresh demonstration, this eight-piece loop fits in the southeast of the map at height 8. Coordinates below are the one-based tile numbers shown to the player, not internal zero-based positions. Each heading is the incoming direction of travel; set it with C before clicking. Pieces can be placed in any order.

| Tile X, Y | Piece | Heading |
| --- | --- | --- |
| 21, 21 | Right curve | North |
| 22, 21 | Station | East |
| 23, 21 | Right curve | East |
| 23, 22 | Straight | South |
| 23, 23 | Right curve | South |
| 22, 23 | Straight | West |
| 21, 23 | Right curve | West |
| 21, 22 | Straight | North |

This example relocates the station. It can run an empty train immediately after Apply/Open; guest service also needs gates, a queue and a connected public exit at the new station. The production proof instead retains the original station and constructs a 28-piece notched loop with both curve directions and a bridge for returning guests.

The process proof uses `--save-check write-track-construction` followed by a fresh `--save-check verify-track-construction` process with the same new absolute `--save-dir`. It exercises saved partial drafts, directed geometry, invalid/occupied refusal, atomic replacement, passenger operation, deterministic continuation and paused backup restoration. Existing continuation proofs remain part of the checks.

Capture scenes are `track-draft`, `track-invalid`, `track-ready`, `track-built` and `track-occupied`. Inspect the editor and world in all four rotations, Surface/Underground and normal/minimum windows. Rendered fixtures establish presentation separately from actual mouse/keyboard acceptance.

## Persistence

Strict schema 13/content 13 adds required bounded `track_draft` state. Explicit migration from schemas 1–12 preserves the prior park and supplies an empty draft. Original circuit, passenger, carousel, service and capital histories remain unchanged. Older readers reject the new schema without rewriting either slot. Keep closed-game legacy saves for rollback; loading alone does not overwrite them.

Incomplete and subsequently obstructed drafts are valid planning states and must remain saveable. Apply performs the world-clearance and full-loop check again. Structural corruption, unsupported pieces, invalid endpoints/heights and oversized draft arrays are rejected during loading. Existing byte, object and map limits remain enforced.

## Simulation and presentation boundary

`TrackBuildKind::spec(pos, height, direction)` creates a level `TrackPieceSpec`; direction is incoming travel, and entry is the opposite tile edge. `Park::track_draft()` exposes the plan, `validate_track_draft_piece` provides a pure placement check, and `set_track_draft_piece` replaces an existing tile or appends a new one. `remove_track_draft_piece` and `clear_track_draft` never remove live rails. `validate_track_draft` orders all pieces from the unique station. `apply_track_draft` performs the atomic replacement; reapplying an identical layout does not allocate new object IDs.

The editor uses the same sampled geometry as train movement. Saved draft rails, direction arrows and pointer previews remain presentation-only. Draft updates invalidate the displayed geometry status; ordinary guest ticks do not repeatedly validate the full plan. Applying track refreshes the world mesh without changing public-path connectivity. Existing unrelated track remains occupied and cannot be silently adopted. A faulted train may be rebuilt safely, but unowned remnants remain obstacles.

## Verification

Verification is in progress. Do not attribute the previous 0.23 executable's passes to this changed source. Final evidence will record tests, fresh-process continuation, authentic migration, inspected captures, native input scope, exact executable/package identity and GitHub CI. Raw receipts belong in ignored `artifacts/track-construction-0.24.0/`.

The Windows package and all bundled guides will be refreshed together. Git carries source and guides; ignored `target`, `dist` and raw evidence need OneDrive or deliberate transfer. Both PC paths and the shared-folder handoff instructions remain in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md). Same-PC relocation is separate from clean-machine acceptance.

## Next stage and remaining scope

Extend the editor with sloped track, chain lifts and brakes, including height transitions and operating feedback. Track economics, broader ride families, multiple coasters, realistic physics and complete ride ratings remain later work. General construction and management checklist families remain in progress.

Clean-machine offline/standard-user compatibility, signing, source licensing, final title and representative performance hardware remain separate distribution gates. No local performance batch or performance acceptance is required for this gameplay stage.
