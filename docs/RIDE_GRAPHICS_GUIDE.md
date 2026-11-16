# Detailed ride graphics — 0.93.0

The classic isometric art now gives the attractions more physical detail in the playable park. No extra graphics pack is required.

- Festival Carousel has sculpted horses with separate hooves, ears, manes and saddles, brass poles, canopy ribs and a decorated central column.
- Meadow Wheel has tubular A-frame supports, twin wheel rims and braces, and eight bevelled open gondolas with shaped roofs. The occupied gondola follows the saved passenger positions.
- Whirling Cups has rounded, open porcelain bowls, recessed interiors, saucers, brass rims and handles that turn with the seats. The rotating pairs and passengers still use the game's simulation.
- Skyline Tower has segmented framing and a rounded glass observation cabin with window mullions and a shaped roof.
- Skyward Swing and Beacon Drop have additional bracing, tower guides and crown details.
- Corsair Ship has a tapered wooden hull, six bench rows, brass rails, side fittings and raised bows. The suspension connects to fixed world-space pivots.
- Coaster cars have divided seating, individual restraints, contrasting lower bodies and pinstripes.
- Water channels have rim fittings and small foam streaks. Rapids rafts have rounded rubber bumpers and a raised inner seating ring.
- Park Railway has sleepers, a faceted locomotive boiler, brass bands and carriage lining. Vista Monorail has additional cab and window details. Catalogue track stations have slim platform canopies, while coaster track has cross ties.

The appearance controls in **Rides → Colors** continue to work. Select **Play Sunhaven Gardens** to see the improved seven-ride park. The rest of the models are in the existing construction catalogue.

## Development and portability

Shared local Git root: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.

Branch `codex/ride-graphics-overhaul` builds on `69044094a68bad6d1f00cd07db4bcac1352efafa`. New geometry is original Rust code, with no additional dependencies or downloaded art. Saves remain schema/content 51/51.

`world/ride_detail.rs` contains reusable unprojected surfaces, rounded profiles, bevelled shells and tubes. Flat rides transform these surfaces into the existing isometric depth and picking system.  
`catalogue_render.rs`, `carousel_render.rs` and `train_render.rs` own the specific models. Static detail remains in the existing cached chunks. Moving parts use current seat/train positions.

For close-up verification, pass `--capture-scene catalogue --capture-ride FerrisWheel` (or another exact RideModel name) with the existing absolute `--capture-frame` option. `--capture-rotation 0` through `3` tests camera directions. This is a diagnostic option, not a player requirement.

Run formatting, strict release Clippy, relevant rendering tests, all workspace tests, a release build, the demo and catalogue writer/verifier pairs, and extracted Windows package verification. Inspect the actual captures. This development PC is excluded from performance acceptance.
