# Beacon Drop — 0.63.0

New games start on flat ground with the build height set to **0**, so new paths sit on that ground. The existing demonstration paths and coaster remain elevated, and **Terrain [T]** can switch the ground to hills. Loading a saved park restores its terrain instead of applying this new-game choice.

## Build and operate

Open **Rides [Y]** and choose **Drop $420**. Close the panel, choose a build height in **Build**, then click a clear 3×3 footprint. The ride starts closed. Add its own queue with **J**, entrance with **I**, and public exit path with **K**. Open it from the ride panel or the selected-ride controls after access is ready.

The six-seat carriage climbs a guide tower, pauses, drops and brakes at the loading deck. Guests pay the ride's own ticket price when they depart. **F6/F7** changes that price. The tower has its own queue, reviews, upkeep and maintenance. It costs **$0.95 per 900 operating ticks**, and demolition returns **$210** of the $420 paid construction cost. Close and empty it before removing it. The park can have one active tower, one carousel and one swing at the same time if it has enough cash and space.

The tower uses the usual ride colors and name controls. Its default name is **Beacon Drop**. The carriage and passengers use the same saved cycle position, so loading, motion and reloading remain aligned. The six-step clearance reserves room for the full lift; taller custom tower construction and physical freefall tuning remain future work.

## Continue on another Windows PC

Use the branch and shared OneDrive checkout listed in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md). Player saves live in `%LOCALAPPDATA%\Parkbound\saves` outside that checkout. Close the game and transfer both the primary save and `.bak` if you want to continue the same park. Save schema/content **33/33** reads version 32 and earlier parks; older executables should not be used to open version-33 saves.
