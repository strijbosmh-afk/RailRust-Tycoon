# Phase 3–5 foundations checkpoint (0.74.0)

This checkpoint advances three linked roadmap phases while keeping their full acceptance gates open.

## Implemented

- **Graphics:** a project visual bible, stable-ID original asset manifest, validation in Windows CI and package creation, and a packaged provenance record. Guest appearance templates expand from 12 to 48 with optional caps. The reference and cached renderers use the same geometry and physical depth.
- **World:** New Park offers empty 64, 128 and 256 tile square sandboxes beside the existing 32-tile starter. Flat or seed 1–3 rolling terrain is generated deterministically before construction. The ordinary park library and single-slot load paths accept validated square 32–256 parks. New and loaded large parks configure camera zoom and pan for their size. The park save schema/content remains 39/39 because dimensions and tile heights were already represented.
- **Coasters:** an isolated closed-coaster test now captures the peak train speed on every installed track piece and displays the three fastest pieces. This telemetry is derived from production movement and is never persisted or applied to guest/ride safety. Existing draft, apply and import/export rules remain authoritative.

## Verification and limits

The asset validator checks unique IDs, existing source paths, redistribution rights and hashes/dimensions for any future authored files. Package verification compares its bundled manifest with the source manifest. The complete release suite passed **518 game + 390 simulation tests (908 total)**. Strict release Clippy, formatting and the Windows release build passed after two lint-only corrections. Tests cover deterministic generation, large-map edit/save/load, 32-tile starter retention, guest cached/reference equality, and piece-level test telemetry against real movement. The executable SHA-256 is `0019a130c0552b8f0f78ea0f84a81b5759a55d1a571718f461477100e51aecdc`.

The New Park screen was inspected at 1024×680; Map, Landscape and Create remain visible, and Enter focuses Create. The ride test at 1920×1080/150% shows all three speed hotspots. At 1024×680/125%, the Rides dialog stays bounded and scrolls to Test. Four 1920×1080/150% captures of the existing busy overview show the coaster, paths, guests, lake and scenery in all rotations without visible depth reversal. A fresh Unicode/space package extraction verified **135 payload hashes and 122 current guides**, then passed front continuation and two diagnostic capture processes. They do not establish native pointer accuracy or full Phase 3 acceptance. Final package receipts are in section 42 of `RESUME_DEVELOPMENT.md`.

This checkpoint does **not** complete the full Phase 3, 4 or 5 gates. Phase 3 still needs a representative busy-park art comparison across four rotations, resolutions and native scales, plus more detailed ride/service silhouettes and night/weather. Phase 4 still needs ownership/construction rights, water and brush tools, consistent cost/refund/undo for all tools, and large-map guest/object/route caps stress. Phase 5 still needs banked/inverted/launch track families with validated transitions, multiple independent coasters and safe block operation, reliability/breakdowns and repair, and a full custom-build-to-import acceptance flow. No representative performance target is inferred from this PC.
