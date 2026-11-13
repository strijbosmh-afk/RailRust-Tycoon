//! Small, local campaign record shared by separate parks. Park saves remain
//! deterministic and independent of this player's progression file.
use std::{fs, path::PathBuf};

use bevy::prelude::*;
use park_sim::{CampaignScenario, ScenarioKind};
use serde::{Deserialize, Serialize};

use crate::{
    Simulation, ViewState,
    save_store::{SaveLock, SaveStore, StagedFile, StoreError, check_file, read_file},
};

const PRIMARY: &str = "campaign.json";
const BACKUP: &str = "campaign.json.bak";
const MAX_CAMPAIGN_BYTES: usize = 1024;
const RETRY_TICKS: u64 = 900;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CampaignProgress {
    format: CampaignFormat,
    version: u32,
    pub meadow_awarded: bool,
    pub boardwalk_awarded: bool,
    #[serde(default)]
    pub later_awards: [bool; 4],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
enum CampaignFormat {
    #[default]
    #[serde(rename = "parkbound-campaign")]
    Parkbound,
}

impl CampaignProgress {
    fn empty() -> Self {
        Self {
            format: CampaignFormat::Parkbound,
            version: 2,
            meadow_awarded: false,
            boardwalk_awarded: false,
            later_awards: [false; 4],
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_CAMPAIGN_BYTES {
            return Err("campaign record is too large".into());
        }
        let header: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|e| format!("invalid campaign record: {e}"))?;
        if let Some(version) = header.get("version").and_then(serde_json::Value::as_u64)
            && !matches!(version, 1 | 2)
        {
            return Err(format!("unsupported campaign version {version}"));
        }
        let mut progress: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("invalid campaign record: {e}"))?;
        if !matches!(progress.version, 1 | 2) {
            return Err(format!("unsupported campaign version {}", progress.version));
        }
        if progress.version == 2 && header.get("later_awards").is_none() {
            return Err("campaign awards field is missing".into());
        }
        progress.version = 2;
        Ok(progress)
    }

    fn bytes(self) -> Vec<u8> {
        serde_json::to_vec_pretty(&self).expect("campaign record serializes")
    }

    pub(crate) fn unlocked(self, kind: ScenarioKind) -> bool {
        match kind {
            ScenarioKind::MeadowChallenge => true,
            // Existing Boardwalk winners from older park saves keep access.
            ScenarioKind::BoardwalkSprint => self.meadow_awarded || self.boardwalk_awarded,
        }
    }

    pub(crate) fn earned(self, kind: CampaignScenario) -> bool {
        match kind {
            CampaignScenario::Meadow => self.meadow_awarded,
            CampaignScenario::Boardwalk => self.boardwalk_awarded,
            _ => self.later_awards[kind.index() - 2],
        }
    }
    pub(crate) fn unlocked_scenario(self, kind: CampaignScenario) -> bool {
        kind.index() == 0
            || self.earned(kind)
            || self.earned(CampaignScenario::ALL[kind.index() - 1])
    }
    fn award_scenario(&mut self, kind: CampaignScenario) -> bool {
        if let Some(legacy) = kind.legacy() {
            return self.award(legacy);
        }
        let award = &mut self.later_awards[kind.index() - 2];
        let changed = !*award;
        *award = true;
        changed
    }
    fn award(&mut self, kind: ScenarioKind) -> bool {
        let awarded = match kind {
            ScenarioKind::MeadowChallenge => &mut self.meadow_awarded,
            ScenarioKind::BoardwalkSprint => &mut self.boardwalk_awarded,
        };
        if *awarded {
            false
        } else {
            *awarded = true;
            true
        }
    }
}

#[derive(Resource)]
pub(crate) struct Campaign {
    store: Option<SaveStore>,
    pub progress: CampaignProgress,
    pub notice: Option<String>,
    blocked: bool,
    next_retry_tick: Option<u64>,
}

impl Campaign {
    pub(crate) fn load(store: Option<SaveStore>) -> Self {
        let mut result = Self {
            store,
            progress: CampaignProgress::empty(),
            notice: None,
            blocked: false,
            next_retry_tick: None,
        };
        let Some(location) = &result.store else {
            result.notice = Some("Campaign folder unavailable. Check launch settings.".into());
            result.blocked = true;
            return result;
        };
        match read_progress(location) {
            Ok((progress, recovered)) => {
                result.progress = progress;
                if recovered {
                    result.notice = Some("Campaign awards recovered from backup.".into());
                }
            }
            Err(error) => {
                result.notice = Some(format!("Campaign record needs recovery: {error}"));
                result.blocked = true;
            }
        }
        result
    }

    pub(crate) fn preview() -> Self {
        Self {
            store: None,
            progress: CampaignProgress {
                meadow_awarded: true,
                ..CampaignProgress::empty()
            },
            notice: None,
            blocked: false,
            next_retry_tick: None,
        }
    }

    pub(crate) fn unlocked(&self, kind: ScenarioKind) -> bool {
        !self.blocked && self.progress.unlocked(kind)
    }

