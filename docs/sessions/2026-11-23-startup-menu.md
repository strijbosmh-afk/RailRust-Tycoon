# Stable startup menu

- Project `railrust-tycoon`
- Session `startup-menu-2026-11-23` on `MICHIEL-BUILD`, 23 November 2026 UTC
- Branch `codex/right-click-settings`, baseline `a57ad723ff29ccd3a75336f49ca8b432a00f2700`

Removed the obsolete periodic intro rebuild and applied cover sizing after
menu creation, before UI layout. The startup regression verifies the intro
entity remains stable and the first title frame is fitted in a 1024×680
window. Spell out the navigation keys because the embedded body font
does not display the arrow glyphs. Use the intro and title captures for visual comparison. Existing
Windows build requirements apply.
