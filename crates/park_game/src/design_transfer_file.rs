//! Strict design file transfer, separate from park saves and the design catalog.
//!
//! Export overwrite consent belongs to the caller's native Save dialog. Files
//! are flushed and atomically renamed from a unique sibling, never truncated in
//! place. Links/reparse points are refused, including directory ancestors. Like
//! the local save store, this does not defend against hostile ancestor changes.

use park_sim::{MAX_TRACK_DESIGN_BYTES, TrackDesign, TrackDesignError};
use std::{
    fmt,
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const EXTENSION: &str = ".parktrack.json";
static UNIQUE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) enum TransferFileError {
    InvalidPath {
        path: PathBuf,
    },
    InvalidExtension,
    UnsafePath {
        path: PathBuf,
    },
    NotRegularFile {
        path: PathBuf,
    },
    TooLarge,
    InvalidUtf8(std::str::Utf8Error),
    InvalidDesign(TrackDesignError),
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    CleanupFailed {
        original: Box<TransferFileError>,
        path: PathBuf,
        source: io::Error,
    },
}

impl TransferFileError {
    pub(crate) fn summary(&self) -> &'static str {
        match self {
            Self::InvalidPath { .. } => "Choose an absolute, ordinary file path",
            Self::InvalidExtension => "Export filename must end in .parktrack.json",
            Self::UnsafePath { .. } => "Design file path contains a link",
            Self::NotRegularFile { .. } => "Design path is not a regular file",
            Self::TooLarge => "Design file exceeds the 128 KiB limit",
            Self::InvalidUtf8(_) => "Design file is not valid UTF-8 text",
            Self::InvalidDesign(_) => "Design file is invalid or unsupported",
            Self::Io { .. } => "Could not read or write design file",
            Self::CleanupFailed { .. } => "Export failed; temporary file cleanup needed",
        }
    }
}

impl fmt::Display for TransferFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { path }
            | Self::UnsafePath { path }
            | Self::NotRegularFile { path } => write!(f, "{}: {}", self.summary(), path.display()),
            Self::InvalidExtension | Self::TooLarge => f.write_str(self.summary()),
            Self::InvalidUtf8(source) => write!(f, "{}: {source}", self.summary()),
            Self::InvalidDesign(source) => source.fmt(f),
            Self::Io {
                action,
                path,
                source,
            } => {
                write!(f, "could not {action} {}: {source}", path.display())
            }
            Self::CleanupFailed {
                original,
                path,
                source,
            } => write!(
                f,
                "{original}; could not remove temporary export {}: {source}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for TransferFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUtf8(source) => Some(source),
            Self::InvalidDesign(source) => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::CleanupFailed { original, .. } => Some(original),
            _ => None,
        }
    }
}

fn io_error(action: &'static str, path: &Path, source: io::Error) -> TransferFileError {
    TransferFileError::Io {
        action,
        path: path.to_owned(),
        source,
    }
}

/// Read only the independent version-1 format; the filename is not trusted as a type tag.
pub(crate) fn read_design(path: &Path) -> Result<TrackDesign, TransferFileError> {
    check_path(path, false)?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Open a leaf reparse point itself rather than following it. Keep the
        // opened file from being renamed/deleted while its bounded read runs.
        options.custom_flags(0x0020_0000).share_mode(0x0000_0003);
    }
    let file = options
        .open(path)
        .map_err(|error| io_error("open design file", path, error))?;
    let metadata = file
        .metadata()
        .map_err(|error| io_error("inspect open design file", path, error))?;
    check_metadata(path, &metadata)?;
    let bytes = read_limited(file, metadata.len(), path)?;
    let text = std::str::from_utf8(&bytes).map_err(TransferFileError::InvalidUtf8)?;
    TrackDesign::from_json(text).map_err(TransferFileError::InvalidDesign)
}

