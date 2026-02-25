use std::collections::HashSet;

use crate::common::{
    Completeness, ConfidenceMeta, ElementId, ExecutionOutcome, Layer, NumericalEventType,
    ProvenanceAnchor, SourceLocation, TemporalCoord, Value,
};
use crate::event_kinds::EventKind;
use crate::lel::{LayeredEventLog, TraceEvent, TraceEventBuilder};

pub const MIN_CONVERGENCE_WINDOW: usize = 4;
pub const REL_DELTA_THRESHOLD: f64 = 1.0e-4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvergencePattern {
    Converged,
    Oscillating,
    Stalled,
    Divergent,
    InsufficientData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvergenceConfidence {
    Direct,
    Derived,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalConvergence {
    pub pattern: ConvergencePattern,
    pub confidence: ConvergenceConfidence,
    pub source_metric: String,
    pub source_framework: String,
}

fn energy_total(event: &TraceEvent) -> Option<f64> {
    match &event.kind {
        EventKind::EnergyRecord {
            total: Value::Known(total, _),
            ..
        } => Some(*total),
        _ => None,
    }
}

pub fn derive_energy_convergence_summary(
    events: &[TraceEvent],
    source_file: &str,
) -> Option<TraceEvent> {
    let energy_events: Vec<(&TraceEvent, f64)> = events
        .iter()
        .filter_map(|event| energy_total(event).map(|total| (event, total)))
        .collect();

    if energy_events.len() < MIN_CONVERGENCE_WINDOW {
        return None;
    }

    let window = &energy_events[energy_events.len() - MIN_CONVERGENCE_WINDOW..];
    let deltas: Vec<f64> = window.windows(2).map(|pair| pair[1].1 - pair[0].1).collect();
    if deltas.is_empty() {
        return None;
    }

    let energy_scale =
        (window.iter().map(|(_, value)| value.abs()).sum::<f64>() / window.len() as f64).max(1.0);
    let rel_abs_deltas: Vec<f64> = deltas
        .iter()
        .map(|delta| delta.abs() / energy_scale)
        .collect();
    let max_rel_delta = rel_abs_deltas.iter().copied().fold(0.0_f64, f64::max);
    let mean_rel_delta = rel_abs_deltas.iter().sum::<f64>() / rel_abs_deltas.len() as f64;
    let sign_changes = deltas
        .windows(2)
        .filter(|pair| pair[0] * pair[1] < 0.0)
        .count();

    let (metric_name, metric_value, converged, note) =
        if sign_changes >= 2 && mean_rel_delta > REL_DELTA_THRESHOLD {
            (
                "derived_oscillation_rel_delta_mean",
                mean_rel_delta,
                Some(false),
                "energy deltas alternate sign across the derivation window",
            )
        } else if max_rel_delta <= REL_DELTA_THRESHOLD {
            (
                "derived_convergence_rel_delta_max",
                max_rel_delta,
                Some(true),
                "max relative energy delta is below convergence threshold",
            )
        } else {
            (
                "derived_stall_rel_delta_mean",
                mean_rel_delta,
                Some(false),
                "energy deltas remain above threshold without oscillation",
            )
        };

    let mut causal_refs = window
        .iter()
        .map(|(event, _)| event.id)
        .collect::<Vec<_>>();
    if let Some(exec_event) = events
        .iter()
        .rev()
        .find(|event| matches!(event.kind, EventKind::ExecutionStatus { .. }))
    {
        causal_refs.push(exec_event.id);
    }
    if let Some(numerical_event) = events
        .iter()
        .rev()
        .find(|event| matches!(event.kind, EventKind::NumericalStatus { .. }))
    {
        causal_refs.push(numerical_event.id);
    }

    let from_elements = causal_refs.iter().map(|id| ElementId(id.0)).collect();
    let simulation_step = window.last().map(|(event, _)| event.temporal.simulation_step)?;
    let logical_sequence = events
        .last()
        .map(|event| event.temporal.logical_sequence + 1)
        .unwrap_or(1);

    Some(
        TraceEventBuilder::new()
            .layer(Layer::Methodology)
            .kind(EventKind::ConvergencePoint {
                iteration: simulation_step,
                metric_name: metric_name.to_string(),
                metric_value: Value::Known(metric_value, "relative".to_string()),
                converged,
            })
            .temporal(TemporalCoord {
                simulation_step,
                wall_clock_ns: None,
                logical_sequence,
            })
            .causal_refs(causal_refs)
            .provenance(ProvenanceAnchor {
                source_file: source_file.to_string(),
                source_location: SourceLocation::ExternalInput,
                raw_hash: 0,
            })
            .confidence(ConfidenceMeta {
                completeness: Completeness::Derived { from_elements },
                field_coverage: 1.0,
                notes: vec![note.to_string()],
            })
            .build(),
    )
}

pub fn derive_vasp_scf_convergence_summary(
    events: &[TraceEvent],
    source_file: &str,
) -> Option<TraceEvent> {
    let de_events: Vec<&TraceEvent> = events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                EventKind::ConvergencePoint { ref metric_name, .. } if metric_name == "dE"
            )
        })
        .collect();

    let converged_steps: HashSet<u64> = de_events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::ConvergencePoint {
                converged: Some(true),
                ..
            } => Some(event.temporal.simulation_step),
            _ => None,
        })
        .collect();

    let non_converged_de_events: Vec<(&TraceEvent, f64)> = de_events
        .into_iter()
        .filter(|event| !converged_steps.contains(&event.temporal.simulation_step))
        .filter_map(|event| match &event.kind {
            EventKind::ConvergencePoint {
                metric_value: Value::Known(value, _),
                ..
            } => Some((event, *value)),
            _ => None,
        })
        .collect();

    if non_converged_de_events.len() < MIN_CONVERGENCE_WINDOW {
        return None;
    }

    let window =
        &non_converged_de_events[non_converged_de_events.len() - MIN_CONVERGENCE_WINDOW..];
    let window_values: Vec<f64> = window.iter().map(|(_, value)| *value).collect();
    let sign_changes = window_values
        .windows(2)
        .filter(|pair| pair[0] * pair[1] < 0.0)
        .count();

    let first_abs = window_values.first()?.abs();
    let normalization_scale = first_abs.max(1.0);
    let normalized_abs_values: Vec<f64> = window_values
        .iter()
        .map(|value| value.abs() / normalization_scale)
        .collect();
    let mean_normalized =
        normalized_abs_values.iter().sum::<f64>() / normalized_abs_values.len() as f64;

    let (metric_name, note) =
        if sign_changes >= 2 && mean_normalized > REL_DELTA_THRESHOLD {
            (
                "derived_vasp_scf_oscillation_dE",
                "SCF dE values alternate sign across the derivation window",
            )
        } else {
            (
                "derived_vasp_scf_stall_dE",
                "SCF dE values remain above threshold without oscillation",
            )
        };

    let last_event = window.last().map(|(event, _)| *event)?;
    let simulation_step = last_event.temporal.simulation_step;
    let logical_sequence = last_event.temporal.logical_sequence + 1;
    let causal_refs = window.iter().map(|(event, _)| event.id).collect::<Vec<_>>();
    let from_elements = causal_refs.iter().map(|id| ElementId(id.0)).collect();

    Some(
        TraceEventBuilder::new()
            .layer(Layer::Methodology)
            .kind(EventKind::ConvergencePoint {
                iteration: simulation_step,
                metric_name: metric_name.to_string(),
                metric_value: Value::Known(mean_normalized, "relative".to_string()),
                converged: Some(false),
            })
            .temporal(TemporalCoord {
                simulation_step,
                wall_clock_ns: None,
                logical_sequence,
            })
            .causal_refs(causal_refs)
            .provenance(ProvenanceAnchor {
                source_file: source_file.to_string(),
                source_location: SourceLocation::ExternalInput,
                raw_hash: 0,
            })
            .confidence(ConfidenceMeta {
                completeness: Completeness::Derived { from_elements },
                field_coverage: 1.0,
                notes: vec![note.to_string()],
            })
            .build(),
    )
}

