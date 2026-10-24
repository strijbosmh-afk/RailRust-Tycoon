# Saved track design previews — 0.29.0

Updated 24 October 2026. This milestone starts from `bf003fdb10da371e66099700f4a7a8e51f35bcf3` on `codex/named-track-designs` / [draft PR #26](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/26). Work is on `codex/track-design-previews`. The [design library report](TRACK_DESIGN_LIBRARY_REPORT.md) preserves the separate 0.28 runtime and delivery evidence.

## Player workflow

Open Track tools, then **Designs** or **Ctrl+D**. The library now shows four clickable thumbnails per page and a larger selected preview. Previous/Next still selects individual entries and follows them onto the appropriate page; the selected thumbnail has a gold border. A partly filled page hides its unused cards. Confirmation prompts freeze gallery selection until confirmed or cancelled.

The image follows the saved pieces' actual curves, directions and grades. Stations are gold, with distinct lift, drop and brake details and an anchor travel arrow. **Turn** rotates the larger preview. It is a normalized shape view: changing X/Y or entry height updates placement checks, while the shape stays centered and fitted to its image. The 900 × 600 modal fits the supported 1024 × 680 minimum window; selected images are 320 × 180 and thumbnails are 156 × 70.

Below the image, the current placement reports its construction purchase, refund, net settlement and available cash, or explains why a quote cannot be supplied. Readiness also checks the coaster's operating state and affordability. Incomplete designs still have previews and may be loaded for editing. Invalid translation or height prevents loading as before. Preview evaluation uses the same candidate validation and quote logic as Apply, without temporarily replacing the current draft.

Browsing, rotating and quoting do not change live rails, the draft, Undo/Redo, object IDs, cash, paid basis, passengers or cached path data. **Load into draft** remains one free undoable replacement; **Apply** rechecks the current world and is the only construction transaction. Library Save/Update/Delete keep their existing persistence and confirmation rules.

## Derived images and compatibility

Images are generated from validated saved geometry, with signed anchor-relative coordinates so even a design whose rotated placement would leave the map can be displayed. The rasterizer uses bounded dimensions, deterministic supersampling and geometric strokes. No external assets or new dependencies were added.

Images are not written into the design catalog or park save. Restarting recreates them. The memory cache holds at most 32 catalog thumbnails plus one selected preview; renaming or reselecting unchanged geometry reuses assets, rotation updates the selected image in place, and updates/deletions release replaced images. Gallery paging does not accumulate assets.

Park schema/content remain **15/15**; design and catalog formats remain **version 1**. Existing named libraries need no migration. The 32-design/256-piece limits and 32 × 32 normal map remain unchanged.

## Verification

Verification is in progress. Release compilation has passed; final formatting, strict all-target Clippy, release tests, production fresh-process checks, visual inspection, exact-source Windows CI and packaged-executable checks are pending. The latest verified portable delivery remains 0.28 until these receipts are recorded.

Focused coverage includes immutable candidate quotes and operating guards; all piece kinds and directions; rotated-geometry equivalence; extreme heights and bounded image sizes; asset reuse and release; real thumbnail button actions, paging, confirmations, update/delete refreshes and draft/history preservation.

The existing production commands now also compare ten derived pixel files across separate processes (two saved geometries, each with its thumbnail and all four preview rotations):

```powershell
Parkbound.exe --save-check write-designs --save-dir <new-absolute-directory>
Parkbound.exe --save-check verify-designs --save-dir <same-directory>
```

These files are proof output, not player-library data. The verifier previews one saved design while a different draft is active, checks unchanged park/history, and compares the preview quote with the subsequent loaded-draft quote. Existing persistence, Undo/Redo, confirmation, invalid-placement, park Save/Load/Backup and paid Apply checks remain in the same proof.

New capture scenes are `library-gallery`, `library-gallery-last`, `library-preview-ready`, `library-preview-blocked`, `library-preview-incomplete` and `library-preview-tall`. The seven prior design scenes remain available. All use in-memory catalogs; presentation-purity coverage spans every scene, all four camera rotations and both world views.

Raw receipts belong under ignored `artifacts/track-design-previews-0.29.0/`; portable delivery belongs under ignored `dist/`. Transfer these separately when moving PCs. The [resume guide](../RESUME_DEVELOPMENT.md) retains all three checkout paths.

## Remaining scope

Native mouse/keyboard acceptance, clean-PC/offline/standard-user compatibility and representative-hardware performance remain separate open checks. This development PC is not the performance reference. Import/export dialogs, search, shared catalogs, multiple coasters, broader geometry/banking, ride settings and realistic physics remain later work; the broader track-design and management criteria remain in progress.
