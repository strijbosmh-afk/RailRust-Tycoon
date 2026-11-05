# Open and close the park — 0.64.0

Open **Guests [L] → Visitor flow**. Select an ordinary public path and choose **Set entry**; it must connect to another public path to admit visitors. Choose **Toggle arrivals** to enable the existing one-guest-per-30-simulation-seconds flow. The park starts open, but arrivals remain opt-in.

Choose **Open / close park** to close the park. The status line changes to **CLOSED**, no new guests arrive, and everyone currently in the park begins leaving. Guests already riding or using a service finish that visit first, keeping their ticket or service purchase, receipt and review. They then walk to the selected entry and leave. If the entry is removed, departures wait until a new entry is set; an isolated entry can also prevent guests from reaching it. Closing does not close individual rides or services, so their operating costs and maintenance still follow their own rules.

Choose **Open / close park** again to reopen. A full 30-second arrival interval starts at reopening if arrivals are enabled and the entry is connected. Guests who were already leaving continue toward the exit. **Toggle arrivals** independently controls whether new guests can enter while the park is open. Space pauses the simulation; controls and status can still be changed while paused.

The park status is saved with the park in strict schema/content **34/34**. Version-33 and earlier parks load as open, preserving their previous behavior. Older game versions cannot read version-34 saves. For the same park on another Windows PC, close the game and transfer both the primary and `.bak` files from `%LOCALAPPDATA%\Parkbound\saves`. See [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md) for the shared OneDrive source path and build steps.
