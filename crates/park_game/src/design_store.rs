//! A local named design library, independent of park saves and editor history.
//!
//! Fixed filenames never include a design name. Writers take a separate lock,
//! reread the catalog, and flush before same-directory replacement. A damaged
//! catalog is never replaced implicitly: close every Parkbound instance and
//! restore `track-designs.json.bak` manually. A crash can leave the `.lock` file;
//! remove that exact lock only after closing all instances. This is a local
//! single-user store, not protection against hostile concurrent path changes.

use park_sim::TrackDesign;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fmt,
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) const MAX_TRACK_DESIGNS: usize = 32;
const MAX_CATALOG_BYTES: usize = 8 * 1024 * 1024;
const CATALOG_NAME: &str = "track-designs.json";
const BACKUP_NAME: &str = "track-designs.json.bak";
const LOCK_NAME: &str = "track-designs.json.lock";
const FORMAT: &str = "parkbound-track-design-library";
static UNIQUE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub(super) struct DesignStore {
    directory: PathBuf,
}

#[derive(Debug)]
pub(super) enum DesignStoreError {
    DuplicateName,
    Full,
    MissingDesign,
    Changed,
    InvalidDirectory {
        path: PathBuf,
    },
    UnsafePath {
        path: PathBuf,
    },
    NotRegularFile {
        path: PathBuf,
    },
    Busy {
        path: PathBuf,
    },
    TooLarge {
        path: PathBuf,
    },
    InvalidCatalog {
        reason: String,
    },
    UnsupportedVersion,
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    RollbackFailed {
        original: Box<DesignStoreError>,
        rollback: Box<DesignStoreError>,
        recovery: Option<PathBuf>,
    },
}

impl DesignStoreError {
    pub(super) fn summary(&self) -> &'static str {
        match self {
            Self::DuplicateName => "A design with this name already exists",
            Self::Full => "Design library is full (32 designs)",
            Self::MissingDesign => "Selected design no longer exists",
            Self::Changed => "Selected design changed; refresh and try again",
            Self::InvalidDirectory { .. } => "Design folder unavailable",
            Self::UnsafePath { .. } => "Design path contains a link",
            Self::NotRegularFile { .. } => "Design path is not a file",
            Self::Busy { .. } => "Design library locked by another instance",
            Self::TooLarge { .. } => "Design library exceeds size limit",
            Self::InvalidCatalog { .. } => "Design library is invalid; see log for recovery",
            Self::UnsupportedVersion => "Design library version is unsupported",
            Self::Io { .. } => "Could not read or write design library",
            Self::RollbackFailed { .. } => "Design recovery needed; see log",
        }
    }
}

impl fmt::Display for DesignStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateName
            | Self::Full
            | Self::MissingDesign
            | Self::Changed
            | Self::UnsupportedVersion => f.write_str(self.summary()),
            Self::InvalidDirectory { path } => write!(
                f,
                "design directory must be an absolute directory without file components: {}",
                path.display()
            ),
            Self::UnsafePath { path } => write!(
                f,
                "refusing symbolic link or reparse point: {}",
                path.display()
            ),
            Self::NotRegularFile { path } => {
                write!(f, "design path is not a regular file: {}", path.display())
            }
            Self::Busy { path } => write!(
                f,
                "design library is locked at {}; after a crash, close all Parkbound instances before removing this lock",
                path.display()
            ),
            Self::TooLarge { path } => write!(
                f,
                "{} exceeds the {MAX_CATALOG_BYTES}-byte design library limit",
                path.display()
            ),
            Self::InvalidCatalog { reason } => write!(
                f,
                "invalid design library: {reason}; close all Parkbound instances and restore {BACKUP_NAME} manually if available"
            ),
            Self::Io {
                action,
                path,
                source,
            } => write!(f, "could not {action} {}: {source}", path.display()),
            Self::RollbackFailed {
                original,
                rollback,
                recovery,
            } => {
                write!(f, "{original}; backup rollback also failed: {rollback}")?;
                if let Some(path) = recovery {
                    write!(f, "; original backup retained at {}", path.display())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for DesignStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::RollbackFailed { original, .. } => Some(original),
            _ => None,
        }
    }
}

