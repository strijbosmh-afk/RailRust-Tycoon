# Ride palette presets — 0.46.0

This stage starts from the verified 0.45.0 ride-color build on `codex/ride-colors` (`a122c907c41ac19e3d7962bd31c5cfc7d56d7cb5`, draft PR #44). The current branch is `codex/ride-palette-presets`. Parkbound remains an original Rust game intended for free Windows distribution. This development PC is excluded as a performance reference.

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

All **794 release tests (459 game / 335 simulation)** passed on the final code, including both new preset tests and the extended color continuation test. Formatting and strict release Clippy for all targets passed. The production build and package/relocation checks are in progress; record their exact identities and limits below when complete.

## Continue building

Next add free-form player ride names with a bounded Unicode policy and strict migration from schema 23. Then continue player-editable scenery and ride-specific/spatial sound. Preserve all three PC paths in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md), prior packages, original content, existing map/save caps and the no-reference-PC policy. Native mouse/keyboard and clean-machine distribution remain separate acceptance checks.
