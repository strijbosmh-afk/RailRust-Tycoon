# Ride names and planted scenery

## Name a ride

Open **Rides [Y]** and select the coaster or carousel. Choose **Edit name**, type a name, and press **Enter** or **Apply name**. **Default** restores that ride's original name. Names are scoped to a stable ride identity: naming the coaster does not rename the carousel. A newly built replacement ride starts with its default name.

Names accept up to 28 Unicode characters and 96 UTF-8 bytes. Leading or trailing spaces are removed on Apply. Empty names, control characters and overlong text are rejected with a notice. Editing is free, works while a ride operates, and takes effect in the rides list and selected ride title. Name labels use the installed Windows UI font when available; uncommon scripts may depend on its glyph coverage. Save the park to retain names. The primary and backup slots may each contain different names if you saved between edits.

While the name field is active, ordinary game shortcuts do not trigger. **Ctrl+A** selects the draft, **Backspace** removes a character, **Enter** applies it, and **Esc** leaves the editor. Select another ride to edit its name separately.

## Plant scenery

Use **Build → Tree** or **Build → Flowers**, then point to an open visible grass or sand tile and click. Both are free, decorative, single-tile objects. Placement follows the tile's ground height and is refused on water, occupied or obscured tiles. The limit is 512 planted scenery objects per park, within the existing 4,096 object save limit. The existing decorative landscape remains separate from player-built objects.

Use **Build → Demolish [X]** on a planted object, or inspect/select it and use the normal removal control. A removed tile can be planted again. Raising/lowering land must still obey object-clearance rules. Trees and flower beds do not change guests, finances, path connectivity or ride ratings in this stage.

## Save and move to another PC

Park saves now use strict schema/content **24/24**. Earlier saves from versions 1–23 load through explicit migration; version 23 gains empty ride-name records and preserves all other fields. Both new scenery kinds are saved as normal surface objects. Save slots live in `%LOCALAPPDATA%\Parkbound\saves`, outside the shared source checkout. Close the game before copying the primary and `.bak` files together. The app ZIP carries the game and assets; players do not need Rust or RollerCoaster Tycoon files.

For the exact shared OneDrive path, Git branch, package identity, checks and remaining work, see [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md) and [the stage report](RIDE_NAMES_SCENERY_REPORT.md).
