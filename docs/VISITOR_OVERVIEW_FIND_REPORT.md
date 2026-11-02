# Visitor overview and location — 0.49.0

This stage follows 0.48.0 visitor stats and appearance. It adds two player-facing steps: a park-wide condition snapshot and a way to locate one selected visitor. This PC is excluded as a performance reference.

## Park stats

The guest modal now has Guest list, Park stats and Visitor flow views. Park stats derives counts from currently active, authoritative guests each refresh. The satisfaction bands are 0–39, 40–69 and 70–100. The need bars count hunger/thirst/toilet values at least 70; their totals can overlap. The activity line reports riding, queueing, service visits and lost guests as separate live counts. Bars show exact text values and are safe for an empty park. This is a snapshot, not a time series; no new simulation or save fields are introduced.

## Find guest

The selected guest's Find control resolves their current authoritative `Park::guest_position`, including a ride seat where applicable. It closes the guest window, zooms and pans the camera, and draws a gold halo from that position in the dynamic selection layer. The halo follows movement and camera rotation and disappears after the guest departs. A stale or missing identity leaves the camera and modal unchanged. It does not alter path picking or simulation state. Strict save schema/content remains **24/24**.

## Verification and limits

The release workspace suite passed **805 tests (466 game, 339 simulation)**. Strict formatting, release all-target Clippy with warnings denied and the Windows release build passed. Final executable SHA-256: **`40e85ec935d255db04241e2611099b9bc2c971e4b82770c5f4a0e9cb8c0b82d5`**. The production `write-visitors`/`verify-visitors` pair passed from a new Unicode/space save folder. Its receipt confirms real visitor controls, money/ledger reconciliation, a **3,600-tick exact restart continuation** and unchanged primary/backup bytes. The final `guest-stats`, `guest-found` and regression `feedback-guests` captures were inspected at **1024 × 680**. The portable Windows ZIP passed **81 payload hashes and 69 current guides**. Its extracted executable passed the visitor proof pair and both new capture scenes from an isolated Unicode/space folder; extracted screenshots were inspected. The adjacent `.zip.sha256` identifies the final archive. Raw receipts are kept under ignored `artifacts/visitor-overview-find-0.49.0/`. Native pointer gestures, long-lived trend history and clean-machine acceptance remain separate. No timings from this development PC are used as product performance targets.