fn io_error(action: &'static str, path: &Path, source: io::Error) -> DesignStoreError {
    DesignStoreError::Io {
        action,
        path: path.to_owned(),
        source,
    }
}

fn invalid(reason: impl ToString) -> DesignStoreError {
    DesignStoreError::InvalidCatalog {
        reason: reason.to_string().chars().take(512).collect(),
    }
}

fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    format: String,
    version: u32,
    // Keep each design's original JSON intact for its strict codec, including
    // duplicate-key validation. A serde_json::Value intermediate loses keys.
    #[serde(deserialize_with = "bounded_designs")]
    designs: Vec<String>,
}

fn bounded_designs<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    struct Designs;
    impl<'de> serde::de::Visitor<'de> for Designs {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "at most {MAX_TRACK_DESIGNS} encoded track designs")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut designs = Vec::new();
            while let Some(design) = seq.next_element::<String>()? {
                if designs.len() == MAX_TRACK_DESIGNS {
                    return Err(serde::de::Error::custom("too many track designs"));
                }
                if design.len() > park_sim::MAX_TRACK_DESIGN_BYTES {
                    return Err(serde::de::Error::custom(
                        "encoded track design exceeds size limit",
                    ));
                }
                designs.push(design);
            }
            Ok(designs)
        }
    }
    deserializer.deserialize_seq(Designs)
}

fn decode(bytes: &[u8]) -> Result<Vec<TrackDesign>, DesignStoreError> {
    if bytes.iter().find(|byte| !byte.is_ascii_whitespace()) != Some(&b'{') {
        return Err(invalid("the catalog must be a JSON object"));
    }
    let catalog: Catalog = serde_json::from_slice(bytes).map_err(invalid)?;
    if catalog.format != FORMAT {
        return Err(invalid("unrecognized format"));
    }
    if catalog.version != 1 {
        return Err(DesignStoreError::UnsupportedVersion);
    }
    let mut names = BTreeSet::new();
    let mut designs = Vec::with_capacity(catalog.designs.len());
    for json in catalog.designs {
        let design = TrackDesign::from_json(&json).map_err(invalid)?;
        if !names.insert(name_key(design.name())) {
            return Err(invalid("duplicate design names"));
        }
        designs.push(design);
    }
    designs.sort_by_key(|design| name_key(design.name()));
    Ok(designs)
}

fn encode(designs: &[TrackDesign]) -> Result<Vec<u8>, DesignStoreError> {
    let catalog = Catalog {
        format: FORMAT.to_owned(),
        version: 1,
        designs: designs
            .iter()
            .map(TrackDesign::to_json)
            .collect::<Result<_, _>>()
            .map_err(invalid)?,
    };
    let bytes = serde_json::to_vec_pretty(&catalog).map_err(invalid)?;
    // Validate the entire replacement before creating any staging file.
    decode(&bytes)?;
    Ok(bytes)
}

impl DesignStore {
    pub(super) fn new(save_directory: PathBuf) -> Self {
        Self {
            directory: save_directory.join("track-designs"),
        }
    }

    pub(super) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(super) fn catalog_path(&self) -> PathBuf {
        self.directory.join(CATALOG_NAME)
    }

    pub(super) fn list(&self) -> Result<Vec<TrackDesign>, DesignStoreError> {
        if !self.check_directory(false)? {
            return Ok(Vec::new());
        }
        self.read_catalog().map(|(_, designs)| designs)
    }

    pub(super) fn load(&self, name: &str) -> Result<TrackDesign, DesignStoreError> {
        let key = name_key(name);
        self.list()?
            .into_iter()
            .find(|design| name_key(design.name()) == key)
            .ok_or(DesignStoreError::MissingDesign)
    }

    pub(super) fn save_new(&self, design: &TrackDesign) -> Result<(), DesignStoreError> {
        self.mutate(|designs| {
            if designs
                .iter()
                .any(|entry| name_key(entry.name()) == name_key(design.name()))
            {
                return Err(DesignStoreError::DuplicateName);
            }
            if designs.len() == MAX_TRACK_DESIGNS {
                return Err(DesignStoreError::Full);
            }
            designs.push(design.clone());
            Ok(())
        })
    }

