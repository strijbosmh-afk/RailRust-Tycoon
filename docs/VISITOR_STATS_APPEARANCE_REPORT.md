# Guest statistics and visitor appearance — 0.48.0

This stage follows the 0.47.0 ride names and scenery branch. The user-facing goal is to make visitor condition legible in **Guests [L]** and make visitors easier to distinguish on the map. This development PC remains excluded as a performance reference.

Runtime source commit: `2f54b32c2227d7a9763f4e52a435e34899a61d35` on `codex/visitor-stats-appearance`, stacked in [draft PR #47](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/47). Subsequent documentation-only commits preserve the executable identity below.

## Implementation

The guest list keeps stable-ID ordering and eight fixed rows per page. Each occupied row shows satisfaction, activity and a shirt swatch. The detail panel replaces the dense undifferentiated text block with four numeric, colored meters and grouped activity, finances, preferences, thought and selected-ride choice. Meter width and warning color are computed from authoritative guest fields at refresh; no simulation state is changed by opening or browsing the panel. Exact numbers and words remain available without color.

Twelve authored looks are derived from `GuestId` modulo 12. The persistent guest renderer retains templates for poses and looks, while the independent full-mesh oracle paints the same varied shirt, skin, hair and bag geometry. Lost visitors retain their orange warning shirt. Looks add no RNG calls, serialized fields, asset files or dependencies. Strict save schema/content remains **24/24**.

## Verification and limits

The final release workspace suite passed **802 tests (463 game, 339 simulation)** with no failures. `cargo fmt --all -- --check`, strict release all-target workspace Clippy and the Windows release build passed. The final `parkbound.exe` SHA-256 is **`71d7ea56e6f909b27bef38b13525f2d9ac595ccf0484a57669a111c5970f264b`**. The production `write-visitors`/`verify-visitors` pair passed against that binary from a fresh Unicode/space-containing save directory, preserving exact primary/backup bytes and a 3,600-tick restart continuation. The existing CI workflow also runs that pair.

Three final **1024 × 680** in-engine captures were visually inspected: `feedback-guests-final.png` shows the stat layout and matching shirt swatches, `visitor-looks-final.png` shows the larger map figures, and `lost-final.png` checks the orange lost-visitor state. Raw logs and captures are under ignored `artifacts/visitor-stats-appearance-0.48.0/`. Rendering tests compare the retained renderer and full-mesh oracle across movement, camera rotations and view modes.

The portable Windows package passed **79 payload hashes and 67 current Markdown guides** with no extra archive entries. Its extracted executable was byte-identical to the final build. From a fresh space/é output folder and unrelated working directory, the two-process visitor continuation and all three capture scenes passed. The extracted guest-panel and close-up map captures were visually inspected. The adjacent `dist/Parkbound-0.48.0-windows-x64.zip.sha256` is the final archive identity and remains outside the ZIP. Same-PC relocation does not prove clean-machine compatibility. Captures establish layout and drawn output, while native pointer/keyboard interaction and representative-hardware performance remain separate acceptance work.

The meters expose the existing three needs; energy, nausea, staff assistance and more detailed guest appearance controls are future gameplay/presentation stages. The small map figures remain decorations over path targets, and no new map guest-picking behavior is claimed. Continue the broader [gameplay checklist](GAMEPLAY_CHECKLIST.md).
