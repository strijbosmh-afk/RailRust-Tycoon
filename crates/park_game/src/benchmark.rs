//! Repeatable CPU measurements. This path never starts Bevy or uses player saves.

use std::{
    ffi::OsString,
    hint::black_box,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bevy::app::AppExit;
use park_sim::{
    Park, Speed, TICKS_PER_SECOND,
    diagnostics::{DiagnosticCase, DiagnosticPathGraph, build_fixture},
};
use serde_json::{Value, json};

use crate::{CROWD_SEED, measurement, save_store::SaveStore, world};

const PHASE_BUDGET: Duration = Duration::from_secs(30);
const RUN_BUDGET: Duration = Duration::from_secs(150);

#[derive(Debug)]
struct Options {
    output: PathBuf,
    case: DiagnosticCase,
    ticks: usize,
}

fn options(args: &[OsString]) -> Result<Options, String> {
    let mut output = None;
    let mut case = None;
    let mut ticks = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let value = iter.next().ok_or("every benchmark option needs a value")?;
        match arg.to_str() {
            Some("--benchmark") if output.is_none() => output = Some(PathBuf::from(value)),
            Some("--benchmark-case") if case.is_none() => {
                case = Some(match value.to_str() {
                    Some("baseline") => DiagnosticCase::Baseline,
                    Some("save-scale") => DiagnosticCase::SaveScale,
                    Some("stress") => DiagnosticCase::Stress,
                    _ => return Err("benchmark case must be baseline, save-scale or stress".into()),
                });
            }
            Some("--benchmark-ticks") if ticks.is_none() => {
                ticks = Some(
                    value
                        .to_str()
                        .and_then(|v| v.parse::<usize>().ok())
                        .filter(|v| (30..=1800).contains(v))
                        .ok_or("benchmark ticks must be an integer from 30 through 1800")?,
                );
            }
            _ => return Err("unknown or repeated benchmark option".into()),
        }
    }
    let output = output.ok_or("--benchmark requires an absolute new .json output path")?;
    measurement::validate_output(&output)?;
    Ok(Options {
        output,
        case: case.unwrap_or(DiagnosticCase::Baseline),
        ticks: ticks.unwrap_or(300),
    })
}

pub(super) fn run(args: &[OsString]) -> Option<AppExit> {
    if !args.iter().any(|v| v == "--benchmark") {
        return None;
    }
    Some(
        match options(args).and_then(|options| {
            let report = measure(&options)?;
            measurement::write_json_new(&options.output, &report)
        }) {
            Ok(()) => AppExit::Success,
            Err(error) => {
                eprintln!("Performance benchmark failed: {error}");
                AppExit::error()
            }
        },
    )
}

/// Budgets are checked BETWEEN complete operations, never cancel a simulation
/// tick or omit a slow sample. The wrapper adds a hard child-process timeout.
fn phase(
    requested: usize,
    run_start: Instant,
    mut operation: impl FnMut(usize) -> Result<usize, String>,
) -> Result<Value, String> {
    let start = Instant::now();
    let mut samples = Vec::with_capacity(requested);
    let mut work_units = 0_u64;
    for index in 0..requested {
        if start.elapsed() >= PHASE_BUDGET || run_start.elapsed() >= RUN_BUDGET {
            break;
        }
        let sample_start = Instant::now();
        work_units += black_box(operation(index)?) as u64;
        samples.push(sample_start.elapsed());
    }
    Ok(json!({
        "status": if samples.len() == requested {"complete"} else {"budget_exceeded"},
        "requested_samples": requested,
        "timing": measurement::summarize(&samples),
        "samples_ms": samples.iter().map(|d| d.as_secs_f64() * 1000.0).collect::<Vec<_>>(),
        "work_units": work_units
    }))
}