fn confidence_from_completeness(completeness: &Completeness) -> ConvergenceConfidence {
    match completeness {
        Completeness::FullyObserved => ConvergenceConfidence::Direct,
        Completeness::Derived { .. } => ConvergenceConfidence::Derived,
        _ => ConvergenceConfidence::Absent,
    }
}

fn has_divergent_status(log: &LayeredEventLog) -> bool {
    log.events.iter().any(|event| match &event.kind {
        EventKind::NumericalStatus { event_type, .. } => {
            matches!(
                event_type,
                NumericalEventType::NaNDetected | NumericalEventType::InfDetected
            )
        }
        EventKind::ExecutionStatus { status, .. } => {
            matches!(status, ExecutionOutcome::CrashDivergent)
        }
        _ => false,
    })
}

pub fn classify_convergence(
    event: &TraceEvent,
    framework: &str,
    log: &LayeredEventLog,
) -> CanonicalConvergence {
    let source_framework = framework.to_string();
    let source_metric = match &event.kind {
        EventKind::ConvergencePoint { metric_name, .. } => metric_name.clone(),
        _ => "unknown".to_string(),
    };
    let completeness_confidence = confidence_from_completeness(&event.confidence.completeness);

    if has_divergent_status(log) {
        return CanonicalConvergence {
            pattern: ConvergencePattern::Divergent,
            confidence: completeness_confidence,
            source_metric,
            source_framework,
        };
    }

    let EventKind::ConvergencePoint {
        metric_name,
        converged,
        ..
    } = &event.kind
    else {
        return CanonicalConvergence {
            pattern: ConvergencePattern::InsufficientData,
            confidence: ConvergenceConfidence::Absent,
            source_metric,
            source_framework,
        };
    };

    let framework_is_gromacs_openmm =
        framework.eq_ignore_ascii_case("gromacs") || framework.eq_ignore_ascii_case("openmm");
    let framework_is_vasp = framework.eq_ignore_ascii_case("vasp");

    let (pattern, confidence) = if framework_is_gromacs_openmm
        && metric_name == "derived_convergence_rel_delta_max"
        && *converged == Some(true)
    {
        (ConvergencePattern::Converged, completeness_confidence)
    } else if framework_is_gromacs_openmm
        && metric_name == "derived_oscillation_rel_delta_mean"
        && *converged == Some(false)
    {
        (ConvergencePattern::Oscillating, completeness_confidence)
    } else if framework_is_gromacs_openmm
        && metric_name == "derived_stall_rel_delta_mean"
        && *converged == Some(false)
    {
        (ConvergencePattern::Stalled, completeness_confidence)
    } else if framework_is_vasp && metric_name == "dE" && *converged == Some(true) {
        (ConvergencePattern::Converged, completeness_confidence)
    } else if framework_is_vasp && metric_name == "dE" && converged.is_none() {
        (ConvergencePattern::InsufficientData, completeness_confidence)
    } else if framework_is_vasp
        && metric_name == "derived_vasp_scf_oscillation_dE"
        && *converged == Some(false)
    {
        (ConvergencePattern::Oscillating, completeness_confidence)
    } else if framework_is_vasp
        && metric_name == "derived_vasp_scf_stall_dE"
        && *converged == Some(false)
    {
        (ConvergencePattern::Stalled, completeness_confidence)
    } else {
        (
            ConvergencePattern::InsufficientData,
            ConvergenceConfidence::Absent,
        )
    };

    CanonicalConvergence {
        pattern,
        confidence,
        source_metric,
        source_framework,
    }
}

pub fn classify_all_convergence(
    log: &LayeredEventLog,
    framework: &str,
) -> Vec<CanonicalConvergence> {
    log.events
        .iter()
        .filter(|event| matches!(event.kind, EventKind::ConvergencePoint { .. }))
        .map(|event| classify_convergence(event, framework, log))
        .collect()
}
