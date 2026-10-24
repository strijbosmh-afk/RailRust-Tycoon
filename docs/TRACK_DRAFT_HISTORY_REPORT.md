# Installed-track copying and draft undo/redo — 0.27.0

Updated 24 October 2026. This milestone starts from the 0.26 documentation checkpoint `925bae82076a95098aafa582b3d7a936fd6ddd28` on `codex/track-economy` / [draft PR #24](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/24). New work is on `codex/track-draft-history`. The [track economy report](TRACK_ECONOMY_REPORT.md) preserves the separate 0.26 runtime, CI, executable and package evidence. Those results do not verify this milestone.

Parkbound remains an original, free Windows theme park game developed in playable stages. Normal play remains 32 × 32. No representative performance PC is selected; development machines remain excluded from performance acceptance.

## Player contract

Open Track tools with Z or the ride panel. **Copy live** replaces the draft with the installed coaster's exact pieces, including their world tiles, directions and endpoint heights. Copying works while the coaster is open or occupied because it changes only the plan. It does not stop the train, move visitors, allocate object IDs or alter cash, paid basis, portals or ledger entries. A missing or faulted circuit refuses the copy and preserves the existing draft. Legacy demonstration rails copy without becoming paid rails.

The copy is one undoable action, so replacing an existing unfinished draft does not lose it. Placement, replacement of a draft tile, erasing, Clear and Copy live each record a step only when the draft actually changes. Clicking an identical piece, clearing an empty plan, copying an exact match or receiving an invalid placement refusal does not consume history or discard redo. Palette selection, heading and height controls, camera changes and previews are not draft edits.

**Undo / Ctrl+Z** restores the previous draft. **Redo / Ctrl+Y / Ctrl+Shift+Z** restores an undone draft. A new successful edit after Undo discards the redo branch. Up to 64 edits are retained per session; the oldest step is dropped when that bound is reached. The buttons show when history is unavailable. Closing and reopening the editor retains its history. An externally replaced draft starts a new history rather than restoring snapshots from an unrelated plan.

Undo and Redo change the draft only. They never reverse construction payments, refunds, demolition, train movement or guest activity. **Successful Apply clears both history stacks**, including an identical zero-charge Apply. Invalid, unaffordable or occupied Apply keeps the draft and its history. The existing closed/stopped/unoccupied checks and current-price settlement remain authoritative. An unchanged copied circuit quotes $0; changing a paid $10 straight into $20 brakes quotes a $5 refund and $15 net purchase. Undo restores the $0 plan, Redo restores the $15 plan, and only Apply settles the transaction.

The compact track panel retains the existing footprint at 1024 × 680. Copy live sits beside Erase and Clear; Undo/Redo have a separate row. Shortcuts consume their Y/Z keys rather than opening Rides or closing Track tools. Modal transitions, Done, Escape and loading take precedence over a history edit. A button and its matching history shortcut in the same frame perform one step. Saving in the same frame records the resulting draft.

## Persistence and implementation

Save schema/content remain **15/15**. The draft already belongs to the saved park; undo/redo stacks belong only to the editor session and are never serialized. Save preserves the current draft and keeps the current session history. Successful Load or Backup restores the saved draft, pauses the game and resets history through the existing view reset. A failed load preserves the running park and history.

`Park::copy_installed_track_to_draft()` validates the installed circuit before copying its exact specifications. `Park::replace_track_draft()` validates a complete snapshot's size, piece shapes, map/height bounds and unique tiles before mutation. Both return whether the draft actually changed. Rejected snapshots leave the park unchanged. A restored snapshot may be incomplete or newly obstructed by a world edit: Apply rechecks joins, clearance, operating state and affordability. This preserves the established distinction between a saved plan and a buildable circuit.

History stores bounded draft snapshots, never whole-park snapshots. Each change invalidates preview/status text; quotes read the current park so undo/redo cannot offer stale cash or refunds. The initial legacy 46-piece demonstration circuit is copied without geometry conversion.

## Reproduction and evidence

The existing production commands are extended, without adding another CLI mode:

```powershell
Parkbound.exe --save-check write-track-construction --save-dir <new-absolute-directory>
Parkbound.exe --save-check verify-track-construction --save-dir <same-directory>
```

Use separate writer and verifier processes. The original graded paid passenger and finance proof remains, including 7,200 ticks, retained IDs, saved remnants, ledger rollover and atomic refusals. It now copies the live layout through production commands, changes pieces through the production picker, undoes/redoes the changes and checks the complete live state. A separate `editing-session` primary/backup pair stores edited and copied plans; the fresh verifier loads them exactly, checks empty session history and verifies that Apply clears history without permitting a later refund through Undo.

New capture scenes are `track-copied`, `track-edited`, `track-undone`, `track-redone` and `track-history-empty`. Existing draft, invalid, occupied, finance and help scenes remain available. Review normal/minimum windows, four rotations, Flat/Terraced and Surface/Underground. Capture fixtures check presentation does not alter park bytes or history counts. Screenshots do not establish native mouse/keyboard acceptance.

Verification is in progress. The full **224-test release simulation suite** has passed, including seven new copying/snapshot regressions. Record the final runtime source, full game/simulation counts, exact CI, executable checksum, direct capture inspection and portable-package checks here before delivery. The currently verified delivered archive remains 0.26 until the new executable completes those checks.

Raw receipts will be retained under ignored `artifacts/track-draft-history-0.27.0/`; that folder and `dist` require OneDrive or explicit transfer between PCs. The [resume guide](../RESUME_DEVELOPMENT.md) preserves all three checkout paths. Native input, clean-PC/offline/standard-user compatibility, signing/title/licensing decisions and representative-hardware performance remain open. No local performance batch is required for this bounded editing milestone.

## Remaining scope

This milestone completes copying the installed circuit and reversible draft editing. Reusable named track designs, broader geometry, multiple coasters, station/access editing assistance, ride settings and realistic physics remain later work. The bounded slice does not complete general coaster-design or management acceptance.