/// Atomically write an approved export destination, preserving its old bytes on failure.
pub(crate) fn write_design(path: &Path, design: &TrackDesign) -> Result<(), TransferFileError> {
    check_extension(path)?;
    check_path(path, true)?;
    let json = design.to_json().map_err(TransferFileError::InvalidDesign)?;
    let directory = path
        .parent()
        .ok_or_else(|| TransferFileError::InvalidPath {
            path: path.to_owned(),
        })?;
    let (mut pending, mut file) = StagedExport::create(directory)?;
    let written = file
        .write_all(json.as_bytes())
        .and_then(|()| file.sync_all());
    drop(file);
    let result = written
        .map_err(|error| io_error("write and flush temporary design", &pending.path, error))
        .and_then(|()| check_path(path, true))
        .and_then(|()| {
            // Same-directory rename replaces a file atomically on supported
            // Windows/Unix targets. Never remove or truncate the destination.
            fs::rename(&pending.path, path)
                .map_err(|error| io_error("replace exported design", path, error))
        });
    match result {
        Ok(()) => {
            pending.owned = false;
            Ok(())
        }
        Err(error) => Err(pending.cleanup_after(error)),
    }
}

/// A safe suggested basename; a design's Unicode display name remains unchanged.
pub(crate) fn default_filename(design: &TrackDesign) -> String {
    let replaced: String = design
        .name()
        .chars()
        .map(|character| {
            if invalid_character(character) {
                '_'
            } else {
                character
            }
        })
        .collect();
    let trimmed = replaced.trim_matches([' ', '.']);
    let stem = if trimmed.is_empty() {
        "Track design"
    } else {
        trimmed
    };
    let prefix = if reserved_device(stem) { "_" } else { "" };
    format!("{prefix}{stem}{EXTENSION}")
}

fn check_extension(path: &Path) -> Result<(), TransferFileError> {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(EXTENSION))
    {
        Ok(())
    } else {
        Err(TransferFileError::InvalidExtension)
    }
}

fn invalid_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
        )
}

fn reserved_device(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || ["COM", "LPT"].iter().any(|prefix| {
        stem.strip_prefix(*prefix).is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    })
}

fn check_path(path: &Path, missing_allowed: bool) -> Result<(), TransferFileError> {
    let invalid = || TransferFileError::InvalidPath {
        path: path.to_owned(),
    };
    if !path.is_absolute() || path.file_name().is_none() {
        return Err(invalid());
    }
    for component in path.components() {
        match component {
            Component::Normal(name) => {
                let name = name.to_str().ok_or_else(invalid)?;
                if name.chars().any(invalid_character)
                    || name.ends_with([' ', '.'])
                    || reserved_device(name)
                {
                    return Err(invalid());
                }
            }
            Component::ParentDir | Component::CurDir => return Err(invalid()),
            #[cfg(windows)]
            Component::Prefix(prefix) => {
                use std::path::Prefix;
                if !matches!(
                    prefix.kind(),
                    Prefix::Disk(_)
                        | Prefix::VerbatimDisk(_)
                        | Prefix::UNC(_, _)
                        | Prefix::VerbatimUNC(_, _)
                ) {
                    return Err(invalid());
                }
            }
            _ => {}
        }
    }
    let ancestors: Vec<_> = path.parent().ok_or_else(invalid)?.ancestors().collect();
    for ancestor in ancestors.into_iter().rev() {
        let metadata = fs::symlink_metadata(ancestor)
            .map_err(|error| io_error("inspect design directory", ancestor, error))?;
        if is_link(&metadata) {
            return Err(TransferFileError::UnsafePath {
                path: ancestor.to_owned(),
            });
        }
        if !metadata.is_dir() {
            return Err(TransferFileError::InvalidPath {
                path: ancestor.to_owned(),
            });
        }
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) => check_metadata(path, &metadata),
        Err(error) if missing_allowed && error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("inspect design file", path, error)),
    }
}

