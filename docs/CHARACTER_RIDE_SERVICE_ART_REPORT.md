# Character, ride and service art — 0.54.0

This stage improves close-view readability in the original mesh renderer. Visitors gain shaded shirt sides, shoe soles, bag straps, eyes and mouths. Both the retained renderer and the independent full-mesh oracle emit matching geometry and depth. The retained renderer now derives its reservation bound once from the largest cached body and marker templates plus the two clipped deck shadows, rather than growing a fixed per-guest estimate. These are presentation changes derived from stable guest IDs; simulation and saves do not gain appearance state.

The coaster's physical car faces gain side panels, bright trim and paired leading lamps within the track envelope. The carousel platform rises from 24 to 48 radial deck segments and gains a brass rim; moving mounts gain tail, mane and bridle geometry. Static, moving, selection and construction-preview paths keep their current source geometry relationships, and ride repainting retains the physical mesh.

Food, Drink, Toilet and Rest facilities each gain small distinct meshes: a menu, dispenser taps/straws, a door plaque/vents, and bench slats/hanging light. Every new vertex stays within the one-tile footprint and the established three-step clearance. Raised foundations, lower crossings and all four camera rotations continue through the existing face pipeline. No imported asset, new dependency, RNG draw, park save field or change to strict schema/content **25/25** is introduced. The development PC remains excluded as a performance reference.

## Verification scope

Release tests compare retained and independent guest rendering, bound geometry in large and normal maps, inspect dynamic car and carousel positions, verify repainting leaves geometry unchanged, and check all service types' picking/preview/selection envelopes. Review close visitor, train, carousel, Food/Drink/Toilet and raised Rest captures at 1024 × 680, including a rotated view. The final test, package, save continuation and GitHub identities are recorded in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md).