fn measure(options: &Options) -> Result<Value, String> {
    let run_start = Instant::now();
    let setup_start = Instant::now();
    let mut fixture = build_fixture(options.case, CROWD_SEED).map_err(|e| e.to_string())?;
    let setup_ms = setup_start.elapsed().as_secs_f64() * 1000.0;
    let warmup = phase(60, run_start, |_| {
        fixture.park.advance(Speed::Normal);
        Ok(1)
    })?;
    let first_tick = fixture.park.ticks();
    let simulation = phase(options.ticks, run_start, |_| {
        fixture.park.advance(Speed::Normal);
        black_box(&fixture.park);
        Ok(1)
    })?;
    let last_tick = fixture.park.ticks();
    let park = &fixture.park;

    // Graph timing includes the diagnostic stable-ID vector. It is therefore
    // slightly more work than production graph construction alone.
    let graph = DiagnosticPathGraph::build(park);
    let graph_build = phase(30, run_start, |_| {
        let graph = DiagnosticPathGraph::build(black_box(park));
        let nodes = graph.node_ids().len();
        black_box(graph);
        Ok(nodes)
    })?;
    let ids = graph.node_ids();
    let route = |index: usize| -> Result<usize, String> {
        let from = index.wrapping_mul(37) % ids.len();
        let offset = 1 + index.wrapping_mul(17) % (ids.len() - 1);
        let to = (from + offset) % ids.len();
        let route = graph
            .route(ids[from], ids[to])
            .ok_or("fixture route is disconnected")?;
        let length = route.len();
        black_box(route);
        Ok(length)
    };
    for index in 0..16 {
        if run_start.elapsed() < RUN_BUDGET {
            route(index)?;
        }
    }
    let pathfinding = phase(128, run_start, route)?;

    // Timing is CPU geometry generation and deallocation; no GPU is created.
    let mesh = |kind: &str| {
        let mesh = match kind {
            "terrain" => world::build_scene(park, 0, true, world::ViewMode::Surface),
            "guests" => world::guest_mesh(park, 0, world::ViewMode::Surface),
            _ => world::train_mesh(park, 0, world::ViewMode::Surface),
        };
        let vertices = mesh.count_vertices();
        black_box(mesh);
        Ok(vertices)
    };
    for kind in ["terrain", "guests", "train"] {
        if run_start.elapsed() < RUN_BUDGET {
            mesh(kind)?;
        }
    }
    let terrain_mesh = phase(5, run_start, |_| mesh("terrain"))?;
    let guest_mesh = phase(30, run_start, |_| mesh("guests"))?;
    let train_mesh = phase(30, run_start, |_| mesh("train"))?;

    let persistence = if fixture.metadata.save_supported && run_start.elapsed() >= RUN_BUDGET {
        json!({"status":"budget_exceeded", "reason":"Run budget expired before codec/store measurements."})
    } else if fixture.metadata.save_supported {
        let bytes = park.to_save_bytes().map_err(|e| e.to_string())?;
        if Park::from_save_bytes(&bytes).map_err(|e| e.to_string())? != *park {
            return Err("codec correctness check failed before measuring".into());
        }
        let encode = phase(30, run_start, |_| {
            let bytes = park.to_save_bytes().map_err(|e| e.to_string())?;
            let length = bytes.len();
            black_box(bytes);
            Ok(length)
        })?;
        let decode = phase(30, run_start, |_| {
            black_box(Park::from_save_bytes(black_box(&bytes)).map_err(|e| e.to_string())?);
            Ok(bytes.len())
        })?;
        let disk = measure_store(park, run_start)?;
        json!({"status":"supported", "encoded_bytes":bytes.len(),
            "encode":encode, "decode":decode, "filesystem":disk})
    } else {
        let error = park
            .to_save_bytes()
            .expect_err("stress must exceed production save limits");
        json!({"status":"unsupported", "reason":error.to_string(),
            "note":"The 5,000-guest fixture exceeds the unchanged 256-guest save cap; no timings invented."})
    };
    let phases = [
        &warmup,
        &simulation,
        &graph_build,
        &pathfinding,
        &terrain_mesh,
        &guest_mesh,
        &train_mesh,
    ];
    let partial = phases.iter().any(|p| p["status"] != "complete")
        || (fixture.metadata.save_supported
            && (persistence["encode"]["status"] != "complete"
                || persistence["decode"]["status"] != "complete"
                || persistence["filesystem"]["save"]["status"] != "complete"
                || persistence["filesystem"]["load"]["status"] != "complete"));
    Ok(json!({
        "schema_version":1, "kind":"parkbound-cpu-benchmark",
        "version":env!("CARGO_PKG_VERSION"), "status":if partial {"partial"} else {"complete"},
        "build_profile":if cfg!(debug_assertions) {"debug"} else {"release"},
        "os":std::env::consts::OS, "architecture":std::env::consts::ARCH,
        "recorded_unix_seconds":SystemTime::now().duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs()),
        "fixture":fixture.metadata,
        "method": {
            "clock":"std::time::Instant wall elapsed; release build required for comparison",
            "percentiles":"nearest rank; all samples retained; no outliers discarded",
            "phase_soft_budget_seconds":30, "run_soft_budget_seconds":150,
            "setup_excluded_ms":setup_ms, "elapsed_seconds":run_start.elapsed().as_secs_f64(),
            "ticks_per_simulation_second":TICKS_PER_SECOND,
            "simulation_scope":"complete production Park::advance(Normal), including crowds, graph rebuild and train",
            "graph_scope":"production path graph plus diagnostic stable-ID list, allocation and drop",
            "route_scope":"production full BFS plus route reconstruction; 16 warmups, deterministic distinct endpoint pairs",
            "mesh_scope":"CPU allocation, geometry generation and drop; 1 warmup per mesh; geometry.terrain is the entire static scene including paths, tracks and decorations; no rendering/GPU",
            "codec_scope":"production validation and encode/decode, allocation and drop; round-trip checked before timing",
            "filesystem_scope":"production SaveStore with flush and backup rotation, warm OS cache, isolated OS-temp directory; includes codec; not cold-disk or OneDrive throughput"
        },
        "target": {
            "map":[256,256], "guests":5000, "rides":50, "resolution":[1920,1080], "frames_per_second":60,
            "status":"unsupported",
            "limitations":["Current model supports one circuit and one train, not 50 rides.",
                "CPU stress fixture is not a playable 256-map renderer benchmark.",
                "Frame profiling measures the normal 32-map demo; no GPU timestamp or presentation timing."]
        },
        "warmup":warmup, "simulation":simulation,
        "simulation_ticks":{"before":first_tick,"after":last_tick,"advanced":last_tick-first_tick},
        "navigation":{"graph":graph.stats(),"build":graph_build,"route":pathfinding},
        "geometry":{"terrain":terrain_mesh,"guests":guest_mesh,"train":train_mesh},
        "persistence":persistence
    }))
}

