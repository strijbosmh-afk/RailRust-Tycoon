//! Uses the executable's icon resources for the native window and taskbar.
use bevy::prelude::*;

pub(crate) fn configure(app: &mut App) {
    #[cfg(windows)]
    app.init_non_send_resource::<MainThread>()
        .add_systems(Update, apply);
    #[cfg(not(windows))]
    let _ = app;
}

#[cfg(windows)]
#[derive(Default)]
struct MainThread(std::marker::PhantomData<std::rc::Rc<()>>);

#[cfg(windows)]
fn apply(
    _main_thread: NonSend<MainThread>,
    windows: Query<&bevy::window::RawHandleWrapper, Added<bevy::window::RawHandleWrapper>>,
) {
    for wrapper in &windows {
        if let raw_window_handle::RawWindowHandle::Win32(window) = wrapper.get_window_handle()
            && let Err(error) =
                set_icons(windows::Win32::Foundation::HWND(window.hwnd.get() as *mut _))
        {
            warn!("Could not set the Parkbound window icon: {error}");
        }
    }
}

#[cfg(windows)]
fn set_icons(window: windows::Win32::Foundation::HWND) -> windows::core::Result<()> {
    use windows::{
        Win32::{
            Foundation::{HINSTANCE, LPARAM, WPARAM},
            System::LibraryLoader::GetModuleHandleW,
            UI::{
                HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
                WindowsAndMessaging::{
                    IMAGE_ICON, LR_SHARED, LoadImageW, SM_CXICON, SM_CXSMICON, SendMessageW,
                    WM_SETICON,
                },
            },
        },
        core::PCWSTR,
    };
    // These APIs run on the window's main thread. LR_SHARED keeps ownership of
    // the loaded resource handles with Windows for the module's lifetime.
    unsafe {
        let module = HINSTANCE(GetModuleHandleW(None)?.0);
        let dpi = GetDpiForWindow(window);
        for (kind, metric) in [(1, SM_CXICON), (0, SM_CXSMICON)] {
            let size = GetSystemMetricsForDpi(metric, dpi);
            let icon = LoadImageW(
                Some(module),
                // Win32 represents integer resource IDs as low-valued pointers.
                PCWSTR(std::ptr::without_provenance(1)),
                IMAGE_ICON,
                size,
                size,
                LR_SHARED,
            )?;
            SendMessageW(
                window,
                WM_SETICON,
                Some(WPARAM(kind)),
                Some(LPARAM(icon.0 as isize)),
            );
        }
    }
    Ok(())
}
