# Named track design library — 0.28.0

Updated 24 October 2026. This milestone starts from `f33b96e99bd5ad1790348b526581a03940b9034e` on `codex/track-draft-history` / [draft PR #25](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/25). Work is on `codex/named-track-designs`. The [draft history report](TRACK_DRAFT_HISTORY_REPORT.md) retains the separate 0.27 runtime and delivery evidence.

## Player workflow

Open Track tools, then **Designs** or **Ctrl+D**. **Save new** stores the current draft under a name of up to 32 Unicode characters. A nonempty unfinished draft can be saved; geometry, piece order, directions and heights are preserved exactly. Empty names, control characters, oversized designs and structurally invalid pieces are rejected. Names are trimmed and matched case-insensitively using Unicode lowercase; they are never used as filenames. The library holds at most 32 designs, each with at most 256 pieces.

Use Previous/Next to choose a design. **Update selected** saves the current draft and entered name over the selected design after confirmation. **Delete selected** also requires confirmation. The confirmation freezes the selected design and proposed replacement. A newer change made by another instance causes refusal rather than being overwritten. Cancel and Close take precedence over other commands, and Confirm cannot approve a prompt first opened in that same frame. Saved-library updates and deletions are separate from draft Undo.

Choose the anchor X/Y, entry height and clockwise rotation before **Load into draft**. The anchor is the first station in stored order, or the first piece for a design without a station. Coordinates are shown one-based. Rotation turns both tile offsets and port directions around that anchor; vertical translation preserves every grade. Out-of-map or invalid-height placements refuse the entire load rather than clamping or cropping pieces. Loaded plans can remain incomplete or obstructed; Apply checks the actual world, connections and quote.

Loading a changed design replaces the draft as **one undoable action**. Undo restores the previous draft; Redo restores the loaded design. Loading an identical plan leaves existing history and redo intact. Loading never changes live rails, object IDs, cash, paid basis, passengers or train state. **Apply** remains the only construction transaction and retains its closed/stopped/unoccupied, clearance and affordability checks. A successful Apply clears draft history as before.

The name field accepts text, Backspace and Ctrl+A; Enter finishes typing. While the library is open, typing and library controls cannot trigger park hotkeys, world edits, Save/Load or draft Undo. Escape cancels a confirmation or closes the library. Existing Ctrl+Z/Ctrl+Y/Ctrl+Shift+Z work again after returning to Track tools.

## Persistence and recovery

The library lives under the selected save directory in `track-designs/track-designs.json`, independently of `park.park.json`. Normal launching uses the existing user save folder; `--save-dir` isolates both parks and designs. Park Save, Load and Backup do not revert or delete library entries. Restarting and opening the library reads the persisted catalog. Transfer the whole `track-designs` folder with the save directory to reuse the library on another PC.

Park save schema/content remain **15/15**. Designs use independent format `parkbound-track-design`, version **1**; the catalog uses `parkbound-track-design-library`, version **1**. Designs contain only names and geometry, never ride identity, paid basis or financial history. Strict bounded codecs reject unknown/duplicate fields, unsupported versions, positional object forms, oversized arrays and malformed geometry. An individual encoded design is limited to 128 KiB and the catalog to 8 MiB.

Catalog mutations lock, reread and validate the current library before writing. Staging files are flushed and replaced in the same directory; the previous catalog is retained as `track-designs.json.bak`. Failed replacement rolls back the backup, and rollback failures report their recovery file. Fixed filenames and symbolic-link/reparse-point checks protect against accidental path substitution. This is a local store, not a guarantee against hardware power loss or hostile concurrent filesystem changes.

A damaged or unsupported catalog is refused without silently discarding entries or replacing it with an empty library. Close all Parkbound instances before manually restoring `track-designs.json.bak` if needed. A crash can leave `track-designs.json.lock`; only remove that exact lock after closing all instances. The application never guesses that a lock is stale. Detailed failures, including the directory, are logged; the panel shows a short explanation and keeps the current draft.

## Verification

Verification is in progress. Ten new simulation tests passed for strict design codecs, exact legacy/graded geometry, all rotations and checked transformations. The local full release checks, exact-source Windows CI, production-process checks, screenshots and portable package checks must complete before the 0.28 delivery is marked verified. The latest verified delivered build remains 0.27 until then.

The new production modes use separate processes and isolated absolute directories:

```powershell
Parkbound.exe --save-check write-designs --save-dir <new-absolute-directory>
Parkbound.exe --save-check verify-designs --save-dir <same-directory>
```

They exercise actual library commands and disk persistence, Unicode naming, update/delete/cancel, fresh-process reload, transformed draft replacement, Undo/Redo, invalid-placement refusal, park Save/Load/Backup with empty restored history and retained designs, and costs only at Apply. Retained track construction/economics proofs remain available separately.

Capture scenes are `library-empty`, `library-saved`, `library-transformed`, `library-confirm-update`, `library-confirm-delete`, `library-invalid` and `loaded-design`. In-memory fixtures avoid player-library access. Presentation purity covers all four rotations and both views; direct screenshots must inspect normal and 1024 × 680 windows, the 32-character name, confirmations, placement feedback and the compact Track tools header.

Raw receipts belong under ignored `artifacts/named-track-designs-0.28.0/`; delivery belongs under ignored `dist/`. These require OneDrive or explicit transfer between PCs. The [resume guide](../RESUME_DEVELOPMENT.md) retains all three checkout paths. Native input, clean-PC/offline/standard-user compatibility and representative-hardware performance remain separate open acceptance checks. This development PC is not a performance reference.

## Remaining scope

This milestone covers local named plans, placement transforms and safe draft reuse. Design thumbnails, import/export dialogs, library search, shared catalogs, multiple coasters, broader geometry/banking, ride settings and realistic physics remain later work. The wider track-design and management acceptance criteria remain in progress.
