//! Reporting helpers shared by opt-in diagnostics, never the simulation clock.

use std::{ffi::OsString, fs::OpenOptions, io::Write, path::Path, time::Duration};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct TimingSummary {
    pub samples: usize,
    pub total_ms: f64,
    pub min_ms: f64,
    pub mean_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
}

/// Nearest-rank percentiles, including every measured sample and no trimming.
pub(super) fn summarize(samples: &[Duration]) -> Option<TimingSummary> {
    if samples.is_empty() {
        return None;
    }
    let mut values: Vec<_> = samples.iter().map(|v| v.as_secs_f64() * 1000.0).collect();
    values.sort_by(f64::total_cmp);
    let total_ms = values.iter().sum::<f64>();
    let percentile = |percent: usize| values[(values.len() * percent).div_ceil(100) - 1];
    Some(TimingSummary {
        samples: values.len(),
        total_ms,
        min_ms: values[0],
        mean_ms: total_ms / values.len() as f64,
        p50_ms: percentile(50),
        p95_ms: percentile(95),
        p99_ms: percentile(99),
        max_ms: values[values.len() - 1],
    })
}

pub(super) fn validate_output(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || path.extension().is_none_or(|v| v != "json") {
        return Err("measurement output must be an absolute path ending in .json".into());
    }
    if path.exists() || !path.parent().is_some_and(Path::is_dir) {
        return Err("measurement output must be new, with an existing parent directory".into());
    }
    Ok(())
}

/// An existing report can never be replaced, even if created after validation.
pub(super) fn write_json_new(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    validate_output(path)?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.sync_all())
        .map_err(|e| e.to_string())
}

/// Refuse ambiguous diagnostics before opening a window or touching save files.
pub(super) fn validate_modes(args: &[OsString]) -> Result<(), String> {
    let modes = [
        "--benchmark",
        "--profile-frames",
        "--capture-frame",
        "--save-check",
    ];
    let count = args
        .iter()
        .filter(|v| modes.iter().any(|m| *v == m))
        .count();
    if count > 1 {
        return Err("choose exactly one diagnostic mode; repeated modes are invalid".into());
    }
    for (flags, mode) in [
        (
            &["--benchmark-case", "--benchmark-ticks"][..],
            "--benchmark",
        ),
        (
            &["--profile-samples", "--profile-no-vsync"][..],
            "--profile-frames",
        ),
    ] {
        if args.iter().any(|v| flags.iter().any(|flag| v == flag))
            && !args.iter().any(|v| v == mode)
        {
            return Err(format!("these measurement options require {mode}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_include_outliers_and_handle_empty_input() {
        assert!(summarize(&[]).is_none());
        let values: Vec<_> = (1..=100).rev().map(Duration::from_millis).collect();
        let s = summarize(&values).unwrap();
        assert_eq!(s.samples, 100);
        assert_eq!((s.min_ms, s.mean_ms, s.max_ms), (1.0, 50.5, 100.0));
        assert_eq!((s.p50_ms, s.p95_ms, s.p99_ms), (50.0, 95.0, 99.0));
        assert_eq!(summarize(&[Duration::ZERO]).unwrap().p99_ms, 0.0);
    }

    #[test]
    fn modes_cannot_mix_or_silently_ignore_measurement_options() {
        let args = |v: &[&str]| v.iter().map(OsString::from).collect::<Vec<_>>();
        assert!(validate_modes(&args(&["--benchmark", "--save-check"])).is_err());
        assert!(validate_modes(&args(&["--profile-frames", "--capture-frame"])).is_err());
        assert!(validate_modes(&args(&["--benchmark", "--benchmark"])).is_err());
        assert!(validate_modes(&args(&["--profile-no-vsync"])).is_err());
        assert!(validate_modes(&args(&["--benchmark", "--benchmark-ticks", "30"])).is_ok());
    }

    #[test]
    fn report_writer_refuses_overwrite_and_missing_parents() {
        let dir = std::env::temp_dir().join(format!(
            "Parkbound report test {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("result.json");
        write_json_new(&path, &serde_json::json!({"first": true})).unwrap();
        let original = std::fs::read(&path).unwrap();
        assert!(write_json_new(&path, &serde_json::json!({"first": false})).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert!(
            write_json_new(&dir.join("absent").join("x.json"), &serde_json::Value::Null).is_err()
        );
        assert!(validate_output(Path::new("relative.json")).is_err());
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
