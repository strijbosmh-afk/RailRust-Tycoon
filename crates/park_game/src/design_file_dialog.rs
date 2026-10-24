//! Native, owner-modal file selection. File validation and persistence live elsewhere.
//!
//! Call on the live game window's thread and keep that window alive until return.
//! The adapter never changes the working directory or rewrites the accepted path.

use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DialogRequest {
    Import,
    Export { suggested_name: String },
}

pub(crate) fn choose_file(request: DialogRequest, owner: usize) -> Result<Option<PathBuf>, String> {
    #[cfg(windows)]
    {
        platform::choose_file(request, owner)
    }
    #[cfg(not(windows))]
    {
        let _ = (request, owner);
        Err("Track design file dialogs are available on Windows only.".into())
    }
}

#[cfg(windows)]
mod platform {
    use std::{
        ffi::OsString,
        os::windows::ffi::{OsStrExt, OsStringExt},
        path::{Path, PathBuf},
    };

    use windows::{
        Win32::{
            Foundation::{ERROR_CANCELLED, HWND, RPC_E_CHANGED_MODE},
            System::{
                Com::{
                    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
                    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
                },
                Threading::GetCurrentThreadId,
            },
            UI::{
                Shell::{
                    Common::COMDLG_FILTERSPEC, FOS_ALLOWMULTISELECT, FOS_DONTADDTORECENT,
                    FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_NOCHANGEDIR, FOS_NOREADONLYRETURN,
                    FOS_NOTESTFILECREATE, FOS_OVERWRITEPROMPT, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS,
                    FOS_STRICTFILETYPES, FileOpenDialog, FileSaveDialog, IFileDialog,
                    SIGDN_FILESYSPATH,
                },
                WindowsAndMessaging::{GetWindowThreadProcessId, IsWindow},
            },
        },
        core::{Error, HRESULT, HSTRING, PWSTR, w},
    };

    use super::DialogRequest;

    const SUFFIX: &str = ".parktrack.json";

    pub(super) fn choose_file(
        request: DialogRequest,
        owner: usize,
    ) -> Result<Option<PathBuf>, String> {
        let owner = checked_owner(owner)?;
        // Declared before every COM interface so it is dropped after they are released,
        // on this same thread, including all early returns and cancellation.
        let _apartment = ComApartment::initialize()?;
        let exporting = matches!(request, DialogRequest::Export { .. });
        let class = if exporting {
            &FileSaveDialog
        } else {
            &FileOpenDialog
        };

        // SAFETY: COM is initialized on this thread. Generated Windows bindings own
        // the returned interface. It is neither shared nor moved to another thread.
        let dialog: IFileDialog = unsafe { CoCreateInstance(class, None, CLSCTX_INPROC_SERVER) }
            .map_err(|error| windows_error("open the file dialog", error))?;

        // SAFETY: the interface is live on its COM apartment. Static wide strings
        // and the filter array remain valid through each call. The dialog copies
        // its configuration; the initial filename also remains alive during its call.
        unsafe {
            let mut options = dialog
                .GetOptions()
                .map_err(|error| windows_error("read file dialog options", error))?;
            options &= !(FOS_ALLOWMULTISELECT | FOS_PICKFOLDERS);
            options |=
                FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST | FOS_NOCHANGEDIR | FOS_DONTADDTORECENT;
            options |= if exporting {
                FOS_OVERWRITEPROMPT
                    | FOS_NOREADONLYRETURN
                    | FOS_NOTESTFILECREATE
                    | FOS_STRICTFILETYPES
            } else {
                FOS_FILEMUSTEXIST
            };
            dialog
                .SetOptions(options)
                .map_err(|error| windows_error("configure the file dialog", error))?;
            dialog
                .SetFileTypes(&[COMDLG_FILTERSPEC {
                    pszName: w!("Parkbound track designs (*.parktrack.json)"),
                    pszSpec: w!("*.parktrack.json"),
                }])
                .map_err(|error| windows_error("set the track design filter", error))?;
            dialog
                .SetFileTypeIndex(1)
                .map_err(|error| windows_error("select the track design filter", error))?;
            match request {
                DialogRequest::Import => {
                    dialog
                        .SetTitle(w!("Import track design"))
                        .map_err(|error| windows_error("set the file dialog title", error))?;
                    dialog
                        .SetOkButtonLabel(w!("Import"))
                        .map_err(|error| windows_error("set the file dialog action", error))?;
                }
                DialogRequest::Export { suggested_name } => {
                    dialog
                        .SetTitle(w!("Export track design"))
                        .map_err(|error| windows_error("set the file dialog title", error))?;
                    dialog
                        .SetOkButtonLabel(w!("Export"))
                        .map_err(|error| windows_error("set the file dialog action", error))?;
                    dialog
                        .SetDefaultExtension(w!("parktrack.json"))
                        .map_err(|error| windows_error("set the track design extension", error))?;
                    let initial_name = HSTRING::from(suggested_export_name(&suggested_name));
                    dialog
                        .SetFileName(&initial_name)
                        .map_err(|error| windows_error("set the suggested filename", error))?;
                }
            }

            // Show runs its own modal message loop with the actual game window as
            // owner. Do not substitute a desktop/null owner or call from a worker.
            if !was_accepted(dialog.Show(Some(owner)))? {
                return Ok(None);
            }
            let item = dialog
                .GetResult()
                .map_err(|error| windows_error("read the selected file", error))?;
            let filename = TaskMemString(
                item.GetDisplayName(SIGDN_FILESYSPATH)
                    .map_err(|error| windows_error("read the selected file path", error))?,
            );
            if filename.0.is_null() {
                return Err("The file dialog returned no file path.".into());
            }
            // GetDisplayName supplies a live NUL-terminated allocation. OsString
            // preserves all Windows UTF-16, including unpaired surrogate code units.
            let path = selected_path(filename.0.as_wide(), exporting)?;
            Ok(Some(path))
        }
    }

