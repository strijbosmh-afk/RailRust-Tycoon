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

Runtime source is `d26fb29a26547c1e792ce0e31e73833f4c863487` on `codex/named-track-designs` / [draft PR #26](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/26), based on PR #25. All **560 local release tests passed: 326 game and 234 simulation**, with strict all-target release Clippy and formatting checks. The local test/Clippy source was parent `34b9d25292823aaf34a1aeb20a4f20e58a13305b`; the final change only shortens the Copy live feedback to fit its panel. The final runtime built and passed separate production `write-designs` / `verify-designs` processes.

[Exact-source Windows CI 40082622037](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40082622037), job `120839112046`, passed all **29 steps** on the final runtime: formatting, strict Clippy, all 560 release tests, build/package, every retained continuation proof, the new named-design writer/verifier and performance-reporting smoke. The smoke checks reporting only, not performance acceptance. The delivery executable comes from this CI artifact:

- Executable size: **48,857,088 bytes**.
- Executable SHA-256: `83dc5b5b1a365713a8b207bbe60e46c900eb582bf6250715925213b65d8f6c1d`.
- GitHub artifact `12555688606` outer ZIP digest: `eacadbe27ee661607367732d5c9b805e184019403d0a8f1e83a0bc7b554acece`.
- Original CI package ZIP checksum: `7bedd28fc20552602555e405be07caafcfd39462b48a375707c1d8bb258182fa`. The local delivery refreshes guides only; its adjacent `.zip.sha256` sidecar identifies that archive.

Both archive checksums and all CI payload hashes matched before running the extracted executable. The exact CI executable passed the named-design writer and verifier in separate processes from `C:\Windows`, with an isolated spaces/Unicode save directory. It also passed the retained **7,200-tick** paid-track and draft-history proof. Original primary, backup and catalog bytes stayed unchanged.

Ordinary strict schema-15 continuation passed in both directions: **0.27 writer / 0.28 reader** and **0.28 writer / 0.27 reader**, each for **3,600 ticks plus commands** and backup restoration. The separate design proof establishes library and history behavior. Authentic schema-14 primary and backup files retained every original field and both original files, then continued under current rules for **3,600 ticks per slot**. These fixtures do not claim arbitrary future-version compatibility or legacy future equivalence.

The added coverage includes ten simulation tests for strict codecs and checked transforms, ten storage tests for persistence and failure handling, six keyboard/command integration tests, capture purity and a fresh-process proof. Input coverage includes same-frame modifier edges and AltGr+A Unicode entry. Preliminary rendering caught clipped draft feedback; both loaded-design and Copy live notices were shortened and visually rechecked.

The new production modes use separate processes and isolated absolute directories:

```powershell
Parkbound.exe --save-check write-designs --save-dir <new-absolute-directory>
Parkbound.exe --save-check verify-designs --save-dir <same-directory>
```

They exercise actual library commands and disk persistence, Unicode naming, update/delete/cancel, fresh-process reload, transformed draft replacement, Undo/Redo, invalid-placement refusal, park Save/Load/Backup with empty restored history and retained designs, and costs only at Apply. Retained track construction/economics proofs remain available separately.

Capture scenes are `library-empty`, `library-saved`, `library-transformed`, `library-confirm-update`, `library-confirm-delete`, `library-invalid` and `loaded-design`. In-memory fixtures avoid player-library access. Presentation purity covers all seven scenes across all four rotations and both views without changing live park bytes.

Ten captures from the exact CI executable were directly inspected: the normal-window saved library; minimum-window empty and saved libraries; transformed placement in rotation 1 / Underground; update and delete confirmations (delete in rotation 2); invalid placement; loaded design in rotation 3 / Flat; retained Copy live; and Help. The 560 × 560 modal fits 1024 × 680, including the 32-character name and wrapped confirmation. The loaded design shows 28 pieces and a $470 quote while actual cash remains $1,000; Copy live still quotes $0 with $720 cash. The new controls, corrected notices and shortcut help are readable without clipping. Screenshots and input tests do not establish native mouse/keyboard acceptance.

The portable delivery is `dist/Parkbound-0.28.0-windows-x64.zip`. A fresh extraction at `C:\Users\strij\AppData\Local\Temp\Parkbound 0.28 relocated package é`, launched from `C:\Windows`, verified **43 payload hashes** with no extra files and **34 current Markdown guides**. All **291 dependency entries** and five dependency/native/notices/runtime files are unchanged from 0.27. That extracted executable passed the named-design writer/verifier; its minimum-window loaded-design capture matched the inspected CI capture byte for byte. Final guide and manifest refreshes preserve the exact executable. The final ZIP sidecar and payload/guide bytes are checked again in a second fresh extraction at `C:\Users\strij\AppData\Local\Temp\Parkbound 0.28 final delivery verification é`. Same-PC relocation does not establish clean-machine compatibility.

Raw receipts belong under ignored `artifacts/named-track-designs-0.28.0/`; delivery belongs under ignored `dist/`. These require OneDrive or explicit transfer between PCs. The [resume guide](../RESUME_DEVELOPMENT.md) retains all three checkout paths. Native input, clean-PC/offline/standard-user compatibility and representative-hardware performance remain separate open acceptance checks. This development PC is not a performance reference.

## Remaining scope

This milestone covers local named plans, placement transforms and safe draft reuse. Design thumbnails, import/export dialogs, library search, shared catalogs, multiple coasters, broader geometry/banking, ride settings and realistic physics remain later work. The wider track-design and management acceptance criteria remain in progress.