    pub(super) fn replace_exact(
        &self,
        expected: &TrackDesign,
        replacement: &TrackDesign,
    ) -> Result<(), DesignStoreError> {
        self.mutate(|designs| {
            let index = designs
                .iter()
                .position(|design| name_key(design.name()) == name_key(expected.name()))
                .ok_or(DesignStoreError::MissingDesign)?;
            if designs[index] != *expected {
                return Err(DesignStoreError::Changed);
            }
            if designs.iter().enumerate().any(|(other, design)| {
                other != index && name_key(design.name()) == name_key(replacement.name())
            }) {
                return Err(DesignStoreError::DuplicateName);
            }
            designs[index] = replacement.clone();
            Ok(())
        })
    }

    pub(super) fn delete_exact(&self, expected: &TrackDesign) -> Result<(), DesignStoreError> {
        self.mutate(|designs| {
            let index = designs
                .iter()
                .position(|design| name_key(design.name()) == name_key(expected.name()))
                .ok_or(DesignStoreError::MissingDesign)?;
            if designs[index] != *expected {
                return Err(DesignStoreError::Changed);
            }
            designs.remove(index);
            Ok(())
        })
    }

    fn mutate(
        &self,
        operation: impl FnOnce(&mut Vec<TrackDesign>) -> Result<(), DesignStoreError>,
    ) -> Result<(), DesignStoreError> {
        self.check_directory(true)?;
        let _lock = LibraryLock::acquire(&self.directory)?;
        let (previous, mut designs) = self.read_catalog()?;
        operation(&mut designs)?;
        designs.sort_by_key(|design| name_key(design.name()));
        let bytes = encode(&designs)?;
        if previous.as_deref() == Some(bytes.as_slice()) {
            return Ok(());
        }
        let primary = self.catalog_path();
        let backup = self.directory.join(BACKUP_NAME);
        check_file(&primary)?;
        check_file(&backup)?;
        let mut pending = StagedFile::write(&self.directory, "catalog", &bytes)?;
        if let Some(previous) = previous {
            let old_backup = read_file(&backup)?;
            let mut rollback = old_backup
                .as_deref()
                .map(|bytes| StagedFile::write(&self.directory, "rollback", bytes))
                .transpose()?;
            let mut pending_backup = StagedFile::write(&self.directory, "backup", &previous)?;
            pending_backup.replace(&backup)?;
            if let Err(original) = pending.replace(&primary) {
                let restored = if let Some(rollback) = &mut rollback {
                    rollback.replace(&backup)
                } else {
                    fs::remove_file(&backup)
                        .map_err(|error| io_error("undo new design backup", &backup, error))
                };
                if let Err(rollback_error) = restored {
                    return Err(DesignStoreError::RollbackFailed {
                        original: Box::new(original),
                        rollback: Box::new(rollback_error),
                        recovery: rollback.as_mut().map(StagedFile::retain),
                    });
                }
                return Err(original);
            }
        } else {
            pending.replace(&primary)?;
        }
        Ok(())
    }

    fn read_catalog(&self) -> Result<(Option<Vec<u8>>, Vec<TrackDesign>), DesignStoreError> {
        match read_file(&self.catalog_path())? {
            Some(bytes) => {
                let designs = decode(&bytes)?;
                Ok((Some(bytes), designs))
            }
            None => {
                if read_file(&self.directory.join(BACKUP_NAME))?.is_some() {
                    return Err(invalid("catalog is missing while its backup exists"));
                }
                Ok((None, Vec::new()))
            }
        }
    }

