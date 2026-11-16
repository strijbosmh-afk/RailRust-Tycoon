//! Isolated long-run replay and save-rotation checks for beta builds.
use std::{ffi::OsString, fs, path::PathBuf, time::Instant};

use bevy::app::AppExit;
use park_sim::{
    Park, Speed,
    diagnostics::{DiagnosticCase, build_fixture},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{measurement, save_store::SaveStore};

const REPORT_NAME: &str = "beta-report.json";
const SEED: u64 = 20261116;

#[derive(Debug)]
struct Options {
    directory: PathBuf,
    case: DiagnosticCase,
    ticks: u64,
    interval: u64,
    seed: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    offset: u64,
    file: String,
    tick: u64,
    bytes: usize,
    guests: usize,
    arrivals: u64,
    departed: u64,
    rides_completed: u64,
    services_used: u64,
    cash_cents: u64,
    operating_profit_cents: i128,
    issues: usize,
    operations: serde_json::Value,
    save_ms: f64,
    load_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    schema_version: u32,
    kind: String,
    version: String,
    case: String,
    seed: u64,
    requested_ticks: u64,
    completed_ticks: u64,
    checkpoint_interval: u64,
    status: String,
    error: Option<String>,
    hardware_role: String,
    fixture: serde_json::Value,
    simulation_timing: Option<measurement::TimingSummary>,
    checkpoints: Vec<Checkpoint>,
}

fn options(args: &[OsString]) -> Result<Options, String> {
    let mut directory = None;
    let mut case = None;
    let mut ticks = None;
    let mut interval = None;
    let mut seed = None;
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        let value = args.next().ok_or("Each beta option needs a value")?;
        match flag.to_str() {
            Some("--beta-check") if directory.is_none() => directory = Some(PathBuf::from(value)),
            Some("--beta-case") if case.is_none() => {
                case = Some(DiagnosticCase::parse(value.to_str().unwrap_or(""))
                    .filter(|case| matches!(case, DiagnosticCase::Starter | DiagnosticCase::MediumMixed
                        | DiagnosticCase::LargeMixed | DiagnosticCase::CrowdCapacity))
                    .ok_or("Beta case must be starter, medium-mixed, large-mixed or crowd-capacity")?);
            }
            Some("--beta-ticks") if ticks.is_none() => ticks = Some(number(value, 900, 1_296_000)?),
            Some("--beta-checkpoint") if interval.is_none() => {
                interval = Some(number(value, 900, 108_000)?)
            }
            Some("--beta-seed") if seed.is_none() => seed = Some(number(value, 0, u64::MAX)?),
            _ => return Err("Unknown or repeated beta option".into()),
        }
    }
    let options = Options {
        directory: directory.ok_or("--beta-check needs a new absolute directory")?,
        case: case.unwrap_or(DiagnosticCase::MediumMixed),
        ticks: ticks.unwrap_or(108_000),
        interval: interval.unwrap_or(9_000),
        seed: seed.unwrap_or(SEED),
    };
    if !options.directory.is_absolute()
        || options.directory.exists()
        || !options.directory.parent().is_some_and(|p| p.is_dir())
    {
        return Err("Beta output must be a new absolute directory with an existing parent".into());
    }
    if options.ticks.div_ceil(options.interval) > 128 {
        return Err("Choose at most 128 beta checkpoint intervals".into());
    }
    Ok(options)
}

fn number(value: &OsString, min: u64, max: u64) -> Result<u64, String> {
    value
        .to_str()
        .and_then(|v| v.parse().ok())
        .filter(|v| (min..=max).contains(v))
        .ok_or_else(|| format!("Expected an integer from {min} through {max}"))
}

