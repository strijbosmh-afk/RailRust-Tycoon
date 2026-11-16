# Sunhaven Gardens playable demo

Version 0.91.0 adds a landscaped 48 × 48 sandbox to the title screen. Choose **Play Sunhaven Gardens · 7 rides** with the mouse or the title menu's arrow keys and Enter. Each launch starts a fresh park. Use **Continue latest park** or **Load Park** to return to a saved visit.

## Explore the park

| Attraction | Type | Ticket |
| --- | --- | --- |
| Sunburst Coaster | Steel coaster with lift hills and drops | $2 |
| Willow Falls | Log flume | $2 |
| Sunhaven Railway | Two-station park railway | $1 |
| Garden Carousel | Carousel | $1 |
| Blossom Cups | Spinning teacups | $1 |
| Golden Wheel | Ferris wheel | $1 |
| Marigold Voyager | Pirate ship | $1 |

All seven rides have a connected queue, entrance, public exit route and checked opening. The railway unloads at its second station. Stations and paths start at ground level on flat land. The coaster and flume climb above the park.

The central service avenue contains food, drinks, toilets, rest, ice cream, souvenirs, information and first aid. Food and drinks cost $1, souvenirs $2, rest $0.50, and toilets, information and first aid are free. Automatic stock ordering is enabled. A mechanic, handyman, security guard, entertainer and service attendant are hired and paid through the ordinary simulation.

Flower borders, tree gardens, a sand-edged lake, benches, bins and covered seating fill the grounds. The west-side entrance charges $1. The opening flyer campaign and five minutes of simulated arrivals populate the park before play begins. The park begins with a $15,000 development budget. Construction, supplies, wages and opening income are recorded in its finance ledger, so the displayed cash is the remaining balance.

## Play and save

- Pan with the movement controls and zoom with the mouse wheel. Use the normal rotation controls to view the attractions from each side. Controls and accessibility settings are available from Help.
- Open **Manage → Rides** to inspect tickets, operation, access, testing and ride appearance.
- Open **Manage → Management → Operations** for problems requiring attention. Supplies and staff continue to cost money during play.
- Use the Build menu to add attractions, paths, scenery or facilities. All research is unlocked. This sandbox has no campaign deadline.
- Press **Space** to pause or resume. Press **F5**, or use **Esc → Save**, to store your visit under Sunhaven Gardens. **Save As New Park** makes an additional copy.

The first save creates an independent library entry. It preserves other saved parks. Reopening a save pauses the simulation, ready to resume with Space. Starting the demo again creates another fresh unsaved park.

## Resume development

The deterministic builder lives in `crates/park_sim/src/demo_park.rs`. Title-screen entry and safe save-library integration live in `crates/park_game/src/front_end.rs`. `crates/park_game/src/save_check/demo_park.rs` exercises actual guest use and independent-process save continuation. Park saves remain schema/content 51/51.

After building the release executable, use separate processes with a new absolute proof directory.

```powershell
./target/release/parkbound.exe --save-check write-demo-park --save-dir 'C:\Parkbound proofs\demo-01'
./target/release/parkbound.exe --save-check verify-demo-park --save-dir 'C:\Parkbound proofs\demo-01'
```

The writer records the opening state, then verifies completed paid visits on all seven rides and facility use after 18,000 further ticks. The verifier reloads the opening state through production persistence, reproduces the same future and checks that the original save bytes remain unchanged. `--capture-scene demo-park` renders the same playable builder for visual checks.

The shared checkout is `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Read `RESUME_DEVELOPMENT.md`, `STATUS.md` and the current project instructions before continuing on another PC. The development machine supplies correctness checks and remains excluded from performance acceptance.
