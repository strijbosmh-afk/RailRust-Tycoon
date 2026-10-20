//! Opt-in observations of Bevy's asynchronously delivered render diagnostics.
//!
//! Bevy 0.17 stamps these measurements when its main-world sync receives them,
//! not when their render frame began. Its private bridge retains only the latest
//! result batch. Receipt frames below therefore never claim source-frame pairing.

use std::collections::BTreeMap;

use bevy::{
    diagnostic::DiagnosticsStore,
    platform::time::Instant,
    prelude::*,
    render::{
        RenderApp, diagnostic::RenderDiagnosticsPlugin, renderer::RenderDevice,
        settings::WgpuFeatures,
    },
};
use serde::Serialize;
use serde_json::{Value, json};

const MAX_PATHS: usize = 128;
const MAX_PATH_BYTES: usize = 512;
const MAX_SAMPLES_PER_PATH: usize = 4096;
const MAX_TOTAL_SAMPLES: usize = 131_072;

/// Called only by the explicitly requested GPU profile. Normal play installs
/// neither the recorder nor this resource and performs no diagnostic readback.
pub(super) fn install(app: &mut App) {
    let render_app_present = app.get_sub_app(RenderApp).is_some();
    app.init_resource::<DiagnosticsStore>();
    if render_app_present && !app.is_plugin_added::<RenderDiagnosticsPlugin>() {
        app.add_plugins(RenderDiagnosticsPlugin);
    }
    app.insert_resource(GpuProbe {
        render_app_present,
        ..default()
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    CpuElapsed,
    GpuElapsed,
    PipelineCounter,
}

impl Kind {
    fn for_path(path: &str) -> Option<Self> {
        if !path.starts_with("render/") {
            return None;
        }
        match path.rsplit('/').next()? {
            "elapsed_cpu" => Some(Self::CpuElapsed),
            "elapsed_gpu" => Some(Self::GpuElapsed),
            "vertex_shader_invocations"
            | "clipper_invocations"
            | "clipper_primitives_out"
            | "fragment_shader_invocations"
            | "compute_shader_invocations" => Some(Self::PipelineCounter),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::CpuElapsed => "cpu_elapsed",
            Self::GpuElapsed => "gpu_elapsed",
            Self::PipelineCounter => "pipeline_counter",
        }
    }

    fn unit(self) -> &'static str {
        if self == Self::PipelineCounter {
            "count"
        } else {
            "ms"
        }
    }
}

#[derive(Debug, Serialize)]
struct Sample {
    observed_main_frame: u64,
    receipt_offset_ms: f64,
    value: f64,
}

#[derive(Debug)]
struct Span {
    kind: Kind,
    last_seen: Option<Instant>,
    samples: Vec<Sample>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Capabilities {
    known: bool,
    timestamp_query: bool,
    inside_encoders: bool,
    inside_passes: bool,
    pipeline_statistics: bool,
}

impl Capabilities {
    fn from_features(features: WgpuFeatures) -> Self {
        Self {
            known: true,
            timestamp_query: features.contains(WgpuFeatures::TIMESTAMP_QUERY),
            inside_encoders: features.contains(WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS),
            inside_passes: features.contains(WgpuFeatures::TIMESTAMP_QUERY_INSIDE_PASSES),
            pipeline_statistics: features.contains(WgpuFeatures::PIPELINE_STATISTICS_QUERY),
        }
    }

    fn pass_timestamps_enabled(self) -> bool {
        // Bevy's FrameData::write_timestamp checks encoder support even when
        // writing inside a pass. TIMESTAMP_QUERY alone is not sufficient.
        self.timestamp_query && self.inside_encoders && self.inside_passes
    }

    fn json(self) -> Value {
        json!({
            "device_features_known": self.known,
            "timestamp_query": self.known.then_some(self.timestamp_query),
            "timestamp_inside_encoders": self.known.then_some(self.inside_encoders),
            "timestamp_inside_passes": self.known.then_some(self.inside_passes),
            "pipeline_statistics_query": self.known.then_some(self.pipeline_statistics),
            "pass_timestamps_enabled": self.known.then_some(self.pass_timestamps_enabled()),
            "adapter_supported_features": null,
            "scope": "enabled RenderDevice features; adapter support was not separately queried",
        })
    }
}

#[derive(Resource, Default)]
pub(super) struct GpuProbe {
    render_app_present: bool,
    capabilities: Capabilities,
    started: Option<Instant>,
    spans: BTreeMap<String, Span>,
    observation_calls: u64,
    first_observed_main_frame: Option<u64>,
    last_observed_main_frame: Option<u64>,
    last_delivery_offset_ms: Option<f64>,
    retained: usize,
    omitted_limit: u64,
    invalid_values: u64,
    pre_window_latest_observations: u64,
}

impl GpuProbe {
    /// Starts the receipt window, independently of the simulation clock.
    /// A late result generated during warmup may still arrive inside this window.
    pub(super) fn begin_window(&mut self) {
        self.begin_at(Instant::now());
    }

    fn begin_at(&mut self, started: Instant) {
        *self = Self {
            render_app_present: self.render_app_present,
            capabilities: self.capabilities,
            started: Some(started),
            ..default()
        };
    }

    /// Observe after Bevy's PreUpdate diagnostic sync, normally in Last.
    /// Repeated observations of the same path/receipt timestamp are ignored.
    pub(super) fn collect(
        &mut self,
        diagnostics: &DiagnosticsStore,
        observed_main_frame: u64,
        device: Option<&RenderDevice>,
    ) {
        if let Some(device) = device {
            self.capabilities = Capabilities::from_features(device.features());
        }
        let Some(started) = self.started else {
            return;
        };
        self.observation_calls += 1;
        self.first_observed_main_frame
            .get_or_insert(observed_main_frame);
        self.last_observed_main_frame = Some(observed_main_frame);
        // Sort only the small set of built-in diagnostic paths so bounded
        // selection and reports remain stable despite DiagnosticsStore's HashMap.
        let mut relevant: Vec<_> = diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.is_enabled && Kind::for_path(diagnostic.path().as_str()).is_some()
            })
            .collect();
        relevant.sort_unstable_by(|a, b| a.path().as_str().cmp(b.path().as_str()));
        for diagnostic in relevant {
            let Some(measurement) = diagnostic.measurement() else {
                continue;
            };
            let path = diagnostic.path().as_str();
            let kind = Kind::for_path(path).expect("filtered above");
            if path.len() > MAX_PATH_BYTES
                || (!self.spans.contains_key(path) && self.spans.len() >= MAX_PATHS)
            {
                self.omitted_limit += 1;
                continue;
            }
            let span = self.spans.entry(path.to_owned()).or_insert_with(|| Span {
                kind,
                last_seen: None,
                samples: Vec::new(),
            });
            if span.last_seen.is_some_and(|last| measurement.time <= last) {
                continue;
            }
            span.last_seen = Some(measurement.time);
            if measurement.time < started {
                self.pre_window_latest_observations += 1;
                continue;
            }
            let expected_suffix = if kind == Kind::PipelineCounter {
                ""
            } else {
                "ms"
            };
            if !measurement.value.is_finite()
                || measurement.value < 0.0
                || diagnostic.suffix.as_ref() != expected_suffix
            {
                self.invalid_values += 1;
                continue;
            }
            let receipt_offset_ms = measurement.time.duration_since(started).as_secs_f64() * 1000.0;
            self.last_delivery_offset_ms = Some(
                self.last_delivery_offset_ms
                    .map_or(receipt_offset_ms, |last| last.max(receipt_offset_ms)),
            );
            if self.retained >= MAX_TOTAL_SAMPLES || span.samples.len() >= MAX_SAMPLES_PER_PATH {
                self.omitted_limit += 1;
                continue;
            }
            span.samples.push(Sample {
                observed_main_frame,
                receipt_offset_ms,
                value: measurement.value,
            });
            self.retained += 1;
        }
    }

    pub(super) fn has_gpu_samples(&self) -> bool {
        self.spans
            .values()
            .any(|span| span.kind == Kind::GpuElapsed && !span.samples.is_empty())
    }

    pub(super) fn report(&self) -> Value {
        let gpu_timing_collected = self.has_gpu_samples();
        let status = if !self.render_app_present {
            "no_render_app"
        } else if gpu_timing_collected {
            "samples_received"
        } else if !self.capabilities.known {
            "device_unavailable"
        } else if !self.capabilities.pass_timestamps_enabled() {
            "unsupported"
        } else {
            "no_samples_received"
        };
        let spans: Vec<_> = self
            .spans
            .iter()
            .map(|(path, span)| {
                json!({
                    "path": path,
                    "kind": span.kind.name(),
                    "unit": span.kind.unit(),
                    "samples": span.samples,
                    "summary": summary(&span.samples),
                })
            })
            .collect();
        json!({
            "status": status,
            "gpu_timing_collected": gpu_timing_collected,
            "method": "bevy_render_diagnostics_async_readback",
            "time_unit": "ms",
            "source_frame_correlation": "unavailable_receipt_window_only",
            "capabilities": self.capabilities.json(),
            "receipt_window": {
                "started": self.started.is_some(),
                "observation_calls": self.observation_calls,
                "first_observed_main_frame": self.first_observed_main_frame,
                "last_observed_main_frame": self.last_observed_main_frame,
                "last_delivery_offset_ms": self.last_delivery_offset_ms,
            },
            "bounded_samples": {
                "max_paths": MAX_PATHS,
                "max_path_bytes": MAX_PATH_BYTES,
                "max_samples_per_path": MAX_SAMPLES_PER_PATH,
                "max_total_samples": MAX_TOTAL_SAMPLES,
                "retained": self.retained,
                "omitted_limit": self.omitted_limit,
                "invalid_values": self.invalid_values,
                "pre_window_latest_observations": self.pre_window_latest_observations,
                "source_dropped_batches": null,
            },
            "spans": spans,
            "upload": {
                "status": "unmeasured",
                "bytes": null,
                "duration_ms": null,
            },
            "notes": [
                "GPU values are real instrumented render-span timestamp durations already converted to milliseconds by Bevy using the queue timestamp period. The built-in scene spans measure render passes; encoder-only spans may also be reported.",
                "CPU values are wall-clock time inside instrumented render encoding spans, not total render-thread CPU time, extraction, asset preparation, upload, presentation, or GPU execution.",
                "Diagnostic timestamps are main-world delivery times. observed_main_frame identifies observation only, not the source render frame.",
                "Readback is asynchronous and Bevy's bridge is latest-only. Source frame IDs and overwritten batch counts are unavailable; raw counts need not match main-loop sample counts.",
                "The receipt window excludes already delivered pre-window samples. Late warmup results may arrive inside it; final in-flight results may be absent. No blocking drain is performed.",
                "Only the latest measurement per path per observation is retained. Identically named spans from multiple views are not individually identified.",
                "Nested or overlapping spans must not be summed as a whole-frame GPU or CPU duration. These are not presentation timestamps.",
                "Unsupported means no GPU samples were received and built-in pass timestamp support is not enabled on this device. Encoder-only spans require timestamp_query and timestamp_inside_encoders but not timestamp_inside_passes; received GPU samples take precedence over the pass-support status. This does not assert that the hardware cannot support disabled features.",
                "Upload bytes and upload duration are unmeasured. CPU mesh work counters elsewhere describe generated mesh work, not GPU transfer completion.",
            ],
        })
    }
}

