# Parkbound 0.99.0 menu presentation

## Shared park colors

The park interface uses forest-green panels, deeper inset surfaces, warm cream
text and gold accents. Construction, ride controls, navigation, visitors,
finances, facilities, track designs, settings and front-screen dialogs use
the shared palette in `crates/park_game/src/ui/theme.rs`.

Hover and disabled colors follow the same palette. Selected gold buttons use
dark lettering. Other buttons use light lettering. The existing high-contrast
mode applies after the normal theme. Original status colors retain their
meaning with brighter shades suited to the dark panels.

## Icon buttons

In-park buttons use compact pictograms, including Path, Find, Gate, Exit,
construction tools and ride controls. The title, new-park, pause and settings
screens retain their original text buttons and dimensions.
The full action label and its existing shortcut appear on hover or keyboard
focus. Dynamic labels continue to update, including open/close states and
selected entries. Adjustment buttons identify what they change in their tooltip.

The Build panel groups paths, landscape tools, and scenery/park items in labelled
inset panels. Path height belongs to Paths. Brush size belongs to Landscape.
The top row opens the ride catalogue, services or demolition. Each group wraps
its controls within the available width and participates in the existing
vertical scroll area on smaller windows.

The original text entities and action handlers remain attached to their buttons.
The icon presentation hides their text nodes, retains accessible button names
and adds original antialiased artwork generated in Rust. Construction icons use
filled shapes and park colors. Signs, patrol routes and brush adjustments have
distinct symbols. Selected and disabled buttons use a monochrome variant for
contrast, as does high-contrast mode. The shared tooltip stays inside
the viewport, including near the lower and right edges.

`crates/park_game/src/ui/button_icons.rs` owns presentation and tooltip behavior.
`crates/park_game/src/ui/pictograms.rs` owns the artwork and action mapping.
`crates/park_game/src/ui/build_tools.rs` owns the construction groups.

## Organized management and advice

The remaining game panels now use labelled inset sections and compact action
rows. Food, drink, toilets, ice cream, gifts, information, first aid, staff,
supplies, advertising and policy have distinct artwork. Editable fields and
selectable roster/design rows retain their readable contents.

The third header shortcut is Tips, a lightbulb with a live remaining-use tooltip.
It shows two state-based suggestions and is limited to three uses per park,
including after save/load, backup restoration and restart. Read
[the advice guide](PARK_TIPS.md) for the persistence rules and verification.

## Startup display

New display settings use borderless fullscreen on the current monitor. The
initial Windows window is created in that mode, with native decorations off.
The Settings screen continues to support explicit Windowed, Borderless and
Fullscreen choices. Previously saved choices remain valid.

**F11** restores a decorated window with automatic interface scale.
`--safe-windowed` starts a temporary windowed session. Deterministic captures
and diagnostics use their existing windowed dimensions. Display preferences
remain version 1. Park saves are now 52/52 for the three-use Tips allowance.

## Verification

Theme contrast, nested button labels, icon generation, live hover/focus labels and display startup
and recovery have focused checks. Runtime screenshots and input checks,
complete tests and distribution results are recorded in the session report.

The source remains in the shared checkout
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Google Drive synchronization remains disabled by the owner. Use GitHub and
the local resume guide for continuing development. This PC supplies
correctness checks and visual review.