    fn check_directory(&self, create: bool) -> Result<bool, DesignStoreError> {
        if !self.directory.is_absolute() {
            return Err(DesignStoreError::InvalidDirectory {
                path: self.directory.clone(),
            });
        }
        let ancestors: Vec<_> = self.directory.ancestors().collect();
        for path in ancestors.into_iter().rev() {
            let metadata = match fs::symlink_metadata(path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound && create => {
                    match fs::create_dir(path) {
                        Ok(()) => {}
                        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                        Err(error) => return Err(io_error("create design directory", path, error)),
                    }
                    fs::symlink_metadata(path)
                        .map_err(|error| io_error("inspect design directory", path, error))?
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
                Err(error) => return Err(io_error("inspect design directory", path, error)),
            };
            if is_link(&metadata) {
                return Err(DesignStoreError::UnsafePath {
                    path: path.to_owned(),
                });
            }
            if !metadata.is_dir() {
                return Err(DesignStoreError::InvalidDirectory {
                    path: path.to_owned(),
                });
            }
        }
        Ok(true)
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

fn check_file(path: &Path) -> Result<(), DesignStoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if is_link(&metadata) => Err(DesignStoreError::UnsafePath {
            path: path.to_owned(),
        }),
        Ok(metadata) if !metadata.is_file() => Err(DesignStoreError::NotRegularFile {
            path: path.to_owned(),
        }),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("inspect design file", path, error)),
    }
}

fn read_file(path: &Path) -> Result<Option<Vec<u8>>, DesignStoreError> {
    check_file(path)?;
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error("open design library", path, error)),
    };
    let metadata = file
        .metadata()
        .map_err(|error| io_error("inspect open design file", path, error))?;
    if !metadata.is_file() {
        return Err(DesignStoreError::NotRegularFile {
            path: path.to_owned(),
        });
    }
    read_limited(file, metadata.len(), path).map(Some)
}

fn read_limited(
    mut reader: impl Read,
    initial_length: u64,
    path: &Path,
) -> Result<Vec<u8>, DesignStoreError> {
    if initial_length > MAX_CATALOG_BYTES as u64 {
        return Err(DesignStoreError::TooLarge {
            path: path.to_owned(),
        });
    }
    let mut bytes = Vec::with_capacity((initial_length as usize).min(64 * 1024));
    let mut chunk = [0_u8; 8192];
    loop {
        let count = match reader.read(&mut chunk) {
            Ok(0) => return Ok(bytes),
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(io_error("read design library", path, error)),
        };
        if count > MAX_CATALOG_BYTES - bytes.len() {
            return Err(DesignStoreError::TooLarge {
                path: path.to_owned(),
            });
        }
        bytes.try_reserve_exact(count).map_err(|error| {
            io_error("allocate library buffer for", path, io::Error::other(error))
        })?;
        bytes.extend_from_slice(&chunk[..count]);
    }
}

struct LibraryLock {
    path: PathBuf,
}

impl LibraryLock {
    fn acquire(directory: &Path) -> Result<Self, DesignStoreError> {
        let path = directory.join(LOCK_NAME);
        check_file(&path)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                if error.kind() == io::ErrorKind::AlreadyExists {
                    DesignStoreError::Busy { path: path.clone() }
                } else {
                    io_error("acquire design lock", &path, error)
                }
            })?;
        let lock = Self { path };
        let written = writeln!(
            file,
            "Parkbound design lock; process {}",
            std::process::id()
        )
        .and_then(|()| file.sync_all());
        drop(file);
        written.map_err(|error| io_error("write design lock", &lock.path, error))?;
        Ok(lock)
    }
}

impl Drop for LibraryLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

struct StagedFile {
    path: PathBuf,
    owned: bool,
}

impl StagedFile {
    fn write(directory: &Path, label: &str, bytes: &[u8]) -> Result<Self, DesignStoreError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for _ in 0..16 {
            let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
            let path = directory.join(format!(
                ".parkbound-design-{label}-{}-{timestamp}-{sequence}.tmp",
                std::process::id()
            ));
            let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(io_error("create temporary design library", &path, error));
                }
            };
            let pending = Self { path, owned: true };
            let written = file.write_all(bytes).and_then(|()| file.sync_all());
            drop(file);
            written.map_err(|error| {
                io_error("write and flush design library", &pending.path, error)
            })?;
            return Ok(pending);
        }
        Err(io_error(
            "create unique temporary design library in",
            directory,
            io::Error::new(io::ErrorKind::AlreadyExists, "temporary name collisions"),
        ))
    }

    fn replace(&mut self, destination: &Path) -> Result<(), DesignStoreError> {
        check_file(destination)?;
        fs::rename(&self.path, destination)
            .map_err(|error| io_error("replace design library", destination, error))?;
        self.owned = false;
        Ok(())
    }

    fn retain(&mut self) -> PathBuf {
        self.owned = false;
        self.path.clone()
    }
}

