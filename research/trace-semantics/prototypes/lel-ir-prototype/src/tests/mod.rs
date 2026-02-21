use std::sync::Once;

use crate::adapter::{DslAdapter, MockOpenMmAdapter};
use crate::common::*;
use crate::event_kinds::EventKind;
use crate::lel::*;
use crate::overlay::CausalOverlay;

/// Helper: initialize the global event ID counter once for the test process.
fn setup() {
    static INIT_EVENT_COUNTER: Once = Once::new();
    INIT_EVENT_COUNTER.call_once(reset_event_id_counter);
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

    assert!(!log.indexes.by_layer.contains_key(&Layer::Methodology));
}

#[test]
fn test_by_id_index_populated() {
    setup();
    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Theory)
                .kind(EventKind::ParameterRecord {
                    name: "alpha".to_string(),
                    specified_value: None,
                    actual_value: Value::Known(1.0, "nm".to_string()),
                    units: Some("nm".to_string()),
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
        .build();

    assert_eq!(log.indexes.by_id.len(), log.events.len());
    for event in &log.events {
        assert!(log.indexes.by_id.contains_key(&event.id));
    }
}

#[test]
fn test_by_id_index_correct_positions() {
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

    for (position, event) in log.events.iter().enumerate() {
        let indexed_position = log
            .indexes
            .by_id
            .get(&event.id)
            .expect("by_id must contain every event id");
        assert_eq!(*indexed_position, position);
    }
}

#[test]
fn test_by_id_serde_roundtrip() {
    setup();
    let adapter = MockOpenMmAdapter;
    let original = adapter.parse_trace("").unwrap();

    let json = serde_json::to_string(&original).expect("Serialization must succeed");
    let restored: LayeredEventLog =
        serde_json::from_str(&json).expect("Deserialization must succeed");

    assert_eq!(original.indexes.by_id.len(), restored.indexes.by_id.len());
    for (event_id, position) in &original.indexes.by_id {
        assert_eq!(restored.indexes.by_id.get(event_id), Some(position));
    }
}

#[test]
fn test_overlay_empty_log() {
    setup();
    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec()).build();
    let overlay = CausalOverlay::from_log(&log);

    assert_eq!(overlay.len(), 0);
    assert!(overlay.is_empty());
    assert!(overlay.entity(0).is_none());
}

#[test]
fn test_overlay_one_to_one_mapping() {
    setup();
    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Theory)
                .kind(EventKind::ParameterRecord {
                    name: "a".to_string(),
                    specified_value: None,
                    actual_value: Value::Known(1.0, "nm".to_string()),
                    units: Some("nm".to_string()),
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
        .build();

    let overlay = CausalOverlay::from_log(&log);
    assert_eq!(overlay.len(), log.events.len());
    for idx in 0..log.events.len() {
        assert_eq!(overlay.entity(idx).unwrap().event_idx, idx);
    }
}

#[test]
fn test_overlay_dag_node_index() {
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
                .dag_node_ref("node_x".to_string())
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
                .dag_node_ref("node_x".to_string())
                .build(),
        )
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Theory)
                .kind(EventKind::ParameterRecord {
                    name: "alpha".to_string(),
                    specified_value: None,
                    actual_value: Value::Known(2.0, "nm".to_string()),
                    units: Some("nm".to_string()),
                    observation_mode: ObservationMode::Observational,
                })
                .temporal(TemporalCoord {
                    simulation_step: 2,
                    wall_clock_ns: None,
                    logical_sequence: 3,
                })
                .dag_node_ref("node_y".to_string())
                .build(),
        )
        .build();

    let overlay = CausalOverlay::from_log(&log);
    assert_eq!(overlay.entity_by_dag_node.get("node_x"), Some(&vec![0, 1]));
    assert_eq!(overlay.entity_by_dag_node.get("node_y"), Some(&vec![2]));
}

