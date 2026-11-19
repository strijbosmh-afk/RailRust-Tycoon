# Parkbound 0.99.0 — park advice and organized menus

## Using Tips

The third shortcut next to Finance and Rides in the park header is a gold lightbulb.
Hover or keyboard focus shows **Tips** and the remaining allowance. Each accepted
click opens two suggestions based on the park at that moment. Close the window
with its close icon or Escape.

Each park has **three uses in total**, not three per session. An exhausted button
is dimmed. Saving, closing the application, loading the park or its previous
backup does not replenish uses. A genuinely new park starts with three. Saving
a used park as another copy retains its spent allowance.

The advice checks connected arrivals, lost guests, ride operating problems,
hunger/thirst/toilet/rest needs, guest satisfaction, cash and recorded operating
results, ticket value refusals and loan interest. It prioritizes immediate
problems and returns two distinct suggestions. Prices, counts and availability
come from the current park. Suggestions are local and deterministic, do not
change prices or build anything, and do not guarantee a particular profit.

The first use in an unsaved park creates a normal park-library slot. The use is
written to its metadata before advice is displayed. Continue to save normally
to retain later building and simulation progress. If storage is unavailable,
no new advice is provided and the error appears in the advice window.

## Durable allowance

`Park::tips_used` is bounded to 0–3 in save schema/content **52/52**. Supported older
saves default to zero. The library's `slot.json` retains the highest spent value
for that slot, under the same lock and atomic replacement used for metadata
writes. Loading the primary or backup merges this value. Renaming and saving
cannot lower it. Construction undo/redo also preserves the current count.

The record is local player data, not an account entitlement. Manually editing or
replacing all player files is outside its persistence guarantee. New versions
read supported older parks; older binaries cannot read version-52 saves.

Implementation is in `crates/park_sim/src/advice.rs`,
`crates/park_game/src/park_library.rs` and `crates/park_game/src/ui/tips.rs`.

## Menu structure

The shared forest, cream and gold palette now has consistent inset groups and
visible section headings across navigation, rides, services, finance, visitors,
park staff, management, scenario editing, audio, Help and track construction.
Related controls stay together. Scroll the panel if the screen is small.

Staff hiring uses separate symbols for mechanics, handymen, security, entertainers
and attendants. Pictograms distinguish food, drinks, toilets, ice cream, rest, gifts, information,
first aid, staff, supplies, advertising and policy. Action buttons retain live
hover/focus descriptions and shortcuts. Editable name/search fields, roster rows
and design previews remain readable. The main menu, new-park, pause and settings
screens retain their original text buttons. Borderless startup and F11 recovery
are unchanged.

## Verification and continuing development

Run `cargo fmt --all -- --check`, strict release Clippy and workspace tests.
The following commands exercise the actual Windows save system in separate
processes, using a new absolute directory each time:

```powershell
& ./target/release/parkbound.exe --save-check write-tips --save-dir 'C:/Temp/Parkbound tips proof'
& ./target/release/parkbound.exe --save-check verify-tips --save-dir 'C:/Temp/Parkbound tips proof'
```

The writer spends three uses and saves an independent new park plus a copy.
The verifier checks primary/backup continuation, independent allowances,
copy preservation and refusal of a fourth use without changing stored files.
The Windows workflow includes this pair. `--capture-panel tips` previews the
layout without consuming an allowance; capture mode blocks interaction.

See the current session report for actual test/build outcomes and release
identities. The current PC supplies correctness checks and visual review only.

Source is in the shared OneDrive checkout:
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Resume from `codex/park-advice-and-menus` and read `RESUME_DEVELOPMENT.md`.
Google Drive synchronization remains disabled. Preserve existing local exports.