    fn checked_owner(owner: usize) -> Result<HWND, String> {
        let hwnd = HWND(owner as *mut std::ffi::c_void);
        let mut process = 0;
        // SAFETY: these Win32 queries accept opaque window handles, including
        // invalid handles. The process pointer is a writable local u32.
        let valid = unsafe {
            owner != 0
                && IsWindow(Some(hwnd)).as_bool()
                && GetWindowThreadProcessId(hwnd, Some(&mut process)) == GetCurrentThreadId()
                && process == std::process::id()
        };
        if valid {
            Ok(hwnd)
        } else {
            Err("Open the file dialog from the live game window's main thread.".into())
        }
    }

    struct ComApartment;

    impl ComApartment {
        fn initialize() -> Result<Self, String> {
            // SAFETY: the reserved parameter is null and this call initializes
            // only the current thread. Both S_OK and S_FALSE require balancing.
            let result =
                unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
            if result == RPC_E_CHANGED_MODE {
                return Err(
                    "The game thread cannot open a Windows file dialog in its current COM mode."
                        .into(),
                );
            }
            result
                .ok()
                .map_err(|error| windows_error("initialize the Windows file dialog", error))?;
            Ok(Self)
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            // SAFETY: created only after successful CoInitializeEx, never moved
            // out of choose_file, and released after that call's COM interfaces.
            unsafe { CoUninitialize() };
        }
    }

    struct TaskMemString(PWSTR);

    impl Drop for TaskMemString {
        fn drop(&mut self) {
            // SAFETY: GetDisplayName transfers one CoTaskMem allocation to us;
            // this guard frees it exactly once after all borrowed reads finish.
            unsafe { CoTaskMemFree(Some(self.0.as_ptr().cast())) };
        }
    }

    fn was_accepted(result: windows::core::Result<()>) -> Result<bool, String> {
        match result {
            Ok(()) => Ok(true),
            Err(error) if error.code() == HRESULT::from_win32(ERROR_CANCELLED.0) => Ok(false),
            Err(error) => Err(windows_error("show the file dialog", error)),
        }
    }

    fn windows_error(action: &str, error: Error) -> String {
        format!(
            "Could not {action} (Windows error 0x{:08X}).",
            error.code().0 as u32
        )
    }

    fn selected_path(wide: &[u16], exporting: bool) -> Result<PathBuf, String> {
        if wide.is_empty() || wide.contains(&0) {
            return Err("The file dialog returned an invalid file path.".into());
        }
        let path = PathBuf::from(OsString::from_wide(wide));
        if !path.is_absolute() || path.file_name().is_none() {
            return Err("The file dialog must select an absolute file path.".into());
        }
        if exporting && !has_export_suffix(&path) {
            return Err("Choose a filename ending in .parktrack.json and export again.".into());
        }
        // Never append/replace an extension here: native overwrite confirmation
        // has already approved this exact destination, not a derived path.
        Ok(path)
    }

    fn has_export_suffix(path: &Path) -> bool {
        let Some(name) = path.file_name() else {
            return false;
        };
        let name: Vec<u16> = name.encode_wide().collect();
        let suffix = SUFFIX.as_bytes();
        name.len() > suffix.len()
            && name[name.len() - suffix.len()..]
                .iter()
                .zip(suffix)
                .all(|(&actual, &expected)| {
                    actual == u16::from(expected)
                        || actual == u16::from(expected.to_ascii_uppercase())
                })
    }