#[test]
fn test_overlay_causal_parents_resolution() {
    setup();
    let event1 = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "p0".to_string(),
            specified_value: None,
            actual_value: Value::Known(1.0, "nm".to_string()),
            units: Some("nm".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .build();
    let event1_id = event1.id;

    let event2 = TraceEventBuilder::new()
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
        .causal_refs(vec![event1_id])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(event1)
        .add_event(event2)
        .build();

    let overlay = CausalOverlay::from_log(&log);
    assert_eq!(overlay.entity(0).unwrap().causal_parents, Vec::<usize>::new());
    assert_eq!(overlay.entity(1).unwrap().causal_parents, vec![0]);
}

#[test]
fn test_overlay_dangling_ref_skipped() {
    setup();
    let parent = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "k".to_string(),
            specified_value: None,
            actual_value: Value::Known(1.0, "nm".to_string()),
            units: Some("nm".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .build();
    let parent_id = parent.id;

    let child = TraceEventBuilder::new()
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
        .causal_refs(vec![parent_id, EventId(999_999)])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(parent)
        .add_event(child)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    assert_eq!(overlay.entity(1).unwrap().causal_parents, vec![0]);
}

#[test]
fn test_overlay_linear_chain_ancestors() {
    setup();
    let e0 = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "e0".to_string(),
            specified_value: None,
            actual_value: Value::Known(1.0, "nm".to_string()),
            units: Some("nm".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .build();
    let e0_id = e0.id;

    let e1 = TraceEventBuilder::new()
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
        .causal_refs(vec![e0_id])
        .build();
    let e1_id = e1.id;

    let e2 = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ExecutionStatus {
            status: ExecutionOutcome::Success,
            framework_error_id: None,
        })
        .temporal(TemporalCoord {
            simulation_step: 2,
            wall_clock_ns: None,
            logical_sequence: 3,
        })
        .causal_refs(vec![e1_id])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(e0)
        .add_event(e1)
        .add_event(e2)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    assert_eq!(overlay.transitive_ancestors(2), vec![1, 0]);
}

#[test]
fn test_overlay_diamond_ancestors() {
    setup();
    let root = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "root".to_string(),
            specified_value: None,
            actual_value: Value::Known(1.0, "nm".to_string()),
            units: Some("nm".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .build();
    let root_id = root.id;

    let left = TraceEventBuilder::new()
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
        .causal_refs(vec![root_id])
        .build();
    let left_id = left.id;

    let right = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ExecutionStatus {
            status: ExecutionOutcome::Success,
            framework_error_id: None,
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 3,
        })
        .causal_refs(vec![root_id])
        .build();
    let right_id = right.id;

    let sink = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ExecutionStatus {
            status: ExecutionOutcome::Success,
            framework_error_id: None,
        })
        .temporal(TemporalCoord {
            simulation_step: 2,
            wall_clock_ns: None,
            logical_sequence: 4,
        })
        .causal_refs(vec![left_id, right_id])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(root)
        .add_event(left)
        .add_event(right)
        .add_event(sink)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let mut ancestors = overlay.transitive_ancestors(3);
    ancestors.sort_unstable();
    assert_eq!(ancestors, vec![0, 1, 2]);
}

#[test]
fn test_overlay_serde_roundtrip() {
    setup();
    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Theory)
                .kind(EventKind::ParameterRecord {
                    name: "a".to_string(),
                    specified_value: None,
                    actual_value: Value::Known(1.0, "nm".to_string()),
                    units: Some("nm".to_string()),
                    observation_mode: ObservationMode::Observational,
                })
                .temporal(TemporalCoord {
                    simulation_step: 0,
                    wall_clock_ns: None,
                    logical_sequence: 1,
                })
                .dag_node_ref("node_a".to_string())
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
        .build();
    let original = CausalOverlay::from_log(&log);

    let json = serde_json::to_string(&original).expect("Serialization must succeed");
    let restored: CausalOverlay =
        serde_json::from_str(&json).expect("Deserialization must succeed");

    assert_eq!(original.len(), restored.len());
    assert_eq!(original.entity_by_dag_node, restored.entity_by_dag_node);
    assert_eq!(
        original.entity(0).unwrap().dag_node,
        restored.entity(0).unwrap().dag_node
    );
}

