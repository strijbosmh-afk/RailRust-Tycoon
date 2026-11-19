//! Opens the bundled, offline player manual without sending player data anywhere.
use std::path::{Path, PathBuf};

const DOCUMENT: &str = "docs/PLAYER_MANUAL.html";

fn locate(executable: &Path, workspace: &Path) -> Option<PathBuf> {
    executable
        .parent()
        .map(|parent| parent.join(DOCUMENT))
        .into_iter()
        .chain(std::iter::once(workspace.join(DOCUMENT)))
        .find(|path| path.is_file())
}

pub(crate) fn open() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = locate(&executable, &workspace).ok_or(
        "Player manual missing. Extract the complete game package, including the docs folder.",
    )?;
    launch(&path)
}

#[cfg(windows)]
fn launch(path: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
        core::{PCWSTR, w},
    };

    // Explorer interprets forward slashes as switches and can silently open
    // Documents instead. Use the file association API with a literal UTF-16
    // filename, preserving spaces/non-ASCII names and native separators.
    let filename: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .map(|unit| {
            if unit == u16::from(b'/') {
                u16::from(b'\\')
            } else {
                unit
            }
        })
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: the NUL-terminated filename lives through this synchronous call;
    // the optional argument and working-directory pointers are null.
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(filename.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    let code = result.0 as usize;
    if code <= 32 {
        Err(format!(
            "Windows could not open the manual (code {code}). Open docs/PLAYER_MANUAL.html manually."
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn launch(_path: &Path) -> Result<(), String> {
    Err("Open docs/PLAYER_MANUAL.html in your browser. Automatic opening requires Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn bundled_manual_wins_and_source_is_a_development_fallback() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "parkbound manual {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let package = root.join("package with spaces");
        let workspace = root.join("source");
        let executable = package.join("Parkbound.exe");
        assert_eq!(locate(&executable, &workspace), None);
        std::fs::create_dir_all(workspace.join("docs")).unwrap();
        std::fs::write(workspace.join(DOCUMENT), "source manual").unwrap();
        assert_eq!(
            locate(&executable, &workspace),
            Some(workspace.join(DOCUMENT))
        );
        std::fs::create_dir_all(package.join("docs")).unwrap();
        std::fs::write(package.join(DOCUMENT), "packaged manual").unwrap();
        assert_eq!(
            locate(&executable, &workspace),
            Some(package.join(DOCUMENT))
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
