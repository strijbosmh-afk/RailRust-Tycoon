# Classic isometric graphics overhaul

Version 0.92.0 implements the selected classic isometric direction. The title screen uses original illustrated park artwork, forest-green panels, gold borders, a prominent Sunhaven Gardens button and live keyboard focus. Gloock headings and Barlow interface text are embedded with their SIL Open Font License files. The opening background preserves its aspect ratio while filling the window.

The playable park has detailed oak foliage, richer planted flowers, water glints and shore edges on maps through 64×64, decorative station platform canopies, expanded shop façades and attendants, and new ride trim. Carousel poles and lights, tea-cup handles and rims, Ferris wheel cabin roofs and supports, pirate-ship rails and bows, railway wheels and flume planks add detail to the existing working rides. Visitors gain collar, pocket, sleeve, hair and accessory details through the cached character renderer.

## Playing

Launch `Parkbound.exe` and choose **Play Sunhaven Gardens**. Every menu action uses the real game command. Arrow keys or Tab change the selection, Enter activates it, and Escape returns. The intro advances to the title after its existing short delay. The title menu retains campaign, scenario editor, save library, settings and credits.

The normal camera rotation, zoom, surface selection and build previews remain available. Help contains controls and accessibility settings. All additions retain the existing paid construction and visitor simulation. Save schema/content remains 51/51.

## Assets and distribution

`assets/art/title-gardens.png` and `assets/art/oak.png` are original AI-generated artwork made with the built-in image generator for this project. The former is opening art; the latter is a transparent scenery texture. `assets/manifest.json` records their actual dimensions and SHA256 hashes. Runtime embeds both assets, so they load when launched outside the checkout or offline. The Windows package also includes the source artwork, fonts and licenses.

The Barlow and Gloock font files come from the Google Fonts repository. Their OFL notices are distributed beside them and included in third-party notices. The dependency lockfile is retained.

## Development handoff

Branch `codex/classic-graphics-overhaul` is based on `2b0df01b22013e4a913a2f363b9963256efefcc7`, the playable Sunhaven Gardens delivery. The shared checkout is `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.

`visual_style.rs` owns embedded images/fonts, title button feedback, background fit and the retained alpha-tested foliage mesh. `front_end.rs` supplies the actual opening/menu commands. World rendering remains in `world.rs` and its ride, service, guest and atmosphere modules. The foliage is rebuilt on map changes and camera rotation, and follows the same lighting tint as the world. It is hidden in the underground view.

Run formatting, strict release Clippy, the workspace tests and the release build. The `demo-park` capture has an optional `--capture-detail` view for comparing close artwork. Inspect the title at 1024×680 and 2560×1440, and the demo at all four rotations. Run the existing demo save/load proof and the extracted-package verification script. This PC is excluded from performance acceptance.

## Original image prompts

The title prompt specified a text-free original isometric park, saturated emerald foliage, red and gold coaster, Ferris wheel, carousel, striped shops, teal lake and a darker forest area on the left for live controls. It requested detailed materials and grounded depth, rendered through the built-in image tool.

The oak prompt specified a single transparent tree sprite, orthographic isometric angle, dense emerald leaves, warm light from the upper left, a brown branching trunk and visible roots, with no ground patch, text or interface. The generated image's alpha channel is retained.
