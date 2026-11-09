//! A bounded library of independent parks. Folder names are generated IDs;
//! display names never become paths. Deleted parks are retained in an archive.
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use park_sim::Park;
use serde::{Deserialize, Serialize};

use crate::save_store::{SaveLock, SaveStore, StagedFile, check_file, read_file};

pub(super) const MAX_PARKS: usize = 32;
const META: &str = "slot.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Entry {
    pub id: u64,
    pub name: String,
    pub updated: u64,
}

#[derive(Clone)]
pub(super) struct ParkLibrary {
    root: PathBuf,
}

impl ParkLibrary {
    pub fn new(save_root: &Path) -> Self {
        Self {
            root: save_root.join("parks"),
        }
    }

    fn directory(&self, id: u64) -> PathBuf {
        self.root.join(format!("park-{id:016x}"))
    }
    fn store(&self, id: u64) -> SaveStore {
        SaveStore::new(self.directory(id))
    }

    pub fn entries(&self) -> Result<Vec<Entry>, String> {
        match SaveStore::new(self.root.clone()).check_directory(false) {
            Ok(()) => {}
            Err(crate::save_store::StoreError::MissingSave { .. }) => return Ok(Vec::new()),
            Err(e) => return Err(e.to_string()),
        }
        let mut entries = Vec::new();
        for item in fs::read_dir(&self.root).map_err(|e| e.to_string())? {
            let item = item.map_err(|e| e.to_string())?;
            let Some(id) = parse_id(&item.file_name().to_string_lossy()) else {
                continue;
            };
            let store = self.store(id);
            store.check_directory(false).map_err(|e| e.to_string())?;
            // An interrupted create may have a valid park and no metadata.
            if store.load().is_err() && store.load_backup().is_err() {
                continue;
            }
            let path = self.directory(id).join(META);
            let entry = match read_file(&path) {
                Ok(bytes) if bytes.len() <= 4096 => serde_json::from_slice::<Entry>(&bytes)
                    .ok()
                    .filter(|entry| entry.id == id && valid_name(&entry.name))
                    .unwrap_or_else(|| fallback(id)),
                _ => fallback(id),
            };
            entries.push(entry);
        }
        entries.sort_by(|a, b| b.updated.cmp(&a.updated).then_with(|| b.id.cmp(&a.id)));
        Ok(entries)
    }

    pub fn create(&self, name: &str, park: &Park) -> Result<Entry, String> {
        let name = clean_name(name)?;
        SaveStore::new(self.root.clone())
            .check_directory(true)
            .map_err(|e| e.to_string())?;
        let _lock = SaveLock::acquire(&self.root).map_err(|e| e.to_string())?;
        let entries = self.entries()?;
        if entries.len() >= MAX_PARKS {
            return Err(format!("Library is full ({MAX_PARKS} parks)."));
        }
        let mut next = 1_u64;
        for folder in [&self.root, &self.root.join("deleted-parks")] {
            if let Ok(items) = fs::read_dir(folder) {
                for item in items.flatten() {
                    let text = item.file_name().to_string_lossy().into_owned();
                    if let Some(id) = parse_id(&text) {
                        next = next.max(id.saturating_add(1));
                    }
                }
            }
        }
        if self.directory(next).exists()
            || self
                .root
                .join("deleted-parks")
                .join(format!("park-{next:016x}"))
                .exists()
        {
            return Err("Park ID space is exhausted; no existing park was changed.".into());
        }
        let store = self.store(next);
        store.save(park).map_err(|e| e.to_string())?;
        let entry = Entry {
            id: next,
            name,
            updated: now(),
        };
        self.write_entry(&entry)?;
        Ok(entry)
    }

    pub fn save(&self, id: u64, park: &Park) -> Result<Entry, String> {
        let mut entry = self.find(id)?;
        self.store(id).save(park).map_err(|e| e.to_string())?;
        entry.updated = now();
        self.write_entry(&entry)?;
        Ok(entry)
    }