impl Drop for StagedFile {
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
                "Parkbound design tests – forêt {} {timestamp} {sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn store(&self) -> DesignStore {
            DesignStore::new(self.0.join("my park – été"))
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
                        .starts_with("Parkbound design tests ")
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

    fn different_design(name: &str) -> TrackDesign {
        let spec = TrackBuildKind::Straight.spec(TilePos { x: 3, y: 4 }, 8, TrackDirection::East);
        TrackDesign::from_draft(name, &[spec]).unwrap()
    }

    fn disk(store: &DesignStore) -> (Vec<u8>, Option<Vec<u8>>) {
        (
            fs::read(store.catalog_path()).unwrap(),
            read_file(&store.directory.join(BACKUP_NAME)).unwrap(),
        )
    }

    fn assert_clean(store: &DesignStore) {
        let names: Vec<_> = fs::read_dir(store.directory())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert!(
            !names
                .iter()
                .any(|name| name.to_string_lossy().starts_with(".parkbound-design-"))
        );
        assert!(!store.directory.join(LOCK_NAME).exists());
    }

    #[test]
    fn unopened_library_is_empty_without_creating_any_directories() {
        let scratch = Scratch::new();
        let store = scratch.store();
        assert!(store.list().unwrap().is_empty());
        assert!(matches!(
            store.load("missing"),
            Err(DesignStoreError::MissingDesign)
        ));
        assert!(!store.directory.parent().unwrap().exists());
        assert!(DesignStore::new(PathBuf::from("relative")).list().is_err());
    }

    #[test]
    fn reopening_unicode_library_retains_sorted_exact_designs_and_safe_names() {
        let scratch = Scratch::new();
        let store = scratch.store();
        let forest = design("Forêt 🎢");
        let traversal = different_design("../../harmless");
        store.save_new(&forest).unwrap();
        store.save_new(&traversal).unwrap();
        let reopened = scratch.store();
        assert_eq!(
            reopened.list().unwrap(),
            vec![traversal.clone(), forest.clone()]
        );
        assert_eq!(reopened.load("  FORÊT 🎢 ").unwrap(), forest);
        assert_eq!(reopened.load("../../harmless").unwrap(), traversal);
        assert_eq!(fs::read_dir(reopened.directory()).unwrap().count(), 2);
        assert!(!scratch.0.join("harmless").exists());
        assert_clean(&reopened);
    }

    #[test]
    fn duplicate_capacity_and_missing_selection_fail_without_changing_catalogs() {
        let scratch = Scratch::new();
        let store = scratch.store();
        store.save_new(&design("Forêt")).unwrap();
        let before = disk(&store);
        assert!(matches!(
            store.save_new(&design("  FORÊT  ")),
            Err(DesignStoreError::DuplicateName)
        ));
        assert!(matches!(
            store.delete_exact(&design("missing")),
            Err(DesignStoreError::MissingDesign)
        ));
        assert!(matches!(
            store.replace_exact(&design("missing"), &design("Replacement")),
            Err(DesignStoreError::MissingDesign)
        ));
        assert_eq!(disk(&store), before);
        for i in 1..MAX_TRACK_DESIGNS {
            store
                .save_new(&different_design(&format!("Design {i}")))
                .unwrap();
        }
        let before = disk(&store);
        assert!(matches!(
            store.save_new(&design("One too many")),
            Err(DesignStoreError::Full)
        ));
        assert_eq!(disk(&store), before);
        assert_clean(&store);
    }