/// Only the two known files created in this unique directory are removed.
/// Failed operations retain evidence instead of sweeping unrelated temp files.
fn measure_store(park: &Park, run_start: Instant) -> Result<Value, String> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "Parkbound benchmark {} {suffix}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).map_err(|e| e.to_string())?;
    let store = SaveStore::new(directory.clone());
    store.save(park).map_err(|e| e.to_string())?;
    if store.load().map_err(|e| e.to_string())? != *park {
        return Err("filesystem correctness check failed before measuring".into());
    }
    let save = phase(5, run_start, |_| {
        store.save(black_box(park)).map_err(|e| e.to_string())?;
        Ok(1)
    })?;
    let load = phase(5, run_start, |_| {
        black_box(store.load().map_err(|e| e.to_string())?);
        Ok(1)
    })?;
    for path in [store.save_path(), store.backup_path()] {
        if path.exists() {
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    std::fs::remove_dir(directory).map_err(|e| e.to_string())?;
    Ok(json!({"save":save,"load":load,"temporary_files_removed":true}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_options_are_bounded_and_strict() {
        let path = std::env::temp_dir().join(format!(
            "unused-parkbound-options-{}.json",
            std::process::id()
        ));
        let valid = || vec![OsString::from("--benchmark"), path.clone().into_os_string()];
        assert_eq!(options(&valid()).unwrap().ticks, 300);
        for invalid in ["0", "29", "1801", "banana"] {
            let mut args = valid();
            args.extend([OsString::from("--benchmark-ticks"), OsString::from(invalid)]);
            assert!(options(&args).is_err());
        }
        let mut args = valid();
        args.extend([OsString::from("--save-dir"), OsString::from("elsewhere")]);
        assert!(options(&args).is_err());
        assert!(options(&[OsString::from("--benchmark")]).is_err());
        assert!(
            options(&[
                OsString::from("--benchmark"),
                OsString::from("relative.json")
            ])
            .is_err()
        );
    }

    #[test]
    fn incomplete_phases_report_actual_work_without_fabricated_samples() {
        let old = Instant::now() - RUN_BUDGET;
        let result = phase(30, old, |_| panic!("expired budget must not run work")).unwrap();
        assert_eq!(result["status"], "budget_exceeded");
        assert_eq!(result["work_units"], 0);
        assert!(result["timing"].is_null());
        assert_eq!(result["samples_ms"].as_array().unwrap().len(), 0);
    }
}
