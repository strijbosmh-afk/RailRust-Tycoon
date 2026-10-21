//! Local single-slot persistence. Saves never overwrite files in place.
//!
//! File contents are flushed before same-directory rename. Backup and primary
//! are two separate names, so a failed final rename rolls the backup back; a
//! rollback failure reports and retains its recovery file. This is not a
//! filesystem-wide transaction or a guarantee against hardware power loss.
//!
//! A crash can leave `park.park.json.lock`. Close every Parkbound instance
//! before removing that exact lock to recover. We never guess that it is stale
//! and never sweep temporary files belonging to another save attempt.

use park_sim::{MAX_SAVE_BYTES, Park, SaveError};
use std::{
    ffi::OsString,
    fmt,
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const PRIMARY_NAME: &str = "park.park.json";
const BACKUP_NAME: &str = "park.park.json.bak";
const LOCK_NAME: &str = "park.park.json.lock";
static UNIQUE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub(super) struct SaveStore {
    directory: PathBuf,
}

#[derive(Debug)]
pub(super) enum StoreError {
    MissingLocalAppData,
    InvalidDirectory {
        path: PathBuf,
        reason: &'static str,
    },
    UnsafePath {
        path: PathBuf,
    },
    NotRegularFile {
        path: PathBuf,
    },
    MissingSave {
        path: PathBuf,
    },
    Busy {
        path: PathBuf,
    },
    TooLarge {
        path: PathBuf,
    },
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    InvalidSave {
        path: Option<PathBuf>,
        source: SaveError,
    },
    RollbackFailed {
        original: Box<StoreError>,
        rollback: Box<StoreError>,
        recovery: Option<PathBuf>,
    },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLocalAppData => write!(
                f,
                "LOCALAPPDATA is unavailable; choose an absolute save directory with --save-dir"
            ),
            Self::InvalidDirectory { path, reason } => {
                write!(f, "save directory {} is invalid: {reason}", path.display())
            }
            Self::UnsafePath { path } => write!(
                f,
                "refusing symbolic link or reparse point: {}",
                path.display()
            ),
            Self::NotRegularFile { path } => {
                write!(f, "save path is not a regular file: {}", path.display())
            }
            Self::MissingSave { path } => write!(f, "no saved park exists at {}", path.display()),
            Self::Busy { path } => write!(
                f,
                "save is locked at {}; close other Parkbound instances; after a crash, close all instances before removing this lock",
                path.display()
            ),
            Self::TooLarge { path } => write!(
                f,
                "{} exceeds the {MAX_SAVE_BYTES}-byte save limit",
                path.display()
            ),
            Self::Io {
                action,
                path,
                source,
            } => write!(f, "could not {action} {}: {source}", path.display()),
            Self::InvalidSave {
                path: Some(path),
                source,
            } => write!(f, "invalid saved park at {}: {source}", path.display()),
            Self::InvalidSave { path: None, source } => {
                write!(f, "park could not be saved: {source}")
            }
            Self::RollbackFailed {
                original,
                rollback,
                recovery,
            } => {
                write!(f, "{original}; backup rollback also failed: {rollback}")?;
                if let Some(path) = recovery {
                    write!(f, "; the original backup is retained at {}", path.display())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidSave { source, .. } => Some(source),
            Self::RollbackFailed { original, .. } => Some(original),
            _ => None,
        }
    }
}

impl StoreError {
    pub(super) fn summary(&self) -> &'static str {
        match self {
            Self::MissingLocalAppData | Self::InvalidDirectory { .. } => "Save folder unavailable",
            Self::UnsafePath { .. } => "Save path contains a link",
            Self::NotRegularFile { .. } => "Save path is not a file",
            Self::MissingSave { .. } => "No saved park found",
            Self::Busy { .. } => "Save locked by another instance",
            Self::TooLarge { .. }
            | Self::InvalidSave {
                source: SaveError::TooLarge { .. },
                ..
            } => "Save file exceeds size limit",
            Self::Io { .. } => "Could not read or write save",
            Self::InvalidSave {
                source: SaveError::UnsupportedSchema(_) | SaveError::UnsupportedContent(_),
                ..
            } => "Save version is unsupported",
            Self::InvalidSave { .. } => "Save data is invalid",
            Self::RollbackFailed { .. } => "Recovery: see save folder",
        }
    }
}

