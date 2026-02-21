use std::fmt;

use crate::common::{
    BoundaryClassification, Completeness, ConfidenceMeta, ControlledVariable, ExecutionOutcome,
    ExperimentRef, Layer, ObservationMode, ProvenanceAnchor, SourceLocation, SpecElementId,
    TemporalCoord, Value,
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

/// Mock OpenMM adapter that produces hardcoded sample events.
/// Demonstrates layer diversity, temporal ordering, and Hybrid upgrade fields.
pub struct MockOpenMmAdapter;

impl DslAdapter for MockOpenMmAdapter {
    fn parse_trace(&self, _raw: &str) -> Result<LayeredEventLog, AdapterError> {
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

        // Event 4: Implementation layer — energy record at step 1000
        let event3_id = event3.id;
        let event4 = TraceEventBuilder::new()
            .layer(Layer::Implementation)
            .kind(EventKind::EnergyRecord {
                total: Value::Known(-45023.7, "kJ/mol".to_string()),
                components: vec![
                    (
                        "kinetic".to_string(),
                        Value::Known(12500.3, "kJ/mol".to_string()),
                    ),
                    (
                        "potential".to_string(),
                        Value::Known(-57524.0, "kJ/mol".to_string()),
                    ),
                ],
            })
            .temporal(TemporalCoord {
                simulation_step: 1000,
                wall_clock_ns: Some(1_500_000),
                logical_sequence: 4,
            })
            .causal_refs(vec![event3_id])
            .provenance(default_provenance.clone())
            .confidence(default_confidence.clone())
            .build();

        // Event 5: Implementation layer — execution completed successfully
        let event5 = TraceEventBuilder::new()
            .layer(Layer::Implementation)
            .kind(EventKind::ExecutionStatus {
                status: ExecutionOutcome::Success,
                framework_error_id: None,
            })
            .temporal(TemporalCoord {
                simulation_step: 10000,
                wall_clock_ns: Some(15_000_000),
                logical_sequence: 5,
            })
            .provenance(default_provenance)
            .confidence(default_confidence)
            .build();

        let log = LayeredEventLogBuilder::new(experiment_ref, spec)
            .add_event(event1)
            .add_event(event2)
            .add_event(event3)
            .add_event(event4)
            .add_event(event5)
            .build();

        Ok(log)
    }
}
