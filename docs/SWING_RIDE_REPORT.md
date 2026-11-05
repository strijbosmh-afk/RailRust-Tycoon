# Skyward Swing stage report — 0.61.0

## Scope

This stage introduces an original swinging flat ride beside the existing carousel. Construction, clearance, entrance/queue/exit connectivity, boarding, six paid seats, guest choice, ride-specific income and upkeep, maintenance, reviews, paint, rename, demolition and save/load all use the established ride contracts. The motion and original procedural A-frame/gondola geometry differ from the carousel. No external asset or dependency is introduced.

The ride costs $350 to build and $0.80 per 900 operating ticks; the existing half-cost demolition rule returns $175. Its appeal challenge is 55. Coaster-preferring guests can treat it as a partial match; carousel preference still favors the carousel. Swing passengers get a larger nausea increase than carousel passengers. One active ride of each flat-ride kind is allowed.

Strict schema/content advances from 30/30 to **31/31**. Older saves have their historical flat-ride records migrated to Carousel. Current records require a kind; old records reject a smuggled kind. The serializer, validator, finance rows and construction history agree on that kind. Existing save slots are loaded without rewriting their bytes until the player saves.

## Verification

The unit suite covers simultaneous construction and refund accounting, exact old-save migration/new-save refusal, swing seat bounds and deterministic mid-cycle replay. A production `write-swing` / `verify-swing` pair creates two paid rides with a naturally running swing, loads the snapshot in a fresh process, replays 3,600 ticks, and checks that the primary slot was unchanged. The Windows workflow runs this pair. `swing-map` and `swing-rides` capture scenes exercise world rendering and management layout.

Final local source checks passed **843 release tests (481 game / 362 simulation)**, `cargo fmt --all -- --check`, strict release all-target Clippy, and the Windows release build. The executable SHA-256 is **`700ba07edf57972d164f948f41f2167137cc17a1726c7e2baca82cb0e782355a`**. The first extracted package verified **106 payload hashes and 94 current guides**. It passed eight independent production processes: Swing, ride choice, paid handyman/bins and wellness writer/verifier pairs. Swing saved two paid rides with one mid-cycle, resumed exactly 3,600 ticks and did not rewrite the source primary slot.

The packaged 1024×680 `swing-map` and `swing-rides` captures passed and were inspected. The world view shows both rides, separate paths and the colored swing; the management view shows both build choices and selected Swing operations without clipping. An additional 350% rotation-1 detail capture from the extracted executable shows the six-seat gondola and support frame on its own footprint. Capture SHA-256 values are `b64afaeb570049d9335c685ea9bbc3299998c3015c4b26901aedb90a20710fd0`, `27ce7e8118e6c262b4cf6cbf1b8fffee5b24482851468586b5c237d55ab50a67`, and `8454a64bc1dbbd74ff6c444d6084a6c483712b9ee45bc0bffd30f34167f110f1`. The final archive is refreshed after documentation changes; its adjacent `.zip.sha256` is authoritative for the local ZIP.

This PC is excluded as a performance reference. Physical input, a clean target Windows PC and the broader classic ride catalogue still need acceptance.