fn io_error(action: &'static str, path: &Path, source: io::Error) -> StoreError {
    StoreError::Io {
        action,
        path: path.to_owned(),
        source,
    }
}

fn directory_from_local_app_data(value: Option<OsString>) -> Result<PathBuf, StoreError> {
    let root = value
        .filter(|value| !value.is_empty())
        .ok_or(StoreError::MissingLocalAppData)?;
    let root = PathBuf::from(root);
    if !root.is_absolute() {
        return Err(StoreError::InvalidDirectory {
            path: root,
            reason: "LOCALAPPDATA must be absolute",
        });
    }
    Ok(root.join("Parkbound").join("saves"))
}

impl SaveStore {
    pub(super) fn user_directory() -> Result<PathBuf, StoreError> {
        directory_from_local_app_data(std::env::var_os("LOCALAPPDATA"))
    }

    pub(super) fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub(super) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(super) fn save_path(&self) -> PathBuf {
        self.directory.join(PRIMARY_NAME)
    }

    pub(super) fn backup_path(&self) -> PathBuf {
        self.directory.join(BACKUP_NAME)
    }

    pub(super) fn save(&self, park: &Park) -> Result<(), StoreError> {
        let bytes = park
            .to_save_bytes()
            .map_err(|source| StoreError::InvalidSave { path: None, source })?;
        self.save_bytes(&bytes)
    }

    fn save_bytes(&self, bytes: &[u8]) -> Result<(), StoreError> {
        // Validate the complete snapshot before creating even a directory or lock.
        Park::from_save_bytes(bytes)
            .map_err(|source| StoreError::InvalidSave { path: None, source })?;
        self.check_directory(true)?;
        let _lock = SaveLock::acquire(&self.directory)?;
        let primary = self.save_path();
        let backup = self.backup_path();
        check_file(&primary)?;
        check_file(&backup)?;

        // A corrupt/oversized primary may be replaced, but cannot poison the
        // last good backup. Permission and other I/O failures still stop saving.
        let previous = match read_file(&primary) {
            Ok(bytes) if Park::from_save_bytes(&bytes).is_ok() => Some(bytes),
            Ok(_) | Err(StoreError::MissingSave { .. } | StoreError::TooLarge { .. }) => None,
            Err(error) => return Err(error),
        };
        let mut pending_primary = StagedFile::write(&self.directory, "primary", bytes)?;
        if let Some(previous) = previous {
            // Keep the exact former backup bytes for rollback, even when that
            // former backup itself was corrupt. A failed save changes neither.
            let original_backup = match read_file(&backup) {
                Ok(bytes) => Some(bytes),
                Err(StoreError::MissingSave { .. }) => None,
                Err(error) => return Err(error),
            };
            let mut rollback = original_backup
                .as_deref()
                .map(|bytes| StagedFile::write(&self.directory, "rollback", bytes))
                .transpose()?;
            let mut pending_backup = StagedFile::write(&self.directory, "backup", &previous)?;
            pending_backup.replace(&backup)?;
            if let Err(original) = pending_primary.replace(&primary) {
                let restored = if let Some(rollback) = &mut rollback {
                    rollback.replace(&backup)
                } else {
                    // This name did not exist before this transaction. Undo
                    // only the backup we just created; never remove a primary.
                    fs::remove_file(&backup)
                        .map_err(|error| io_error("undo new backup", &backup, error))
                };
                if let Err(rollback_error) = restored {
                    let recovery = rollback.as_mut().map(StagedFile::retain);
                    return Err(StoreError::RollbackFailed {
                        original: Box::new(original),
                        rollback: Box::new(rollback_error),
                        recovery,
                    });
                }
                return Err(original);
            }
        } else {
            pending_primary.replace(&primary)?;
        }
        Ok(())
    }

