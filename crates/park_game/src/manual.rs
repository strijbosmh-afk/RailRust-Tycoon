//! Opens the bundled, offline player manual without sending player data anywhere.
use std::path::{Path, PathBuf};

const DOCUMENT: &str = "docs/HANDLEIDING_NL.html";

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
    let path = locate(&executable, &workspace)
        .ok_or("Handleiding ontbreekt. Pak het volledige spelpakket uit, inclusief de map docs.")?;
    launch(&path)
}

#[cfg(windows)]
fn launch(path: &Path) -> Result<(), String> {
    // Pass one literal path argument; never interpret a shell command or a URL.
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Open docs/HANDLEIDING_NL.html handmatig: {error}"))
}

#[cfg(not(windows))]
fn launch(_path: &Path) -> Result<(), String> {
    Err(
        "Open docs/HANDLEIDING_NL.html in je browser. De automatische opener vereist Windows."
            .into(),
    )
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
