# Organized menus and durable advice session

Project `railrust-tycoon`, session `01a15441-13d3-7142-b742-e896afad208c`,
PC `PC1830`, 2026-11-19 UTC. Base commit
`72f272cdc51a4327abe187905ce9e09a1da77415`; work branch
`codex/park-advice-and-menus`.

The owner requested the same clear organization for remaining menus, improved
icons, preservation of main-menu text, and a third Tips shortcut next to Rides.
Each accepted request must give two current-state profit suggestions, with a
three-use allowance that survives saving and restarting.

Version 0.99.0 applies shared inset groups to navigation, rides, finance,
facilities, visitor flow, staffing, management, scenario tools, Help, audio and
track tools. Data-entry fields, roster rows and design previews stay readable.
Original pictograms distinguish facility types and management actions.

Advice is computed locally from park data. Save schema/content 52/52 stores the
bounded counter; library metadata stores its high-water value. A request commits
metadata before displaying advice. Primary and backup loading, copies, renaming,
saving and construction undo preserve usage. New parks begin with zero spent.
An unsaved park gets a library slot on its first successful request. A storage
failure withholds advice. Details and commands are in `docs/PARK_TIPS.md`.

## Verification

- Formatting, strict release Clippy and all 1,057 workspace tests passed
  (591 application tests, 466 simulation tests).
- Tests cover real header clicks, exactly two tips, storage refusal, three-use
  persistence, rejection of a fourth use, old-save migration and undo/redo.
- The first full run exposed a hidden scenario-section title lookup outside its
  valid range and two icons without antialiased outer edges. Both were corrected;
  the complete suite then passed.
- The final icon-to-action refinements also passed all 35 UI tests.
- All 25 original asset families passed their source and redistribution records.
- Windows build, screen captures, extracted-package continuation and installation
  verification are still in progress at this checkpoint.

## Storage and resume

Source root: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Project identity and GitHub ownership were checked. Google Drive synchronization
remains disabled; local standard prompt 2.1.0 was used. Existing local exports are
retained. No Drive upload was attempted. This PC is not a performance reference.

Continue from the latest published commit on `codex/park-advice-and-menus` and
read `RESUME_DEVELOPMENT.md` before starting the next phase. Follow the existing
Windows setup in `INSTALLATIE.md` and use the full player-data directory when
moving parks to another PC.
