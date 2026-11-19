# Menu theme and fullscreen session

Project `railrust-tycoon`, session `01a15441-13d3-7142-b742-e896afad208c`,
PC `PC1830`, UTC date 2026-11-19. Branch `codex/themed-menus-borderless`
starts at `4933b0ec75fa33c4d62c1a251a2803fb61674213`.

The user requested game-themed menu colors, an icon on the menu button with
text on hover, and borderless fullscreen startup. The implementation uses a
shared forest/cream/gold palette and a native three-bar icon on the top-right
park-menu button. Hover and keyboard focus show the Park menu label.

New display preferences default to borderless fullscreen. The initial window
uses that mode and removes its native decorations. Existing explicit display
choices, F11 recovery and safe-windowed startup remain supported. Captures keep
their deterministic windowed layout. Saves remain 51/51 and display settings
remain version 1.

The registered root and GitHub repository were checked. Google Drive remains
disabled. The cached standard prompt is 2.1.0 and its cloud copy was not read.
Build, visual/input checks, full tests and distribution verification are in
progress. See `docs/MENU_THEME.md` for behavior and source locations.

Shared source path
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
This PC is excluded from performance acceptance.