fn is_link(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn check_metadata(path: &Path, metadata: &Metadata) -> Result<(), TransferFileError> {
    if is_link(metadata) {
        Err(TransferFileError::UnsafePath {
            path: path.to_owned(),
        })
    } else if !metadata.is_file() {
        Err(TransferFileError::NotRegularFile {
            path: path.to_owned(),
        })
    } else {
        Ok(())
    }
}

fn read_limited(
    mut reader: impl Read,
    initial_length: u64,
    path: &Path,
) -> Result<Vec<u8>, TransferFileError> {
    if initial_length > MAX_TRACK_DESIGN_BYTES as u64 {
        return Err(TransferFileError::TooLarge);
    }
    let mut bytes = Vec::with_capacity((initial_length as usize).min(8192));
    let mut chunk = [0_u8; 8192];
    loop {
        let count = match reader.read(&mut chunk) {
            Ok(0) => return Ok(bytes),
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(io_error("read design file", path, error)),
        };
        if count > MAX_TRACK_DESIGN_BYTES - bytes.len() {
            return Err(TransferFileError::TooLarge);
        }
        bytes.try_reserve_exact(count).map_err(|error| {
            io_error(
                "allocate design read buffer for",
                path,
                io::Error::other(error),
            )
        })?;
        bytes.extend_from_slice(&chunk[..count]);
    }
}

struct StagedExport {
    path: PathBuf,
    owned: bool,
}

impl StagedExport {
    fn create(directory: &Path) -> Result<(Self, File), TransferFileError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for _ in 0..16 {
            let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
            let path = directory.join(format!(
                ".parkbound-export-{}-{timestamp}-{sequence}.tmp",
                std::process::id()
            ));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => return Ok((Self { path, owned: true }, file)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(io_error("create temporary design file", &path, error)),
            }
        }
        Err(io_error(
            "create unique temporary design file in",
            directory,
            io::Error::new(io::ErrorKind::AlreadyExists, "temporary name collisions"),
        ))
    }

    fn cleanup_after(mut self, original: TransferFileError) -> TransferFileError {
        let result = fs::remove_file(&self.path);
        self.owned = false;
        match result {
            Ok(()) => original,
            Err(error) if error.kind() == io::ErrorKind::NotFound => original,
            Err(source) => TransferFileError::CleanupFailed {
                original: Box::new(original),
                path: self.path.clone(),
                source,
            },
        }
    }
}

impl Drop for StagedExport {
    fn drop(&mut self) {
        if self.owned {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use park_sim::{Park, TilePos, TrackBuildKind, TrackDirection};

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "Parkbound transfer tests – forêt {} {timestamp} {sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn file(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
        fn assert_no_temporary_files(&self) {
            assert!(fs::read_dir(&self.0).unwrap().all(|entry| {
                !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".parkbound-export-")
            }));
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            if let (Ok(path), Ok(root)) = (
                fs::canonicalize(&self.0),
                fs::canonicalize(std::env::temp_dir()),
            ) && path.parent() == Some(root.as_path())
                && path.file_name().is_some_and(|name| {
                    name.to_string_lossy()
                        .starts_with("Parkbound transfer tests ")
                })
            {
                let _ = fs::remove_dir_all(path);
            }
        }
    }

    fn design(name: &str) -> TrackDesign {
        let mut park = Park::new(32, 32);
        park.install_train_fixture().unwrap();
        TrackDesign::from_draft(name, park.circuit().unwrap().specs()).unwrap()
    }

    fn unfinished(name: &str) -> TrackDesign {
        let spec = TrackBuildKind::Straight.spec(TilePos { x: 3, y: 4 }, 8, TrackDirection::East);
        TrackDesign::from_draft(name, &[spec]).unwrap()
    }

