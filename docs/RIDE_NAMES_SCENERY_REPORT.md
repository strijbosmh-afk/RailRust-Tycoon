# Ride names and planted scenery — 0.47.0

This two-stage delivery starts from the CI-verified 0.46.0 ride-palette branch. The product remains an original Rust park simulation for free Windows distribution. This development PC is excluded as a performance reference.

## Stage 1: player ride names

The coaster and each carousel have independent names under stable ride IDs. **Rides [Y]** exposes Edit name, Apply name and Default. Text entry accepts Unicode, handles Backspace, Ctrl+A and Enter, and suppresses game shortcuts while focused. Applying trims spaces and validates a nonempty, control-free value of at most 28 characters and 96 bytes. Invalid or stale-owner commands leave simulation state unchanged. Default names are canonical and are omitted from saved name records. Naming never changes cost, operation, visitor state, random state, track or appearance. Name labels load the installed Windows Segoe UI face for accented text; the original bundled UI face is the fallback when it is unavailable. Glyph coverage still depends on the system face for less common scripts.

Strict save schema/content advances from **23/23 to 24/24**. A dedicated exact schema-23 reader migrates prior saves with no names. Name records are bounded, sorted, unique and tied to known ride IDs. The reader rejects malformed names and unknown ride identities; old schemas reject new scenery variants. Existing primary/backup recovery and all earlier migration paths remain in force.

## Stage 2: player-built scenery

**Build → Tree** and **Build → Flowers** plant original decorative objects on open visible grass or sand. Trees use three height steps of clearance and flower beds use one; both sit at the actual terrain height. Placement checks land, overlap, height, identity and a 512-scenery limit before any mutation. The existing 4,096-object cap also applies. Save validation requires canonical ground placement, clearance, axis, flat slope and no queue owner. Demolish and selected-object removal work with both kinds, without a refund because planting is free. Objects are painted and picked as part of the existing visibility/depth pipeline. Scenery currently has no gameplay modifiers.

## Evidence

The final release suite passed **801 tests (462 game, 339 simulation)**, with zero failures. `cargo fmt --all -- --check`, strict release workspace/all-target Clippy and the Windows release build passed. The final `parkbound.exe` SHA-256 is `b128cc6aa2b0f01c9d1eb09d24457b64c0eb6cf030306cbd01bc856d9812b324`. The production `write-names`/`verify-names` and `write-scenery`/`verify-scenery` modes all passed on that binary, each with an isolated Unicode/space-containing save directory, exact primary/backup comparisons and a 3,600-tick continuation in a fresh process. The scenery proof retains paid visitors, colors and names. CI runs the same writer/verifier pairs.

Three final 1024 × 680 in-engine captures were visually inspected: `names-coaster-final.png`, `names-carousel-final.png` and `scenery-final.png`. The accented name, controls and planted objects render correctly. The initial visual inspection found a missing accented glyph and an overlong Build label; both were corrected before these captures. Raw logs and captures are under ignored `artifacts/ride-names-scenery-0.47.0/`.

The portable Windows package verified **77 payload hashes and 65 current Markdown guides**, with no extra archive entries. Its extracted executable was byte-identical to the final build. From a fresh space/é directory and unrelated working directory, both two-process continuation pairs and all three capture scenes passed. The extracted coaster and scenery captures were visually inspected. The adjacent `dist/Parkbound-0.47.0-windows-x64.zip.sha256` identifies the final archive; it stays outside the ZIP to avoid a self-referential digest. Same-PC extraction does not establish clean-machine or minimum hardware acceptance, and scripted capture does not establish native mouse/keyboard interaction.

## Remaining work

This is one bounded scenery family, not the full classic scenery catalog. Add a real ghost preview, several original scenery families and placement costs as their design warrants; preserve validation and save migration. The next broad product stages include richer ride-specific sound, more construction and management features, and the remaining [gameplay checklist](GAMEPLAY_CHECKLIST.md). Do not use measurements from this PC as performance targets.
