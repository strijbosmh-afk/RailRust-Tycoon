# Phase 10 — sound, guidance and accessibility

Version 0.89.0 continues the Windows game with original sound and player assistance.

## Start and learn

New parks open a seven-step first-park guide. Follow the camera introduction, extend a public path, place a Carousel, connect its entrance queue and exit, welcome a visitor, save, and return through the park library. Each construction step observes your actual park. Next becomes available when its action succeeds. Skip guide ends it immediately. Help → First park guide starts it again in the current park.

The guide opens the existing construction and management controls. A queue joins the ride entrance to one public path endpoint. The exit needs its own public connection. Use Rides → Access for the missing connection reason and suggested build locations. For coasters, use the preview’s next height, choose an ascent or descent, complete the circuit and run its test before opening. The Help page also explains terrain edges, facilities and staff.

## Keyboard and visibility

Settings → Controls & accessibility is available at the title and from the park. Help also opens it. Tab and Shift+Tab move the visible focus outline between controls. Enter activates the focused control. Long panels scroll to the selected control. Escape returns to the previous screen.

Eighteen gameplay actions have configurable keys. Click an action repeatedly to cycle through unused keys. Assignments are unique. Camera arrows, zoom, Escape, F1 help, F2 navigation, F10 sound, F11 safe window recovery and modifier-based editing commands retain their established roles. The panel shows every current assignment. Toolbar and navigation labels update for the corresponding action. Default shortcut reference text remains marked as default.

High contrast uses dark and light text against panel backgrounds. Existing status words, shapes and error reasons remain visible. Reduced motion freezes decorative water and weather animation, the intro pulse and guest walking poses. Moving vehicles and guest positions continue to show the simulation. Interface scale remains under Display settings.

## Sound

F10 opens independent Master, Music, Ambience, Rides, Alerts & interface, and Guest & staff reactions levels. Music alternates between two original themes. Rain joins the ambience in rainy park weather. Completed rides trigger a short guest response. Resolved staff incidents trigger a staff response. Mute and focus fading apply to the whole mix.

Meaningful build, removal, ride closure, save and reaction cues also appear as brief captions. Captions work while muted or when no output device is available. Controls & accessibility can disable them.

The audio worker retries after an unavailable or stalled output device. Reconnect audio device reopens the Windows default output without restarting the park. This also lets you pick up a changed Windows default output.

## Saved settings and continuation

Park saves remain schema/content 51/51. Accessibility preferences use a separate version-1 `accessibility-settings.json` beside the existing settings in the active save directory. The sound settings format is version 2. Version-1 sound files retain all five levels and gain a 60% reactions level. Invalid preference files produce defaults with a visible explanation. Settings writes use the shared lock and atomic replacement.

The normal location is `%LOCALAPPDATA%\Parkbound\saves`. A `--save-dir` launch isolates all preferences and park files. Transfer settings and park libraries with the game closed. Technical cross-PC instructions and the shared local source path are in `RESUME_DEVELOPMENT.md`.
