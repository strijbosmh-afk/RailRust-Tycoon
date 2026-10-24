# Saved track design import and export — 0.30.0

Updated 24 October 2026. This milestone starts from `8439032fe36126f87d412e7c38f0a29f0f2a0189` on `codex/track-design-previews` / [draft PR #27](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/27). Work is on `codex/track-design-transfer`. The [design previews report](TRACK_DESIGN_PREVIEWS_REPORT.md) preserves the verified 0.29 runtime and delivery evidence.

## Player workflow

Open Track tools, then **Designs** or **Ctrl+D**. **Export selected** opens the Windows Save dialog for an individual `.parktrack.json` file. It exports the selected persisted design's name and exact ordered pieces, directions and grades. The current draft, edited name field and placement rotation/position/height do not change that file. If another library instance has changed the selected design, reopen the library before exporting.

**Import file** opens the Windows Open dialog. A valid file is added and selected in the local library, with its thumbnail and preview ready. Existing names are compared case-insensitively; a collision receives the first available ` (2)`, ` (3)` and subsequent suffix, shortening the original name by Unicode characters when necessary to retain the 32-character limit. Import never overwrites an existing design. The existing 32-design library limit applies.

Import and export do not alter installed rails, current draft, Undo/Redo or money. **Load into draft** remains the explicit free, undoable step; **Apply** remains the construction transaction. Cancel and Close take precedence over pending commands. Confirmation prompts and outstanding transfers block other library actions, including actions queued in the same confirmation frame.

## Files and native dialogs

The portable file uses the existing strict **TrackDesign version 1** format. Park saves remain **schema/content 15/15**, and the local catalog remains **version 1**. No image payload, migration or new geometry is introduced. Names, piece limits, map bounds and geometry are checked by the same simulation codec used by the library. Reads are limited to 128 KiB and require valid UTF-8; malformed, unsupported and oversized inputs leave the catalog untouched.

Imports reread and update the catalog under its existing lock, so collision naming also handles another instance's intervening additions. Full, locked or malformed catalogs refuse the operation. Exports write and sync a unique sibling temporary file before atomically replacing the chosen destination. A failed replacement preserves the previous destination and cleans up its own temporary file. Export only accepts `.parktrack.json` destinations; park and catalog paths cannot be overwritten through that action. Regular absolute paths are required; symlink/reparse paths and unsafe Windows device/stream components are refused.

Windows Common Item Dialogs run on the game's owning main thread and use its actual window handle. The Save dialog confirms replacement of the exact selected path; the adapter never rewrites that path after confirmation. A different explicit extension is refused with a retry message. Dialogs leave the working directory unchanged. Waiting in a dialog must not advance the simulation through catch-up ticks; the selected speed is preserved. Physical native picker gestures and overwrite-prompt acceptance remain separate checks.

The direct Windows dependencies `windows` 0.61.3 and `raw-window-handle` 0.6.2 already existed in the locked graph. No new package version or external visual asset is added. The 900 × 600 gallery and 1024 × 680 minimum window are retained.

## Verification

Formatting, strict all-target release Clippy, production build and all **594 local release tests (355 game / 239 simulation)** passed. Rendered capture inspection, exact-source Windows CI, downloaded artifact identity, compatibility proofs and relocated package checks are in progress. Do not treat the 0.29 evidence as a 0.30 result.

The new production proof runs through the same request/completion boundary used by the real dialogs, with file choices injected for automation:

```powershell
Parkbound.exe --save-check write-design-transfer --save-dir <new-absolute-directory>
Parkbound.exe --save-check verify-design-transfer --save-dir <same-directory>
```

The writer exports a selected saved graded design while another draft and unsaved placement/name changes are active, tests approved replacement and cancellation, and preserves primary/backup/catalog bytes. The separate verifier imports into a fresh independent library, imports a numbered duplicate, reopens the catalog, refuses invalid/unsupported/oversized files, and loads/undoes the imported design without changing the live park. Both record receipts and explicitly exclude native dialog gestures. The Windows workflow retains all earlier proofs and adds this pair.

Focused tests cover strict file bounds, Windows path and filename handling, failed replacement cleanup, collision naming, locked/full/corrupt catalogs, real Import/Export button commands, pending/confirmation ordering, stale saved selections, and immutable park/draft/history. Four additional in-memory capture scenes are `library-imported`, `library-exported`, `library-import-refused` and `library-transfer-cancelled`; no capture fixture reads a player library or opens a native dialog.

Raw evidence belongs under ignored `artifacts/track-design-transfer-0.30.0/`; portable delivery belongs under ignored `dist/`. The [resume guide](../RESUME_DEVELOPMENT.md) retains all three checkout paths.

## Remaining scope

Native mouse/keyboard and file-picker acceptance, clean-PC/offline/standard-user compatibility and representative-hardware performance remain separate open checks. This development PC is not the performance reference. Catalog search, bulk transfer/shared catalogs, multiple coasters, broader geometry/banking, ride settings and realistic physics remain later work. The broader track-design and management criteria remain in progress.
