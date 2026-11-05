# Skyward Swing — player guide (0.61.0)

Open **Rides [Y]** and choose **Swing / $350**. Move the cursor over an empty 3×3 area and click to place the closed ride. Its footprint obeys the same terrain clearance and build-height rules as the carousel. Connect a queue to its entrance and a public path to its exit, then open it from the ride panel. Guests can choose it independently of the carousel and coaster.

Skyward Swing has six seats and a visible side-to-side pendulum cycle. It has its own ticket price, completed visits, guest reviews, operating cost, maintenance state and ride income. The initial build costs $350; demolition follows the existing half-cost refund rule. Upkeep is $0.80 per 900 operating ticks. Existing timing controls in **Rides → Operations** work on either flat ride. The default name is **Skyward Swing** and the existing Colors and Rename controls apply.

A player may keep one active carousel and one active swing ride. Each has its own entrance, queue, exit and independent passengers. Demolishing one leaves the other operating. Ride history is retained within the current bounded record limit.

Save and load use strict park schema/content **31/31**. Older saves migrate their carousel records to the carousel kind; a current save records the flat ride kind explicitly. Park saves remain in `%LOCALAPPDATA%\Parkbound\saves` unless a save directory is supplied. Close the game before copying both the primary and `.bak` files to another PC. The shared source checkout is `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`; see [the handoff](../RESUME_DEVELOPMENT.md) for other PC paths.
