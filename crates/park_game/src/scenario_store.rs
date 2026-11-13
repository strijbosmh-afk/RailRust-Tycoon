//! Separate, bounded scenario files with atomic saves and recoverable deletion.
use crate::save_store::{SaveLock, SaveStore, StagedFile, check_file};
use park_sim::{MAX_SCENARIO_BYTES, ScenarioDocument};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub(crate) struct ScenarioEntry {
    pub id: u64,
    pub name: String,
    pub summary: String,
    pub error: Option<String>,
}
#[derive(Clone)]
pub(crate) struct ScenarioStore {
    root: PathBuf,
}
impl ScenarioStore {
    pub fn new(profile: &Path) -> Self {
        Self {
            root: profile.join("scenarios"),
        }
    }
    fn path(&self, id: u64) -> PathBuf {
        self.root
            .join(format!("scenario-{id:016x}.parkscenario.json"))
    }
    fn prepare(&self) -> Result<(), String> {
        SaveStore::new(self.root.clone())
            .check_directory(true)
            .map_err(|e| e.to_string())
    }
    pub fn list(&self) -> Result<Vec<ScenarioEntry>, String> {
        self.prepare()?;
        let mut rows = Vec::new();
        for file in fs::read_dir(&self.root).map_err(|e| e.to_string())? {
            let file = file.map_err(|e| e.to_string())?;
            let name = file.file_name().to_string_lossy().into_owned();
            let Some(id) = parse_id(&name) else { continue };
            if rows.len() == 64 {
                return Err("Scenario library contains more than 64 records.".into());
            }
            let row = match self.load(id) {
                Ok(doc) => {
                    let park = doc.template()?;
                    ScenarioEntry {
                        id,
                        name: doc.definition.name,
                        summary: format!(
                            "{} × {} · {} goals · {}",
                            park.width(),
                            park.height(),
                            doc.definition.goals.len(),
                            doc.definition.description
                        ),
                        error: None,
                    }
                }
                Err(error) => ScenarioEntry {
                    id,
                    name: format!("Unavailable scenario #{id}"),
                    summary: "Inspect or restore the scenario file before playing.".into(),
                    error: Some(error),
                },
            };
            rows.push(row);
        }
        rows.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then(a.id.cmp(&b.id))
        });
        Ok(rows)
    }
    pub fn load(&self, id: u64) -> Result<ScenarioDocument, String> {
        read_document(&self.path(id))
    }
    pub fn save(&self, id: Option<u64>, doc: &ScenarioDocument) -> Result<u64, String> {
        self.prepare()?;
        let _lock = SaveLock::acquire(&self.root).map_err(|e| e.to_string())?;
        let bytes = doc.to_bytes()?;
        let entries = self.list()?;
        if entries
            .iter()
            .any(|e| Some(e.id) != id && e.name.eq_ignore_ascii_case(&doc.definition.name))
        {
            return Err("A scenario with that name already exists.".into());
        }
        if id.is_none() && entries.len() >= 64 {
            return Err("Scenario library is full (64 records).".into());
        }
        let id = match id {
            Some(id) => {
                self.load(id)?;
                id
            }
            None => {
                let mut maximum = 0;
                for folder in [&self.root, &self.root.join("archive")] {
                    if folder.exists() {
                        for f in fs::read_dir(folder).map_err(|e| e.to_string())? {
                            if let Some(id) = parse_id(
                                &f.map_err(|e| e.to_string())?.file_name().to_string_lossy(),
                            ) {
                                maximum = maximum.max(id);
                            }
                        }
                    }
                }
                maximum
                    .checked_add(1)
                    .ok_or("Scenario identity limit reached")?
            }
        };
        let target = self.path(id);
        check_file(&target).map_err(|e| e.to_string())?;
        if target.exists() {
            let old = read_bytes(&target)?;
            let mut backup = StagedFile::write(&self.root, "scenario-backup", &old)
                .map_err(|e| e.to_string())?;
            backup
                .replace(&target.with_extension("json.bak"))
                .map_err(|e| e.to_string())?;
        }
        let mut staged =
            StagedFile::write(&self.root, "scenario", &bytes).map_err(|e| e.to_string())?;
        staged.replace(&target).map_err(|e| e.to_string())?;
        Ok(id)
    }
    pub fn rename(&self, id: u64, name: &str) -> Result<(), String> {
        let mut doc = self.load(id)?;
        doc.definition.name = name.trim().into();
        self.save(Some(id), &doc).map(|_| ())
    }
    pub fn archive(&self, id: u64) -> Result<(), String> {
        self.prepare()?;
        let _lock = SaveLock::acquire(&self.root).map_err(|e| e.to_string())?;
        let source = self.path(id);
        check_file(&source).map_err(|e| e.to_string())?;
        let archive = self.root.join("archive");
        SaveStore::new(archive.clone())
            .check_directory(true)
            .map_err(|e| e.to_string())?;
        let target = archive.join(source.file_name().unwrap());
        if target.exists() {
            return Err("An archived scenario already has this identity.".into());
        }
        fs::rename(source, target).map_err(|e| e.to_string())
    }
}
fn parse_id(name: &str) -> Option<u64> {
    let hex = name
        .strip_prefix("scenario-")?
        .strip_suffix(".parkscenario.json")?;
    if hex.len() != 16 {
        return None;
    }
    u64::from_str_radix(hex, 16).ok().filter(|id| *id > 0)
}
fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    check_file(path).map_err(|e| e.to_string())?;
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_SCENARIO_BYTES as u64 {
        return Err("Scenario exceeds 32 MiB.".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_SCENARIO_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_SCENARIO_BYTES {
        return Err("Scenario exceeds 32 MiB.".into());
    }
    Ok(bytes)
}
pub(crate) fn read_document(path: &Path) -> Result<ScenarioDocument, String> {
    ScenarioDocument::from_bytes(&read_bytes(path)?)
}
pub(crate) fn export_document(path: &Path, doc: &ScenarioDocument) -> Result<(), String> {
    if !path.is_absolute()
        || !path
            .to_string_lossy()
            .to_lowercase()
            .ends_with(".parkscenario.json")
    {
        return Err("Choose an absolute .parkscenario.json filename.".into());
    }
    let parent = path.parent().ok_or("Missing export folder")?;
    SaveStore::new(parent.into())
        .check_directory(false)
        .map_err(|e| e.to_string())?;
    check_file(path).map_err(|e| e.to_string())?;
    let mut staged = StagedFile::write(parent, "scenario-export", &doc.to_bytes()?)
        .map_err(|e| e.to_string())?;
    staged.replace(path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn directory() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "scenario-store-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        p
    }
    #[test]
    fn save_import_rename_archive_and_identity_are_preserved() {
        let root = directory();
        let store = ScenarioStore::new(&root);
        let doc =
            ScenarioDocument::from_park(&park_sim::Park::new(32, 32), Default::default()).unwrap();
        let id = store.save(None, &doc).unwrap();
        assert!(store.save(None, &doc).is_err());
        store.rename(id, "Renamed scenario").unwrap();
        assert_eq!(store.load(id).unwrap().definition.name, "Renamed scenario");
        let file = root.join("shared.parkscenario.json");
        export_document(&file, &store.load(id).unwrap()).unwrap();
        let imported = read_document(&file).unwrap();
        assert_eq!(imported.template().unwrap(), doc.template().unwrap());
        store.archive(id).unwrap();
        assert!(store.list().unwrap().is_empty());
        assert!(store.save(None, &doc).unwrap() > id);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn malformed_files_are_reported_and_not_replaced() {
        let root = directory();
        let store = ScenarioStore::new(&root);
        store.prepare().unwrap();
        let path = store.path(1);
        fs::write(&path, b"invalid scenario").unwrap();
        assert!(store.list().unwrap()[0].error.is_some());
        let doc =
            ScenarioDocument::from_park(&park_sim::Park::new(32, 32), Default::default()).unwrap();
        assert!(store.save(Some(1), &doc).is_err());
        assert_eq!(fs::read(path).unwrap(), b"invalid scenario");
        fs::remove_dir_all(root).unwrap();
    }
}
