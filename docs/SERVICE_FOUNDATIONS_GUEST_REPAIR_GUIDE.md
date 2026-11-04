# Service foundations and guest path repair

Elevated Food, Drink, Toilet and Rest facilities now show timber posts down to the terrain below. Their serving deck remains at its built height. If another object crosses the same tile below the facility, the posts are omitted there so they do not pass through that crossing. This is a visual change; access and construction still use the facility's actual height.

To help a lost visitor, open **Guests [L]**, select **Lost**, choose a visitor and press **Repair path**. The game centers the visitor's last valid position, marks the visitor, and opens the normal path-building tool at their height. If the path beneath them was removed, the tool also selects the previous path's slope and shows its tile coordinates. Rebuild that piece and connect it to the public network. If the visitor is standing on an isolated path, join that path back to the public network. The visitor resumes through the usual simulation recovery rule after a valid connection is restored.

The button is labelled **Lost only** when no lost visitor is selected and leaves the park unchanged. Preparing a repair does not place a path or spend money. The normal build checks and price apply when you click the map. The **Services [M]** panel also shows how many current visitors have a strong need for the selected facility type; this is a live count, not a promise that all of them will reach or afford it.

Park saves remain strict schema/content **25/25**. The primary and `.bak` files are in `%LOCALAPPDATA%\Parkbound\saves`; move both with the game closed when continuing on another PC. See [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md) for the shared OneDrive checkout and build steps.