    pub(super) fn load(&self) -> Result<Park, StoreError> {
        self.load_path(self.save_path())
    }

    pub(super) fn load_backup(&self) -> Result<Park, StoreError> {
        self.load_path(self.backup_path())
    }

    fn load_path(&self, path: PathBuf) -> Result<Park, StoreError> {
        self.check_directory(false).map_err(|error| match error {
            StoreError::MissingSave { .. } => StoreError::MissingSave { path: path.clone() },
            other => other,
        })?;
        let bytes = read_file(&path)?;
        Park::from_save_bytes(&bytes).map_err(|source| StoreError::InvalidSave {
            path: Some(path),
            source,
        })
    }

    fn check_directory(&self, create: bool) -> Result<(), StoreError> {
        if !self.directory.is_absolute() {
            return Err(StoreError::InvalidDirectory {
                path: self.directory.clone(),
                reason: "an absolute path is required",
            });
        }
        // Check each component before creating below it. This also rejects
        // Windows junctions; fixed filenames never resolve outside this tree
        // through an existing link. Hostile concurrent filesystem edits are
        // outside this local single-user store's synchronization contract.
        let ancestors: Vec<_> = self.directory.ancestors().collect();
        for path in ancestors.into_iter().rev() {
            match fs::symlink_metadata(path) {
                Ok(metadata) => check_directory_metadata(path, &metadata)?,
                Err(error) if error.kind() == io::ErrorKind::NotFound && create => {
                    match fs::create_dir(path) {
                        Ok(()) => {}
                        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                        Err(error) => return Err(io_error("create directory", path, error)),
                    }
                    let metadata = fs::symlink_metadata(path)
                        .map_err(|error| io_error("inspect directory", path, error))?;
                    check_directory_metadata(path, &metadata)?;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    return Err(StoreError::MissingSave {
                        path: self.save_path(),
                    });
                }
                Err(error) => return Err(io_error("inspect directory", path, error)),
            }
        }
        Ok(())
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

fn check_directory_metadata(path: &Path, metadata: &Metadata) -> Result<(), StoreError> {
    if is_link(metadata) {
        Err(StoreError::UnsafePath {
            path: path.to_owned(),
        })
    } else if !metadata.is_dir() {
        Err(StoreError::InvalidDirectory {
            path: path.to_owned(),
            reason: "a path component is a file",
        })
    } else {
        Ok(())
    }
}

fn check_file(path: &Path) -> Result<(), StoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if is_link(&metadata) => Err(StoreError::UnsafePath {
            path: path.to_owned(),
        }),
        Ok(metadata) if !metadata.is_file() => Err(StoreError::NotRegularFile {
            path: path.to_owned(),
        }),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("inspect file", path, error)),
    }
}

fn read_file(path: &Path) -> Result<Vec<u8>, StoreError> {
    check_file(path)?;
    let file = File::open(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            StoreError::MissingSave {
                path: path.to_owned(),
            }
        } else {
            io_error("open saved park", path, error)
        }
    })?;
    let metadata = file
        .metadata()
        .map_err(|error| io_error("inspect open file", path, error))?;
    if !metadata.is_file() {
        return Err(StoreError::NotRegularFile {
            path: path.to_owned(),
        });
    }
    read_limited(file, metadata.len(), path)
}