fn summary(samples: &[Sample]) -> Value {
    if samples.is_empty() {
        return Value::Null;
    }
    let mut values: Vec<_> = samples.iter().map(|sample| sample.value).collect();
    values.sort_unstable_by(f64::total_cmp);
    let percentile = |percent: usize| values[(values.len() * percent).div_ceil(100) - 1];
    // Incremental mean avoids overflowing a sum of otherwise finite values.
    let mean = values.iter().enumerate().fold(0.0, |mean, (i, value)| {
        mean + (value - mean) / (i + 1) as f64
    });
    json!({
        "samples": values.len(),
        "min": values[0],
        "mean": mean,
        "p50": percentile(50),
        "p95": percentile(95),
        "p99": percentile(99),
        "max": values[values.len() - 1],
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::diagnostic::{Diagnostic, DiagnosticMeasurement, DiagnosticPath};

    use super::*;

    fn store_sample(store: &mut DiagnosticsStore, path: &str, time: Instant, value: f64) {
        let key = DiagnosticPath::new(path.to_owned());
        if store.get(&key).is_none() {
            let suffix = if path.contains("elapsed_") { "ms" } else { "" };
            store.add(Diagnostic::new(key.clone()).with_suffix(suffix));
        }
        store
            .get_mut(&key)
            .unwrap()
            .add_measurement(DiagnosticMeasurement { time, value });
    }

    fn probe_at(started: Instant) -> GpuProbe {
        let mut probe = GpuProbe {
            render_app_present: true,
            ..default()
        };
        probe.begin_at(started);
        probe
    }

    #[test]
    fn headless_install_reports_absent_renderer_without_installing_recorder() {
        let mut app = App::new();
        install(&mut app);
        assert!(!app.is_plugin_added::<RenderDiagnosticsPlugin>());
        app.finish();
        app.cleanup();
        app.update();
        let probe = app.world().resource::<GpuProbe>();
        assert_eq!(probe.report()["status"], "no_render_app");
        assert!(!probe.has_gpu_samples());
        assert_eq!(
            probe.report()["capabilities"]["timestamp_query"],
            Value::Null
        );
    }

    #[test]
    fn receipt_window_deduplicates_and_never_treats_stale_or_warmup_as_new() {
        let now = Instant::now();
        let mut store = DiagnosticsStore::default();
        let mut probe = probe_at(now);
        let path = "render/main_transparent_pass_2d/elapsed_gpu";
        store_sample(&mut store, path, now - Duration::from_millis(1), 7.0);
        probe.collect(&store, 120, None);
        probe.collect(&store, 121, None);
        assert!(!probe.has_gpu_samples());
        assert_eq!(probe.pre_window_latest_observations, 1);
        store_sample(&mut store, path, now + Duration::from_millis(3), 2.5);
        probe.collect(&store, 122, None);
        probe.collect(&store, 123, None);
        let report = probe.report();
        assert_eq!(report["status"], "samples_received");
        assert_eq!(report["bounded_samples"]["retained"], 1);
        assert_eq!(report["spans"][0]["samples"][0]["observed_main_frame"], 122);
        assert_eq!(report["spans"][0]["samples"][0]["receipt_offset_ms"], 3.0);
        assert_eq!(report["spans"][0]["samples"][0]["value"], 2.5);
        assert_eq!(
            report["bounded_samples"]["source_dropped_batches"],
            Value::Null
        );
    }

    #[test]
    fn warmup_collects_nothing_and_begin_clears_previous_window() {
        let now = Instant::now();
        let mut probe = GpuProbe::default();
        let mut store = DiagnosticsStore::default();
        store_sample(&mut store, "render/ui/elapsed_cpu", now, 1.0);
        probe.collect(&store, 1, None);
        assert_eq!(probe.observation_calls, 0);
        probe.begin_at(now);
        probe.collect(&store, 2, None);
        assert_eq!(probe.retained, 1);
        probe.begin_at(now + Duration::from_millis(1));
        assert_eq!(probe.retained, 0);
        probe.collect(&store, 3, None);
        assert_eq!(probe.pre_window_latest_observations, 1);
    }

    #[test]
    fn feature_status_requires_all_flags_and_distinguishes_no_samples() {
        let mut probe = probe_at(Instant::now());
        assert_eq!(probe.report()["status"], "device_unavailable");
        for features in [
            WgpuFeatures::empty(),
            WgpuFeatures::TIMESTAMP_QUERY,
            WgpuFeatures::TIMESTAMP_QUERY | WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        ] {
            probe.capabilities = Capabilities::from_features(features);
            assert_eq!(probe.report()["status"], "unsupported");
            assert_eq!(
                probe.report()["capabilities"]["pass_timestamps_enabled"],
                false
            );
        }
        probe.capabilities = Capabilities::from_features(
            WgpuFeatures::TIMESTAMP_QUERY
                | WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS
                | WgpuFeatures::TIMESTAMP_QUERY_INSIDE_PASSES,
        );
        assert_eq!(probe.report()["status"], "no_samples_received");
        assert_eq!(probe.report()["gpu_timing_collected"], false);
    }

    #[test]
    fn received_encoder_only_timestamps_do_not_require_pass_timestamp_support() {
        let now = Instant::now();
        let mut probe = probe_at(now);
        probe.capabilities = Capabilities::from_features(
            WgpuFeatures::TIMESTAMP_QUERY | WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        );
        let mut store = DiagnosticsStore::default();
        store_sample(&mut store, "render/custom_encoder/elapsed_gpu", now, 0.75);
        probe.collect(&store, 121, None);
        let report = probe.report();
        assert_eq!(report["status"], "samples_received");
        assert_eq!(report["gpu_timing_collected"], true);
        assert_eq!(report["capabilities"]["timestamp_query"], true);
        assert_eq!(report["capabilities"]["timestamp_inside_encoders"], true);
        assert_eq!(report["capabilities"]["timestamp_inside_passes"], false);
        assert_eq!(report["capabilities"]["pass_timestamps_enabled"], false);
        assert_eq!(report["bounded_samples"]["retained"], 1);
        assert_eq!(report["spans"][0]["samples"][0]["value"], 0.75);
    }

    #[test]
    fn cpu_and_pipeline_samples_stay_separate_and_invalid_values_are_counted() {
        let now = Instant::now();
        let mut probe = probe_at(now);
        let mut store = DiagnosticsStore::default();
        store_sample(&mut store, "render/ui/elapsed_cpu", now, 0.5);
        store_sample(
            &mut store,
            "render/ui/fragment_shader_invocations",
            now,
            900.0,
        );
        store_sample(&mut store, "frame_time", now, 42.0);
        probe.collect(&store, 1, None);
        let gpu = "render/ui/elapsed_gpu";
        for (i, value) in [-1.0, f64::NAN, f64::INFINITY].into_iter().enumerate() {
            store_sample(
                &mut store,
                gpu,
                now + Duration::from_millis(i as u64 + 1),
                value,
            );
            probe.collect(&store, i as u64 + 2, None);
        }
        assert_eq!(probe.invalid_values, 3);
        assert_eq!(probe.retained, 2);
        assert!(!probe.has_gpu_samples());
        let report = probe.report();
        assert_eq!(report["spans"][0]["kind"], "cpu_elapsed");
        assert_eq!(report["spans"][0]["unit"], "ms");
        assert_eq!(report["spans"][2]["kind"], "pipeline_counter");
        assert_eq!(report["spans"][2]["unit"], "count");
        assert_eq!(report["upload"]["duration_ms"], Value::Null);
    }

    #[test]
    fn retained_data_is_bounded_and_omissions_are_not_claimed_source_drops() {
        let now = Instant::now();
        let mut probe = probe_at(now);
        let mut store = DiagnosticsStore::default();
        for i in 0..MAX_SAMPLES_PER_PATH + 3 {
            store_sample(
                &mut store,
                "render/ui/elapsed_gpu",
                now + Duration::from_micros(i as u64),
                i as f64,
            );
            probe.collect(&store, i as u64, None);
        }
        assert_eq!(probe.retained, MAX_SAMPLES_PER_PATH);
        assert_eq!(probe.omitted_limit, 3);
        for i in 0..MAX_PATHS {
            store_sample(&mut store, &format!("render/span{i}/elapsed_cpu"), now, 1.0);
        }
        probe.collect(&store, 5000, None);
        assert_eq!(probe.spans.len(), MAX_PATHS);
        assert_eq!(probe.omitted_limit, 4);
        probe.retained = MAX_TOTAL_SAMPLES;
        store_sample(
            &mut store,
            "render/ui/elapsed_gpu",
            now + Duration::from_secs(1),
            1.0,
        );
        probe.collect(&store, 5001, None);
        assert_eq!(probe.retained, MAX_TOTAL_SAMPLES);
        assert!(probe.omitted_limit >= 5);
    }
}