#[test]
fn test_detect_confounders_all_controlled() {
    setup();
    let mut spec = test_spec();
    spec.controlled_variables = vec![ControlledVariable {
        id: SpecElementId(99),
        parameter: "conf".to_string(),
        held_value: Value::Known(1.0, "arb".to_string()),
    }];

    let conf = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "conf".to_string(),
            specified_value: None,
            actual_value: Value::Known(1.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .dag_node_ref("conf".to_string())
        .build();
    let conf_id = conf.id;

    let intervention = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "dose".to_string(),
            specified_value: None,
            actual_value: Value::Known(2.0, "mg".to_string()),
            units: Some("mg".to_string()),
            observation_mode: ObservationMode::Interventional,
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .causal_refs(vec![conf_id])
        .dag_node_ref("dose".to_string())
        .build();

    let observable = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "outcome".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(10.0, "unit".to_string()),
            uncertainty: None,
            conditions: "test".to_string(),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 2,
            wall_clock_ns: None,
            logical_sequence: 3,
        })
        .causal_refs(vec![conf_id])
        .dag_node_ref("outcome".to_string())
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(conf)
        .add_event(intervention)
        .add_event(observable)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let candidates = overlay.detect_confounders(&log, "outcome", "dose");
    assert!(candidates.is_empty());
}

#[test]
fn test_detect_confounders_uncontrolled_detected() {
    setup();
    let spec = test_spec();

    let conf = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "conf".to_string(),
            specified_value: None,
            actual_value: Value::Known(5.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .dag_node_ref("conf".to_string())
        .build();
    let conf_id = conf.id;

    let intervention = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "dose".to_string(),
            specified_value: None,
            actual_value: Value::Known(2.0, "mg".to_string()),
            units: Some("mg".to_string()),
            observation_mode: ObservationMode::Interventional,
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .causal_refs(vec![conf_id])
        .dag_node_ref("dose".to_string())
        .build();

    let observable = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "outcome".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(12.0, "unit".to_string()),
            uncertainty: None,
            conditions: "test".to_string(),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 2,
            wall_clock_ns: None,
            logical_sequence: 3,
        })
        .causal_refs(vec![conf_id])
        .dag_node_ref("outcome".to_string())
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(conf)
        .add_event(intervention)
        .add_event(observable)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let candidates = overlay.detect_confounders(&log, "outcome", "dose");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].dag_node, "conf");
    assert_eq!(candidates[0].observable_ancestor_events, vec![0]);
    assert_eq!(candidates[0].intervention_ancestor_events, vec![0]);
}

#[test]
fn test_detect_confounders_intervention_excluded() {
    setup();
    let spec = test_spec();

    let intervention_root = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "dose".to_string(),
            specified_value: None,
            actual_value: Value::Known(1.0, "mg".to_string()),
            units: Some("mg".to_string()),
            observation_mode: ObservationMode::Interventional,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .dag_node_ref("dose".to_string())
        .build();
    let intervention_root_id = intervention_root.id;

    let intervention_child = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "dose".to_string(),
            specified_value: None,
            actual_value: Value::Known(2.0, "mg".to_string()),
            units: Some("mg".to_string()),
            observation_mode: ObservationMode::Interventional,
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .causal_refs(vec![intervention_root_id])
        .dag_node_ref("dose".to_string())
        .build();
    let intervention_child_id = intervention_child.id;

    let observable = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "outcome".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(8.0, "unit".to_string()),
            uncertainty: None,
            conditions: "test".to_string(),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 2,
            wall_clock_ns: None,
            logical_sequence: 3,
        })
        .causal_refs(vec![intervention_child_id])
        .dag_node_ref("outcome".to_string())
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(intervention_root)
        .add_event(intervention_child)
        .add_event(observable)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let candidates = overlay.detect_confounders(&log, "outcome", "dose");
    assert!(candidates.is_empty());
}

#[test]
fn test_detect_confounders_no_common_ancestors() {
    setup();
    let spec = test_spec();

    let left_root = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "left_root".to_string(),
            specified_value: None,
            actual_value: Value::Known(1.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .dag_node_ref("left_root".to_string())
        .build();
    let left_root_id = left_root.id;

    let right_root = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "right_root".to_string(),
            specified_value: None,
            actual_value: Value::Known(2.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Interventional,
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .dag_node_ref("right_root".to_string())
        .build();
    let right_root_id = right_root.id;

    let intervention = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "dose".to_string(),
            specified_value: None,
            actual_value: Value::Known(3.0, "mg".to_string()),
            units: Some("mg".to_string()),
            observation_mode: ObservationMode::Interventional,
        })
        .temporal(TemporalCoord {
            simulation_step: 2,
            wall_clock_ns: None,
            logical_sequence: 3,
        })
        .causal_refs(vec![right_root_id])
        .dag_node_ref("dose".to_string())
        .build();

    let observable = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "outcome".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(9.0, "unit".to_string()),
            uncertainty: None,
            conditions: "test".to_string(),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 3,
            wall_clock_ns: None,
            logical_sequence: 4,
        })
        .causal_refs(vec![left_root_id])
        .dag_node_ref("outcome".to_string())
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(left_root)
        .add_event(right_root)
        .add_event(intervention)
        .add_event(observable)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let candidates = overlay.detect_confounders(&log, "outcome", "dose");
    assert!(candidates.is_empty());
}

