# Asset provenance

This register covers the 0.4.0 path/crowd implementation and retained overlap/terrain/foundation assets. Current build evidence is recorded in [the crowd report](CROWD_REPORT.md). Add an entry before importing any new graphics, fonts, sounds, music, maps, or data.

| Asset | Source | Rights and distribution treatment |
| --- | --- | --- |
| Landscape tiles, lake and shore | Original geometry and colors in `crates/park_game/src/world.rs` | Project-authored procedural art; no external source files |
| Terraced hill profile, exposed cliffs and contour details | Original integer preset in `crates/park_sim/src/lib.rs` and geometry in `crates/park_game/src/world.rs` | Project-authored data and procedural art; no imported maps or textures |
| Elevated-path and underground-track fixtures | Original 21-object fixture in `crates/park_sim/src/surfaces.rs` and procedural geometry in `crates/park_game/src/world.rs` | Project-authored proof data and art; no imported track models or ride definitions |
| Expanded path network and junction decks | Original 37-node fixture in `crates/park_sim/src/crowds.rs` and connected deck geometry in `crates/park_game/src/world.rs` | Project-authored topology and procedural art; no imported maps |
| Walking/waiting guests and lost markers | Original procedural geometry/colors in `crates/park_game/src/world.rs`, positioned from `park_sim` guest state | Project-authored guest art; no RCT sprites, imported characters, or external textures |
| Underground reference floor and visibility indicators | Original geometry/UI in `crates/park_game/src/world.rs` and `crates/park_game/src/ui.rs` | Project-authored presentation; floor is decorative, not terrain or a usable facility |
| Trees, entrance arch, soil sides and shadows | Original geometry in `crates/park_game/src/world.rs` | Project-authored decoration; no RCT sprites or branding |
| Interface layout and colors | Original implementation in `crates/park_game/src/ui.rs` | Project-authored interface |
| Embedded Fira Mono font subset | Bevy 0.17.3 `bevy_text`, `default_font` feature | SIL Open Font License; include the font's license and attribution in the packaged notices |
| Engine, shaders, and dependencies | Locked Cargo dependency graph | Packaging collects exact package metadata and license/notice texts; reviewed supplementary notices cover packages that omit their license text |

No game audio or external textures are included in version 0.4.0. The crowd proof introduces no external art asset. The working title Parkbound has not undergone a public-name availability review. Project source licensing remains a separate pending decision.