pub(super) fn run(args: &[OsString]) -> Option<AppExit> {
    let result = if args.iter().any(|v| v == "--beta-check") {
        options(args).and_then(|options| execute(&options))
    } else if args.iter().any(|v| v == "--beta-verify") {
        if args.len() != 2 || args[0] != "--beta-verify" {
            Err("Use --beta-verify with one absolute beta directory".into())
        } else {
            verify(&PathBuf::from(&args[1]))
        }
    } else {
        return None;
    };
    Some(match result {
        Ok(()) => AppExit::Success,
        Err(error) => {
            eprintln!("Beta verification failed: {error}");
            AppExit::error()
        }
    })
}

fn execute(options: &Options) -> Result<(), String> {
    let fixture = build_fixture(options.case, options.seed)?;
    fs::create_dir(&options.directory).map_err(|e| e.to_string())?;
    let mut report = Report {
        schema_version: 1,
        kind: "parkbound-beta-replay".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        case: options.case.name().into(),
        seed: options.seed,
        requested_ticks: options.ticks,
        completed_ticks: 0,
        checkpoint_interval: options.interval,
        status: "running".into(),
        error: None,
        hardware_role:
            "diagnostics only; representative hardware acceptance is recorded separately".into(),
        fixture: serde_json::to_value(fixture.metadata).map_err(|e| e.to_string())?,
        simulation_timing: None,
        checkpoints: Vec::new(),
    };
    let result = exercise(options, fixture.park, &mut report);
    report.status = if result.is_ok() { "complete" } else { "failed" }.into();
    report.error = result.as_ref().err().cloned();
    measurement::write_json_new(
        &options.directory.join(REPORT_NAME),
        &serde_json::to_value(&report).map_err(|e| e.to_string())?,
    )?;
    result
}

fn exercise(options: &Options, mut uninterrupted: Park, report: &mut Report) -> Result<(), String> {
    let store = SaveStore::new(options.directory.clone());
    let mut restored = uninterrupted.clone();
    let mut samples = Vec::new();
    loop {
        if uninterrupted != restored {
            return Err(format!(
                "Replay diverged at offset {}",
                report.completed_ticks
            ));
        }
        check_accounts(&uninterrupted)?;
        let start = Instant::now();
        store.save(&uninterrupted).map_err(|e| e.to_string())?;
        let save_ms = start.elapsed().as_secs_f64() * 1000.0;
        let bytes = fs::read(store.save_path()).map_err(|e| e.to_string())?;
        let start = Instant::now();
        restored = store.load().map_err(|e| e.to_string())?;
        let load_ms = start.elapsed().as_secs_f64() * 1000.0;
        if restored != uninterrupted {
            return Err("Save/load changed park state".into());
        }
        if let Some(previous) = report.checkpoints.last() {
            let backup = store.load_backup().map_err(|e| e.to_string())?;
            let expected =
                fs::read(options.directory.join(&previous.file)).map_err(|e| e.to_string())?;
            if backup != Park::from_save_bytes(&expected).map_err(|e| e.to_string())? {
                return Err("Backup rotation lost the previous checkpoint".into());
            }
        }
        let file = format!("checkpoint-{:03}.park.json", report.checkpoints.len());
        crate::save_check::write_new(&options.directory.join(&file), &bytes)?;
        let finance = uninterrupted.finance_summary();
        let departed = uninterrupted.departed_visitors();
        report.checkpoints.push(Checkpoint {
            offset: report.completed_ticks,
            file,
            tick: uninterrupted.ticks(),
            bytes: bytes.len(),
            guests: uninterrupted.guests().len(),
            arrivals: uninterrupted.visitor_flow_stats().spawned,
            departed: departed.count,
            rides_completed: uninterrupted
                .guests()
                .iter()
                .map(|g| g.rides_completed)
                .sum::<u64>()
                + departed.rides.iter().map(|r| r.completed).sum::<u64>(),
            services_used: uninterrupted
                .guests()
                .iter()
                .map(|g| g.services_used)
                .sum::<u64>()
                + departed.services_used,
            cash_cents: finance.cash_cents,
            operating_profit_cents: finance.operating_profit_cents,
            issues: uninterrupted.operation_issues().len(),
            operations: operation_snapshot(&uninterrupted),
            save_ms,
            load_ms,
        });
        if report.completed_ticks == options.ticks {
            break;
        }
        let count = options.interval.min(options.ticks - report.completed_ticks);
        for _ in 0..count {
            let start = Instant::now();
            uninterrupted.advance(Speed::Normal);
            samples.push(start.elapsed());
            restored.advance(Speed::Normal);
        }
        report.completed_ticks += count;
    }
    report.simulation_timing = measurement::summarize(&samples);
    Ok(())
}

