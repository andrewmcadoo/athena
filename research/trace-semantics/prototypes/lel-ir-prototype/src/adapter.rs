use std::fmt;

use crate::common::{
    BoundaryClassification, Completeness, ConfidenceMeta, ControlledVariable, ExecutionOutcome,
    ExperimentRef, Layer, NumericalEventType, ObservationMode, ProvenanceAnchor, Severity,
    SourceLocation, SpecElementId, TemporalCoord, Value,
};
use crate::convergence;
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

fn parse_openmm_energy_series(raw: &str) -> Vec<(u64, f64)> {
    let non_empty_lines: Vec<&str> = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if let Some(first_line) = non_empty_lines.first().copied() {
        if first_line.starts_with("#\"Step\"") || first_line.starts_with("#\"") {
            return parse_openmm_csv_energy_series(&non_empty_lines);
        }
    }

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

fn parse_openmm_csv_fields(line: &str) -> Vec<String> {
    line.split(',')
        .map(|field| {
            field
                .trim()
                .trim_start_matches('#')
                .trim()
                .trim_matches('"')
                .to_string()
        })
        .collect()
}

fn parse_openmm_csv_energy_series(lines: &[&str]) -> Vec<(u64, f64)> {
    let Some(header_line) = lines.first().copied() else {
        return Vec::new();
    };
    let header = parse_openmm_csv_fields(header_line);
    let Some(step_idx) = header
        .iter()
        .position(|column| column.eq_ignore_ascii_case("Step"))
    else {
        return Vec::new();
    };
    let Some(energy_idx) = header
        .iter()
        .position(|column| column.contains("Potential Energy"))
    else {
        return Vec::new();
    };

    lines[1..]
        .iter()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            if step_idx >= fields.len() || energy_idx >= fields.len() {
                return None;
            }

            let step = fields[step_idx].trim().trim_matches('"').parse::<u64>().ok()?;
            let energy = fields[energy_idx]
                .trim()
                .trim_matches('"')
                .parse::<f64>()
                .ok()?;
            Some((step, energy))
        })
        .collect()
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
            let energy_event_id = energy_event.id;
            events.push(energy_event);

            if !total_energy.is_finite() {
                let (event_type, detail) = if total_energy.is_nan() {
                    (
                        NumericalEventType::NaNDetected,
                        "NaN detected in potential energy".to_string(),
                    )
                } else {
                    (
                        NumericalEventType::InfDetected,
                        "Inf detected in potential energy".to_string(),
                    )
                };

                let numerical_event = TraceEventBuilder::new()
                    .layer(Layer::Implementation)
                    .kind(EventKind::NumericalStatus {
                        event_type,
                        affected_quantity: "Potential Energy".to_string(),
                        severity: Severity::Warning,
                        detail: Value::KnownCat(detail),
                    })
                    .temporal(TemporalCoord {
                        simulation_step: step,
                        wall_clock_ns: Some(step.saturating_mul(1500)),
                        logical_sequence,
                    })
                    .causal_refs(vec![energy_event_id])
                    .provenance(default_provenance.clone())
                    .confidence(default_confidence.clone())
                    .build();
                logical_sequence += 1;
                events.push(numerical_event);
            }
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

        if let Some(summary_event) =
            convergence::derive_energy_convergence_summary(&events, "simulation.log")
        {
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
