# Individual ride and service settings — 0.99.1

Right-click a visible attraction or service building and release without dragging.
Its existing settings window opens with that exact owner selected. Coaster track,
ride entrances/exits and owned queue paths lead to their ride's settings too.

For rides, the window starts on **Operating settings**. Use its tabs for operation,
station/path access, testing and colors. The selected name is shown at the top.
Mechanical changes retain their existing close/stop/empty-ride requirements.
Ticket pricing remains in the selected ride inspector: close the window and use
its price buttons or F6/F7. For services, use the selected facility's price and
open/close controls. Hover over an icon to read its action and shortcut.

A short click opens settings on release. Right-drag still pans the map; middle-drag
also pans. A movement beyond four logical pixels counts as a drag. Small pointer
jitter does not pan the park. Empty ground and unowned scenery do not open a
settings window. The visible foreground surface wins, so a hidden ride is never
selected through terrain or another object.

Opening these windows does not itself change park money, objects or tip
allowance. The simulation keeps its selected speed; Space pauses it. Existing modals and read-only captures block the shortcut. Losing
focus/the cursor, starting another pointer action, changing the camera or loading
a park during the gesture cancels the pending click. The gesture is transient
and is not saved. Park saves remain schema/content 52/52.

Implementation uses the existing `world::pick_surface` depth rules, with a small
click/drag tracker in `crates/park_game/src/object_settings.rs`. Regression tests
in `crates/park_game/src/object_settings_input_tests.rs` exercise owner selection,
coaster track, owned queues, all four rotations, DPI/UI scaling, dragging,
read-only modes, input cancellation and exact save-byte preservation.

The source remains in the shared OneDrive checkout:
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Continue from `codex/right-click-settings`, following `RESUME_DEVELOPMENT.md` and
the current session report. Google Drive synchronization remains disabled.