fn check_accounts(park: &Park) -> Result<(), String> {
    crate::save_check::summary::check(park)?;
    let f = park.finance_summary();
    if i128::from(f.starting_cash_cents) + f.net_cash_cents != i128::from(f.cash_cents) {
        return Err("Cash does not reconcile to the ledger".into());
    }
    let flow = park.visitor_flow_stats();
    if flow.tracked as u64 + flow.departed != flow.spawned {
        return Err("Visitor admissions and departures do not reconcile".into());
    }
    Ok(())
}

fn operation_snapshot(park: &Park) -> serde_json::Value {
    let rides: Vec<_> = park
        .all_rides()
        .map(|ride| {
            json!({
                "id":ride.id.0, "name":park.ride_name(ride.id),
                "open":park.ride_is_open(ride.id), "boarded":ride.boarded,
                "completed":ride.completed, "abandoned":ride.abandoned,
                "revenue_cents":ride.revenue_cents, "ticket_price_cents":ride.ticket_price_cents,
                "unaffordable_rejections":ride.unaffordable_rejections,
                "value_rejections":ride.value_rejections,
                "average_rating_tenths":park.ride_experience_stats(ride.id).average_rating_tenths,
            })
        })
        .collect();
    let facilities: Vec<_> = park.services().iter().map(|service| json!({
        "id":service.id.0, "kind":service.kind, "open":service.open,
        "served":service.served, "revenue_cents":service.revenue_cents,
        "upkeep_cents":service.upkeep_cents, "supplies_cents":service.supplies_cents,
        "price_cents":service.price_cents, "unaffordable_rejections":service.unaffordable_rejections,
    })).collect();
    let staff: Vec<_> = park
        .park_staff()
        .iter()
        .map(|member| {
            json!({
                "role":member.role, "working":member.working, "completed":member.completed,
                "wages_cents":member.wages_paid_cents,
            })
        })
        .collect();
    let mut thoughts = std::collections::BTreeMap::<String, usize>::new();
    for guest in park.guests() {
        *thoughts.entry(guest.thought.to_string()).or_default() += 1;
    }
    json!({"rides":rides,"facilities":facilities,"staff":staff,
        "guest_thoughts":thoughts,"satisfaction":park.guest_experience_summary().average_satisfaction,
        "research_spent_cents":park.management_status().research_spent,
        "lost_guests":park.crowd_stats().lost,
        "weather":park.park_life_status().weather})
}

