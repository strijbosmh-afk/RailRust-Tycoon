# Paid handyman and path bins — 0.60.0

Open **Guests [L] → Park stats** to assign or dismiss the park's one handyman. An assigned worker costs **$0.50 every 900 simulation ticks**. Finance records each payment as a handyman wage and includes it in total staff wages. If the park cannot pay when a wage is due, the worker is dismissed and the wage clock resets. Pause stops the clock. The existing mechanic wage is separate.

To place a bin, click a **public path** on the map and use **Build → Bin on selected path**. Click the same button again to remove it. Bins are free in this phase, limited to 64 per park, and hold four pieces of litter each. Installing a bin starts grounds activity even if no handyman is assigned. Litter made on its path goes into the bin while there is room; a full bin leaves ordinary visible litter. Nausea spills always remain on the path. The worker follows connected public paths to empty a used bin. A disconnected bin remains until a route is available. Demolishing its path removes the bin.

**Park stats** shows the bin count, number full, number emptied, worker wage paid, visible litter and spills, and completed cleanups. Worker movement and bin fill survive save/load. Current park saves use strict schema/content **30/30**; version 29 loads with empty bins and no handyman wage history. Copy both the primary and `.bak` park saves from `%LOCALAPPDATA%\Parkbound\saves` with the game closed when moving PCs.

These are original procedural graphics. See [the stage report](PAID_HANDYMAN_BINS_REPORT.md) for verification and limits.
