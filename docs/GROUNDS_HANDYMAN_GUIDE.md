# Grounds and handyman — 0.59.0

Open **Guests [L] → Park stats** to see the number of littered paths, spills and completed cleanups. Use **Assign / dismiss handyman** to switch one worker on or off. The first assignment starts grounds activity; after dismissal, guests can still leave mess and the existing dirt remains until the worker is reassigned or the path is demolished.

While visitors are present, a public path can gain litter or a spill. Nauseous visitors create spills. Guests on dirty paths gradually lose satisfaction. The handyman starts at the configured park entry, walks the connected public-path network and cleans reachable mess. They return toward the entry when there is nothing to clean. A disconnected path cannot be reached. Removing a dirty path also removes its mess, but does not count as a cleaning.

The handyman assignment is free in this stage; there is no wage or supply charge yet. This is a single worker and a bounded grounds model, not a full staffing roster. Park save/load preserves dirt, cleaned count and the worker's current edge and destination. Current park save schema/content is **29/29**; versions 1–28 load with idle grounds. Keep both primary and `.bak` saves from `%LOCALAPPDATA%\Parkbound\saves` when moving PCs.

The game uses original procedural colors and geometry. No assets from another game are required. See [GROUNDS_HANDYMAN_REPORT.md](GROUNDS_HANDYMAN_REPORT.md) for the implementation and verification.
