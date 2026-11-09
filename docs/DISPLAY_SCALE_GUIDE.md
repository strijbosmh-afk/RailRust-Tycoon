# Parkbound Phase 2 checkpoint: display and interface scale (0.72.0)

## Display settings

Open **Settings** from the title screen or the paused park menu. **Smaller interface** and **Larger interface** cycle through Auto, 100%, 125%, 150%, and 200%. Auto chooses a readable size from the window's logical width. If a requested scale would leave too little map space, the effective scale is capped; Settings shows both the requested choice and the effective percentage.

The window mode choices are Windowed, Borderless, and Fullscreen. The display selector starts at **Current display** and cycles through connected displays. A missing saved display falls back to the primary display. In Windowed mode, selecting a specific display centers the game there. Press **F11** to restore Windowed, Current display, and Auto scale at any time. To bypass a saved fullscreen choice at startup, launch `Parkbound.exe --safe-windowed`; this recovery launch does not overwrite the saved setting until you change or reset it in Settings.

Display preferences save separately in `%LOCALAPPDATA%\Parkbound\saves\display-settings.json`, or under the selected `--save-dir`. They do not change a park save or campaign awards. Copy the whole saves folder after closing the game to carry the preferences to another PC. If the settings file is unreadable or from a future version, the game uses defaults for that session and preserves the file until you explicitly choose **Reset display settings**.

## Layout and controls

Title and park dialogs, Help, and Finance use a shared capped layout. They stay within the window and scroll vertically when their content is taller than the available space. The left sidebar and Build/Track panels also scroll at larger scales. Wheel over a panel scrolls it; wheel over the map still zooms the park. Finance and Rides remain available in the left sidebar when their top-bar shortcuts hide in compact layouts. The map camera, cursor picking, drag and wheel handling use the same effective interface scale as the visible chrome.

This checkpoint starts Phase 2. Other management panels still need the shared layout conversion, grouped navigation, full display matrix, and mixed-DPI native acceptance before the Phase 2 gate can be marked complete. This development PC is not a performance reference.

## Developer captures

Add `--capture-ui-scale auto|100|125|150|200` to a capture command to inspect a requested scale without writing player settings. The existing `--capture-small-window` and `--capture-large-window` modes create 1024×680 and 1920×1080 screenshots. Use `--capture-window-size WIDTHxHEIGHT` for other bounded display checks, up to 3840×2160, including 1366×768, 2560×1440 and 3440×1440. Choose only one window-size flag per capture. Compare title, Settings, Finance, flat park, and map picking at multiple sizes. Diagnostic captures default to 100% for repeatability; normal play defaults to Auto.
