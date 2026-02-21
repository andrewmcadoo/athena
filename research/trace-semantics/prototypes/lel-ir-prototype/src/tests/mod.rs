use crate::adapter::{DslAdapter, MockOpenMmAdapter};
use crate::common::*;
use crate::event_kinds::EventKind;
use crate::lel::*;

/// Helper: reset the global event ID counter before each test.
fn setup() {
    reset_event_id_counter();
}

/// Helper: create a minimal provenance anchor for tests.
fn test_provenance() -> ProvenanceAnchor {
    ProvenanceAnchor {
        source_file: "test.log".to_string(),
        source_location: SourceLocation::LineRange { start: 1, end: 1 },
        raw_hash: 0,
    }
}

/// Helper: create a minimal experiment spec for tests.
fn test_spec() -> ExperimentSpec {
    ExperimentSpec {
        preconditions: vec![ContractTerm {
            id: SpecElementId(1),
            description: "System must be solvated".to_string(),
            layer: Layer::Theory,
        }],
        postconditions: vec![ContractTerm {
            id: SpecElementId(2),
            description: "Energy must be finite".to_string(),
            layer: Layer::Implementation,
        }],
        predictions: Vec::new(),
        interventions: Vec::new(),
        controlled_variables: Vec::new(),
        dag_refs: Vec::new(),
        provenance: test_provenance(),
    }
}

/// Helper: create a minimal experiment ref for tests.
fn test_experiment_ref() -> ExperimentRef {
    ExperimentRef {
        experiment_id: "test-001".to_string(),
        cycle_id: 0,
        hypothesis_id: "H1".to_string(),
    }
}

#[test]
fn test_event_construction() {
    setup();
    let event = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ExecutionStatus {
            status: ExecutionOutcome::Success,
            framework_error_id: None,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .build();

    assert_eq!(event.layer, Layer::Implementation);
    assert_eq!(event.boundary, BoundaryClassification::PrimaryLayer);
    assert!(event.causal_refs.is_empty());
}

#[test]
fn test_layer_classification() {
    setup();
    let theory_event = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "force_field".to_string(),
            specified_value: None,
            actual_value: Value::KnownCat("amber14".to_string()),
            units: None,
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .build();

    let impl_event = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ResourceStatus {
            platform_type: "CPU".to_string(),
            device_ids: vec![],
            memory_allocated: None,
            memory_peak: None,
            parallelization: None,
            warnings: vec![],
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .build();

    assert_eq!(theory_event.layer, Layer::Theory);
    assert_eq!(impl_event.layer, Layer::Implementation);
    assert_ne!(theory_event.layer, impl_event.layer);
}

#[test]
fn test_boundary_classification() {
    setup();
    // DualAnnotated: timestep is primarily Methodology but also Implementation
    let dual_event = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .boundary(BoundaryClassification::DualAnnotated {
            secondary_layer: Layer::Implementation,
            rationale: "Timestep affects numerical stability".to_string(),
        })
        .kind(EventKind::ParameterRecord {
            name: "dt".to_string(),
            specified_value: None,
            actual_value: Value::Known(0.002, "ps".to_string()),
            units: Some("ps".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .build();

    match &dual_event.boundary {
        BoundaryClassification::DualAnnotated {
            secondary_layer,
            rationale,
        } => {
            assert_eq!(*secondary_layer, Layer::Implementation);
            assert!(!rationale.is_empty());
        }
        other => panic!("Expected DualAnnotated, got {:?}", other),
    }

    // ContextDependent
    let ctx_event = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .boundary(BoundaryClassification::ContextDependent {
            default_layer: Layer::Implementation,
            context_note: "ALGO is Theory-adjacent for pathological systems".to_string(),
        })
        .kind(EventKind::ParameterRecord {
            name: "ALGO".to_string(),
            specified_value: None,
            actual_value: Value::KnownCat("Normal".to_string()),
            units: None,
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .build();

    match &ctx_event.boundary {
        BoundaryClassification::ContextDependent {
            default_layer,
            context_note,
        } => {
            assert_eq!(*default_layer, Layer::Implementation);
            assert!(context_note.contains("pathological"));
        }
        other => panic!("Expected ContextDependent, got {:?}", other),
    }
}

#[test]
fn test_log_append_and_query_by_layer() {
    setup();
    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Theory)
                .kind(EventKind::ParameterRecord {
                    name: "force_field".to_string(),
                    specified_value: None,
                    actual_value: Value::KnownCat("amber14".to_string()),
                    units: None,
                    observation_mode: ObservationMode::Observational,
                })
                .temporal(TemporalCoord {
                    simulation_step: 0,
                    wall_clock_ns: None,
                    logical_sequence: 1,
                })
                .build(),
        )
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::ExecutionStatus {
                    status: ExecutionOutcome::Success,
                    framework_error_id: None,
                })
                .temporal(TemporalCoord {
                    simulation_step: 1,
                    wall_clock_ns: None,
                    logical_sequence: 2,
                })
                .build(),
        )
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::ResourceStatus {
                    platform_type: "CUDA".to_string(),
                    device_ids: vec!["GPU:0".to_string()],
                    memory_allocated: None,
                    memory_peak: None,
                    parallelization: None,
                    warnings: vec![],
                })
                .temporal(TemporalCoord {
                    simulation_step: 2,
                    wall_clock_ns: None,
                    logical_sequence: 3,
                })
                .build(),
        )
        .build();

    assert_eq!(log.events.len(), 3);

    let theory_ids = log.indexes.by_layer.get(&Layer::Theory).unwrap();
    assert_eq!(theory_ids.len(), 1);

    let impl_ids = log.indexes.by_layer.get(&Layer::Implementation).unwrap();
    assert_eq!(impl_ids.len(), 2);

    assert!(log.indexes.by_layer.get(&Layer::Methodology).is_none());
}

#[test]
fn test_query_by_event_kind() {
    setup();
    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::ExecutionStatus {
                    status: ExecutionOutcome::Success,
                    framework_error_id: None,
                })
                .temporal(TemporalCoord {
                    simulation_step: 0,
                    wall_clock_ns: None,
                    logical_sequence: 1,
                })
                .build(),
        )
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::ResourceStatus {
                    platform_type: "CPU".to_string(),
                    device_ids: vec![],
                    memory_allocated: None,
                    memory_peak: None,
                    parallelization: None,
                    warnings: vec![],
                })
                .temporal(TemporalCoord {
                    simulation_step: 1,
                    wall_clock_ns: None,
                    logical_sequence: 2,
                })
                .build(),
        )
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::EnergyRecord {
                    total: Value::Known(-100.0, "kJ/mol".to_string()),
                    components: vec![],
                })
                .temporal(TemporalCoord {
                    simulation_step: 2,
                    wall_clock_ns: None,
                    logical_sequence: 3,
                })
                .build(),
        )
        .build();

    let exec_ids = log.indexes.by_kind.get(&EventKindTag::ExecutionStatus).unwrap();
    assert_eq!(exec_ids.len(), 1);

    let resource_ids = log.indexes.by_kind.get(&EventKindTag::ResourceStatus).unwrap();
    assert_eq!(resource_ids.len(), 1);

    let energy_ids = log.indexes.by_kind.get(&EventKindTag::EnergyRecord).unwrap();
    assert_eq!(energy_ids.len(), 1);

    assert!(log
        .indexes
        .by_kind
        .get(&EventKindTag::ExceptionEvent)
        .is_none());
}

