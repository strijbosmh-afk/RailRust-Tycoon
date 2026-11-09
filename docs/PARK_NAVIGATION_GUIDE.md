# Parkbound grouped park navigation (0.73.0)

Open **Explore [F2]** from the left sidebar, or press **F2** while playing. Choose **Build**, **Manage**, **Inspect**, **Park**, or **Help / Settings**. The number keys **1–5** switch those groups while Explore is open. **Esc** or **F2** returns to the park. The existing direct shortcuts, such as **P** for paths, **L** for guests and **N** for finances, still work.

**Build** opens path, queue, terrain, scenery, ride, service, coaster track and saved design tools. **Manage** leads to finances, staff and grounds, visitor flow and advertising, ride operations and service operations. **Inspect** opens guest, ride and facility details. **Park** opens scenarios, statistics, the normal save action and the pause menu. **Help / Settings** opens controls, sound or display settings directly. Choosing a build tool closes Explore and returns to the map; choosing a report opens its existing management panel.

Guests, Services, Rides and Track Designs windows now stay within the display and scroll vertically when needed. Wide ride and design content wraps into columns at compact sizes. Hover a dialog and use the mouse wheel to reach lower controls. The world does not zoom while a management window is open. A click that closes a dialog does not also build or select underneath it.

The display controls from 0.72.0 remain under Title or Pause → Settings. F11 restores Windowed/Auto/Current display if a choice becomes hard to use. On another PC, copy the whole `%LOCALAPPDATA%\Parkbound\saves` folder after closing the game; display preferences and park saves are separate files there.

## Capture checks

For development screenshots, use `--capture-frame <path> --capture-scene flat-start --capture-panel build|manage|inspect|park|help|guests|services|rides|designs` with one size flag (`--capture-small-window`, `--capture-large-window`, or `--capture-window-size WIDTHxHEIGHT`) and `--capture-ui-scale auto|100|125|150|200`. Supply `--save-dir <isolated-path>` for profile checks. Diagnostic captures do not save player display settings. `--capture-panel` cannot be combined with `--capture-front`.