    #[test]
    fn explicit_replace_can_rename_and_delete_keeps_prior_catalog_backup() {
        let scratch = Scratch::new();
        let store = scratch.store();
        store.save_new(&design("Original")).unwrap();
        store.save_new(&design("Other")).unwrap();
        let before = disk(&store);
        assert!(matches!(
            store.replace_exact(&design("Original"), &design("OTHER")),
            Err(DesignStoreError::DuplicateName)
        ));
        assert_eq!(disk(&store), before);
        let changed = different_design("Renamed");
        store.replace_exact(&design("Original"), &changed).unwrap();
        assert_eq!(store.load("renamed").unwrap(), changed);
        assert!(store.load("Original").is_err());
        let before_delete = fs::read(store.catalog_path()).unwrap();
        store.delete_exact(&design("Other")).unwrap();
        assert_eq!(store.list().unwrap(), vec![changed]);
        assert_eq!(
            fs::read(store.directory.join(BACKUP_NAME)).unwrap(),
            before_delete
        );
        assert_clean(&store);
    }

    #[test]
    fn stale_confirmations_cannot_overwrite_or_delete_another_instances_changes() {
        let scratch = Scratch::new();
        let store = scratch.store();
        let original = design("Shared");
        store.save_new(&original).unwrap();
        let changed = different_design("Shared");
        let other_instance = scratch.store();
        other_instance.replace_exact(&original, &changed).unwrap();
        let before = disk(&store);
        assert!(matches!(
            store.replace_exact(&original, &design("Renamed")),
            Err(DesignStoreError::Changed)
        ));
        assert!(matches!(
            store.delete_exact(&original),
            Err(DesignStoreError::Changed)
        ));
        assert_eq!(disk(&store), before);
        assert_eq!(store.load("Shared").unwrap(), changed);
        store.delete_exact(&changed).unwrap();
        assert!(other_instance.list().unwrap().is_empty());
        assert_clean(&store);
    }

    #[test]
    fn damaged_catalog_never_gets_silently_replaced_or_auto_recovered() {
        let scratch = Scratch::new();
        let store = scratch.store();
        store.save_new(&design("First")).unwrap();
        store.save_new(&design("Second")).unwrap();
        fs::write(store.catalog_path(), b"{broken").unwrap();
        let before = disk(&store);
        assert!(matches!(
            store.list(),
            Err(DesignStoreError::InvalidCatalog { .. })
        ));
        assert!(store.save_new(&design("Third")).is_err());
        assert!(
            store
                .replace_exact(&design("First"), &design("Replacement"))
                .is_err()
        );
        assert!(store.delete_exact(&design("First")).is_err());
        assert_eq!(disk(&store), before);
        fs::remove_file(store.catalog_path()).unwrap();
        assert!(matches!(
            store.list(),
            Err(DesignStoreError::InvalidCatalog { .. })
        ));
        assert!(store.save_new(&design("Third")).is_err());
        assert!(!store.catalog_path().exists());
        assert_eq!(
            fs::read(store.directory.join(BACKUP_NAME)).unwrap(),
            before.1.unwrap()
        );
        assert_clean(&store);
    }

