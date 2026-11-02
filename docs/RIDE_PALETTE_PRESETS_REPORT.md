# Ride palette presets — 0.46.0

This stage starts from the verified 0.45.0 ride-color build on `codex/ride-colors` (`a122c907c41ac19e3d7962bd31c5cfc7d56d7cb5`, draft PR #44). The 0.46.0 runtime source is `65a249efd759abe69aeeb6089611128ac0951d1a` on `codex/ride-palette-presets` in [draft PR #45](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/45). Parkbound remains an original Rust game intended for free Windows distribution. This development PC is excluded as a performance reference.

## Player behavior

Open **Rides [Y] → Colors** and select Harvest, Seaside or Twilight. Each button sets Structure, Supports and Vehicles at once on the selected ride, including an operating coaster or carousel. The three existing color controls can then refine any channel. Restore classic colors returns all three channels to the original ride look. Presets are free and do not change physical geometry, guest choice, operation, money, ratings or value.

| Preset | Structure | Supports | Vehicles |
| --- | --- | --- | --- |
| Harvest | Sunflower | Meadow | Coral |
| Seaside | Lagoon | Sky | Cream |
| Twilight | Violet | Charcoal | Sky |

The choices are project-authored combinations of the nine existing colors. No external art, sound or dependency is added. Save the park to keep the selected appearance. Each stable ride ID owns its own appearance; replacement ride IDs start Classic. The three preset buttons occupy one row in the existing Colors panel and remain within its minimum-window width.

## Persistence and implementation

The preset is a one-time edit to the existing `RideAppearance` record. Strict park save schema/content remains **23/23**, so no new field or migration is introduced. Saved records still hold the three explicit color values, allowing individual edits and deterministic continuation. The existing schema-22 reader and earlier migrations remain in place. `RidePalettePreset::appearance(ride)` constructs the three-channel value; `Park::apply_ride_palette_preset` delegates to the existing owner/install validation and atomic setter. UI commands use the existing modal, browse, load and held-click guards.

The production color continuation writer now uses the actual preset buttons on both rides before saving and replaying paid visits. Unit and UI tests cover all preset mappings, ride isolation, unchanged nonpaint state, invalid owner rejection, individual edits after applying a preset, and saved reload. Rendered scenes show the actual resulting colors.

All **794 release tests (459 game / 335 simulation)** passed on the runtime source, including both new preset tests and the extended color continuation test. Formatting, strict release Clippy for all targets and the production build passed. The 51,000,320-byte executable has SHA-256 `41ae000c66e3747c4a808d3774f9c8fe6911e35a3d8e1a0570d37b97911769d9`.

A separate writer and verifier process used the release executable with a fresh save directory containing spaces and `é`. The backup preserves Harvest (Sunflower/Meadow/Coral) on the coaster and Seaside (Lagoon/Sky/Cream) on the carousel. The primary additionally preserves a later individual edit on the coaster. Both saved slots exactly match their expected file hashes, and the verifier replayed 3,600 ticks without modifying either slot. Two directly inspected 1024 × 680 captures show the actual coaster and carousel palettes, the three preset buttons, the individual controls and the full panel notice inside the window. Captures establish rendering, not native physical input.

The portable `dist/Parkbound-0.46.0-windows-x64.zip` contains the tested executable, **75 manifest-verified payloads**, 60 Markdown guides, the original starter designs, one bundled MSVC runtime DLL, and license/copyright notices for 291 dependencies plus the default font and Rust standard library. The adjacent `.zip.sha256` is the authoritative archive checksum; repackaging changed guides changes that checksum. A fresh extraction under a path containing spaces and `é` passed every manifest hash and the ZIP sidecar check. Its executable, launched with `C:\Windows` as the working directory, passed two complete independent writer/verifier pairs in separate fresh Unicode save directories. An initial relocation writer exited early after producing partial proof files; it did not write the final expected replay. The subsequent two clean writer/verifier pairs passed. This isolated interruption is recorded in the local receipts; the relocated result is same-PC evidence, not clean-PC acceptance.

[GitHub's Windows CI for the runtime source](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/41433509166) is in progress at this documentation checkpoint. Raw local checks and captures are under ignored `artifacts/ride-palette-presets-0.46.0/`; the package and its checksum live under ignored `dist/` and require OneDrive or separate transfer to another PC.

## Continue building

Next add free-form player ride names with a bounded Unicode policy and strict migration from schema 23. Then continue player-editable scenery and ride-specific/spatial sound. Preserve all three PC paths in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md), prior packages, original content, existing map/save caps and the no-reference-PC policy. Native mouse/keyboard and clean-machine distribution remain separate acceptance checks.