    pub(crate) fn unlocked_scenario(&self, kind: CampaignScenario) -> bool {
        !self.blocked && self.progress.unlocked_scenario(kind)
    }
    pub(crate) fn record_completed(&mut self, kind: ScenarioKind, tick: u64) {
        self.record_scenario_completed(
            match kind {
                ScenarioKind::MeadowChallenge => CampaignScenario::Meadow,
                ScenarioKind::BoardwalkSprint => CampaignScenario::Boardwalk,
            },
            tick,
        );
    }
    pub(crate) fn record_scenario_completed(&mut self, kind: CampaignScenario, tick: u64) {
        if self.blocked || self.next_retry_tick.is_some_and(|next| tick < next) {
            return;
        }
        let mut updated = self.progress;
        if !updated.award_scenario(kind) {
            return;
        }
        let Some(store) = &self.store else {
            return;
        };
        match write_progress(store, updated) {
            Ok(()) => {
                self.progress = updated;
                self.next_retry_tick = None;
                self.notice = Some(format!(
                    "{} award earned! Campaign progress saved.",
                    kind.label()
                ));
            }
            Err(error) => {
                self.next_retry_tick = Some(tick.saturating_add(RETRY_TICKS));
                self.notice = Some(format!("Campaign award could not be saved: {error}"));
                warn!("Campaign award save failed: {error}");
            }
        }
    }
}

pub(crate) fn observe(
    mut campaign: ResMut<Campaign>,
    sim: Res<Simulation>,
    mut view: ResMut<ViewState>,
) {
    if let Some(status) = sim.0.challenge_status()
        && status.completed_tick.is_some()
        && let Some(kind) = status.definition.campaign
    {
        campaign.record_scenario_completed(kind, sim.0.ticks());
    }
    if let Some(status) = sim.0.scenario_status()
        && let Some(tick) = status.completed_tick
    {
        let before = campaign.progress;
        let previous_notice = campaign.notice.clone();
        campaign.record_completed(status.kind, tick.max(sim.0.ticks()));
        if campaign.progress != before || campaign.notice != previous_notice {
            view.scenario_notice = campaign.notice.clone();
        }
    }
}

fn paths(store: &SaveStore) -> (PathBuf, PathBuf) {
    (
        store.directory().join(PRIMARY),
        store.directory().join(BACKUP),
    )
}

fn read_progress(store: &SaveStore) -> Result<(CampaignProgress, bool), String> {
    let (primary, backup) = paths(store);
    match store.check_directory(false) {
        Ok(()) => {}
        Err(StoreError::MissingSave { .. }) => return Ok((CampaignProgress::empty(), false)),
        Err(error) => return Err(error.to_string()),
    }
    match read_file(&primary) {
        Ok(bytes) => match CampaignProgress::from_bytes(&bytes) {
            Ok(progress) => return Ok((progress, false)),
            Err(error) if error.starts_with("unsupported campaign version") => return Err(error),
            Err(error) => {
                let backup_bytes = read_file(&backup)
                    .map_err(|backup_error| format!("{error}; backup: {backup_error}"))?;
                return CampaignProgress::from_bytes(&backup_bytes)
                    .map(|progress| (progress, true))
                    .map_err(|backup_error| format!("{error}; backup: {backup_error}"));
            }
        },
        Err(StoreError::MissingSave { .. }) => {}
        Err(error) => return Err(error.to_string()),
    }
    match read_file(&backup) {
        Ok(bytes) => CampaignProgress::from_bytes(&bytes).map(|progress| (progress, true)),
        Err(StoreError::MissingSave { .. }) => Ok((CampaignProgress::empty(), false)),
        Err(error) => Err(error.to_string()),
    }
}

