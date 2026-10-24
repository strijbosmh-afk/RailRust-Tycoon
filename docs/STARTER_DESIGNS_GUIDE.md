# Starter coaster designs

The portable package includes three original, complete designs in the **designs** folder beside **Parkbound.exe**. They use the normal `.parktrack.json` format and can be imported, previewed, rotated, moved, edited and exported through the existing design library.

| Design | File | Pieces | Footprint | Height at original placement | Fresh track price |
| --- | --- | ---: | --- | --- | ---: |
| Pocket Oval | pocket-oval.parktrack.json | 16 | 5 × 5 tiles | 8 | $270 |
| Meadow Winder | meadow-winder.parktrack.json | 18 | 6 × 5 tiles | 8 | $300 |
| Meadow Climb | meadow-climb.parktrack.json | 24 | 8 × 6 tiles | 8–11 | $420 |

Footprints cover track only. Leave additional room for gates, queues and return paths. Prices are for a fresh build with no retained track or refunds; your live placement quote reflects the actual park. Meadow Climb includes three lift pieces, a two-step drop, a one-step drop and a brake. Layout estimates use the game's approximate motion model.

## Import and build

1. Extract the full package. Open **Designs** from Track tools or use **Ctrl+D**.
2. Choose **Import file**, browse to the package's **designs** folder and select one file. Repeat to import the other designs.
3. Select a thumbnail. Adjust the station's **X**, **Y**, entry height and rotation until the preview is ready. The original station is at tile **9,11**, height **8**; coordinates shown in the game start at 1.
4. Close the coaster and allow its guests to leave. Choose **Load into draft**, inspect the track tools' quote, then **Apply** to construct it. Draft loading is free and undoable; Apply settles the actual purchase/refund.
5. Open **Rides [Y] → Access**. Place the station gates, an owned queue and a public return path. **Prepare suggested public path** selects the next legal tile, including ramps for a return bridge. Move to a clear edge of its ghost and click; reopen Access for the next tile.
6. Run a mechanics test while closed, then choose the car count/loading policy in **Operations** and open the ride. Save the park with **F5** after closing the panel.

The current park supports one coaster and one carousel. Applying a design replaces the current coaster's installed track. Existing paths, facilities, gates or queues may obstruct the proposed layout; move the design or remove the obstruction through normal construction tools. A moved station may need new gates and paths. The preview and Apply share the same placement rules.

Flat closed loops can enclose their exit landing. A return path may need two ramps before crossing over low track, then ramps back down to the public network. The Access helper has been tested to complete a legal return route for each starter from its original position in the demonstration terrain. Further terrain or player construction can require manual changes.

Design files store geometry. Set the receiving coaster's car count and operating settings in Operations. Importing preserves the source file and adds a new library entry; duplicate names receive a numbered suffix. Your own edits can be saved under a new name or exported separately.
