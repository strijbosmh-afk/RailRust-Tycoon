# Phase 6 — ride and facility catalogue

The catalogue was completed in 0.85.0. Current development version 0.85.1 adds automatic coaster height and heading continuation, documented in the [construction guide](PHASES_3_5_FOUNDATIONS_GUIDE.md). Park schema/content remains 48/48.

The agreed scope is 14 rides across five families and eight facilities. The catalogue is available through **Build → Catalogue**, the Rides window and the Services window. Choose a category and an entry, position the preview, use **C** to rotate and click to build. Height controls apply to the entire prefab. Construction validates the complete footprint and charges only after the whole operation succeeds.

## Rides

| Family | Entry | Seats per cycle or train | Layout and operation |
| --- | --- | --- | --- |
| Gentle | Festival Carousel | 6 | Rotating horses under the festival canopy |
| Gentle | Meadow Wheel | 6 | Upright observation gondola on a tall wheel |
| Gentle | Whirling Cups | 12 | Six two-seat spinning cups |
| Gentle | Skyline Tower | 8 | Slow observation lift, viewing pause and descent |
| Thrill | Skyward Swing | 6 | Pendulum gondola |
| Thrill | Beacon Drop | 6 | Lift, hold, drop and braking return |
| Thrill | Corsair Ship | 12 | Swinging timber boat and support frames |
| Water | Timber Falls | 4 | Log boat, chain lift and splashdown channel |
| Water | River Rapids | 6 | Level water channel and passenger raft |
| Transport | Park Railway | 12 | Steam engine and coaches serving two stations |
| Transport | Vista Monorail | 12 | Beam-supported coaches serving two stations |
| Coaster | Classic Coaster | 6 | Timber-supported track and configurable trains |
| Coaster | Steel Launch Coaster | 6 | Steel track, launch, banked curve and heartline roll |
| Coaster | Inverted Coaster | 6 | Overhead track and visible suspended-seat hangers |

The listed tracked capacity is the supplied train. Water rides use one vehicle per train. Coasters support one to three cars and one to three trains when block spacing permits. Water rides support up to three block-controlled boats or rafts. Transport uses a single train of one to three vehicles. The existing ride operations window changes supported settings.

Fixed footprints range from 3×3 to 5×5. Tracked prefabs occupy 10×7 tiles before rotation. New fixed rides reserve their complete height and footprint. Tracked construction uses the existing segment envelopes and overlap checks. Water and transport families restrict the track pieces their vehicle can use.

## Access and operation

1. Connect public paths to the park entrance.
2. Add an entrance facing the ride and a queue leading to public paths.
3. Add an exit facing the ride, with an outward public path connected to the park network.
4. Open the ride in its Operations window and set a ticket price.

Round-trip prefabs have a three-tile station. Both gates can face the outer edge at different station tiles, allowing flat ground access. For railway and monorail, put the entrance at the first station and the exit at the far station. Passengers unload at that destination. The empty train then returns to the entrance station. A close request finishes the current journey and returns the train home.

Rides retain separate stable identities, names, paint, queues, fares, ratings, maintenance counters, costs and receipts. Multiple examples of a catalogue model can coexist. The existing one-per-kind helper functions remain available to historical fixtures.

## Facilities

| Facility | Build price | Default visit price | Effect |
| --- | --- | --- | --- |
| Food | $120 | $2 | Reduces hunger |
| Drinks | $80 | $1 | Reduces thirst |
| Toilets | $100 | Free | Relieves bathroom need |
| Rest | $140 | Free | Restores energy and eases nausea |
| Ice cream | $110 | $1.50 | Reduces hunger and thirst together |
| Souvenirs | $160 | $4 | Satisfies the desire for a souvenir |
| Information | $90 | Free | Satisfies the need for park information |
| First aid | $180 | Free | Treats nausea and restores energy |

All facilities occupy one tile. Face the serving edge toward a public path. Prices, opening, visitor demand, served counts and operating receipts are available in Services. Each facility charges its own supplies per completed visit and recurring upkeep. Demolition uses the existing refund and occupied-service rules.

Souvenir and information demand build gradually during a park visit. First aid and rest share some wellness effects and have different operating costs. The bright kiosk artwork includes ice-cream displays, gift boxes, information signage and a first-aid cross.

## Saves and continuation

New saves use schema/content 48/48. The file stores ride models, fixed-ride orientation, transport destination state and the two new visitor demand values. Existing rides retain their legacy defaults. Older saves keep their original ride identities, finances and operational state. New catalogue fields and new facility kinds are rejected when embedded under an older schema header.

The complete showcase fixture uses explicit diagnostic historical admission receipts. Normal new parks keep the existing starting balance and default ground-level starter rides.

## Development verification

The catalogue acceptance tests cover each model in all four orientations, atomic overlap refusal, saved identity and capacity, all 14 models coexisting with eight facilities, real paid guest trips, destination unloading, a close request at the far station, facility purchases and exact replay after loading.

The game exposes a fresh-process continuation proof.

```powershell
.\target\release\parkbound.exe --save-check write-catalogue --save-dir 'C:\Parkbound-Proofs\catalogue'
.\target\release\parkbound.exe --save-check verify-catalogue --save-dir 'C:\Parkbound-Proofs\catalogue'
```

Use a new absolute directory for the writer. The verifier preserves the original save and compares the same 2,400 future simulation ticks.

Capture scenes are `catalogue`, `catalogue-menu`, `catalogue-water`, `catalogue-transport` and `catalogue-facilities`. They use production construction and rendering. Rendering checks on this development PC establish correctness. Its speed is excluded from performance targets.

The final local verification results, package identity and GitHub checkpoint are recorded in `RESUME_DEVELOPMENT.md`.
