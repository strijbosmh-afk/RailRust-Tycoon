# Parkbound release notes

## 0.99.1 — right-click object settings

- Right-click a ride, its track, gates or owned queue to open that ride’s settings.
- Right-click a service building to open its individual price and operating controls.
- Preserve right/middle drag panning and ignore clicks through foreground terrain or UI.
- Cancel pending clicks on camera, park-load and focus changes. Opening settings does not change the park or consume Tips.
- No new dependencies or save-format changes.

Dezelfde versie bevat een Nederlandstalige spelershandleiding met 25
hoofdstukken, zoekfunctie, hoofdstuklinks en afdrukweergave. Open haar via
Help of `docs/HANDLEIDING_NL.html`. De Markdown-bron wordt met een
dependencyvrije generator omgezet; CI controleert links en actualiteit.

## 0.99.0 — organized menus and park advice

- Labelled inset sections and distinct icons across the remaining park menus.
- Main-menu text buttons retained; editable fields and roster entries stay readable.
- New Tips lightbulb beside Rides. Each request returns two local suggestions
  based on park state, with three uses per park.
- Allowance persists through normal saves, primary/backup loads, restart, copied
  parks and construction undo. A genuinely new park starts with three uses.
- Saves migrate to schema/content 52/52. Supported older parks start with zero
  uses spent. Older binaries cannot open new saves.
- No new runtime dependencies. Google Drive remains disabled.

See `docs/PARK_TIPS.md` and the current session report for verification evidence.

## 0.98.1 — structured construction menus

Build tools are grouped into Paths, Landscape and Scenery/park items. Their
colored pictograms distinguish paths, queues, signs, patrol routes and brush
adjustments. Hover and keyboard focus reveal the action and shortcut.
The title, new-park, pause and settings screens use their original text buttons.

## Parkbound 0.98.0 menu theme and borderless startup

Park menus use forest-green panels, cream lettering and gold accents, with
matching hover, selected and disabled states. All menu buttons use compact
antialiased icons with their full action labels on hover or keyboard focus.
Version 0.98.1 restores text navigation to the front screens. New display
preferences start borderless fullscreen. F11 and safe-windowed recovery remain
available. Existing display choices and park saves remain compatible.

## Parkbound 0.97.0 soundtrack and spatial audio

Three original instrumental tracks have fuller arrangements, rhythmic and
melodic variations and crossfaded transitions. Zooming toward rides brings
their actual water, lift, rail and mechanical sounds forward. Nearby visitors
cheer with a retained CC0 recording. Music, rides, ambience and reactions keep
their independent F10 controls. Existing sound settings and park saves remain
compatible.

The owner disabled the project's Google Drive synchronization. Local source
and GitHub publication remain in use.

## Parkbound 0.96.1 game icon

An original red coaster and golden loop on a forest-green badge now identify
Parkbound in Explorer, the game window, taskbar, Start menu, desktop shortcut
and installer. Seven icon sizes cover 16 through 256 pixels. Executable file
properties include the Parkbound name and release version. Source artwork and
reproduction instructions are included. Park saves remain 51/51.

## Parkbound 0.96.0 detailed park graphics

Rides have inlaid platforms, fine balusters, globe lamps, gondola decoration,
painted cups and round coaster wheels. Facilities gain rounded products and
roof signs, glazing, window-box plants and shaped attendants. Flowers now have
stems, leaves and raised petals. Ferns, pine clusters, stone edges and timber
fixings enrich the landscape. Visitor outfits include stripes, vests, bags,
scarves and sun hats. The normal zoom controls support twice the previous close
view. Original geometry is embedded in the game. Park saves remain 51/51.

This continues the unsigned Windows test distribution. The source-license
decision remains deferred. See `docs/PARK_DETAIL_GRAPHICS.md` for validation.

## Parkbound 0.95.0 Windows test release

This release adds a per-user Windows installer, Start menu entries, an optional
desktop shortcut, program removal, package identity metadata and distribution
checks. Setup supports English and Dutch. The portable ZIP remains available.

Close running Parkbound instances before updating or uninstalling. Setup detects
the running game and asks you to close it. Choose the same program folder to
update. Player data is stored separately and survives an update, repair,
reinstallation or uninstall.

The seven-ride Sunhaven Gardens demo, original campaign, sandbox and editors are
included. The preceding graphics updates add detailed ride bodies, stone paths,
timber bridges, terrain materials and varied planting.

Park saves retain schema/content 51/51. Accessibility preferences retain version
1 and sound settings retain version 2. Copy your complete player-data folder with
the game closed when moving computers.

## Test coverage and release status

The installer targets Windows 10/11 x64. The exact tested installation and
upgrade results are in `docs/PHASE_12_DISTRIBUTION_VERIFICATION.md`. Independent
clean-PC and representative-hardware acceptance remain to be completed. This
development PC supplies correctness checks, not performance targets.

The Windows binaries are unsigned. The owner has deferred the open-source license
decision. This is the Windows test channel, version 0.95.0. See `RIGHTS.md` and
`PRIVACY.md` for distribution and data-handling information.