fn read_limited(
    mut reader: impl Read,
    initial_length: u64,
    path: &Path,
) -> Result<Vec<u8>, StoreError> {
    if initial_length > MAX_SAVE_BYTES as u64 {
        return Err(StoreError::TooLarge {
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
            Err(error) => return Err(io_error("read saved park", path, error)),
        };
        // Check again after every read: a file can grow after metadata(). Keep
        // both retained data and allocation bounded, rather than read_to_end.
        if count > MAX_SAVE_BYTES - bytes.len() {
            return Err(StoreError::TooLarge {
                path: path.to_owned(),
            });
        }
        bytes
            .try_reserve_exact(count)
            .map_err(|error| io_error("allocate save buffer for", path, io::Error::other(error)))?;
        bytes.extend_from_slice(&chunk[..count]);
    }
}

struct SaveLock {
    path: PathBuf,
}

impl SaveLock {
    fn acquire(directory: &Path) -> Result<Self, StoreError> {
        let path = directory.join(LOCK_NAME);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                if error.kind() == io::ErrorKind::AlreadyExists {
                    StoreError::Busy { path: path.clone() }
                } else {
                    io_error("acquire save lock", &path, error)
                }
            })?;
        let lock = Self { path };
        let written = writeln!(file, "Parkbound save lock; process {}", std::process::id())
            .and_then(|()| file.sync_all());
        drop(file);
        written.map_err(|error| io_error("write save lock", &lock.path, error))?;
        Ok(lock)
    }
}

impl Drop for SaveLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

struct StagedFile {
    path: PathBuf,
    owned: bool,
}