    fn suggested_export_name(name: &str) -> String {
        let name = name.trim();
        let stem = name
            .get(name.len().saturating_sub(SUFFIX.len())..)
            .filter(|ending| ending.eq_ignore_ascii_case(SUFFIX))
            .map_or(name, |_| &name[..name.len() - SUFFIX.len()]);
        let cleaned: String = stem
            .chars()
            .take(80)
            .map(|character| {
                if character.is_control() || "<>:\"/\\|?*".contains(character) {
                    '_'
                } else {
                    character
                }
            })
            .collect();
        let stem = cleaned.trim_matches([' ', '.']);
        let stem = if stem.is_empty() {
            "Track design"
        } else {
            stem
        };
        // Windows device names remain reserved when followed by an extension.
        let base = stem.split('.').next().unwrap_or(stem).to_ascii_uppercase();
        let reserved = matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || base
                .strip_prefix("COM")
                .or_else(|| base.strip_prefix("LPT"))
                .is_some_and(|number| {
                    matches!(
                        number,
                        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                    )
                });
        format!("{}{stem}{SUFFIX}", if reserved { "_" } else { "" })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn selected_path_preserves_unicode_and_windows_code_units() {
            let mut wide: Vec<u16> = r"C:\Users\Zoë\設計 🚂 ".encode_utf16().collect();
            wide.push(0xD800); // Valid Windows path unit without a Unicode scalar.
            wide.extend(".parktrack.json".encode_utf16());
            let path = selected_path(&wide, true).unwrap();
            assert_eq!(path.as_os_str().encode_wide().collect::<Vec<_>>(), wide);
            assert!(
                selected_path(
                    &r"\\server\設計\Loop.parktrack.json"
                        .encode_utf16()
                        .collect::<Vec<_>>(),
                    true
                )
                .is_ok()
            );
        }

        #[test]
        fn export_accepts_only_the_confirmed_path_with_the_right_suffix() {
            let path = r"C:\designs\Loop.PARKTRACK.JSON";
            assert_eq!(
                selected_path(&path.encode_utf16().collect::<Vec<_>>(), true).unwrap(),
                PathBuf::from(path)
            );
            for path in [
                r"C:\designs\Loop.json",
                r"C:\designs\Loop.parktrack.json.txt",
                r"C:\designs\.parktrack.json",
                r"relative\Loop.parktrack.json",
                r"C:Loop.parktrack.json",
                r"C:\",
                "",
                "C:\\designs\\Loop.parktrack.json\0hidden",
            ] {
                assert!(
                    selected_path(&path.encode_utf16().collect::<Vec<_>>(), true).is_err(),
                    "{path:?}"
                );
            }
            // Import does not confuse an extension filter with content validation.
            assert!(
                selected_path(
                    &r"C:\designs\renamed.json"
                        .encode_utf16()
                        .collect::<Vec<_>>(),
                    false
                )
                .is_ok()
            );
        }

        #[test]
        fn suggestions_are_single_safe_unicode_filenames_with_one_suffix() {
            assert_eq!(
                suggested_export_name("  Zoë's 設計 🚂  "),
                "Zoë's 設計 🚂.parktrack.json"
            );
            assert_eq!(
                suggested_export_name("Loop.PARKTRACK.JSON"),
                "Loop.parktrack.json"
            );
            assert_eq!(
                suggested_export_name("../a\\b:*?\0"),
                "_a_b____.parktrack.json"
            );
            assert_eq!(
                suggested_export_name("...  "),
                "Track design.parktrack.json"
            );
            for name in [
                "CON",
                "prn",
                "AUX",
                "NUL",
                "COM1",
                "LPT9",
                "COM¹",
                "con.backup",
            ] {
                assert!(suggested_export_name(name).starts_with('_'), "{name}");
            }
            assert_eq!(suggested_export_name("Station"), "Station.parktrack.json");
        }

        #[test]
        fn cancellation_is_distinct_from_native_failure_without_opening_a_dialog() {
            assert_eq!(was_accepted(Ok(())), Ok(true));
            assert_eq!(
                was_accepted(Err(Error::from_hresult(HRESULT::from_win32(
                    ERROR_CANCELLED.0
                )))),
                Ok(false)
            );
            assert!(was_accepted(Err(Error::from_hresult(RPC_E_CHANGED_MODE))).is_err());
            assert!(choose_file(DialogRequest::Import, 0).is_err());
        }
    }
}