#[test]
fn test_detect_confounders_unknown_variable() {
    setup();
    let spec = test_spec();

    let event = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "outcome".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(1.0, "unit".to_string()),
            uncertainty: None,
            conditions: "test".to_string(),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(event)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    assert!(overlay
        .detect_confounders(&log, "unknown_observable", "dose")
        .is_empty());
    assert!(overlay
        .detect_confounders(&log, "outcome", "unknown_intervention")
        .is_empty());
}

#[test]
fn test_detect_confounders_multiple_confounders() {
    setup();
    let spec = test_spec();

    let conf_a = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "conf_a".to_string(),
            specified_value: None,
            actual_value: Value::Known(1.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .dag_node_ref("conf_a".to_string())
        .build();
    let conf_a_id = conf_a.id;

    let conf_b = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "conf_b".to_string(),
            specified_value: None,
            actual_value: Value::Known(2.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .dag_node_ref("conf_b".to_string())
        .build();
    let conf_b_id = conf_b.id;

    let intervention = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "dose".to_string(),
            specified_value: None,
            actual_value: Value::Known(4.0, "mg".to_string()),
            units: Some("mg".to_string()),
            observation_mode: ObservationMode::Interventional,
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 3,
        })
        .causal_refs(vec![conf_a_id, conf_b_id])
        .dag_node_ref("dose".to_string())
        .build();

    let observable = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "outcome".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(7.0, "unit".to_string()),
            uncertainty: None,
            conditions: "test".to_string(),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 2,
            wall_clock_ns: None,
            logical_sequence: 4,
        })
        .causal_refs(vec![conf_a_id, conf_b_id])
        .dag_node_ref("outcome".to_string())
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(conf_a)
        .add_event(conf_b)
        .add_event(intervention)
        .add_event(observable)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let candidates = overlay.detect_confounders(&log, "outcome", "dose");
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].dag_node, "conf_a");
    assert_eq!(candidates[1].dag_node, "conf_b");
}

#[test]
fn test_detect_confounders_transitive_chain() {
    setup();
    let spec = test_spec();

    let root = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "root".to_string(),
            specified_value: None,
            actual_value: Value::Known(1.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .build();
    let root_id = root.id;

    let mid = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "mid".to_string(),
            specified_value: None,
            actual_value: Value::Known(2.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .causal_refs(vec![root_id])
        .dag_node_ref("mid".to_string())
        .build();
    let mid_id = mid.id;

    let intervention = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "dose".to_string(),
            specified_value: None,
            actual_value: Value::Known(5.0, "mg".to_string()),
            units: Some("mg".to_string()),
            observation_mode: ObservationMode::Interventional,
        })
        .temporal(TemporalCoord {
            simulation_step: 2,
            wall_clock_ns: None,
            logical_sequence: 3,
        })
        .causal_refs(vec![mid_id])
        .dag_node_ref("dose".to_string())
        .build();

    let observable = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "outcome".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(6.0, "unit".to_string()),
            uncertainty: None,
            conditions: "test".to_string(),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 3,
            wall_clock_ns: None,
            logical_sequence: 4,
        })
        .causal_refs(vec![mid_id])
        .dag_node_ref("outcome".to_string())
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(root)
        .add_event(mid)
        .add_event(intervention)
        .add_event(observable)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let candidates = overlay.detect_confounders(&log, "outcome", "dose");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].dag_node, "mid");
    assert_eq!(candidates[0].observable_ancestor_events, vec![1]);
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

    assert!(!log
        .indexes
        .by_kind
        .contains_key(&EventKindTag::ExceptionEvent));
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
