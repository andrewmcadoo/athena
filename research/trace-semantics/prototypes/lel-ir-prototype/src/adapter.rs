use std::fmt;

use crate::common::{
    BoundaryClassification, Completeness, ConfidenceMeta, ControlledVariable, ExecutionOutcome,
    ElementId, ExperimentRef, Layer, ObservationMode, ProvenanceAnchor, SourceLocation,
    SpecElementId, TemporalCoord, Value,
};
use crate::event_kinds::EventKind;
use crate::lel::{
    ExperimentSpec, LayeredEventLog, LayeredEventLogBuilder, TraceEventBuilder,
};

/// Error type for adapter operations.
#[derive(Debug)]
pub enum AdapterError {
    ParseError(String),
    UnsupportedFormat(String),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdapterError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AdapterError::UnsupportedFormat(msg) => {
                write!(f, "Unsupported format: {}", msg)
            }
        }
    }
}

impl std::error::Error for AdapterError {}

/// Trait for DSL framework adapters.
/// Each adapter translates framework-specific trace output into
/// a LayeredEventLog.
pub trait DslAdapter {
    fn parse_trace(&self, raw: &str) -> Result<LayeredEventLog, AdapterError>;
}

const OPENMM_MIN_CONVERGENCE_WINDOW: usize = 4;
const OPENMM_REL_DELTA_THRESHOLD: f64 = 1.0e-4;

fn parse_openmm_energy_series(raw: &str) -> Vec<(u64, f64)> {
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }

            let mut tokens = trimmed.split_whitespace();
            let step = tokens.next()?.parse::<u64>().ok()?;
            let energy = tokens.next()?.parse::<f64>().ok()?;
            Some((step, energy))
        })
        .collect()
}

fn openmm_energy_total(event: &crate::lel::TraceEvent) -> Option<f64> {
    match &event.kind {
        EventKind::EnergyRecord {
            total: Value::Known(total, _),
            ..
        } => Some(*total),
        _ => None,
    }
}

