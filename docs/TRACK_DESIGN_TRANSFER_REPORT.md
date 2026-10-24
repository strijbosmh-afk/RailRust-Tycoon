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

Runtime source is `e1887838193c8a0ae181247d25f4b46fe655d80e` in [draft PR #28](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/28), stacked on PR #27. Formatting, strict all-target release Clippy, production build and all **594 local release tests (355 game / 239 simulation)** passed. [Exact-source Windows CI 40088580143](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40088580143), job `120856420160`, passed all **30 steps**, including the same source gates, packaging, every retained continuation proof, the new transfer proof and reporting smoke. That smoke verifies reporting only, not performance acceptance. Subsequent changes update Markdown guides only.

The portable delivery uses the executable from that exact CI artifact:

- Executable size: **49,176,576 bytes**.
- Executable SHA-256: `e7fd45d35384e2508829fe62453ac224ba25ed70407b5c0580ae92febf5c86ed`.
- GitHub artifact `12557689646` outer ZIP digest: `0cfc57198782884bd1e57281188702b4b862875e7e3865f48e9b5fe722debe68`.
- Original CI package ZIP checksum: `953d1858ec7962bd9fe71e4e6570acd747894cfb4f83dfe6213195bdc017899d`. The local delivery refreshes guides only; its adjacent `.zip.sha256` sidecar identifies the resulting archive.

Both archive digests and all **45 CI payload hashes** matched before the executable was run. The exact CI executable then passed all **11 separate runtime processes** from `C:\Windows`, using isolated spaces/Unicode save directories. These cover the new file transfer, retained named designs and ten recreated preview pixel files, and the **7,200-tick** paid-track/draft-history proof. Original primary, backup, catalog and exported file bytes stayed unchanged where required.

Ordinary strict schema-15 continuation passed in both directions: **0.29 writer / 0.30 reader** and **0.30 writer / 0.29 reader**, each for **3,600 ticks plus commands** and backup restoration. Authentic schema-14 primary and backup files preserved every original field and both source files, then continued under current rules for **3,600 ticks per slot**. These establish the tested compatibility cases, not arbitrary future-version compatibility or legacy future equivalence.

The new production proof runs through the same request/completion boundary used by the real dialogs, with file choices injected for automation:

```powershell
Parkbound.exe --save-check write-design-transfer --save-dir <new-absolute-directory>
Parkbound.exe --save-check verify-design-transfer --save-dir <same-directory>
```

The writer exports a selected saved graded design while another draft and unsaved placement/name changes are active, tests approved replacement and cancellation, and preserves primary/backup/catalog bytes. The separate verifier imports into a fresh independent library, imports a numbered duplicate, reopens the catalog, refuses invalid/unsupported/oversized files, and loads/undoes the imported design without changing the live park. Both record receipts and explicitly exclude native dialog gestures. The Windows workflow retains all earlier proofs and adds this pair.

Focused tests cover strict file bounds, Windows path and filename handling, failed replacement cleanup, collision naming, locked/full/corrupt catalogs, real Import/Export button commands, pending/confirmation ordering, stale saved selections, and immutable park/draft/history. A regression uses Bevy's actual queued render-time channel: a stale timestamp followed by the timestamp containing the modal delay cannot advance the park, while normal progression resumes with the player's selected speed intact. Four additional in-memory capture scenes are `library-imported`, `library-exported`, `library-import-refused` and `library-transfer-cancelled`; no capture fixture reads a player library or opens a native dialog.

All **18 final CI captures** match the directly inspected preliminary images byte for byte: normal/minimum galleries and partial page, ready/blocked/incomplete/tall/empty/rotated states, update/delete confirmations, invalid placement, loaded draft, Help and all four transfer outcomes. The exact CI gallery was also viewed directly. Controls, full-length names, notices and confirmations fit without clipping at 1024 × 680. Screenshots and command integration tests do not establish physical native dialog gestures.

The portable delivery is `dist/Parkbound-0.30.0-windows-x64.zip`. A fresh extraction at `C:\Users\strij\AppData\Local\Temp\Parkbound 0.30 relocated package é`, launched from `C:\Windows`, verified **45 payload hashes**, no extra files and **36 bundled Markdown guides** against source. All **291 dependency entries** and five dependency/native/notices/runtime files are unchanged from 0.29. The extracted executable passed separate named-design and new file-transfer writer/verifier processes, and reproduced the final minimum-window gallery and imported-design captures byte for byte. Guide refreshes preserve the exact executable; the adjacent ZIP sidecar and payload/guide bytes are checked again after the final refresh. Same-PC relocation does not establish clean-machine compatibility.

The bounded import/export implementation and technical delivery are complete. Native file-picker gestures, including real overwrite-prompt acceptance, remain unverified because native computer-control APIs were unavailable in this session; no physical interaction result is inferred from adapter tests or injected choices.

Raw evidence belongs under ignored `artifacts/track-design-transfer-0.30.0/`; portable delivery belongs under ignored `dist/`. The [resume guide](../RESUME_DEVELOPMENT.md) retains all three checkout paths.

## Remaining scope

Native mouse/keyboard and file-picker acceptance, clean-PC/offline/standard-user compatibility and representative-hardware performance remain separate open checks. This development PC is not the performance reference. Catalog search, bulk transfer/shared catalogs, multiple coasters, broader geometry/banking, ride settings and realistic physics remain later work. The broader track-design and management criteria remain in progress.