#[test]
fn test_temporal_ordering() {
    setup();
    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::ExecutionStatus {
                    status: ExecutionOutcome::Success,
                    framework_error_id: None,
                })
                .temporal(TemporalCoord {
                    simulation_step: 0,
                    wall_clock_ns: Some(0),
                    logical_sequence: 1,
                })
                .build(),
        )
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::EnergyRecord {
                    total: Value::Known(-100.0, "kJ/mol".to_string()),
                    components: vec![],
                })
                .temporal(TemporalCoord {
                    simulation_step: 1000,
                    wall_clock_ns: Some(500_000),
                    logical_sequence: 2,
                })
                .build(),
        )
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::ExecutionStatus {
                    status: ExecutionOutcome::Success,
                    framework_error_id: None,
                })
                .temporal(TemporalCoord {
                    simulation_step: 10000,
                    wall_clock_ns: Some(5_000_000),
                    logical_sequence: 3,
                })
                .build(),
        )
        .build();

    // Verify monotonic logical_sequence ordering
    for window in log.events.windows(2) {
        assert!(
            window[0].temporal.logical_sequence < window[1].temporal.logical_sequence,
            "Events must maintain monotonic logical_sequence ordering"
        );
        assert!(
            window[0].temporal.simulation_step <= window[1].temporal.simulation_step,
            "Events must maintain non-decreasing simulation_step ordering"
        );
    }
}

#[test]
fn test_havoc_value() {
    setup();
    let havoc = Value::Havoc {
        expected_type: ValueType::Scalar,
        reason: HavocReason::NotLogged,
    };

    match &havoc {
        Value::Havoc {
            expected_type,
            reason,
        } => {
            assert_eq!(*expected_type, ValueType::Scalar);
            assert_eq!(*reason, HavocReason::NotLogged);
        }
        other => panic!("Expected Havoc, got {:?}", other),
    }

    // Temporal gap variant
    let gap_havoc = Value::Havoc {
        expected_type: ValueType::Vector,
        reason: HavocReason::TemporalGap {
            last_known_step: 500,
            gap_steps: 100,
        },
    };

    match &gap_havoc {
        Value::Havoc {
            reason:
                HavocReason::TemporalGap {
                    last_known_step,
                    gap_steps,
                },
            ..
        } => {
            assert_eq!(*last_known_step, 500);
            assert_eq!(*gap_steps, 100);
        }
        other => panic!("Expected Havoc with TemporalGap, got {:?}", other),
    }
}