fn derive_openmm_convergence_summary(
    events: &[crate::lel::TraceEvent],
) -> Option<crate::lel::TraceEvent> {
    let energy_events: Vec<(&crate::lel::TraceEvent, f64)> = events
        .iter()
        .filter_map(|event| openmm_energy_total(event).map(|total| (event, total)))
        .collect();

    if energy_events.len() < OPENMM_MIN_CONVERGENCE_WINDOW {
        return None;
    }

    let window = &energy_events[energy_events.len() - OPENMM_MIN_CONVERGENCE_WINDOW..];
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
    let max_rel_delta = rel_abs_deltas
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let mean_rel_delta = rel_abs_deltas.iter().sum::<f64>() / rel_abs_deltas.len() as f64;
    let sign_changes = deltas
        .windows(2)
        .filter(|pair| pair[0] * pair[1] < 0.0)
        .count();

    let (metric_name, metric_value, converged, note) =
        if sign_changes >= 2 && mean_rel_delta > OPENMM_REL_DELTA_THRESHOLD {
            (
                "derived_oscillation_rel_delta_mean",
                mean_rel_delta,
                Some(false),
                "energy deltas alternate sign across the derivation window",
            )
        } else if max_rel_delta <= OPENMM_REL_DELTA_THRESHOLD {
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
                source_file: "simulation.log".to_string(),
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

/// Mock OpenMM adapter that produces hardcoded sample events.
/// Demonstrates layer diversity, temporal ordering, and Hybrid upgrade fields.
pub struct MockOpenMmAdapter;

impl DslAdapter for MockOpenMmAdapter {
    fn parse_trace(&self, raw: &str) -> Result<LayeredEventLog, AdapterError> {
        let experiment_ref = ExperimentRef {
            experiment_id: "openmm-mock-001".to_string(),
            cycle_id: 0,
            hypothesis_id: "H1-force-field-comparison".to_string(),
        };

        let spec = ExperimentSpec {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            predictions: Vec::new(),
            interventions: Vec::new(),
            controlled_variables: vec![ControlledVariable {
                id: SpecElementId(1),
                parameter: "temperature".to_string(),
                held_value: Value::Known(300.0, "K".to_string()),
            }],
            dag_refs: Vec::new(),
            provenance: ProvenanceAnchor {
                source_file: "experiment_spec.json".to_string(),
                source_location: SourceLocation::ExternalInput,
                raw_hash: 0xDEAD_BEEF,
            },
        };

        let default_provenance = ProvenanceAnchor {
            source_file: "simulation.log".to_string(),
            source_location: SourceLocation::LineRange { start: 1, end: 10 },
            raw_hash: 0xCAFE_BABE,
        };

        let default_confidence = ConfidenceMeta {
            completeness: Completeness::FullyObserved,
            field_coverage: 1.0,
            notes: Vec::new(),
        };

        // Event 1: Theory layer — force field selection (ParameterRecord)
        let event1 = TraceEventBuilder::new()
            .layer(Layer::Theory)
            .kind(EventKind::ParameterRecord {
                name: "force_field".to_string(),
                specified_value: Some(Value::KnownCat("amber14-all".to_string())),
                actual_value: Value::KnownCat("amber14-all".to_string()),
                units: None,
                observation_mode: ObservationMode::Observational,
            })
            .temporal(TemporalCoord {
                simulation_step: 0,
                wall_clock_ns: Some(0),
                logical_sequence: 1,
            })
            .provenance(default_provenance.clone())
            .confidence(default_confidence.clone())
            .build();

        // Event 2: Methodology layer — timestep configuration (ParameterRecord)
        let event2 = TraceEventBuilder::new()
            .layer(Layer::Methodology)
            .boundary(BoundaryClassification::DualAnnotated {
                secondary_layer: Layer::Implementation,
                rationale:
                    "Timestep affects both sampling methodology and numerical stability".to_string(),
            })
            .kind(EventKind::ParameterRecord {
                name: "timestep".to_string(),
                specified_value: Some(Value::Known(0.002, "ps".to_string())),
                actual_value: Value::Known(0.002, "ps".to_string()),
                units: Some("ps".to_string()),
                observation_mode: ObservationMode::Observational,
            })
            .temporal(TemporalCoord {
                simulation_step: 0,
                wall_clock_ns: Some(100),
                logical_sequence: 2,
            })
            .provenance(default_provenance.clone())
            .confidence(default_confidence.clone())
            .build();

        // Event 3: Implementation layer — platform/resource status
        let event3 = TraceEventBuilder::new()
            .layer(Layer::Implementation)
            .kind(EventKind::ResourceStatus {
                platform_type: "CUDA".to_string(),
                device_ids: vec!["GPU:0".to_string()],
                memory_allocated: Some(Value::Known(2048.0, "MB".to_string())),
                memory_peak: None,
                parallelization: Some("SingleGPU".to_string()),
                warnings: Vec::new(),
            })
            .temporal(TemporalCoord {
                simulation_step: 0,
                wall_clock_ns: Some(500),
                logical_sequence: 3,
            })
            .provenance(default_provenance.clone())
            .confidence(default_confidence.clone())
            .build();

        let mut events = Vec::new();
        events.push(event1);
        events.push(event2);
        events.push(event3);

        // Mock reporter stream: if `raw` contains lines in "<step> <energy>" format,
        // use them as the energy series; otherwise fall back to the single baseline sample.
        let parsed_series = parse_openmm_energy_series(raw);
        let energy_series = if parsed_series.is_empty() {
            vec![(1000_u64, -45023.7_f64)]
        } else {
            parsed_series
        };

        let resource_event_id = events[2].id;
        let mut logical_sequence = 4_u64;
        for (step, total_energy) in energy_series {
            let energy_event = TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::EnergyRecord {
                    total: Value::Known(total_energy, "kJ/mol".to_string()),
                    components: vec![
                        (
                            "kinetic".to_string(),
                            Value::Known(12500.3, "kJ/mol".to_string()),
                        ),
                        (
                            "potential".to_string(),
                            Value::Known(total_energy - 12500.3, "kJ/mol".to_string()),
                        ),
                    ],
                })
                .temporal(TemporalCoord {
                    simulation_step: step,
                    wall_clock_ns: Some(step.saturating_mul(1500)),
                    logical_sequence,
                })
                .causal_refs(vec![resource_event_id])
                .provenance(default_provenance.clone())
                .confidence(default_confidence.clone())
                .build();
            logical_sequence += 1;
            events.push(energy_event);
        }

        // Execution completed successfully (mock default path).
        let mut execution_builder = TraceEventBuilder::new()
            .layer(Layer::Implementation)
            .kind(EventKind::ExecutionStatus {
                status: ExecutionOutcome::Success,
                framework_error_id: None,
            })
            .temporal(TemporalCoord {
                simulation_step: 10000,
                wall_clock_ns: Some(15_000_000),
                logical_sequence,
            })
            .provenance(default_provenance)
            .confidence(default_confidence);

        if let Some(last_energy_id) = events
            .iter()
            .rev()
            .find(|event| matches!(event.kind, EventKind::EnergyRecord { .. }))
            .map(|event| event.id)
        {
            execution_builder = execution_builder.causal_refs(vec![last_energy_id]);
        }

        events.push(execution_builder.build());

        if let Some(summary_event) = derive_openmm_convergence_summary(&events) {
            events.push(summary_event);
        }

        let mut log_builder = LayeredEventLogBuilder::new(experiment_ref, spec);
        for event in events {
            log_builder = log_builder.add_event(event);
        }
        let log = log_builder.build();

        Ok(log)
    }
}
