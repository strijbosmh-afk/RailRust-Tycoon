# Parkbound — complete player manual

Version 0.99.2 · Windows · available entirely offline · English names match the game's controls.

## 1. Finding your way around this manual

Parkbound is a park-building and management game. Build a connected park, install attractions, look after visitors and balance income against expenses. Campaign scenarios add specific objectives and sometimes a deadline. In Sandbox, you choose what you want to achieve.

Press **H** or **F1** in a park to open **Help**, then choose the document icon labelled **Player manual - open in browser**. Hover over an icon or focus it with the keyboard to reveal its full name. You can also double-click `docs/PLAYER_MANUAL.html` beside `Parkbound.exe`. No internet connection is required. This Markdown document contains the same instructions.

The HTML version has a contents panel with chapter and topic links, a search field, a clear-search button and a print button. Try searching for “queue”, “stock”, “interest” or “height”. Search shows matching chapters and highlights your words. Clicking a contents entry restores the full document and takes you to that topic. **Ctrl+F** uses the browser's own search. **/** focuses the manual's search box; **Escape** clears its filter while the box is focused. The browser's Back button returns to a previous heading.

**Start here:** [your first playable park](#3-your-first-playable-park-step-by-step), [finding controls](#5-screen-layout-and-menus), [individual ride settings](#9-managing-one-ride), [making money](#15-finances-and-profit), [the campaign](#19-the-six-scenario-campaign), [solving problems](#23-troubleshooting).

This guide describes the current game. A grey control or red construction preview usually has a specific reason: research, money, ownership, clearance, connectivity, operating status or scenario rules. Read that reason before trying again. Older technical guides in the docs folder record earlier releases and may mention superseded limits; use this manual for the current playing workflow.

## 2. Installing, starting and choosing a game

### Install on Windows or use the portable package

1. Run the Parkbound Windows installer, or extract the complete portable ZIP.
2. Keep the **assets**, **designs** and **docs** folders with the portable executable. Copying only the executable leaves parts of the distribution behind.
3. Start Parkbound from its shortcut or `Parkbound.exe`.
4. New display preferences start borderless. Open **Settings** to choose a display, window mode and interface scale.
5. If the display becomes unusable, press **F11** to recover Windowed mode, Current display and Auto interface scale. You can also start `Parkbound.exe --safe-windowed`.

### What the main-menu choices do

| Choice | What happens | Best for |
| --- | --- | --- |
| Play Sunhaven Gardens | Starts a fresh landscaped demonstration with seven working rides | Exploring and learning management |
| Continue latest park | Loads your most recent saved park | Continuing a visit |
| New Park | Configures a fresh Sandbox | Building your own park |
| Campaign | Starts a scenario with objectives and unlocks | A directed challenge |
| Load Park | Selects a saved park or its previous valid backup | Managing several parks |
| Scenarios and editor | Creates, imports, edits and plays custom scenarios | Designing your own challenges |
| Settings | Changes display, sound, controls and accessibility | Making the game comfortable |

The main menu keeps text buttons. Many in-park actions use icons. Hold the pointer over one to read its name, shortcut and any price. **Tab** and **Shift+Tab** move keyboard focus; **Enter** activates the focused control. **Escape** returns to the previous screen.

### Configure a Sandbox

Name your park, then select a map with **32, 64, 128 or 256 tiles per side**, an available landscape and the starting rules. The 32-tile option has flat terrain and starter infrastructure. Larger maps begin empty and can use flat or rolling landscapes. Starting budgets are $500, $1,000, $5,000 and $10,000. You can configure research availability, visitor arrivals, loans and charging policy.

For your first custom park, choose **32**, flat terrain and a comfortable budget. Select fully available research if you want to practise construction without research locks. Choose **Create Park**; that is the initial keyboard selection when this screen opens. Sandbox has no compulsory finishing score or deadline. A larger map provides space but also requires more entrance and path construction.

## 3. Your first playable park step by step

### Route A: learn in Sunhaven Gardens

1. Choose **Play Sunhaven Gardens**. Each use starts a new demonstration; it does not load your previous saved visit.
2. Press **Space** to pause. Locate the seven rides, service avenue and entrance.
3. Right-drag to move the camera. Use the wheel over the map to zoom. Briefly right-click a building to manage that particular object.
4. Open **Guests** and inspect satisfaction and needs. Select a visitor, then choose **Find** to locate that person.
5. Open **Management and economy → Overview**, then **History**. Compare sales with operating expenses.
6. Right-click a facility. Check its opening state and price; inspect its stock under **Supplies**.
7. Resume with **Space**. Watch guests queue, board and return to public paths after their rides.
8. Close the panels and press **F5** to save your own visit. Later use **Continue latest park** or **Load Park**.

Sunhaven contains Sunburst Coaster, Willow Falls, Sunhaven Railway, Garden Carousel, Blossom Cups, Golden Wheel and Marigold Voyager. All eight facilities and all five staff roles are present. Research is unlocked and there is no campaign deadline. The opening allocation is $15,000, but construction and simulated activity have already been recorded. The displayed cash is the remaining balance, not necessarily $15,000.

### Route B: build your own first park

1. Create a Sandbox and pause immediately. For guided practice, open **Help → First park guide**. Its seven steps observe your actual park actions.
2. Locate the public paths. On a large empty map, build from the west edge through the available land corridor towards your building area.
3. Set a public path as the visitor entrance through **Visitor flow**. Enable arrivals and open the park.
4. Use **Build → Catalogue** to select an available gentle ride, such as a carousel. Place it on clear owned land with space for access and an exit.
5. Right-click the ride and open **Access**. Build its entrance, join it to a queue and connect the other end of that queue to public paths.
6. Build a separate exit. Its public path must return to the park network. Use the suggestions in **Access** to find missing connections.
7. Open **Operations**, resolve all reported problems and open the ride. Start with a modest ticket price.
8. Add drinks, food and toilets with their serving sides facing public paths. Check opening, prices and stock.
9. Add a few benches and bins. Leave cash for supplies, wages and upkeep.
10. Resume. Check actual admissions, completed guest rides and served customers. A moving ride alone does not prove that guests can reach it.
11. Press **F5**. Add a second ride after the first receives customers and you understand its costs.

### A useful first-play routine

Build while paused, run the park for a short period, inspect the result and change one problem at a time. Check connectivity first, then needs, operating status, stock and finances. Save after a successful expansion and before major rebuilding. Fast-forward is useful once you know visitors are moving correctly and recurring expenses are affordable.

## 4. Objectives and the simulation clock

In Sandbox you decide your objective: several distinct rides, satisfied guests, no lost visitors or a succession of profitable periods. Campaign scenarios define their own conditions. Reaching one number is insufficient when the scenario requires several objectives at once.

**Space** pauses or resumes. Outside text fields and special panels, **1**, **2** and **3** select 1×, 2× and 4× speed. In Explore, those number keys select menu groups instead. A payment or campaign interval expressed in seconds refers to **simulation time**, not the Windows clock. Fast-forward also accelerates needs, expenses, research and deadlines.

Ride and facility settings do not automatically pause the park. Pause with Space if you want time to read. Opening this manual in a browser does not pause it either. A loaded park begins paused so that you can inspect its state before resuming.

## 5. Screen layout and menus

### Header, sidebar and Explore

The header contains the park name and key figures. Finance, Rides and **Tips** shortcuts sit together. The sidebar contains general actions and grouped construction tools. Selecting a guest, path or attraction changes the contextual information. The lower status message often explains why an action was refused.

**Explore / F2** is the central route through the controls:

| Group | Main destinations |
| --- | --- |
| Build | Paths, queues, terrain, rides, facilities, coaster track and saved designs |
| Manage | Finance, economy, visitor flow, staff and scenario editing |
| Inspect | Guest, ride and facility details |
| Park | Objectives, park statistics, saving and the park menu |
| Help / Settings | Instructions, sound, accessibility and display |

Panels scroll when their contents do not fit. Place the pointer over the panel and turn the wheel; over the map, the wheel zooms. Escape first closes a panel or cancels a tool, then opens the park menu from the clear map. Leave a text field before expecting letter keys to act as shortcuts.

### Reading icons

Track symbols lead to coaster tools, winding lines to paths, gates to entrances, outward arrows to exits, tools to maintenance, coins and charts to finances, people to visitors and a lightbulb to Tips. Plus and minus change the value in their own group. Check the tooltip to distinguish path height from brush size. Names, editable fields and list entries remain readable text. Main-menu, New Park and Settings choices retain their text labels.

## 6. Camera, display, sound and accessibility

### Inspect the map

Hold and drag the **right or middle mouse button** to pan. A short right-click without dragging opens object settings. Use **WASD** or arrow keys when no text field or list has those keys. **Q/E** rotate the map by a quarter turn. Use the wheel or **+/−** to zoom, **Home** to fit the park, **G** for the grid and **U** for underground view.

Rotate when a tall building hides an entrance. Zoom out to inspect connections and zoom in for gates or height changes. Click a visible part of the attraction, rather than empty space beside a tall structure. Right-click selection accounts for rotation and interface scaling.

### Use a higher-resolution display

Open **Menu → Settings**. Choose the display, Windowed, Borderless or Fullscreen, and interface scale **Auto, 100%, 125%, 150% or 200%**. The effective scale may be capped to keep controls usable; Settings shows that effective value. Smaller screens require more panel scrolling. F11 restores a safe window configuration.

### Music, ride sounds and reactions

**F10** opens sound controls. Adjust Master, Music, Ambience, Rides, Alerts & interface and Guest & staff reactions separately. Camera position and zoom affect ride-sound proximity and stereo placement. Water rides, chain and rolling sounds, and guest reactions follow relevant activity. Reduce Music to hear the park more clearly; use Mute or reduce Master for silence.

Sound fades when the game loses focus. **Reconnect audio device** retries the current Windows output after a device change or loss. Short captions communicate important actions even without sound. Music and ambience may continue while paused; ride activity stops with the simulation.

### Accessibility and custom keys

Open **Controls & accessibility** from Settings or Help. Eighteen gameplay actions can use alternative available keys. The panel shows current assignments; fixed navigation and recovery keys keep their roles. The shortcut table later in this manual lists defaults. **High contrast** improves separation between text and backgrounds. **Reduced motion** reduces decorative movement while vehicles and guests still show their real positions. Captions can be controlled separately.

## 7. Terrain, ownership and gardens

### Owned land and terrain height

Construction requires owned land. Large maps start with a central owned area and a narrow corridor to the western edge. **Buy Land** purchases adjoining tiles for $5 each. Ground colouring and boundaries show ownership.

**Raise** and **Lower** change terrain by one level. The brush covers 1×1, 3×3 or 5×5 tiles. The preview shows its entire area and exact cost. Raising or lowering costs $1 per changed tile, grass/sand painting $0.50 and water $2. If any tile is invalid or the whole action is unaffordable, nothing is purchased. Clear obstructing construction before changing its terrain.

Changing ground height does not automatically lower an existing elevated ride. Building height, path height and ground height are separate. Set the desired placement height before building a new ride. Major changes to an existing attraction can require rebuilding or editing its track.

### Water, bridges and tunnels

Water needs suitable level ground. Ground-level paths and objects can block flooding. A bridge with sufficient clearance may remain over water, but a new ground path cannot be placed in the water. A path cannot pass partly through solid ground.

To build a tunnel, enable **Underground view**, select Path or Queue and lower the construction height far enough for the complete path clearance to fit below the surface. A slope needs extra room. The preview says **Tunnel** when valid. Surface view hides buried sections and shows tunnel mouths where they connect outside. Return underground to select or edit buried paths.

### Trees, flowers and path furniture

Place trees and flowers on clear owned grass or sand. Trees cost $2.50 and flower beds $1.50. Removing paid scenery returns half its original price. Free existing decoration has no paid purchase value to refund. Nearby scenery supports satisfaction but does not replace food, drink or rest.

Benches and bins belong on public paths. Signs and fences provide visual wayfinding and edges; by themselves they do not change guest route selection. **Cover a path** adds a roof to a bench. A new bench costs $1; covering an existing bench is free. Keep entrances visible and walking routes compact rather than filling every available tile.

## 8. Paths, queues, gates and heights

### Four different pieces of infrastructure

| Item | Purpose | Required connection |
| --- | --- | --- |
| Public path / Path | Circulation for visitors and staff | Other public paths and the park entrance |
| Queue | Waiting for one particular ride | Public access at one end, that ride's entrance at the other |
| Entrance / Gate | Boarding the selected ride | The correct ride and its own queue |
| Exit | Leaving the ride | The ride and a public route back into the park |

A working arrangement looks like this:

```text
park entrance === public path === facilities === public park network
                       |
                    queue A --> entrance A --> ride A
                                                  |
                         public return path <---- exit A
```

A queue is not a public through-route. A facility needs public path access on its serving side. Queues and gates belong to the selected ride: check its name before building them. **Rides → Access** is the best place to check the owner and missing connections.

### Build a connection step by step

1. Select **Path / P** for the public section or **Queue / J** for the waiting section.
2. Set height and shape before placing. C changes path shape/direction; for gates and facilities, C changes orientation.
3. Place neighbouring tiles with matching **edge heights**. Adjacent tiles at different levels do not connect automatically.
4. Use a one-level slope between consecutive heights. Its high edge must meet the high neighbouring edge.
5. Place the entrance with **I**, facing the ride, and its exit with **K**, with a separate public return connection.
6. Check **Access**, resolve each failed condition and then watch a real guest use the route.

A level public path costs $1 and a sloped public path $1.50. Level queues cost $2 and sloped queues $2.50. Hover for the selected tile's quote. Invalid placement spends nothing. Paths can appear close together without connecting; inspect their heights, orientation, type and ownership.

### Demolition and undo

**Demolish / X** removes the pointed-at item when allowed. **Delete** removes a selected object. Close rides and allow passengers to leave before major demolition. Removing a path can also remove its furniture and disconnect the park.

Paid paths return half their purchase price, plus applicable furniture refunds. Rides and facilities use their own refund rules and actual paid cost. A refund is a separate transaction from a new purchase; replacement construction is not automatically free.

While park construction is paused, **Ctrl+Z** and **Ctrl+Y** reverse or redo up to sixteen recent edits. Resuming, loading or switching parks clears this temporary history. Coaster drafts have their own undo/redo. Saves store the resulting park, not the undo stack. Undo never replenishes spent Tips.

## 9. Managing one ride

### Open the correct settings with a right-click

Briefly **right-click** a visible ride, its track, owned gate or queue. **Rides** opens with the owner of that item selected. Right-click a facility building to open **Services** instead. Empty ground opens nothing. Holding the button and dragging pans the camera without opening settings.

Changing camera, park or foreground panel during the gesture cancels it. This prevents a stale click from selecting a different object after a drag or load. A foreground panel covers the map; close it before clicking an attraction behind it.

### Operations, names and colours

**Operations** shows operating state, opening, capacity and the settings supported by the selected model. A carousel does not expose the same equipment as a coaster or transport train. Change vehicle counts and similar equipment only while the ride is closed, stopped and empty. A close request lets already departed journeys finish.

Adjust loading and waits to suit demand. A high minimum load can hold a quiet ride at the station. Additional trains do not help when station or block space is insufficient. **Reset** restores default operating settings. This panel also reports maintenance, running costs and reasons opening is refused.

| Setting | Meaning | When to change it |
| --- | --- | --- |
| Cars / Trains | Vehicle and train count where supported | Add capacity when demand and block spacing support it |
| Station wait | Base waiting period at the station | Balance prompt departure with time for boarding |
| Minimum load | Desired minimum passenger count | Lower it in quiet parks; zero means no minimum |
| Max extra wait | Additional time to seek the minimum load | Prevent unnecessarily long waits |
| Lift speed | Speed on lift sections | Consider the following track and retest afterwards |
| Brake speed | Track braking setting | Control approach speeds and retest afterwards |
| Cycle duration | Length of a fixed ride's operating cycle | Balance experience against throughput |

Choose **Edit name**, type a name and use **Apply name**. **Restore default** restores the model name. **Colors** selects available paint schemes. Renaming or repainting does not repair access or change ticket prices.

### Tickets and the selected ride inspector

The right-click leaves that ride selected. Close Rides with **Use ride / Escape** to return to its map inspector. Use **F6/F7** or its price icons for $0.50 changes. **O** opens or closes the selected ride and **F** locates it. Different rides retain independent prices. **Management → Policies** also offers price presets that affect all installed rides.

Scenario rules can require free rides. A guest with a reserved fare keeps that quote; later prices apply to later reservations. Payment occurs on actual departure. Cancelled boarding is not a sale.

### Access, testing and maintenance

**Access** checks the entrance, queue and exit route. Choose a suggested construction action, place its preview, then reopen Access for the next missing item. A preview is not yet a built path. **Test** runs an isolated mechanics lap for an installed coaster; close and empty it first.

For wear or a maintenance shutdown, use the offered **Service ride** action or provide a reachable mechanic. The ride must be safe, stopped and empty. Resolve cash and access problems too. A mechanic cannot reach an isolated path island, and adding money does not automatically reopen a ride closed for unpaid costs.

## 10. The full ride catalogue

Open **Build → Catalogue**, choose a family and select an available model. Read its quote, footprint and description. Rotate with **C** and place only when the entire preview is valid. Multiple instances can have separate names, customers and accounts. Some older quick-build controls have narrower limits; the catalogue contains the full selection.

| Family | Model | Supplied capacity | Character and considerations |
| --- | --- | --- | --- |
| Gentle | Festival Carousel | 6 | Gentle rotating horses; a useful first ride |
| Gentle | Meadow Wheel | 6 | Observation wheel with a tall gondola structure |
| Gentle | Whirling Cups | 12 | Six rotating two-person cups |
| Gentle | Skyline Tower | 8 | Observation lift, viewing pause and descent |
| Thrill | Skyward Swing | 6 | Pendulum gondola; demand depends on preferences |
| Thrill | Beacon Drop | 6 | Lift, hold, rapid drop and braking return |
| Thrill | Corsair Ship | 12 | Swinging pirate ship |
| Water | Timber Falls | 4 | Log boat, lift and splashdown |
| Water | River Rapids | 6 | Passenger raft on a water channel |
| Transport | Park Railway | 12 | Railway between two stations |
| Transport | Vista Monorail | 12 | Monorail between two stations |
| Coaster | Classic Coaster | 6 | Classic track with wooden supports |
| Coaster | Steel Launch Coaster | 6 | Steel track with launch and transition pieces |
| Coaster | Inverted Coaster | 6 | Suspended seats beneath the track |

Capacities describe the supplied configurations. Supported train and car settings can change them. Each track family restricts its available pieces. The live catalogue quote determines cost, clearance and research availability. Leave money for paths, gates, stock, wages and upkeep after the initial purchase.

Fixed rides reserve their full footprint and height. Supplied tracked layouts occupy approximately 10×7 tiles before rotation. Leave room beyond the visible vehicles. Transport uses a boarding station and a destination: put the entrance at the first station and the exit at the destination. The empty train returns to the boarding station.

## 11. Building a coaster from station to first ride

### Start with a short working circuit

1. Pause and choose **Coaster track / Z**. Choose **New coaster** for a separate ride; select an existing ride in Rides before editing its track.
2. Choose clear owned land and start a station at ground level. Leave outside space for its entrance, queue and exit.
3. Keep **Follow heights** enabled. Select a straight and use **Add next piece / Enter**. The next preview inherits the preceding exit direction and height.
4. Build a small circuit with curves. Establish a working loop before adding complicated elevation changes.
5. Match the final piece to the station's direction and height. Visually nearby ends do not necessarily connect.
6. Read the quote and choose **Apply**. This installs the draft and settles actual costs/refunds. Drawing pieces in the draft alone does not build an operating attraction.
7. Open **Rides → Access** and build its entrance, owned queue and public exit route.
8. Keep the ride closed and use **Test → Run one isolated lap**. Resolve track problems.
9. Select suitable train and loading settings, open the ride, watch a guest complete a journey and save.

### Climbs, descents and the next piece's height

The preview shows **Start → End** levels. With **Follow heights**, each piece starts at the previous exit. Consecutive lifts climb automatically; a drop starts at the reached elevation and leaves the next piece lower. Curves update the next heading too.

**PgUp/PgDn** manually changes the entry height. Manual height or direction changes disable following because they deliberately depart from the previous endpoint. Select **Follow heights on** to return to that open end. Reopening an incomplete draft finds its last open connection. At the floor or height limit, the editor can fall back to a straight; inspect the preview before placing.

Pieces include Station, Straight, left/right Curve, Bank, Lift, one/two-level Drop, Brakes, Launch, Camelback, Smooth rise, Smooth drop and Heartline roll. A camelback rises inside its tile but ends at its original height. Smooth rise/drop changes level gradually. A roll needs extra clearance above the track. Track families may reject particular pieces; the red preview explains the restriction.

### Trains, blocks and tests

Coasters support one to three trains with one to three two-seat cars when the layout permits it. Multiple trains require **more blocks than trains**, and each block must hold an entire train. The station and brake pieces begin blocks. Space brakes sufficiently far apart. A train stopped at a red signal may simply be waiting for the next block to clear.

Change the fleet only when the ride is closed, stopped and empty. Minimum load applies to the train at the station. Closing lets dispatched trains return and unload. Water rides use their own vehicle restrictions; transport uses a single train and a destination.

Tests report speed, height, features, estimated forces and layout ratings. These are model estimates, not engineering certification. The isolated lap excludes queueing, passenger loading and block waits. Track or operating changes invalidate the previous result. Retest, then observe actual guest reactions. Testing does not advance the park clock or change guests and cash.

### Safely modify an existing ride

Save first. Close the correct coaster and let everyone leave. Open its draft, change a few pieces, inspect connections and cost, apply and retest. **Next coaster** switches saved workspaces; check the name before Apply. Up to eight coaster records can coexist with fixed rides. Draft undo is separate from paused park-construction undo.

## 12. Saving and sharing coaster designs

**Designs / Ctrl+D** opens the design library. A design stores track geometry, not a complete park, guest population or all operating settings. Save a draft with a meaningful name. The library supports search, previews, renaming, updating a selected design, import and export.

The package includes **Pocket Oval**, **Meadow Winder** and **Meadow Climb** in designs. Import a `.parktrack.json`, select it, adjust position, rotation and height, then inspect its preview. **Load into draft** loads geometry; **Apply** constructs it. Close and empty an existing ride before replacement. Moving a station can require new gates and paths.

A supplied design's original placement height need not be ground level. Adjust it deliberately; Follow heights then preserves connections between pieces. Check land, tunnels, research, cash and nearby construction. Export a successful design and try importing it into another park before sharing. Importing geometry does not automatically unlock the receiving park's research.

## 13. Facilities, prices and stock

### Eight services with different purposes

| Facility | Build cost | Default visit price | Main effect |
| --- | --- | --- | --- |
| Food | $120 | $2 | Relieves hunger |
| Drinks | $80 | $1 | Relieves thirst |
| Toilets | $100 | Free | Relieves bathroom need |
| Rest | $140 | Free | Restores energy and helps nausea |
| Ice cream | $110 | $1.50 | Helps hunger and thirst together |
| Souvenirs | $160 | $4 | Satisfies souvenir demand |
| Information | $90 | Free | Provides park information |
| First aid | $180 | Free | Helps nauseous and exhausted visitors |

Scenarios and existing parks can use other configured prices; Sunhaven deliberately charges less for some services. These are model defaults, not mandatory prices. Free service still consumes supplies and incurs upkeep.

### Configure one facility

1. Right-click its building and check the selected name and kind in **Services**.
2. Open or close it and change its price with the plus/minus controls. F6/F7 also adjust prices in this panel.
3. Check customers, sales, costs and its serving edge. A nearby public path on the wrong side is insufficient.
4. Open **Management and economy → Supplies** and select that same facility.
5. Inspect inventory, automatic ordering and cash. Order twenty units or enable automatic ordering as needed.
6. Resume and check that served-customer counts actually increase.

### How inventory works

With management enabled, each facility holds up to one hundred consumable units. Toilets, rest, information and first aid also use supplies. Automatic ordering buys twenty units when an open facility falls below five. The complete batch must be affordable. A failed order retries later while automatic ordering remains enabled.

Supplies are paid for on delivery. A completed service consumes one unit and charges its agreed price. A price change does not alter an existing quote. Empty stock prevents normal service sales. Closing stops automatic ordering and upkeep, but does not remove visitors' needs.

Reports expense stock when purchased. A large delivery can therefore make a period look unprofitable before those supplies serve later customers. Demolition does not erase the historical expense or turn unused stock into additional sales.

## 14. Understanding and helping guests

### Individual visitors and parties

Open **Guests / L**. Select a person to inspect their money, satisfaction, current activity, preferences, needs and available journey/group information. **Find** centres the map on that person. Page and filter controls help you locate specific problems.

New arrivals can form three-person parties with family, thrill-seeking or relaxed preferences. Members pay individually and can choose different affordable activities, but regroup between visits and leave together. Satisfied departing parties may attempt a return later; that is not an immediate guaranteed sale. Wallets are finite, so an apparently popular park can lose repeat business through excessive prices.

### Set the park entrance and admission policy

Select a suitable public path on the map, open **Visitor flow** and choose **Set entry**. Check its reported connection. **Toggle arrivals** controls new arrival attempts; **Open / close park** controls admission. Entry price has its own minus/plus controls. Pricing policy can require free entry, free rides or allow flexible charging. A scenario can lock that choice.

Candidates consider their willingness and available money before entering. High admission leaves less for rides and facilities. Closing the park prevents new admission but does not immediately remove existing guests. **Send guest home** asks the selected visitor to leave; existing activity and an exit route still matter. Sending people home does not repair disconnected paths.

### Needs, satisfaction and ride ratings

Hunger, thirst, bathroom need, tiredness, nausea and other needs change during a visit. Heat increases thirst. Demanding rides can increase the need for rest or first aid. Choices depend on affordability, preferences, connections and current condition. Offer variety in both rides and services.

Satisfaction is different from a ride rating. A visitor can enjoy a ride but still be unhappy through thirst or lack of rest. Scenery improves the surroundings; entertainers help satisfaction. Neither substitutes for a reachable toilet. Persistent unhappiness leads to departure. Adjust prices, shorten routes or provide missing services according to the reported problem.

### Use filters and trend charts

**All**, **Urgent**, **Unhappy** and **Lost** filter the guest list. Urgent means hunger, thirst or bathroom need is at least 70. Unhappy means satisfaction below 40. A visitor can meet several filters at once.

**Park stats** combines current averages with trends. The visitor chart samples every thirty simulation seconds and retains up to twenty-four samples. Loading starts a fresh session chart; it does not discard guests' saved state. Inspect several subsequent samples to see whether an intervention helped. Do not confuse a current snapshot with the park's complete sales history.

## 15. Finances and profit

### Four different figures

| Figure | Meaning | Common mistake |
| --- | --- | --- |
| Cash | Money available now | Treating a loan as earned profit |
| Sales / Revenue | Receipts from guest purchases | Treating revenue as profit |
| Operating result | Sales less recorded running expenses | Assuming a busy park must be profitable |
| Park value / assets | Book value of paid installed construction | Assuming asset value is immediately spendable cash |

**Finance / N** provides cashflows, park summaries, loans, park value and transactions. Management adds trends and actionable problems. Historical expenses remain after a ride or facility is demolished. Recent transactions are bounded, but older amounts remain represented in totals and owner accounts.

### Reconcile sales with available cash

A useful reconciliation is: starting allocation plus operating result, plus demolition refunds, minus construction, plus borrowing, minus principal repayments and paid interest equals cash. Operating costs include ride/facility upkeep, delivered supplies, wages, research and advertising. Reports separate these categories so you do not have to infer sales from the bank balance.

Use **Management → History** to compare successive periods. It retains sixty financial snapshots, one per simulation minute. Construction and supplies can make an expansion period negative by design. Check later whether the extension earns enough extra business. Keep a cash reserve for payments due before the next customer sale.

### Make the park more profitable

1. Repair disconnected access and closed sales outlets first. Unreachable capacity earns nothing.
2. Inspect actual customers and operating costs for each ride and facility.
3. Check whether guests reject prices or have run out of money. Change one price at a time.
4. Temporarily close unnecessary loss-making services or build more compactly, while keeping essential needs covered.
5. Research things you can afford and intend to build. Avoid continuous funding without a construction plan.
6. Hire staff for visible work; inspect unpaid workers and patrol restrictions.
7. Advertise after the park can feed, queue, entertain and release additional visitors.
8. Compare a new financial period. More revenue with even faster-growing costs is not improved profit.

**Policies** offers $2, $3 and $5 price presets for all installed rides. These check the rules before changing prices and preserve existing guest quotes. Use them deliberately: a gentle carousel and an intense coaster need not perform best at the same price.

### Use loans deliberately

**Finance → Loans** borrows or repays $100 at a time within the permitted limit and scenario rules. The general outstanding principal limit is $2,000. Interest is **0.5% per simulation minute**: a $100 loan costs $0.50 for a full minute. This is not an annual rate. Faster simulation advances that cost faster.

Interest accrues gradually and is collected at minute boundaries. Unpaid amounts remain due; the game does not automatically borrow more. Principal repayment also requires payment of accrued whole-cent interest. Borrow for a specific, reachable income opportunity and retain operating cash. A loan does not automatically reopen attractions closed for funds.

## 16. Staff, maintenance, weather and incidents

### Put your team to work

Open **Manage → Staff and park life**. New parks enable these systems. Older parks may offer **Enable park life**. Hire up to twenty team members; each role costs $1 per thirty simulation seconds. Inspect each employee's assignment and paid status.

| Role | Work | Check when idle |
| --- | --- | --- |
| Mechanic | Services rides that are ready for maintenance | Is the ride closed, empty, stopped and reachable? |
| Handyman | Removes litter and vomit, empties bins | Is the task on a reachable public path within patrol? |
| Security | Resolves vandalism and guest-distress incidents | Can the worker reach an incident in the patrol area? |
| Entertainer | Improves nearby visitors' satisfaction | Is the worker paid and near guests? |
| Service attendant | Assists at facilities and helps tired or nauseous visitors | Is there a reachable open counter or a guest needing help? |

Select a worker and use **Patrol tile** on public paths to include or exclude work destinations. **Whole park** clears the restriction. A patrol limits destinations; travel can use the connected public network. **Find selected** locates the employee. **Dismiss selected** ends employment without deleting past wages from the accounts.

**Management → Policies** provides a $0–$20 team payroll cap per thirty-second payment window. Insufficient cash or a low cap suspends work until payment succeeds. The cap does not cover all other park costs or older standalone staff contracts. **Grounds controls** retains the earlier separate mechanic/handyman controls; avoid paying for unintended duplicate staffing.

### Weather and shelter

Park life supports sunny, hot, rainy and storm conditions. Automatic weather can change every sixty simulation seconds; weather policy can also fix one condition. Heat increases thirst. Rain and storms reduce outdoor satisfaction. Covered benches shelter visitors while they rest.

Storms can close exposed rides while dispatched guests finish their journeys. Suitable covered rides can continue. When conditions improve, the game checks whether reopening is safe. Your own close request, broken access, maintenance or unpaid costs can still keep a ride shut. The **Lighting / Weather** choices in display Settings concern presentation; manage simulated weather through **Staff and park life**.

### Resolve incidents

Orange markers and staff reports identify incidents. Severe tiredness or nausea can trigger guest distress; unhappy visitors may damage furniture. A vandalised shelter stops working properly until repaired.

Choose **Find first incident**, inspect the need or damage, check security's route and provide relevant services or attendants. Hiring more security alone does not solve a persistent shortage of rest, drinks or connected paths. For maintenance, also inspect the individual ride's status.

## 17. Research and economic controls

Open **Management and economy → Research**. Choose a category and funding: paused, $1, $2.50 or $5 per thirty simulation seconds. A project requires $40 of actual paid research. When cash is limited, smaller affordable payments may be processed. Switching categories preserves investment; completed categories stop charging.

| Category | Normally available at the start | Research unlocks |
| --- | --- | --- |
| Gentle | Carousel, Wheel, Cups | Skyline Tower |
| Thrill | None | Swing, Drop, Corsair Ship |
| Water | None | Timber Falls, River Rapids |
| Transport | Park Railway | Vista Monorail |
| Coasters | Classic Coaster | Steel Launch Coaster, Inverted Coaster |
| Facilities | Food, Drinks, Toilets, Rest, Information | Ice cream, Souvenirs, First aid |

The catalogue shows availability for your current park. A fully unlocked Sandbox or custom scenario can differ from this starting set. Research normally begins paused so it does not quietly spend your opening budget. Older parks may first need **Enable management**; read its notice because stock and economic rules change afterwards.

**Overview** summarises money and warnings. **Research** controls unlocks. **Supplies** controls stock. **Marketing** offers promotions. **Policies** contains shared price actions and payroll limits. **History** compares periods. **Operations** groups problems at specific objects. Use Operations to move from a broad warning to the relevant ride, facility or visitor.

## 18. Marketing campaigns

A marketing campaign purchases reach, not guaranteed customers or profit. First open the park, enable arrivals, connect the entrance and ensure spare capacity. Advertising is paid immediately. Duration uses simulation time; closing the park or disabling arrivals neither refunds nor pauses a paid campaign. Space pauses the whole simulation.

| Campaign | Price | Duration | Effect |
| --- | --- | --- | --- |
| Local flyers | $25 | 5 minutes | Shortens the candidate interval from 30 to 15 seconds |
| Radio ads | $40 | 5 minutes | Candidates accept up to $3 more admission, within their actual wallets |
| Family week | $50 | 5 minutes | Themed arrivals at a 10-second interval; requires an open gentle ride |
| Coaster enthusiasts | $50 | 5 minutes | Themed arrivals at a 10-second interval; requires an open coaster |

Flyers and radio are available through **Visitor flow** and marketing routes. Each type can have one active campaign; flyers and radio can overlap. Only one themed promotion can run at a time. Do not assume overlapping campaigns add all their benefits arithmetically: inspect the active arrival cadence and real results. Scenario policies can prohibit particular promotions.

### Evaluate a campaign

1. Inspect your cash reserve, available capacity and the last financial period before buying.
2. Choose the appropriate effect: flyers for more arrival opportunities, radio for admission willingness, or a theme matched to open attractions.
3. Note guests, sales and costs. Resume until enough campaign time has passed.
4. Inspect candidates, admitted visitors and radio's extra admissions. A candidate is not necessarily a paying customer.
5. Check needs and queues: overcrowding can consume the benefit of additional sales.
6. Compare extra revenue with the advertising bill and additional operating costs before buying again.

Radio does not add money to guest wallets. A candidate can still refuse because of budget or rules. Sales attributed to an active promotion describe activity in its window; they do not prove every sale was caused solely by advertising.

## 19. The six-scenario campaign

The game campaign is separate from advertising. Choose **Campaign** at the title. Each scenario has objectives, initial rules and an unlock condition. Awards belong to your player profile and unlock the next scenario. **Later campaign** cycles through the four later challenges.

| Scenario | Objectives | Deadline | Starting allocation |
| --- | --- | --- | --- |
| Meadow Challenge | 20 new visitors, 5 completed guest rides, at least $750 cash | None | $1,000 |
| Boardwalk Sprint | 24 new visitors, 12 completed guest rides, at least $900 cash | 10 minutes | $1,000 |
| Cedar Valley | 30 new visitors and 65 satisfaction | 20 minutes | $2,000 |
| Research Ridge | Complete 1 funded research project and 20 guest rides | 20 minutes | $2,000 |
| Lakeside Services | Install 3 facilities, serve 12 customers and welcome 30 visitors | 20 minutes | $2,000 |
| Grand Festival | Welcome 60 visitors, earn $150 sales and reach 65 satisfaction | 20 minutes | $2,500 |

Inspect progress and state in **Scenarios and goals**. A ride in a guest objective is a completed passenger trip, not a new track or empty test lap. New-arrival totals start from the challenge's beginning. Existing activity is not automatically a head start. A cash target is a balance requirement, unlike an operating-profit target.

### Approach Meadow and Boardwalk

**Meadow** requires free admission. Ride tickets may earn money. Radio and themed promotions are unavailable, but flyers are allowed. Research is capped at $2.50 per thirty seconds. Establish a connected open ride, keep expenses controlled and let enough new visitors complete trips. Protect the $750 balance while building.

**Boardwalk** requires free rides. Consider admission income and suitable facility prices rather than relying on coaster tickets. Pause to plan, but remember the ten-minute clock advances when resumed. Watch admissions, completed rides and cash together; improving one alone does not win.

### Approach the four later challenges

For **Cedar Valley**, keep needs reachable and routes manageable to protect satisfaction. **Research Ridge** requires paid research; placing an already available model does not complete a funded project. Choose funding and category early. **Lakeside Services** needs real customers, so incorrect orientation or empty stock prevents progress even with three buildings. **Grand Festival** combines capacity, sales and satisfaction; monitor all three instead of buying arrivals alone.

A failed deadline remains saved with that park. Load an earlier saved state or start a new attempt. Earned profile awards remain. Start separate attempts rather than trying to change a challenge already in progress.

## 20. Custom scenarios and the editor

### Design a challenge

1. Begin with a Sandbox containing the landscape and starting buildings you want.
2. Open **Scenarios and editor** and choose **New from this park**. The editor preserves your original park separately.
3. Choose **Build park** to edit terrain, paths, scenery, rides and services. The banner identifies this authoring state.
4. Return to the editor and enter a name and description. Explain special restrictions to the player.
5. On **Goals and rules**, choose up to four distinct objectives, their metrics and targets, and an optional deadline.
6. Set starting allocation, admission policy, research, loans and arrivals. **Available content** restricts constructible ride models and facilities.
7. Choose **Save scenario**, then **Test scenario** to play an actual test run.
8. Use **Return to editor** to restore your design and adjust it. Test play does not award official campaign medals.
9. **Return to park**, followed by confirmation, restores your original park. Saved scenarios remain available.

Metrics include arrivals, rides, customers, sales, operating profit, cash, satisfaction, facilities, park value and funded research. Allow time for travel, service and affordable construction. Restrictions on new construction do not remove buildings already present in the template.

### Set fair financial objectives

Starting budget is the financial allocation; existing construction spending still belongs to the template park. A budget below recorded spending is refused. Sales goals count new scenario sales. Operating-result goals can include the template's existing operating history. Begin with a deliberate, tested template and inspect the actual progress figures.

### Share scenarios

The library searches by name and shows description, map dimensions, goals and compatibility. **Play selected** starts an attempt after confirmation. **Edit selected** opens an editable copy. Rename and archive require confirmation; archiving moves the file to the scenario archive.

**Export** writes a `.parkscenario.json`. **Import** validates the selected file before adding it to the library. A scenario carries its starting park and rules, not a Windows program or campaign profile. Imported custom challenges do not grant official campaign awards. Unsupported or damaged files show an error; retain the original when investigating a problem.

## 21. Tips: three uses per park

The **lightbulb** next to Finance and Rides produces **two suggestions** about current conditions, such as disconnected access, needs, unhappiness, cash, losses, rejected prices or interest. Advice does not change the park automatically or promise profit. Read both suggestions, then inspect the relevant figures.

Each park receives **three uses in total**, not three per launch. Saving, restarting, loading a backup and construction undo do not replenish the allowance. **Save As New Park** carries spent uses into the copy. A genuinely new park receives a new allowance.

The first advice request in an unsaved park creates a library slot to persist the count. This does not replace normal saving of later progress: keep using F5. If the allowance cannot be stored, the game shows an error and withholds new advice. Advice is computed locally; no internet service is needed.

## 22. Saves, backups and another PC

### Preserve your park

**F5** or **Menu → Save** writes to the current park's library slot. A new park receives its own slot on first save. **Save As New Park** creates a separate copy. **Rename** changes the library display name. **Delete** archives a park rather than immediately erasing it permanently from disk.

**Load Park** lists saved parks. **Continue latest park** selects the latest save. **F9** loads the current stored park; **Shift+F9** requests the previous valid backup. Save your current state separately first if you want to preserve it. A backup is the previous save, not an unlimited timeline. Read confirmations when replacing or leaving unsaved work.

Saves contain terrain, construction, visitors, vehicles, prices, stock, finances, staff, research, running campaigns, scenario progress and Tips use. Temporary charts, selection and undo history need not all persist. The current park schema is 52; older executables may not read newer saves. Keep a copy of the player folder when upgrading.

### Where files live

| File or folder | Purpose |
| --- | --- |
| `%LOCALAPPDATA%\Parkbound\saves` | Player parks, preferences, profile and libraries |
| `parks` inside the player folder | Independent park slots with metadata and backups |
| `deleted-parks` | Archived park slots |
| `campaign.json` and its backup | Earned campaign awards |
| `display-settings.json`, `sound-settings.json`, `accessibility-settings.json` | Personal preferences |
| `docs/PLAYER_MANUAL.html` beside the executable | This offline player manual |

Close Parkbound before copying data. Transfer the **whole player folder** to another PC, not just one park file. Install the same or a compatible newer version there, restore the folder, load the park and check display settings for the new monitor. The source-code checkout on OneDrive does not automatically synchronise AppData.

The development checkout is `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. That contains source and technical documents, not automatically your player profile. Developers should read **RESUME_DEVELOPMENT.md** before continuing work. Google Drive synchronisation is disabled.

## 23. Troubleshooting

### No visitors are entering

1. Resume the simulation; a paused park produces no new candidates.
2. Open **Visitor flow** and check park opening, enabled arrivals and the selected entrance.
3. Confirm that entrance is a public path connected to the park network.
4. Inspect candidates and refusals. Expensive admission can prevent access even to affordable rides.
5. Check the active visitor limit and allow existing visitors time to leave.
6. Buy advertising only after these checks. Marketing does not build an entrance.

### A ride will not open or has no passengers

1. Right-click the correct ride and read the specific Operations reason.
2. Check Access: its own entrance, queue with public access and separate connected exit.
3. Inspect edge heights, especially after terrain edits or slope construction.
4. Check the circuit, required test, fleet/block space and whether configuration changes require a stopped empty ride.
5. Resolve maintenance, weather or funds closure. Reopen explicitly when required.
6. If open but empty, inspect price, preferences, wallets and minimum load. Follow a visitor with Find.

### Guests or queues appear stuck

Check the public return to the park entrance and the ride's exit path first. Ride A's queue does not provide access to ride B. Find the first disconnected edge, wrong level or reversed gate. Rotate to reveal hidden pieces and use Underground view for buried paths. After repairing a connection, let the park run briefly before adding more paths.

### A facility makes no sales

Check opening, stock, funds for a whole delivery and a public path on its serving side. Then inspect price and actual guest demand. A souvenir shop does not relieve thirst. A customer may still be travelling or receiving service; sales register on completion. Verify the selected facility in both Services and Supplies to avoid adjusting a different building.

### A busy park keeps losing cash

Pause and open **Finance** and **Management → History**. Separate construction, supplies, wages, upkeep, advertising, research and interest. Stop unnecessary recurring expenses, repair unreachable capacity and adjust prices methodically. A loan provides breathing room but adds interest; it does not correct an unprofitable operation by itself.

### Research or staff seem inactive

Check paused funding, completed categories, insufficient cash and scenario restrictions. For staff, inspect wages, payroll cap, public access, patrol area and suitable work. A mechanic correctly waits for an empty, stopped ride. A worker on an isolated path cannot reach a task elsewhere.

### Controls, heights or windows behave unexpectedly

Read the tooltip and leave text entry first. PgUp/PgDn can change a list page or construction height depending on context. C can rotate a building or change a path shape. Manually changing coaster height disables Follow heights. Right-dragging is not a context click. Escape returns towards inspection so you can choose the intended tool again.

For small text, increase interface scale. For hidden controls, scroll over the panel. For the wrong display, use F11. For missing audio, check Mute, volume levels, Windows output and Reconnect audio device. For a damaged save, try the offered backup and retain the damaged original for investigation. Follow recovery messages instead of overwriting your only copy blindly.

## 24. Keyboard shortcuts and context

These are defaults. **Controls & accessibility** shows customised assignments. Typing in a field enters text; arrow keys may select list items; number keys choose groups in Explore. Context deliberately changes some keys' meanings.

| Key or gesture | Action |
| --- | --- |
| Short right-click | Settings for the clicked ride or facility |
| Right-drag / middle-drag | Pan the camera |
| WASD / arrows | Pan outside special input contexts |
| Wheel / + / − over map | Zoom |
| Wheel over a panel | Scroll panel contents |
| Q / E | Rotate the map |
| Home | Fit the park in view |
| Space | Pause/resume |
| 1 / 2 / 3 | 1× / 2× / 4× outside Explore and text entry |
| F2 | Explore; 1–5 select its groups |
| L / M / N / Y | Guests / Services / Finance / Rides |
| Z | Coaster editor |
| Ctrl+D | Design library |
| P / J | Public path / queue |
| I / K | Ride entrance / exit |
| X / Delete | Demolition tool / remove selected item |
| B / V | Raise / lower terrain |
| C | Path shape or orientation; track direction in the editor |
| PgUp / PgDn | Build height; page/selection in relevant panels |
| Enter in coaster editor | Add the next piece |
| G / U | Grid / underground view |
| O / F | Open/close selected ride or facility / locate ride, in the appropriate context |
| F6 / F7 | Price − / + $0.50 for the active selection |
| Ctrl+Z / Ctrl+Y | Undo / redo paused construction or the active draft |
| F5 / F9 / Shift+F9 | Save / load / previous backup |
| H / F1 | Help |
| F10 | Sound |
| F11 | Recover safe window settings |
| Tab / Shift+Tab / Enter | Next focus / previous focus / activate |
| Escape | Close the active panel/tool, then open the park menu |

**T** and **R** are older terrain/demonstration restoration actions with particular conditions, not normal ways to rebuild an occupied park safely. Use explicit building tools for normal construction and Home for the camera. Save before using a restoration action.

## 25. Glossary and next reading

| Term | Meaning |
| --- | --- |
| Ride | An independently managed attraction with its own access, operations and accounts |
| Facility / Service | A building that relieves a need and can consume stock |
| Public path | The general circulation network |
| Queue | Waiting path assigned to one ride |
| Edge height | Level where a path joins its neighbour |
| Draft | Editable track layout before Apply |
| Block | A protected track section reserving space for a train |
| Candidate | Someone considering entry, not necessarily admitted |
| Admission | An actual park entry |
| Revenue / Sales | Guest receipts before expenses |
| Upkeep | Recurring operating costs |
| Payroll cap | Maximum team wages paid per payment window |
| Cashflow | Money actually entering or leaving accounts |
| Operating result | Operating receipts minus recorded running expenses |
| Scenario | Starting park with objectives and rules |
| Campaign | Official scenario sequence; in marketing, an advertising promotion |
| Backup | Previous valid saved version, not unlimited history |

Start playing with [chapter 3](#3-your-first-playable-park-step-by-step), find controls through [the menu guide](#5-screen-layout-and-menus) and consult [troubleshooting](#23-troubleshooting) when stuck. For construction, keep [paths](#8-paths-queues-gates-and-heights) and [coasters](#11-building-a-coaster-from-station-to-first-ride) handy. For management, begin with [finance](#15-finances-and-profit), [staff](#16-staff-maintenance-weather-and-incidents) and [research](#17-research-and-economic-controls).

This manual is maintained with the source and Windows package. The technical resume guide, version history and session reports document development and checks; they are not additional game objectives. The development PC provides correctness checks and is not a product-performance reference.
