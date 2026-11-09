# Parkbound Phase 1: startup and park library

Parkbound now opens with a short original Parkbound intro. Click **Skip intro**, press **Space**, **Enter**, or **Escape**, or let it finish. The title screen has Continue, New Park, Campaign, Load Park, Settings, Credits, and Quit. Continue opens the most recent park with a valid primary save; when none exists, the title explains why it is unavailable. The original diagnostic and capture modes still bypass normal startup unless a front-screen capture is explicitly requested.

## Starting a park

Choose **New Park**, type a name of 1–40 characters, and select Sandbox, Meadow Challenge, or Boardwalk Sprint. Sandbox is always available. Meadow Challenge requires 20 arrivals, five completed rides, and $750 cash, with free entry and no deadline; winning it unlocks Boardwalk. Boardwalk Sprint requires 24 arrivals, 12 completed rides, and $900 cash before 18,000 ticks, with free rides. Its button explains when it is locked. **Campaign** goes straight to the same selector with Meadow chosen. A new park is made only after **Create Park**.

## Saving and managing parks

Click **Menu** in the top bar or press **Escape** when no tool or panel owns that key. The park pauses while the menu is open. It offers Resume, Save, Save As New Park, Load Park, Settings, Return to Title, and Quit. Save As asks for a name and creates an independent copy. F5 saves the active park. F9 loads its primary save; Shift+F9 explicitly loads its backup. A loaded park starts paused; Space or a speed button resumes it. Return to Title, Quit, and any load ask for confirmation if the current park differs from its last save.

**Load Park** lists up to 32 current parks, four per page, and offers primary and backup loading, rename, and Archive with confirmation. Archiving moves a park's folder to `deleted-parks` for manual recovery; it does not erase its files. The library preserves strict park-file validation and a previous valid backup per park. A damaged primary remains visible if its backup is valid; use **Load selected backup**. Load failures show an error screen and leave the live park intact.

The old single-slot primary and backup remain at `%LOCALAPPDATA%\Parkbound\saves\park.park.json` and `.bak` (or the explicit `--save-dir`). In the title's **Load Park** menu, use **Import old single-slot save** or **Import old single-slot backup**. Each makes a separate named library park and leaves the original bytes unchanged. The new library lives under `%LOCALAPPDATA%\Parkbound\saves\parks\park-<generated-id>\`. Campaign awards stay in the root `campaign.json` and `campaign.json.bak` and are shared across parks.

To move play to another PC, close Parkbound and copy the **whole** `%LOCALAPPDATA%\Parkbound\saves` directory, including `parks`, `deleted-parks`, campaign files, and any legacy single-slot files. The OneDrive source checkout alone does not copy AppData. Copying after the game closes avoids partial saves.

## Controls and verification

Title and menu buttons work with the mouse. Use **Tab** or **Up/Down**, **Enter**, and **Escape** for keyboard navigation; the currently selected action is written at the bottom. Type directly in a New Park, Rename, or Save As name field; Backspace removes a character. In the park, Escape first closes an open panel or cancels an active tool; once clear it opens the menu. The Menu button always opens it.

For an isolated fresh-process library check, run the package executable twice with `--save-check write-front --save-dir <new absolute directory>` and then `--save-check verify-front --save-dir <same directory>`. The verifier compares independent slots, prior backup, rename, archive, both unchanged legacy files, and exact saved continuation. The existing `write-campaign` / `verify-campaign` pair covers fresh-process awards. Capture front screens with `--capture-frame <absolute PNG> --capture-front title|new-park|park-library|pause-menu|settings|credits` (one name per process). Add `--capture-small-window` for 1024×680 or `--capture-large-window` for 1920×1080. A full game capture uses the normal `--capture-scene` mechanism.

Park save schema and content stay **39/39**. Phase 2 of the [roadmap](FULL_GAME_ROADMAP.md) will redesign scaling and high-resolution layout. This development PC is not a performance reference.
