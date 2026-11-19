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

## Runtime checks

A separate `artifacts/menu-live-profile` directory was used for the native
Windows checks. Its sound was muted and its display settings initially absent.
The game filled the 1920 by 1080 display without a native title bar. The top-right
icon showed Park menu on hover and opened the pause menu when clicked. Settings
reported Borderless. F11 restored the native title bar and reported Windowed.
The test process was closed with Alt+F4. Player saves were outside this profile.

Seven engine captures covered construction navigation, management, guests,
settings, new park, the title screen and high contrast. Sizes included
1024 by 680, 1920 by 1080 and 3840 by 2160. Normal panels, active-button text,
dialog text and settings were visually inspected. Capture files live under
`artifacts/menu-*.png`. A high-contrast outline initially counted as keyboard
focus for the tooltip. Keyboard focus now has an explicit component, with
regression coverage. Captures also suppress live hover feedback and tooltips
to keep their output reproducible.

The asset catalogue validates all 25 families, and all seven embedded icon
sizes match the ICO. Full final checks and installer identity are recorded
below once complete.

Shared source path
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
This PC is excluded from performance acceptance.