    #[test]
    fn exports_round_trip_exact_unicode_geometry_and_replace_only_approved_file() {
        let scratch = Scratch::new();
        let source = design("Forêt 🎢");
        let before = source.clone();
        let exported = scratch.file(&default_filename(&source));
        let unrelated = scratch.file("park.park.json");
        fs::write(&unrelated, b"player park data").unwrap();
        write_design(&exported, &source).unwrap();
        assert_eq!(read_design(&exported).unwrap(), source);
        assert_eq!(
            fs::read(&exported).unwrap(),
            source.to_json().unwrap().as_bytes()
        );
        let replacement = unfinished("Nouvelle idée");
        write_design(&exported, &replacement).unwrap();
        assert_eq!(read_design(&exported).unwrap(), replacement);
        assert_eq!(source, before);
        assert_eq!(fs::read(unrelated).unwrap(), b"player park data");
        let uppercase = scratch.file("UPPER.PARKTRACK.JSON");
        write_design(&uppercase, &source).unwrap();
        assert_eq!(read_design(&uppercase).unwrap(), source);
        scratch.assert_no_temporary_files();
    }

    #[test]
    fn filenames_keep_unicode_and_sanitize_separators_devices_and_trailing_dots() {
        for (name, expected) in [
            ("Forêt 🎢", "Forêt 🎢.parktrack.json"),
            ("../ride\\bad:*?<>|\"", "_ride_bad_______.parktrack.json"),
            ("NUL", "_NUL.parktrack.json"),
            ("con.txt", "_con.txt.parktrack.json"),
            ("COM¹", "_COM¹.parktrack.json"),
            ("lpt9", "_lpt9.parktrack.json"),
            ("Trail...", "Trail.parktrack.json"),
            ("...", "Track design.parktrack.json"),
        ] {
            assert_eq!(default_filename(&design(name)), expected, "{name}");
        }
        let name = "🎢".repeat(32);
        let filename = default_filename(&design(&name));
        assert_eq!(filename, format!("{name}{EXTENSION}"));
        assert_eq!(Path::new(&filename).components().count(), 1);
    }

