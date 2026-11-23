# Park planting, shadows and motion

- Project `railrust-tycoon`
- Session `park-atmosphere-2026-11-23` on `MICHIEL-BUILD`, 23 November 2026 UTC
- Branch `codex/right-click-settings`, baseline `9e98fc6321934822ac783c2bbe72b1571a890d08`

Low shrubs grow in cosmetic clusters on open lawns. Trees, raised paths and
stations add terrain-clipped contact shadows. Ride entrances have pennants,
the water has wider glints, and visitors waiting at rides or shops subtly
lean. Reduced motion stops the animated effects. Visitor routing, scenery
objects, finances and save schema are unchanged. The existing Windows and
Rust build requirements apply.

Use the Sunhaven Gardens and landscape captures to review the visuals at
overview and ride zoom. World tests cover scene depth, clipping and the
retained visitor renderer against its reference implementation.
