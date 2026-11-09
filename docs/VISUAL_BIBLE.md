# Parkbound visual bible — Phase 3 working contract

Parkbound uses an original, readable isometric style. The supplied colorful park screenshot is a direction for saturation and ride detail, never an asset source. The current game draws its world and characters with project-authored procedural meshes. A later authored texture or sprite must enter through `assets/manifest.json` with a stable ID, source, dimensions, SHA-256, and redistribution rights before packaging.

## Shape and light

- Keep the 2:1 isometric ground diamond and the same depth coordinate for terrain, paths, track, guests and picking. Every new visible part is tested at rotations 0–3 and at ground and raised heights.
- Top surfaces are warm and saturated; left faces are midtone and right faces darker. Thin highlight edges can separate levels, but must remain clipped to the physical face.
- Use a soft, short ground shadow for small buildings and characters. Moving parts use the same physical depth as their parent. Avoid full-screen effects that hide gates, selection, closed status or cost feedback.
- At the normal 32-tile view, faces, ride moving parts, sign shapes and path material marks should be readable. At far zoom or on large maps, keep silhouettes and status colors first; cap decorative geometry per chunk.

## Palette

| Role | Current anchor | Use |
| --- | --- | --- |
| Meadow top | `#4BAB3A` | Grass base, varied subtly per tile. |
| Water top | `#27A7B7` | Blue-green body with pale shore highlight. |
| Sand top | `#E5CA82` | Warm border around water. |
| Soil left/right | `#8F714B` / `#70573A` | Height and cliff direction. |
| Public path | `#E0BD80` | Timber walking surface. |
| Queue path | `#4E91B5` | Distinct blue surface plus broken gold stripe. |
| Warning / selection | gold / orange | Avoid using these as ordinary scenery accents near actionable objects. |

Guest clothing has vivid warm and cool variants, while skin, hair and accessories retain separate values. The optional cap is a clothing detail, not a status marker. Ride paint remains player-selectable; its outline and moving geometry must still read on every terrain material.

## Asset decisions

| Family | Representation | Stable identity |
| --- | --- | --- |
| Terrain, water, cliffs, paths, supports | Procedural mesh and palette | Terrain/object kind and height in saves. |
| Guests and staff | Bounded procedural body templates | Guest/staff IDs and state; appearance is derived. |
| Coaster rail, trains, flat rides and services | Procedural parts with paint variants | Ride/object IDs, kind and saved paint. |
| Future surface art, signs and icons | Authored sprites or hybrid mesh only after validation | New manifest ID; no image path stored in a park save. |

The rendering grid is **64×32 logical pixels per ground tile** and **16 pixels per elevation step**. Future terrain sprites use a 64×64 transparent canvas at 1× with a 64×32 diamond and foot anchor `(32, 48)`; 2× exports are 128×128 with doubled anchors. Character frames use 24×40 at 1×, foot anchor `(12, 36)`, and exact doubled 2× frames. Multi-tile rides may use larger canvases, but must declare their pixel dimensions and a ground-tile anchor in the manifest before an importer accepts them. These are authoring rules for later sprite work; current shipped families remain procedural.

Authored sprites should be transparent PNG with a trim-safe two-pixel margin and consistent premultiplied-alpha handling. Atlases are named by family (`terrain`, `rides`, `services`, `characters`), capped at 2048×2048 pixels, with two extruded pixels between frames and no frame rotation. Use lowercase dotted manifest IDs and kebab-case filenames. Variants keep the same anchor and footprint so changing art cannot move an existing saved object. The validator rejects missing sources, duplicate IDs, path traversal and unrecorded authored hashes. The packaged manifest documents what ships; project source owns procedural geometry.

## Animation and acceptance

Walk and queue motion is sampled from authoritative guest position. Cars and ride parts use the simulation clock; a paused park must not advance them. Decorative effects may animate separately only when they do not imply a ride is operating. Night and weather treatment is reserved for a later simulation-backed state, with status and selection contrast reviewed in both conditions.

Compare the same busy park at 1024×680, HD, QHD and 4K, and all four rotations. Inspect elevated paths against lower terrain, guest feet against the correct deck, track supports, station/gate visibility and close-view service silhouettes. Captures establish layout and depth; native pointer and mixed-DPI checks remain a separate gate.