impl StagedFile {
    fn write(directory: &Path, label: &str, bytes: &[u8]) -> Result<Self, StoreError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for _ in 0..16 {
            let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
            let name = format!(
                ".parkbound-{label}-{}-{timestamp}-{sequence}.tmp",
                std::process::id()
            );
            let path = directory.join(name);
            let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(io_error("create temporary save", &path, error)),
            };
            let pending = Self { path, owned: true };
            let written = file.write_all(bytes).and_then(|()| file.sync_all());
            drop(file);
            written.map_err(|error| {
                io_error("write and flush temporary save", &pending.path, error)
            })?;
            return Ok(pending);
        }
        Err(io_error(
            "create unique temporary save in",
            directory,
            io::Error::new(io::ErrorKind::AlreadyExists, "temporary name collisions"),
        ))
    }

    fn replace(&mut self, destination: &Path) -> Result<(), StoreError> {
        check_file(destination)?;
        // Rust's documented rename contract replaces an existing file on
        // Windows too. Same-directory staging prevents cross-volume fallback.
        // https://doc.rust-lang.org/std/fs/fn.rename.html
        fs::rename(&self.path, destination)
            .map_err(|error| io_error("replace saved park", destination, error))?;
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
    use park_sim::Speed;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "Parkbound save tests – forêt {} {timestamp} {sequence}",
                std::process::id(),
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn store(&self) -> SaveStore {
            SaveStore::new(self.0.join("my park – été"))
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            // Resolve and check the exact test directory before recursive
            // cleanup; never follow an unexpectedly redirected scratch root.
            if let (Ok(path), Ok(root)) = (
                fs::canonicalize(&self.0),
                fs::canonicalize(std::env::temp_dir()),
            ) && path.parent() == Some(root.as_path())
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("Parkbound save tests "))
            {
                let _ = fs::remove_dir_all(path);
            }
        }
    }

    fn parks() -> (Park, Park, Park) {
        let mut first = Park::new(32, 32);
        first.install_crowd_fixture(42).unwrap();
        first.install_train_fixture().unwrap();
        let mut second = first.clone();
        for _ in 0..150 {
            second.advance(Speed::Normal);
        }
        let mut third = second.clone();
        for _ in 0..350 {
            third.advance(Speed::Normal);
        }
        (first, second, third)
    }

    fn disk(store: &SaveStore) -> (Vec<u8>, Vec<u8>) {
        (
            fs::read(store.save_path()).unwrap(),
            fs::read(store.backup_path()).unwrap(),
        )
    }

    fn assert_no_owned_temps(store: &SaveStore) {
        let names: Vec<_> = fs::read_dir(store.directory())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert!(
            !names
                .iter()
                .any(|name| name.to_string_lossy().starts_with(".parkbound-"))
        );
        assert!(!store.directory().join(LOCK_NAME).exists());
    }

    #[test]
    fn saves_unicode_paths_and_rotates_the_previous_valid_primary() {
        let scratch = Scratch::new();
        let store = scratch.store();
        let (first, second, third) = parks();
        store.save(&first).unwrap();
        assert_eq!(store.load().unwrap(), first);
        assert!(matches!(
            store.load_backup(),
            Err(StoreError::MissingSave { .. })
        ));
        store.save(&second).unwrap();
        assert_eq!(store.load().unwrap(), second);
        assert_eq!(store.load_backup().unwrap(), first);
        store.save(&third).unwrap();
        assert_eq!(store.load().unwrap(), third);
        assert_eq!(store.load_backup().unwrap(), second);
        assert_no_owned_temps(&store);
    }

    #[test]
    fn corrupt_primary_never_replaces_a_good_backup_or_loads_it_implicitly() {
        let scratch = Scratch::new();
        let store = scratch.store();
        let (first, second, third) = parks();
        store.save(&first).unwrap();
        store.save(&second).unwrap();
        let backup = fs::read(store.backup_path()).unwrap();
        fs::write(store.save_path(), b"{broken").unwrap();
        let before = disk(&store);
        assert!(matches!(store.load(), Err(StoreError::InvalidSave { .. })));
        assert_eq!(disk(&store), before);
        assert_eq!(store.load_backup().unwrap(), first);
        store.save(&third).unwrap();
        assert_eq!(store.load().unwrap(), third);
        assert_eq!(fs::read(store.backup_path()).unwrap(), backup);
        assert_no_owned_temps(&store);
    }

    #[test]
    fn rejected_serialized_state_touches_no_files_or_directories() {
        let scratch = Scratch::new();
        let store = scratch.store();
        assert!(store.save_bytes(b"{invalid").is_err());
        assert!(!store.directory().exists());
        let (first, second, _) = parks();
        store.save(&first).unwrap();
        store.save(&second).unwrap();
        let before = disk(&store);
        assert!(store.save_bytes(b"{}").is_err());
        assert_eq!(disk(&store), before);
        assert_no_owned_temps(&store);
    }

    #[test]
    fn missing_unsupported_and_oversized_loads_leave_both_files_unchanged() {
        let scratch = Scratch::new();
        let store = scratch.store();
        assert!(matches!(store.load(), Err(StoreError::MissingSave { .. })));
        assert!(!store.directory().exists());
        let (first, second, _) = parks();
        store.save(&first).unwrap();
        store.save(&second).unwrap();
        let mut serialized: serde_json::Value =
            serde_json::from_slice(&second.to_save_bytes().unwrap()).unwrap();
        serialized["schema_version"] = 999.into();
        let unsupported = serde_json::to_vec(&serialized).unwrap();
        fs::write(store.save_path(), unsupported).unwrap();
        let before = disk(&store);
        assert!(matches!(
            store.load(),
            Err(StoreError::InvalidSave {
                source: SaveError::UnsupportedSchema(999),
                ..
            })
        ));
        assert_eq!(disk(&store), before);
        File::create(store.save_path())
            .unwrap()
            .set_len(MAX_SAVE_BYTES as u64 + 1)
            .unwrap();
        let before = disk(&store);
        assert!(matches!(store.load(), Err(StoreError::TooLarge { .. })));
        assert_eq!(disk(&store), before);
        assert_eq!(store.load_backup().unwrap(), first);
    }

    #[test]
    fn growing_files_are_bounded_even_after_a_small_metadata_size() {
        let scratch = Scratch::new();
        let path = scratch.0.join("growing.park.json");
        fs::write(&path, b"x").unwrap();
        let reader = File::open(&path).unwrap();
        let length = reader.metadata().unwrap().len();
        OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(MAX_SAVE_BYTES as u64 + 1)
            .unwrap();
        assert!(matches!(
            read_limited(reader, length, &path),
            Err(StoreError::TooLarge { .. })
        ));
    }

    #[test]
    fn an_existing_lock_blocks_saves_and_is_never_removed_by_the_contender() {
        let scratch = Scratch::new();
        let store = scratch.store();
        let (first, second, third) = parks();
        store.save(&first).unwrap();
        store.save(&second).unwrap();
        let before = disk(&store);
        let lock = SaveLock::acquire(store.directory()).unwrap();
        assert!(matches!(store.save(&third), Err(StoreError::Busy { .. })));
        assert!(lock.path.exists());
        assert_eq!(disk(&store), before);
        drop(lock);
        store.save(&third).unwrap();
        assert_no_owned_temps(&store);
    }

    #[test]
    fn invalid_directory_and_backup_destination_do_not_replace_the_primary() {
        let scratch = Scratch::new();
        let store = scratch.store();
        let (first, second, _) = parks();
        store.save(&first).unwrap();
        let primary = fs::read(store.save_path()).unwrap();
        fs::create_dir(store.backup_path()).unwrap();
        assert!(matches!(
            store.save(&second),
            Err(StoreError::NotRegularFile { .. })
        ));
        assert_eq!(fs::read(store.save_path()).unwrap(), primary);
        assert_no_owned_temps(&store);
        let blocked = scratch.0.join("ordinary file");
        fs::write(&blocked, b"untouched").unwrap();
        assert!(SaveStore::new(blocked.join("saves")).save(&first).is_err());
        assert_eq!(fs::read(blocked).unwrap(), b"untouched");
    }

    #[test]
    fn local_app_data_has_no_relative_or_missing_fallback() {
        assert!(directory_from_local_app_data(None).is_err());
        assert!(directory_from_local_app_data(Some(OsString::new())).is_err());
        assert!(directory_from_local_app_data(Some(OsString::from("relative"))).is_err());
        let scratch = Scratch::new();
        assert_eq!(
            directory_from_local_app_data(Some(scratch.0.clone().into_os_string())).unwrap(),
            scratch.0.join("Parkbound").join("saves"),
        );
        assert!(SaveStore::new(PathBuf::from("relative")).load().is_err());
    }

    #[cfg(windows)]
    #[test]
    fn failed_primary_replace_rolls_backup_back_and_preserves_both_originals() {
        use std::os::windows::fs::OpenOptionsExt;
        let scratch = Scratch::new();
        let store = scratch.store();
        let (first, second, third) = parks();
        store.save(&first).unwrap();
        store.save(&second).unwrap();
        let before = disk(&store);
        // Share reads/writes but deliberately deny delete/rename. Backup
        // staging succeeds; the final primary rename must fail on Windows.
        let blocker = OpenOptions::new()
            .read(true)
            .share_mode(0x1 | 0x2)
            .open(store.save_path())
            .unwrap();
        assert!(store.save(&third).is_err());
        assert_eq!(disk(&store), before);
        assert_no_owned_temps(&store);
        drop(blocker);
        store.save(&third).unwrap();
        assert_eq!(store.load().unwrap(), third);
    }

    #[cfg(unix)]
    #[test]
    fn linked_directory_and_file_targets_are_rejected() {
        use std::os::unix::fs::symlink;
        let scratch = Scratch::new();
        let outside = scratch.0.join("outside");
        fs::create_dir(&outside).unwrap();
        let link = scratch.0.join("linked saves");
        symlink(&outside, &link).unwrap();
        let first = Park::new(32, 32);
        assert!(matches!(
            SaveStore::new(link).save(&first),
            Err(StoreError::UnsafePath { .. })
        ));
        let store = scratch.store();
        fs::create_dir(store.directory()).unwrap();
        let target = outside.join("keep.txt");
        fs::write(&target, b"untouched").unwrap();
        symlink(&target, store.save_path()).unwrap();
        assert!(matches!(
            store.save(&first),
            Err(StoreError::UnsafePath { .. })
        ));
        assert_eq!(fs::read(&target).unwrap(), b"untouched");
    }
}
