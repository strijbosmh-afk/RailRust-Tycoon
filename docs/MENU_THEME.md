# Parkbound 0.98.0 menu presentation

## Shared park colors

The park interface uses forest-green panels, deeper inset surfaces, warm cream
text and gold accents. Construction, ride controls, navigation, visitors,
finances, facilities, track designs, settings and front-screen dialogs use
the shared palette in `crates/park_game/src/ui/theme.rs`.

Hover and disabled colors follow the same palette. Selected gold buttons use
dark lettering. Other buttons use light lettering. The existing high-contrast
mode applies after the normal theme. Original status colors retain their
meaning with brighter shades suited to the dark panels.

## Menu button

The top-right park-menu button uses three scalable bars. Its **Park menu**
label appears when the pointer is over it or the keyboard focus reaches it.
The button retains its existing action and provides an accessible name.
The pictogram is generated with native UI nodes and needs no downloaded font
glyph or bitmap.

## Startup display

New display settings use borderless fullscreen on the current monitor. The
initial Windows window is created in that mode, with native decorations off.
The Settings screen continues to support explicit Windowed, Borderless and
Fullscreen choices. Previously saved choices remain valid.

**F11** restores a decorated window with automatic interface scale.
`--safe-windowed` starts a temporary windowed session. Deterministic captures
and diagnostics use their existing windowed dimensions. Display preferences
remain version 1 and park saves remain 51/51.

## Verification

Theme contrast, nested button labels, the hover/focus label and display startup
and recovery have focused checks. Runtime screenshots and input checks,
complete tests and distribution results are recorded in the session report.

The source remains in the shared checkout
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Google Drive synchronization remains disabled by the owner. Use GitHub and
the local resume guide for continuing development. This PC supplies
correctness checks and visual review.