fn verify(directory: &std::path::Path) -> Result<(), String> {
    if !directory.is_absolute() {
        return Err("Use an absolute beta directory".into());
    }
    let report_bytes = fs::read(directory.join(REPORT_NAME)).map_err(|e| e.to_string())?;
    if report_bytes.len() > 4 * 1024 * 1024 {
        return Err("Beta report is too large".into());
    }
    let report: Report = serde_json::from_slice(&report_bytes).map_err(|e| e.to_string())?;
    if report.schema_version != 1
        || report.kind != "parkbound-beta-replay"
        || report.status != "complete"
        || report.completed_ticks != report.requested_ticks
        || report.requested_ticks > 1_296_000
        || report.requested_ticks < 900
        || !(900..=108_000).contains(&report.checkpoint_interval)
        || DiagnosticCase::parse(&report.case).is_none()
        || report.checkpoints.is_empty()
        || report.checkpoints.len() > 129
    {
        return Err("Beta report is incomplete or unsupported".into());
    }
    if report.checkpoints.len() as u64
        != report.requested_ticks.div_ceil(report.checkpoint_interval) + 1
    {
        return Err("Beta report is missing checkpoints".into());
    }
    let store = SaveStore::new(directory.to_path_buf());
    let primary = fs::read(store.save_path()).map_err(|e| e.to_string())?;
    let backup = fs::read(store.backup_path()).ok();
    let mut replay: Option<Park> = None;
    let mut previous_offset = 0;
    for (index, checkpoint) in report.checkpoints.iter().enumerate() {
        if checkpoint.file != format!("checkpoint-{index:03}.park.json")
            || checkpoint.offset
                != (index as u64 * report.checkpoint_interval).min(report.requested_ticks)
            || checkpoint.offset > report.requested_ticks
        {
            return Err("Beta checkpoint order or filename is invalid".into());
        }
        let bytes = fs::read(directory.join(&checkpoint.file)).map_err(|e| e.to_string())?;
        if bytes.len() != checkpoint.bytes {
            return Err("Beta checkpoint size changed".into());
        }
        let expected = Park::from_save_bytes(&bytes).map_err(|e| e.to_string())?;
        if let Some(park) = &mut replay {
            for _ in previous_offset..checkpoint.offset {
                park.advance(Speed::Normal);
            }
            if *park != expected {
                return Err(format!(
                    "Fresh-process replay diverged at checkpoint {index}"
                ));
            }
        } else {
            if checkpoint.offset != 0 {
                return Err("Beta replay needs its initial checkpoint".into());
            }
            replay = Some(expected.clone());
        }
        if expected.ticks() != checkpoint.tick
            || operation_snapshot(&expected) != checkpoint.operations
        {
            return Err("Checkpoint tick mismatch".into());
        }
        check_accounts(&expected)?;
        previous_offset = checkpoint.offset;
    }
    if previous_offset != report.requested_ticks
        || replay.as_ref() != Some(&store.load().map_err(|e| e.to_string())?)
    {
        return Err("Final checkpoint does not match the saved park".into());
    }
    if primary != fs::read(store.save_path()).map_err(|e| e.to_string())?
        || backup != fs::read(store.backup_path()).ok()
    {
        return Err("Replay changed the saved slots".into());
    }
    measurement::write_json_new(
        &directory.join("beta-verification.json"),
        &json!({
            "status":"complete", "version":env!("CARGO_PKG_VERSION"),
            "case":report.case, "ticks":report.completed_ticks,
            "checkpoints":report.checkpoints.len(), "exact_replay":true,
            "primary_and_backup_unchanged":true
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn beta_writer_replays_and_rejects_a_changed_checkpoint() {
        let directory = std::env::temp_dir().join(format!(
            "Parkbound beta test {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let options = Options {
            directory: directory.clone(),
            case: DiagnosticCase::Starter,
            ticks: 900,
            interval: 900,
            seed: SEED,
        };
        execute(&options).unwrap();
        let before = fs::read(directory.join("park.park.json")).unwrap();
        verify(&directory).unwrap();
        assert_eq!(before, fs::read(directory.join("park.park.json")).unwrap());
        fs::write(directory.join("checkpoint-001.park.json"), b"changed").unwrap();
        assert!(verify(&directory).unwrap_err().contains("size changed"));
        for entry in fs::read_dir(&directory).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn beta_options_reject_ambiguous_modes_and_unsafe_outputs() {
        let args = |values: &[&str]| values.iter().map(OsString::from).collect::<Vec<_>>();
        assert!(options(&args(&["--beta-check", "relative"])).is_err());
        assert!(options(&args(&["--beta-check", "relative", "--save-dir", "x"])).is_err());
        assert!(
            options(&args(&[
                "--beta-check",
                "relative",
                "--beta-case",
                "stress"
            ]))
            .is_err()
        );
        assert!(number(&OsString::from("900"), 900, 108_000).is_ok());
        assert!(number(&OsString::from("899"), 900, 108_000).is_err());
    }
}