fn write_progress(store: &SaveStore, progress: CampaignProgress) -> Result<(), String> {
    let bytes = progress.bytes();
    if bytes.len() > MAX_CAMPAIGN_BYTES {
        return Err("campaign record is too large".into());
    }
    store.check_directory(true).map_err(|e| e.to_string())?;
    let _lock = SaveLock::acquire(store.directory()).map_err(|e| e.to_string())?;
    let (primary, backup) = paths(store);
    check_file(&primary).map_err(|e| e.to_string())?;
    check_file(&backup).map_err(|e| e.to_string())?;
    let previous = match read_file(&primary) {
        Ok(bytes) if CampaignProgress::from_bytes(&bytes).is_ok() => Some(bytes),
        Ok(_) | Err(StoreError::MissingSave { .. }) => None,
        Err(error) => return Err(error.to_string()),
    };
    let mut pending =
        StagedFile::write(store.directory(), "campaign", &bytes).map_err(|e| e.to_string())?;
    if let Some(previous) = previous {
        let old_backup = match read_file(&backup) {
            Ok(bytes) => Some(bytes),
            Err(StoreError::MissingSave { .. }) => None,
            Err(error) => return Err(error.to_string()),
        };
        let mut rollback = old_backup
            .as_deref()
            .map(|bytes| StagedFile::write(store.directory(), "campaign-rollback", bytes))
            .transpose()
            .map_err(|e| e.to_string())?;
        let mut pending_backup = StagedFile::write(store.directory(), "campaign-backup", &previous)
            .map_err(|e| e.to_string())?;
        pending_backup.replace(&backup).map_err(|e| e.to_string())?;
        if let Err(original) = pending.replace(&primary) {
            let restored = if let Some(staged) = &mut rollback {
                staged.replace(&backup)
            } else {
                fs::remove_file(&backup).map_err(|e| StoreError::Io {
                    action: "undo campaign backup",
                    path: backup.clone(),
                    source: e,
                })
            };
            if let Err(rollback_error) = restored {
                let recovery = rollback.as_mut().map(StagedFile::retain);
                return Err(format!(
                    "{original}; backup rollback failed: {rollback_error}; recovery: {recovery:?}"
                ));
            }
            return Err(original.to_string());
        }
    } else {
        pending.replace(&primary).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn store() -> SaveStore {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        SaveStore::new(std::env::temp_dir().join(format!(
            "parkbound-campaign-test-{}-{stamp} é",
            std::process::id()
        )))
    }

    #[test]
    fn awards_unlock_next_park_and_survive_reopen() {
        let store = store();
        let mut campaign = Campaign::load(Some(store.clone()));
        assert!(!campaign.unlocked(ScenarioKind::BoardwalkSprint));
        campaign.record_completed(ScenarioKind::MeadowChallenge, 30);
        assert!(campaign.unlocked(ScenarioKind::BoardwalkSprint));
        let primary = fs::read(store.directory().join(PRIMARY)).unwrap();
        campaign.record_completed(ScenarioKind::MeadowChallenge, 31);
        assert_eq!(primary, fs::read(store.directory().join(PRIMARY)).unwrap());
        campaign.record_completed(ScenarioKind::BoardwalkSprint, 60);
        let reopened = Campaign::load(Some(store.clone()));
        assert!(reopened.progress.meadow_awarded && reopened.progress.boardwalk_awarded);
        assert!(
            !CampaignProgress::from_bytes(&fs::read(store.directory().join(BACKUP)).unwrap())
                .unwrap()
                .boardwalk_awarded
        );
        fs::remove_dir_all(store.directory()).unwrap();
    }

    #[test]
    fn invalid_primary_recovers_backup_without_losing_awards() {
        let store = store();
        let mut campaign = Campaign::load(Some(store.clone()));
        campaign.record_completed(ScenarioKind::MeadowChallenge, 1);
        campaign.record_completed(ScenarioKind::BoardwalkSprint, 2);
        fs::write(store.directory().join(PRIMARY), b"broken").unwrap();
        let recovered = Campaign::load(Some(store.clone()));
        assert!(recovered.progress.meadow_awarded);
        assert!(!recovered.progress.boardwalk_awarded);
        assert!(recovered.notice.unwrap().contains("backup"));
        fs::remove_dir_all(store.directory()).unwrap();
    }

    #[test]
    fn unknown_version_and_fields_do_not_unlock() {
        let store = store();
        fs::create_dir(store.directory()).unwrap();
        let mut older = CampaignProgress::empty();
        older.meadow_awarded = true;
        fs::write(store.directory().join(BACKUP), older.bytes()).unwrap();
        fs::write(store.directory().join(PRIMARY), br#"{"format":"parkbound-campaign","version":2,"meadow_awarded":true,"boardwalk_awarded":false,"future_medal":true}"#).unwrap();
        let original = fs::read(store.directory().join(PRIMARY)).unwrap();
        let campaign = Campaign::load(Some(store.clone()));
        assert!(!campaign.unlocked(ScenarioKind::BoardwalkSprint));
        assert!(campaign.blocked);
        assert_eq!(original, fs::read(store.directory().join(PRIMARY)).unwrap());
        let mut value = CampaignProgress::empty().bytes();
        value.extend_from_slice(b"garbage");
        assert!(CampaignProgress::from_bytes(&value).is_err());
        fs::remove_dir_all(store.directory()).unwrap();
    }

    #[test]
    fn completed_loaded_park_awards_once_through_application_system() {
        let store = store();
        let mut park = crate::save_check::challenge::fixture().unwrap();
        crate::save_check::challenge::replay(&mut park).unwrap();
        let original = park.to_save_bytes().unwrap();
        let mut app = App::new();
        app.insert_resource(Simulation(park))
            .insert_resource(ViewState::default())
            .insert_resource(Campaign::load(Some(store.clone())))
            .add_systems(Update, observe);
        app.update();
        assert!(app.world().resource::<Campaign>().progress.meadow_awarded);
        assert!(
            app.world()
                .resource::<ViewState>()
                .scenario_notice
                .as_ref()
                .unwrap()
                .contains("award earned")
        );
        assert_eq!(
            app.world()
                .resource::<Simulation>()
                .0
                .to_save_bytes()
                .unwrap(),
            original
        );
        let award = fs::read(store.directory().join(PRIMARY)).unwrap();
        app.update();
        assert_eq!(fs::read(store.directory().join(PRIMARY)).unwrap(), award);
        fs::remove_dir_all(store.directory()).unwrap();
    }
}