    #[test]
    fn invalid_json_utf8_versions_and_park_files_are_read_only_refusals() {
        let scratch = Scratch::new();
        let path = scratch.file("incoming.json");
        let valid = design("Source").to_json().unwrap();
        let invalids = [
            b"not JSON".to_vec(),
            vec![0xff, 0xfe],
            valid
                .replacen('{', "{\"name\":\"Duplicate\",", 1)
                .into_bytes(),
            valid.replacen('{', "{\"unknown\":true,", 1).into_bytes(),
            valid
                .replace("\"version\": 1", "\"version\": 999")
                .into_bytes(),
            b"{\"format\":\"parkbound-save\",\"version\":15}".to_vec(),
            b"{\"format\":\"parkbound-track-design-library\",\"version\":1,\"designs\":[]}"
                .to_vec(),
        ];
        for bytes in invalids {
            fs::write(&path, &bytes).unwrap();
            assert!(read_design(&path).is_err());
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
        fs::write(&path, &valid).unwrap();
        assert_eq!(read_design(&path).unwrap(), design("Source"));
        assert_eq!(fs::read(&path).unwrap(), valid.as_bytes());
        scratch.assert_no_temporary_files();
    }

    #[test]
    fn reads_are_bounded_at_the_limit_even_if_the_input_grows() {
        let scratch = Scratch::new();
        let path = scratch.file("large.parktrack.json");
        let source = unfinished("Small");
        let mut bytes = source.to_json().unwrap().into_bytes();
        bytes.resize(MAX_TRACK_DESIGN_BYTES, b' ');
        fs::write(&path, &bytes).unwrap();
        assert_eq!(read_design(&path).unwrap(), source);
        bytes.push(b' ');
        fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            read_design(&path),
            Err(TransferFileError::TooLarge)
        ));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(matches!(
            read_limited(io::repeat(b' '), 0, &path),
            Err(TransferFileError::TooLarge)
        ));
        assert!(matches!(
            read_limited(io::empty(), MAX_TRACK_DESIGN_BYTES as u64 + 1, &path),
            Err(TransferFileError::TooLarge)
        ));
    }

    #[test]
    fn unsafe_nonregular_and_wrong_extension_destinations_preserve_existing_files() {
        let scratch = Scratch::new();
        let source = design("Source");
        for name in [
            "park.park.json",
            "track-designs.json",
            "saved.parktrack.json.bak",
            "saved.parktrack.json ",
        ] {
            let path = scratch.file(name);
            // Avoid creating a trailing-space alias on Windows.
            if !name.ends_with(' ') {
                fs::write(&path, b"preserve me").unwrap();
            }
            assert!(matches!(
                write_design(&path, &source),
                Err(TransferFileError::InvalidExtension)
            ));
            if !name.ends_with(' ') {
                assert_eq!(fs::read(path).unwrap(), b"preserve me");
            }
        }
        let directory = scratch.file("directory.parktrack.json");
        fs::create_dir(&directory).unwrap();
        assert!(matches!(
            write_design(&directory, &source),
            Err(TransferFileError::NotRegularFile { .. })
        ));
        assert!(matches!(
            read_design(&directory),
            Err(TransferFileError::NotRegularFile { .. })
        ));
        for path in [
            PathBuf::from("relative.parktrack.json"),
            scratch.file("NUL.parktrack.json"),
            scratch.file("alternate:stream.parktrack.json"),
            scratch.file("../elsewhere.parktrack.json"),
        ] {
            assert!(matches!(
                write_design(&path, &source),
                Err(TransferFileError::InvalidPath { .. })
            ));
            assert!(matches!(
                read_design(&path),
                Err(TransferFileError::InvalidPath { .. })
            ));
        }
        let missing = scratch.file("missing/inside.parktrack.json");
        assert!(write_design(&missing, &source).is_err());
        assert!(!scratch.file("missing").exists());
        scratch.assert_no_temporary_files();
    }

    #[cfg(windows)]
    #[test]
    fn failed_atomic_replace_preserves_destination_and_cleans_only_its_temporary_file() {
        use std::os::windows::fs::OpenOptionsExt;
        let scratch = Scratch::new();
        let path = scratch.file("held.parktrack.json");
        let source = design("Original");
        write_design(&path, &source).unwrap();
        let before = fs::read(&path).unwrap();
        let other_temp = scratch.file(".parkbound-export-other-process.tmp");
        fs::write(&other_temp, b"another export").unwrap();
        let held = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&path)
            .unwrap();
        assert!(write_design(&path, &unfinished("Replacement")).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read(&other_temp).unwrap(), b"another export");
        assert_eq!(fs::read_dir(&scratch.0).unwrap().count(), 2);
        drop(held);
        fs::remove_file(other_temp).unwrap();
        scratch.assert_no_temporary_files();
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_file_and_directory_links_never_redirect_import_or_export() {
        use std::os::unix::fs::symlink;
        let scratch = Scratch::new();
        let source = design("Original");
        let path = scratch.file("original.parktrack.json");
        write_design(&path, &source).unwrap();
        let before = fs::read(&path).unwrap();
        let link = scratch.file("linked.parktrack.json");
        symlink(&path, &link).unwrap();
        let directory_link = scratch.file("linked-directory");
        symlink(&scratch.0, &directory_link).unwrap();
        for path in [link, directory_link.join("original.parktrack.json")] {
            assert!(matches!(
                read_design(&path),
                Err(TransferFileError::UnsafePath { .. })
            ));
            assert!(matches!(
                write_design(&path, &unfinished("Replacement")),
                Err(TransferFileError::UnsafePath { .. })
            ));
        }
        assert_eq!(fs::read(path).unwrap(), before);
        scratch.assert_no_temporary_files();
    }
}