    #[test]
    fn catalog_and_nested_design_validation_rejects_unknown_duplicate_and_oversized_data() {
        assert!(decode(br#"["parkbound-track-design-library",1,[]]"#).is_err());
        let valid = design("First").to_json().unwrap();
        let catalog = |designs: Vec<String>| {
            serde_json::to_vec(&Catalog {
                format: FORMAT.to_owned(),
                version: 1,
                designs,
            })
            .unwrap()
        };
        assert!(decode(&catalog(vec![valid.clone(), valid.clone()])).is_err());
        assert!(decode(&catalog(vec![valid.clone(); MAX_TRACK_DESIGNS + 1])).is_err());
        assert!(
            decode(&catalog(vec![
                " ".repeat(park_sim::MAX_TRACK_DESIGN_BYTES + 1)
            ]))
            .is_err()
        );
        let duplicate_key = valid.replacen('{', "{\"name\":\"Duplicate\",", 1);
        assert!(decode(&catalog(vec![duplicate_key])).is_err());
        let mut value: serde_json::Value = serde_json::from_slice(&catalog(vec![valid])).unwrap();
        value["version"] = 999.into();
        assert!(matches!(
            decode(&serde_json::to_vec(&value).unwrap()),
            Err(DesignStoreError::UnsupportedVersion)
        ));
        value["version"] = 1.into();
        value["unknown"] = true.into();
        assert!(decode(&serde_json::to_vec(&value).unwrap()).is_err());
        assert!(decode(br#"{"format":"parkbound-track-design-library","version":1,"version":1,"designs":[]}"#).is_err());
    }

    #[test]
    fn lock_and_non_file_backup_refuse_mutation_without_losing_existing_data() {
        let scratch = Scratch::new();
        let store = scratch.store();
        store.save_new(&design("First")).unwrap();
        let before = disk(&store);
        let lock = store.directory.join(LOCK_NAME);
        fs::write(&lock, b"another process").unwrap();
        assert!(matches!(
            store.save_new(&design("Second")),
            Err(DesignStoreError::Busy { .. })
        ));
        assert_eq!(disk(&store), before);
        assert_eq!(fs::read(&lock).unwrap(), b"another process");
        fs::remove_file(lock).unwrap();
        fs::create_dir(store.directory.join(BACKUP_NAME)).unwrap();
        assert!(matches!(
            store.save_new(&design("Second")),
            Err(DesignStoreError::NotRegularFile { .. })
        ));
        assert_eq!(fs::read(store.catalog_path()).unwrap(), before.0);
        assert_clean(&store);
    }

    #[test]
    fn reader_is_bounded_even_when_a_file_grows_after_metadata() {
        let path = Path::new("track-designs.json");
        assert!(matches!(
            read_limited(io::repeat(b' '), 0, path),
            Err(DesignStoreError::TooLarge { .. })
        ));
        assert!(matches!(
            read_limited(io::empty(), MAX_CATALOG_BYTES as u64 + 1, path),
            Err(DesignStoreError::TooLarge { .. })
        ));
        let scratch = Scratch::new();
        let store = scratch.store();
        store.save_new(&design("First")).unwrap();
        File::create(store.catalog_path())
            .unwrap()
            .set_len(MAX_CATALOG_BYTES as u64 + 1)
            .unwrap();
        assert!(matches!(
            store.list(),
            Err(DesignStoreError::TooLarge { .. })
        ));
        assert!(matches!(
            store.save_new(&design("Second")),
            Err(DesignStoreError::TooLarge { .. })
        ));
        assert_eq!(
            fs::metadata(store.catalog_path()).unwrap().len(),
            MAX_CATALOG_BYTES as u64 + 1
        );
        assert_clean(&store);
    }

    #[cfg(windows)]
    #[test]
    fn failed_final_replace_restores_existing_backup_and_primary() {
        use std::os::windows::fs::OpenOptionsExt;
        let scratch = Scratch::new();
        let store = scratch.store();
        store.save_new(&design("First")).unwrap();
        store.save_new(&design("Second")).unwrap();
        let before = disk(&store);
        // Permit reading but not deleting/replacing the primary name.
        let held = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(store.catalog_path())
            .unwrap();
        assert!(store.save_new(&design("Third")).is_err());
        assert_eq!(disk(&store), before);
        drop(held);
        assert_clean(&store);
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_directory_and_catalog_are_rejected() {
        use std::os::unix::fs::symlink;
        let scratch = Scratch::new();
        let store = scratch.store();
        store.save_new(&design("First")).unwrap();
        let alias = scratch.0.join("alias");
        symlink(store.directory.parent().unwrap(), &alias).unwrap();
        assert!(matches!(
            DesignStore::new(alias).list(),
            Err(DesignStoreError::UnsafePath { .. })
        ));
        let real = store.directory.join("real.json");
        fs::rename(store.catalog_path(), &real).unwrap();
        symlink(&real, store.catalog_path()).unwrap();
        assert!(matches!(
            store.list(),
            Err(DesignStoreError::UnsafePath { .. })
        ));
        assert!(store.save_new(&design("Second")).is_err());
    }
}
