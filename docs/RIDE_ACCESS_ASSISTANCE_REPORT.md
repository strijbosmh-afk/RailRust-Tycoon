# Ride access assistance — 0.32.0

Version **0.32.0 is locally verified** on `codex/ride-access-assistance`, from verified 0.31 checkpoint `04720f9b916a90c7ef8be994cb2c65dd88ee46fd`. The bounded implementation advances BLD-02, BLD-03 and UI-02. Source/build checks, all **634 release tests (381 game / 253 simulation; zero doc tests)**, production proofs, visual inspection and fresh portable-package verification passed. **Exact-source Windows CI passed** for checkpoint `303cd66a0e1789cb0bf4b3d3365233440efefebb`: [run 40108342235](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40108342235), all 32 steps and 634 tests. Historical version-0.31 evidence remains preserved in [RIDE_OPERATIONS_REPORT.md](RIDE_OPERATIONS_REPORT.md).

## Player behavior

Open **Rides [Y]**, select a ride and choose **Access**. The panel explains entrance/queue and exit/public-path readiness independently. Suggested gate locations follow the installed station or carousel body at its actual height. Existing gates can be located, and suggested construction prepares the existing build tool with a visible preview. Preparing assistance never constructs an object, spends money or moves guests. The player makes a deliberate map click, which uses the normal construction validation again.

Gate actions find an existing gate or prepare the first ranked legal site when absent. Queue/public-path actions prepare the first ranked legal tile, including supported slopes. Preparation closes the panel, retains the selected ride and sets the ordinary tool's location, height, slope and facing. A hidden build plane switches to Underground view. Move to the ghost before clicking: the panel-closing click or a click at the unchanged pointer position cannot construct it.

Suggestions are local next steps. Further paths, terrain work or obstruction removal may still be necessary. Branches, loops, foreign queues and disconnected public routes retain their diagnostic rules. Public suggestions reject queue middles, extra queue entries and contact with another ride's queue. No assistance action bypasses collision, station-side, height, queue-ownership or passenger protections. Unavailable rides receive advice without build suggestions.

The Operations tab retains per-ride settings and Open/Close controls. Park saves remain schema/content **16/16**; design/catalog files remain version **1**. Access reports, selected tabs and prepared previews are transient. No save migration or dependency addition is required.

## Verification

| Check | Current result |
| --- | --- |
| Production `cargo check`, formatting and strict all-target release Clippy | PASS |
| Full workspace release tests | PASS — 634 (381 game / 253 simulation), zero doc tests |
| Production release build | PASS — 47.71 seconds |
| Separate production access/operations/design-transfer/track-construction proofs | PASS — eight extracted-executable processes |
| Bidirectional 0.31 ↔ 0.32 compatibility | PASS — four processes; 7,200-tick operations replay plus backup in each direction |
| Controlled capture inspection | PASS — seven 1024 × 680 images from the extracted executable |
| Exact-source Windows CI | PASS — run 40108342235, job 120914620244, all 32 steps and 634 tests for source `303cd66a0e1789cb0bf4b3d3365233440efefebb` |
| CI artifact 12565218742 | Identified only; not downloaded or executed |
| Fresh portable extraction, payload/current-guide checks and relocated proofs | PASS — 47 payload hashes / 38 guides |
| Dependency/runtime/notice comparison with 0.31 | PASS — 291 dependencies and five unchanged files |
| Native Access/Find/Prepare gestures | Not verified |
| Clean-PC/offline/standard-user and representative performance acceptance | Not verified |

The delivered local production executable is **49,516,544 bytes**, SHA-256 **`b4bb18c338bfb08b7957e7adef42898d4b4106deaa4954022f38162a746932bc`**. Source receipts are `artifacts/ride-access-0.32.0/clippy-final.log`, `tests.log` and `build.log`. [Exact-source Windows CI 40108342235](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40108342235), job **120914620244**, passed all **32 steps** and **634 tests** for head **`303cd66a0e1789cb0bf4b3d3365233440efefebb`**. CI receipts are `artifacts/ride-access-0.32.0/ci-push-40108342235-*`. Artifact **12565218742** was identified but not downloaded or executed; no CI-artifact hash, runtime or pixel-equivalence claim is made. The existing 0.32 package and locally verified executable remain unchanged.

Fresh extraction to `C:\Users\strij\AppData\Local\Temp\Parkbound 0.32 access delivery é`, launched from `C:\Windows`, verified every payload and current guide before executing the eight proof processes. Its receipt is `verification.txt` in that directory. `artifacts/ride-access-0.32.0/compatibility/compatibility.txt` records both compatibility directions, unchanged source slots and unchanged old/current executable hashes. `notices.txt` records the dependency/runtime comparison. The current archive is `dist/Parkbound-0.32.0-windows-x64.zip`; refresh guides and integrity checks after documentation edits, keeping its mutable checksum in the adjacent sidecar.

The access proof uses separate `--save-check write-access` and `verify-access` processes with isolated absolute save directories. It retains the missing-gate backup and connected primary. Each process builds the first suggested gates and queue only in a disposable backup copy, requires real missing-queue/public-path states, and compares those derived reports. Every gate candidate and suggestion uses its normal placement validator; each available Prepare action must match the first ranked tile's mode, location, height, slope, owner, facing and exit status. Inspection/preparation preserve the entire source park and its save bytes. Both ready rides then complete paid journeys over 3,600 ticks with strict roundtrips. The verifier checks original slot bytes remain unchanged.

Source tests cover deterministic planning, station sides/heights, slopes, topology filtering, actual UI actions, pointer/modal guards and save-load reset. The seven directly inspected captures are `access-needs`, `access-ready`, `access-unavailable`, `operations-coaster`, `operations-carousel`, `library-search` and `help`. Actual topology supplies the access states; controls and text fit the minimum window. `artifacts/ride-access-0.32.0/visual-review.txt` records the inspection and exact executable identity.

The reusable source script `scripts/verify-windows-package.ps1` verifies supplied archive and executable SHA-256 values, rejects unsafe archive entries, extracts to a fresh absolute directory, checks every payload and current guide, then runs separate production proofs and captures with isolated profiles from `C:\Windows`. It retains logs and refuses an existing output directory. It never uses the player's save directory. Captured image hashes still need comparison or direct inspection. Mutable delivery ZIP checksums remain in their adjacent `.zip.sha256`, outside bundled guides.

Native gestures and clean-PC acceptance are separate from automated input tests, controlled images and same-PC relocation. This development PC remains excluded as a performance reference; no representative PC is selected and no 60 FPS or full 5,000-guest/50-ride acceptance is claimed. The normal map and save caps are unchanged.

## Following work

Next add an isolated coaster test dashboard with deterministic lap timing, speed/height telemetry, transparent bounded layout estimates and stale-result invalidation, together with bounded loading policies. The unregistered `ride_testing.rs` and `coaster_testing.rs` modules are preparation for 0.33 and are outside this delivery. Layout-informed guest choice and broader construction follow. Multiple coasters, realistic forces, additional ride families, staff, research, scenarios and full management remain unfinished. [GAMEPLAY_CHECKLIST.md](GAMEPLAY_CHECKLIST.md) tracks the wider acceptance criteria.
