# Phase 9 — campaign, sandbox and editors

Version 0.88.0 adds a six-scenario campaign, configurable sandbox starts and portable scenario authoring. Park saves use schema/content 51/51. The campaign profile remains separate from individual parks.

## Campaign

Choose **Campaign** at the title screen. Meadow and Boardwalk retain their original rules. **Later campaign** cycles through the four additional scenarios. The screen shows the selected scenario and its unlock status. Each award unlocks the next scenario. Existing Meadow and Boardwalk awards carry forward.

| Scenario | Objectives | Time limit | Starting budget |
| --- | --- | --- | --- |
| Meadow Challenge | 20 new visitors, 5 completed rides, $750 cash | None | $1,000 |
| Boardwalk Sprint | 24 new visitors, 12 completed rides, $900 cash | 10 minutes | $1,000 |
| Cedar Valley | 30 new visitors and 65 guest satisfaction | 20 minutes | $2,000 |
| Research Ridge | Complete a funded research project and 20 rides | 20 minutes | $2,000 |
| Lakeside Services | Install 3 facilities, serve 12 customers and welcome 30 visitors | 20 minutes | $2,000 |
| Grand Festival | Welcome 60 visitors, earn $150 in sales and reach 65 satisfaction | 20 minutes | $2,500 |

Meadow requires free park entry and Boardwalk requires free rides. The later scenarios permit flexible pricing. Build and connect the starter coaster's access, provide services and manage prices, research and staff using the normal park controls. A deadline failure remains saved with the park. Load an earlier park save or start a new attempt to retry. Earned profile awards remain available across attempts and restarts.

## Sandbox starts

Choose **New Park → Sandbox**. The options select map size, flat or rolling terrain on larger empty maps, starting budget, funded or fully available research, visitor arrivals, loan access and the charging policy. Supported sizes are 32, 64, 128 and 256 tiles per side. The 32-tile option retains the flat starter park. Larger sizes begin with an empty park.

The starting-budget choices are $500, $1,000, $5,000 and $10,000. The Create Park button remains at the top and Enter selects it on opening the screen. Existing park saves retain their recorded budget and rules. An empty map needs a public path and visitor entry before arrivals can enter.

## Scenario authoring

Open **Explore → Manage → Scenarios and editor**, or use **Scenarios and editor** at the title screen. The workspace has separate Library, Goals and rules, and Available content pages.

1. Start from a sandbox with the desired map, or select **New from this park**. The editor keeps a copy of the original park.
2. Select **Build park** to use the existing terrain, water, land, path, scenery, ride and facility tools. The editor banner identifies the current mode. Reopen the workspace to continue authoring.
3. Edit the name and description. Select a field, then type. Backspace edits the selected field.
4. Add up to four distinct goals. Select a goal, cycle its metric and adjust its target. Choose a deadline or remove the deadline. Supported metrics include arrivals, rides, customers served, sales, cash, satisfaction, facilities, park value and funded research projects.
5. Set the opening budget, charging rule, research availability, loan policy and visitor flow. Available content controls which ride models and facilities can be built. Existing buildings remain in the template.
6. Select **Save scenario**. Its file contains the validated park layout, land rights, starting rides, finances, research, content restrictions and objective definition.
7. Select **Test scenario** to play a separate test run. **Return to editor** restores the authoring state. Test play does not earn campaign awards.
8. **Return to park**, followed by confirmation, restores the original park. Saved scenarios remain in the library.

The budget is the opening financial allocation. Existing construction spending still belongs to the park. Reducing the budget below its recorded spending is rejected. Scenario play preserves integer-cent finance reconciliation and quoted guest fares.

## Scenario library and sharing

The library searches by name, pages through saved scenarios and shows their description, map dimensions, goals and compatibility. Select **Play selected** and confirm to start it. **Edit selected** opens a copy for authoring after confirmation. Rename and archive require confirmation. Archived files remain in the profile's scenario archive. Damaged or unsupported files stay visible with an error and can be archived.

**Export** opens a Windows file chooser for `.parkscenario.json`. **Import** validates a chosen scenario before adding it to the library. Names are display text and do not become internal paths. Version 1 scenario files embed a bounded, validated park save and custom objectives. Custom files cannot carry campaign award markers. Importing into another profile makes the scenario available there without requiring campaign progress or another game's files.

The existing track-design library retains its search, previews, import/export and confirmation workflow. **Rename selected** changes the saved name while preserving the selected design's geometry. Use **Update selected** when replacing geometry from the current draft.

## Save and profile compatibility

Schema 51 adds optional play rules and custom challenge state. Older saves keep their original $1,000 opening budget and behavior. The reader continues to support earlier park schemas. Campaign profile version 2 adds four awards and reads version 1 profiles while preserving their existing medals. Unknown versions and malformed data are rejected.

Scenario files use bounded reads and checked park loading. Library writes are staged and replaced atomically, previous versions have backups, and generated file identities are retained after archiving. Native export asks before replacing an existing destination. The editor and test copies do not overwrite the source park's saved slot.

## Verification

The `write-scenario-tools` and `verify-scenario-tools` native proof modes exercise the six campaign starts, actual objective completion, persistent awards, editor export/import to a clean profile and exact custom-scenario continuation. The release tests cover budget reconciliation, content restrictions, loans, save migration, malformed files, editor restoration and library operations. Capture scenes include `scenario-editor` and `scenario-library`.

Final results and the Windows package are recorded in [resume section 57](../RESUME_DEVELOPMENT.md#57-phase-9-campaign-sandbox-and-editors--0880). Development-PC timings are excluded from performance acceptance.