#[test]
fn test_mock_adapter() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace("").unwrap();

    // 5 events from MockOpenMmAdapter
    assert_eq!(log.events.len(), 5);

    // Verify experiment ref
    assert_eq!(log.experiment_ref.experiment_id, "openmm-mock-001");
    assert_eq!(log.experiment_ref.cycle_id, 0);

    // Verify layer diversity: Theory, Methodology, Implementation all present
    assert!(log.indexes.by_layer.contains_key(&Layer::Theory));
    assert!(log.indexes.by_layer.contains_key(&Layer::Methodology));
    assert!(log.indexes.by_layer.contains_key(&Layer::Implementation));

    // Verify kind diversity
    assert!(log.indexes.by_kind.contains_key(&EventKindTag::ParameterRecord));
    assert!(log.indexes.by_kind.contains_key(&EventKindTag::ResourceStatus));
    assert!(log.indexes.by_kind.contains_key(&EventKindTag::EnergyRecord));
    assert!(log.indexes.by_kind.contains_key(&EventKindTag::ExecutionStatus));

    // Verify temporal ordering
    for window in log.events.windows(2) {
        assert!(window[0].temporal.logical_sequence < window[1].temporal.logical_sequence);
    }

    // Verify last event is ExecutionStatus::Success
    let last = log.events.last().unwrap();
    match &last.kind {
        EventKind::ExecutionStatus { status, .. } => {
            assert_eq!(*status, ExecutionOutcome::Success);
        }
        other => panic!("Expected ExecutionStatus, got {:?}", other),
    }
}

#[test]
fn test_hybrid_upgrade_fields_present() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace("").unwrap();

    // All events must have dag_node_ref, spec_ref, and causal_refs fields.
    // In Stage 1 (LEL-only), these are None/empty but structurally present.
    for event in &log.events {
        // dag_node_ref exists (as None for mock data)
        assert!(event.dag_node_ref.is_none());
        // spec_ref exists (as None for mock data)
        assert!(event.spec_ref.is_none());
        // causal_refs exists (as empty or populated vec)
        let _ = &event.causal_refs; // Field access compiles — structural presence confirmed
    }

    // Verify at least one event has non-empty causal_refs (event4 references event3)
    let has_causal_ref = log.events.iter().any(|e| !e.causal_refs.is_empty());
    assert!(
        has_causal_ref,
        "At least one event should demonstrate causal_refs usage"
    );
}

#[test]
fn test_spec_separation() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace("").unwrap();

    // AP1 avoidance: ExperimentSpec is structurally separate from the event stream.
    // The spec is its own entity, not embedded within events.
    let spec = &log.spec;
    let events = &log.events;

    // Spec has controlled variables (from experiment design)
    assert!(
        !spec.controlled_variables.is_empty(),
        "Spec should contain controlled variables"
    );

    // Events contain execution trace data
    assert!(!events.is_empty(), "Event stream should contain trace events");

    // Spec provenance is different from event provenance (different sources)
    assert_ne!(
        spec.provenance.source_file, events[0].provenance.source_file,
        "Spec and events should have different provenance sources"
    );
}

#[test]
fn test_serde_roundtrip() {
    setup();
    let adapter = MockOpenMmAdapter;
    let original = adapter.parse_trace("").unwrap();

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&original).expect("Serialization must succeed");

    // Verify JSON is non-empty and contains expected fields
    assert!(json.contains("openmm-mock-001"));
    assert!(json.contains("force_field"));
    assert!(json.contains("Theory"));
    assert!(json.contains("Implementation"));

    // Deserialize back
    let restored: LayeredEventLog =
        serde_json::from_str(&json).expect("Deserialization must succeed");

    // Verify structural equality
    assert_eq!(original.events.len(), restored.events.len());
    assert_eq!(
        original.experiment_ref.experiment_id,
        restored.experiment_ref.experiment_id
    );
    assert_eq!(original.experiment_ref.cycle_id, restored.experiment_ref.cycle_id);

    // Verify event-level equality
    for (orig, rest) in original.events.iter().zip(restored.events.iter()) {
        assert_eq!(orig.id, rest.id);
        assert_eq!(orig.layer, rest.layer);
        assert_eq!(orig.temporal, rest.temporal);
        assert_eq!(orig.dag_node_ref, rest.dag_node_ref);
        assert_eq!(orig.spec_ref, rest.spec_ref);
        assert_eq!(orig.causal_refs, rest.causal_refs);
    }
}