    pub fn load(&self, id: u64, backup: bool) -> Result<Park, String> {
        self.find(id)?;
        let store = self.store(id);
        (if backup {
            store.load_backup()
        } else {
            store.load()
        })
        .map_err(|e| e.to_string())
    }

    pub fn rename(&self, id: u64, name: &str) -> Result<Entry, String> {
        let mut entry = self.find(id)?;
        entry.name = clean_name(name)?;
        self.write_entry(&entry)?;
        Ok(entry)
    }

    pub fn archive(&self, id: u64) -> Result<(), String> {
        self.find(id)?;
        let source = self.directory(id);
        SaveStore::new(source.clone())
            .check_directory(false)
            .map_err(|e| e.to_string())?;
        let archive = self.root.join("deleted-parks");
        SaveStore::new(archive.clone())
            .check_directory(true)
            .map_err(|e| e.to_string())?;
        let target = archive.join(format!("park-{id:016x}"));
        if target.exists() {
            return Err("Archive target already exists.".into());
        }
        fs::rename(source, target).map_err(|e| e.to_string())
    }

    pub fn import_legacy(&self, legacy: &SaveStore, backup: bool) -> Result<Entry, String> {
        let park = (if backup {
            legacy.load_backup()
        } else {
            legacy.load()
        })
        .map_err(|e| e.to_string())?;
        self.create("Imported park", &park)
    }

    fn find(&self, id: u64) -> Result<Entry, String> {
        self.entries()?
            .into_iter()
            .find(|entry| entry.id == id)
            .ok_or_else(|| "Park is no longer available.".into())
    }

    fn write_entry(&self, entry: &Entry) -> Result<(), String> {
        let folder = self.directory(entry.id);
        SaveStore::new(folder.clone())
            .check_directory(false)
            .map_err(|e| e.to_string())?;
        let path = folder.join(META);
        check_file(&path).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec_pretty(entry).map_err(|e| e.to_string())?;
        let mut staged = StagedFile::write(&folder, "slot", &bytes).map_err(|e| e.to_string())?;
        staged.replace(&path).map_err(|e| e.to_string())
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn fallback(id: u64) -> Entry {
    Entry {
        id,
        name: format!("Recovered park {id}"),
        updated: 0,
    }
}
fn parse_id(name: &str) -> Option<u64> {
    let hex = name.strip_prefix("park-")?;
    if hex.len() != 16 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let id = u64::from_str_radix(hex, 16).ok()?;
    (name == format!("park-{id:016x}")).then_some(id)
}
fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name.chars().count() <= 40 && !name.chars().any(char::is_control)
}
fn clean_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if valid_name(name) {
        Ok(name.to_owned())
    } else {
        Err("Use a name of 1 to 40 characters.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn library_preserves_legacy_and_archives_without_erasing() {
        let root = std::env::temp_dir().join(format!(
            "parkbound-library-{}-{}",
            std::process::id(),
            now()
        ));
        let legacy = SaveStore::new(root.clone());
        let park = Park::new(32, 32);
        legacy.save(&park).unwrap();
        let original = fs::read(legacy.save_path()).unwrap();
        let library = ParkLibrary::new(&root);
        let imported = library.import_legacy(&legacy, false).unwrap();
        assert_eq!(library.load(imported.id, false).unwrap(), park);
        assert_eq!(fs::read(legacy.save_path()).unwrap(), original);
        let renamed = library.rename(imported.id, "Été park").unwrap();
        assert_eq!(renamed.name, "Été park");
        library.archive(imported.id).unwrap();
        assert!(library.entries().unwrap().is_empty());
        assert!(
            root.join("parks/deleted-parks/park-0000000000000001/park.park.json")
                .exists()
        );
        // Test cleanup only of its exact, newly created scratch directory.
        let safe = root
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("parkbound-library-")
            && root.parent() == Some(std::env::temp_dir().as_path());
        if safe {
            fs::remove_dir_all(root).unwrap();
        }
    }
}
