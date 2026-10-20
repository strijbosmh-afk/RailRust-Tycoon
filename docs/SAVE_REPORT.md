# Save and continuation report — 0.6.0

Date: 20 October 2026. Acceptance ID: **PRF-04**. State: **verified for same-build authoritative save/load continuation**, through 131 tests, separate production processes, twelve inspected GPU captures, and extracted-package relocation. Native Save/Load/Backup controls remain unverified; this narrow technical acceptance does not close all controls, broader persistence, or release gates. Implementation `d7a16f6` is pushed in [draft PR #4](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/4); [Windows CI run 39422425034](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39422425034) passed for implementation `d7a16f6`. The earlier [train](TRAIN_REPORT.md), [crowd](CROWD_REPORT.md), [overlap](OVERLAP_REPORT.md), [terrain](TERRAIN_REPORT.md), and [foundation](FOUNDATION_REPORT.md) reports retain their named-build evidence.

## Scope and acceptance

The proof saves the complete existing seeded simulation, validates it before loading, and reproduces same-build authoritative continuation under the same subsequent commands. The saved park includes terrain/materials, stable object identities and allocators, elapsed ticks, crowd random-generator state, guest routes/waits/arrivals, circuit specifications, and train operation/motion state. It does not add construction, boarding, finances, autosave, a save browser, arbitrary map presentation, or historical-version migrations.

PRF-04 requires uninterrupted simulation and save → load → continuation to agree for moving guests/train, station dwell, a pending Close request, and lost/recovered guests. Malformed, truncated, unsupported, oversized, and invalid-reference files must be rejected before replacing live state. Storage checks must cover a previous valid backup, corrupt-primary protection, write/replace failures, Unicode/space paths, and explicit backup recovery. Ordinary Save/Load/Backup controls and the resulting paused scene must be observed where native testing is available; any unobserved controls remain explicit.

## Player behavior

| Control | Behavior |
| --- | --- |
| F5 / Save | Replace the single primary slot with the current authoritative park |
| F9 / Load | Validate and load the primary slot |
| Shift+F9 / Backup | Validate and load the backup explicitly |

Sidebar buttons and a two-line status explain success or failure. Every startup creates the seeded demonstration without automatically loading or saving. A successful load starts paused, resets camera/selection/view, rebuilds derived presentation, and selects 1x as the resume speed. Train Open/Close intent, motion, crowd state, and elapsed ticks come from the save. Camera, presentation settings, selection, and pause/speed preferences are not part of the authoritative schema.

The app accepts only the currently supported 32 × 32 presentation. The simulation codec can validate other bounded park dimensions, but the app rejects them before replacing its live park. Terrain heights are preserved exactly; Flat/Terraced is inferred from heights and other valid height layouts are labelled Custom until the player applies a preset.

## Files and cross-PC use

Default directory: `%LOCALAPPDATA%\Parkbound\saves`.

| File | Purpose |
| --- | --- |
| `park.park.json` | Primary single save slot |
| `park.park.json.bak` | Previous valid primary retained as explicit recovery |
| `park.park.json.lock` | Exclusive writer lock |

`--save-dir` accepts an absolute directory override, including spaces and Unicode. A missing or relative LocalAppData value is an error, rather than a fallback to the executable or repository folder. Save targets reject symlinks/reparse points. This storage is separate from both recorded source checkouts:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon
```

The first is the current shared OneDrive workspace; the second is the separate historical second-PC clone. GitHub source pushes and source-folder synchronization do not transfer LocalAppData saves. Close the game before copying the primary and optional backup to the receiving PC's save directory; preserve any existing receiving-PC save separately. Do not copy a lock file or temporary in-progress file. Use the same supported build/schema and verify a successful paused load before resuming.

After a crash, a stale lock is not silently cleared. Close all Parkbound instances, confirm no save operation is running, and remove only `park.park.json.lock` in the selected save directory. Keep primary, backup, and any reported recovery file. Do not remove the entire saves directory to resolve a lock.

If the UI reports **Recovery: see save folder**, close all Parkbound instances and preserve files matching `.parkbound-rollback-<process-id>-<Unix-nanoseconds>-<sequence>.tmp` in that directory. Such a file contains the exact former backup bytes retained because both the final primary replacement and backup rollback failed. Copy it to a separate recovery location before inspection; validate it through the current codec before restoring a primary/backup filename. Do not delete the existing primary or backup while diagnosing recovery, and do not assume a retained historical backup is valid merely because its bytes were preserved.

## Schema and simulation boundary

`Park::to_save_bytes()` and `Park::from_save_bytes()` define the window-independent codec; `SaveError` reports encoding/validation failures. JSON envelope identifiers are `format: "parkbound-save"`, `schema_version: 1`, and `content_version: 1`. Unknown fields are rejected throughout nested objects. Structs must be JSON objects and enums their declared strings; positional struct arrays and tagged-object unit enums are rejected. Nullable fields must explicitly exist as null or a value. Unsupported versions are rejected explicitly; no upgrade/migration promise is made yet.

The decoder builds a candidate Park and validates it before the game replaces live state. It preserves stored order, stable IDs and next-ID counters, terrain/materials, clock, crowd seed/RNG, guest movement/routes/waits/arrivals, and all public/private train continuation fields. Legitimate lost guests with no current path and a faulted train with no circuit are representable. Geometry is reconstructed from ordered circuit specifications and original object IDs without calling fixture installation or allocating replacement objects.

Rail samples, lengths, station cache, meshes, Bevy entities, render handles, and derived train poses are rebuilt. They are not a second persisted source of truth. Validation covers map and height bounds, unique IDs and monotonic allocators, clearance/occupancy, path adjacency and references, coherent guest state, circuit ports/grades/closure/identity, and train phase/distance/index relationships.

The input limit is 8 MiB (`MAX_SAVE_BYTES`). Collection limits are 4,096 objects, 1,024 paths, 256 guests, 2,048 circuit pieces, and 65,536 remaining route IDs in total. Individual route vectors are capped at 1,024, and valid routes cannot loop/reuse paths. Count checks precede expensive graph work. These are bounded proof-format limits, not the proposed large-park performance target; PRF-05 must measure and revisit them deliberately.

## Filesystem boundary and failure behavior

`SaveStore::user_directory()` resolves LocalAppData. The store owns `save(&Park)`, `load()`, and `load_backup()` and exposes directory/primary/backup paths for diagnostics. Saving validates encoded bytes before writing, stages and syncs temporary files in the same directory, then installs the backup and primary through replacement. A valid existing primary becomes the backup. An invalid primary never replaces a valid backup. Loading does not implicitly fall back to or promote the backup.

The primary and backup are separate filenames and cannot be one atomic transaction through the standard filesystem API. A failed final primary replacement triggers backup rollback from a synced snapshot; rollback failure is reported with a retained recovery file. Do not claim immunity to every power-loss, filesystem, antivirus, or external-writer failure. The exclusive lock coordinates cooperating Parkbound writers, and staging cleanup is restricted to files created by that operation. The Windows failed-primary-replacement test passed and preserved both original files after rollback.

`--save-check write` and `--save-check verify` provide an opt-in fresh-process diagnostic using production save/load actions without initializing a renderer. An explicit absolute `--save-dir` is mandatory. Write requires a new directory whose parent already exists and refuses an existing directory. It records a baseline backup, then a moving train with pending Close and lost guests after path deletion at tick 731, and calculates expected continuation. A separate verify process reloads paused, runs 3,600 ticks, restores paths at continuation tick 100, reopens the train at tick 1,800, compares the whole Park, and explicitly checks the backup. It creates `verification.txt` only on success and refuses to replace an existing receipt. Both commands passed after packaging in hosted Windows run 39422425034 for implementation `d7a16f6`.

Example commands, using a new directory name each time:

```powershell
$proofExe = (Resolve-Path -LiteralPath '.\target\release\parkbound.exe').Path
$proofDirectoryArgument = '"C:\existing-qa-parent\Parkbound save proof é"'
$writer = Start-Process -FilePath $proofExe -ArgumentList @('--save-check', 'write', '--save-dir', $proofDirectoryArgument) -Wait -PassThru -WindowStyle Hidden
if ($writer.ExitCode -ne 0) { throw 'Save proof write failed' }
$reader = Start-Process -FilePath $proofExe -ArgumentList @('--save-check', 'verify', '--save-dir', $proofDirectoryArgument) -Wait -PassThru -WindowStyle Hidden
if ($reader.ExitCode -ne 0) { throw 'Save proof verification failed' }
Get-Content -LiteralPath 'C:\existing-qa-parent\Parkbound save proof é\verification.txt'
```

The placeholder parent `C:\existing-qa-parent` must exist; choose a new child directory. `Start-Process -Wait` collects the GUI-subsystem executable's exit status before the next process begins; verify the receipt as well. Use a new QA directory, never a player's save slot. These diagnostics do not replace native control observation.

Render diagnostics add `--capture-scene saved|loaded|backup|load-error`; every persistence scene requires an explicit `--save-dir`. `saved` executes the production Save action and writes that selected slot, `loaded` loads the primary, and `backup` loads the backup. The latter two retain the normal load's reset view and paused state. `load-error` requires a missing or invalid slot and asserts that the failed production Load leaves the Park unchanged. Prepare isolated test files for the intended case; do not point a `saved` capture at a player's save directory. Combine with `--capture-frame` and the existing minimum-window options to inspect status/help fit and rebuilt geometry.

## Verification evidence

| Check | Current result |
| --- | --- |
| Runtime commit / branch / review | `d7a16f629dd0f89134314ea5f7b35a47d2b6ed6f`, pushed on `codex/save-load-continuation` in [draft PR #4](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/4), stacked on `codex/train-circuit` / PR #3. Base checkpoint `9c95b88`; the train review remains open and unmerged. |
| Formatting / strict Clippy / release build | Passed; final release executable identity is recorded below |
| Simulation/schema/store/application tests | Passed: 131 tests, 0 failures (56 simulation, 27 world/rendering, 28 input-system, 5 persistence, 9 store, 4 main-module, 2 capture) |
| Continuation and corruption/recovery scenarios | Fresh-process production executable write/verify passed: 3,600 ticks plus commands, exact whole-Park continuation, explicit backup, and paused load. Corruption/rollback cases pass in the tests above. |
| GPU captures and minimum-window help/save controls | Twelve final captures exited 0 and were visually inspected; all controls/status/version fit at 1024 × 680 |
| Native Save / Load / Backup / shortcuts | Unverified: an isolated native attempt was obscured by another app; no keyboard or mouse input was issued and only the test's own process was closed |
| Executable SHA-256 | `eb2b21b97e6053dd256034ade6cd17d5cac0c7ca7770db8873f17aab791f04fd` (45,597,184 bytes) |
| ZIP payload, guides, and relocation checks | All 20 payload hashes and 12 current Markdown guides matched after extraction to a fresh temporary path with spaces/Unicode. Extracted fresh-process write/verify from `C:\Windows` passed; loaded-small capture was byte-identical to the original. |
| Hosted Windows CI | [Run 39422425034](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39422425034) completed successfully for implementation `d7a16f629dd0f89134314ea5f7b35a47d2b6ed6f`: formatting, lints, tests, packaging, fresh-process continuation, and artifact upload |

PRF-04 is accepted for the demonstrated technical continuation behavior and its implementation is saved on GitHub with successful hosted verification. The recorded CI result belongs to implementation `d7a16f6`; a later documentation-only head has its own run status and is not implicitly covered by this result. ZIP hashes belong in the adjacent `.zip.sha256` sidecar; embedding the archive hash inside its own packaged guide would make it stale. Repackage documentation-only evidence updates and recheck all guides/hashes before handoff.

The 131-test workspace run covers exact authoritative round trips and continuation, stable allocator gaps/saturated counters, rebuilt circuit caches, malformed/unsupported/oversized saves, invalid reference/state rejection, and preserved live state on failed loads. Store tests include Windows final-primary rename failure with successful backup rollback, growth-aware bounded reads, corrupt-primary backup protection, and refusal to remove another writer's lock. Input/application tests cover both Shift keys, button/keyboard precedence, help/capture gating, held-button single execution, and load replacing all meshes in the same update while clearing selection. These input tests supply Bevy resources; they are not native desktop observations.

The final production executable and the freshly extracted copy passed separate `--save-check write` and `verify` processes using new absolute temporary save directories containing spaces and `é`. Extracted verification ran from `C:\Windows`, independent of the executable/source working directory. Its receipt reported continuation over 3,600 ticks plus commands, explicit previous-backup restoration, and paused loading as passing.

Final inspected captures were `saved-small`, `loaded-small`, `backup-small`, `missing-small`, `invalid-small`, `unsupported-small`, `custom-small`, `help-small`, `path-small`, `train-lift`, `crowd`, and `underground-track`. Every process exited 0. Minimum-window captures showed all controls, status lines, and version footer within 1024 × 680. Loaded/custom/error scenes showed the intended reconstructed park or preserved live park and status. The extracted executable's `loaded-small` PNG was byte-for-byte identical to the original.

Packaging collected 291 dependency notices, the default font attribution, Rust standard-library notices, and one required MSVC DLL. All 20 manifest payload hashes and twelve guides matched the extracted package. These are development-PC relocation checks, not a clean-machine or public-release guarantee.

A separate native attempt launched an isolated extracted instance. The helper initially timed out, then identified the unique game window, but another application's window obscured the game. No keyboard/mouse action was issued; testing stopped and only that test process was closed. Therefore F5/F9/Shift+F9 and the Save/Load/Backup buttons still require ordinary native verification despite passing synthetic input and production-action diagnostics.

## Remaining checks and next step

Retain the earlier native O/F and Open/Close/Find button retests, physical Delete retest, FND-02 keyboard pan/right-middle drag/physical display checks, hardware frame-rate measurements, and clean-PC offline/standard-user package testing. The interrupted 0.5.0 native session verified a completed train lap and Space pause only. No new control or display claim is implied by earlier evidence.

Proceed to **PRF-05 measurement**: record the reference PC and benchmark frame time, simulation/routing time, memory, save size, and load time. Preserve the accepted save/continuation behavior while completing native retests separately. The proposed 256 × 256, 5,000-guest, 50-ride, 60-FPS target is not an existing capability. Stage 3 remains the later first complete park loop. General save-system entries SAV-01/SAV-02 remain subject to their broader gameplay, autosave, and migration acceptance.
