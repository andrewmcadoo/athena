use std::sync::Once;

use crate::adapter::{parse_openmm_energy_series, DslAdapter, MockOpenMmAdapter};
use crate::common::*;
use crate::convergence::{
    classify_all_convergence, classify_convergence, ConvergenceConfidence, ConvergencePattern,
};
use crate::event_kinds::EventKind;
use crate::gromacs_adapter::{
    classify_mdp_parameter, parse_log, parse_mdp, GromacsAdapter,
};
use crate::lel::*;
use crate::overlay::{CausalOverlay, PredictionComparison};
use crate::vasp_adapter::{
    classify_incar_parameter, parse_incar, parse_oszicar, parse_outcar, VaspAdapter,
};

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

/// Helper: create a test spec with caller-provided prediction records.
fn test_spec_with_predictions(predictions: Vec<PredictionRecord>) -> ExperimentSpec {
    let mut spec = test_spec();
    spec.predictions = predictions;
    spec
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
fn test_compare_predictions_empty_log() {
    setup();
    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec()).build();
    let overlay = CausalOverlay::from_log(&log);

    assert!(overlay.compare_predictions(&log).is_empty());
}

#[test]
fn test_compare_predictions_no_comparison_events() {
    setup();
    let spec = test_spec_with_predictions(vec![PredictionRecord {
        id: SpecElementId(1),
        variable: "outcome".to_string(),
        predicted_value: Value::Known(10.0, "unit".to_string()),
        tolerance: Some(0.5),
    }]);

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(
            TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::ObservableMeasurement {
                    variable_name: "outcome".to_string(),
                    measurement_method: "synthetic".to_string(),
                    value: Value::Known(10.1, "unit".to_string()),
                    uncertainty: None,
                    conditions: "test".to_string(),
                    observation_mode: ObservationMode::Observational,
                })
                .temporal(TemporalCoord {
                    simulation_step: 0,
                    wall_clock_ns: None,
                    logical_sequence: 1,
                })
                .build(),
        )
        .build();
    let overlay = CausalOverlay::from_log(&log);

    assert!(overlay.compare_predictions(&log).is_empty());
}

#[test]
fn test_compare_predictions_matched_agreement() {
    setup();
    let spec = test_spec_with_predictions(vec![PredictionRecord {
        id: SpecElementId(101),
        variable: "energy".to_string(),
        predicted_value: Value::Known(-100.0, "kJ/mol".to_string()),
        tolerance: Some(5.0),
    }]);

    let observation = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "energy".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(-98.0, "kJ/mol".to_string()),
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
    let observation_id = observation.id;

    let comparison = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ComparisonResult {
            prediction_id: "101".to_string(),
            observation_id,
            result: ComparisonOutcome {
                agreement: true,
                divergence: None,
                detail: "within tolerance".to_string(),
            },
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(observation)
        .add_event(comparison)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let comparisons = overlay.compare_predictions(&log);
    assert_eq!(comparisons.len(), 1);
    assert_eq!(comparisons[0].comparison_event_idx, 1);
    assert_eq!(comparisons[0].prediction_id, Some(SpecElementId(101)));
    assert_eq!(comparisons[0].variable, "energy");
    assert!(!comparisons[0].is_falsified);
    assert!(comparisons[0].outcome.agreement);
}

#[test]
fn test_compare_predictions_matched_falsified() {
    setup();
    let spec = test_spec_with_predictions(vec![PredictionRecord {
        id: SpecElementId(202),
        variable: "rdf_peak".to_string(),
        predicted_value: Value::Known(1.5, "arb".to_string()),
        tolerance: Some(0.1),
    }]);

    let observation = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "rdf_peak".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(2.2, "arb".to_string()),
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
    let observation_id = observation.id;

    let comparison = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ComparisonResult {
            prediction_id: "202".to_string(),
            observation_id,
            result: ComparisonOutcome {
                agreement: false,
                divergence: Some(DivergenceMeasure::AbsoluteDifference(0.7)),
                detail: "outside tolerance".to_string(),
            },
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(observation)
        .add_event(comparison)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let comparisons = overlay.compare_predictions(&log);
    assert_eq!(comparisons.len(), 1);
    assert_eq!(comparisons[0].prediction_id, Some(SpecElementId(202)));
    assert_eq!(comparisons[0].variable, "rdf_peak");
    assert!(comparisons[0].is_falsified);
    assert!(!comparisons[0].outcome.agreement);
    assert!(matches!(
        comparisons[0].outcome.divergence,
        Some(DivergenceMeasure::AbsoluteDifference(_))
    ));
}

#[test]
fn test_compare_predictions_unresolvable_prediction_id() {
    setup();
    let spec = test_spec_with_predictions(vec![PredictionRecord {
        id: SpecElementId(303),
        variable: "pressure".to_string(),
        predicted_value: Value::Known(1.0, "bar".to_string()),
        tolerance: Some(0.05),
    }]);

    let observation = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "pressure".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(1.2, "bar".to_string()),
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
    let observation_id = observation.id;

    let comparison = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ComparisonResult {
            prediction_id: "not-a-u64".to_string(),
            observation_id,
            result: ComparisonOutcome {
                agreement: false,
                divergence: Some(DivergenceMeasure::AbsoluteDifference(0.2)),
                detail: "malformed prediction id".to_string(),
            },
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(observation)
        .add_event(comparison)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let comparisons = overlay.compare_predictions(&log);
    assert_eq!(comparisons.len(), 1);
    assert_eq!(comparisons[0].prediction_id, None);
    assert_eq!(comparisons[0].variable, "unknown");
}

#[test]
fn test_compare_predictions_multiple_predictions() {
    setup();
    let spec = test_spec_with_predictions(vec![
        PredictionRecord {
            id: SpecElementId(1),
            variable: "var_a".to_string(),
            predicted_value: Value::Known(10.0, "unit".to_string()),
            tolerance: Some(1.0),
        },
        PredictionRecord {
            id: SpecElementId(2),
            variable: "var_b".to_string(),
            predicted_value: Value::Known(20.0, "unit".to_string()),
            tolerance: Some(1.0),
        },
    ]);

    let observation_a = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "var_a".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(10.2, "unit".to_string()),
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
    let observation_a_id = observation_a.id;

    let observation_b = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "var_b".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(22.0, "unit".to_string()),
            uncertainty: None,
            conditions: "test".to_string(),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .build();
    let observation_b_id = observation_b.id;

    let comparison_a = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ComparisonResult {
            prediction_id: "1".to_string(),
            observation_id: observation_a_id,
            result: ComparisonOutcome {
                agreement: true,
                divergence: None,
                detail: "prediction A matched".to_string(),
            },
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 3,
        })
        .build();

    let comparison_b = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ComparisonResult {
            prediction_id: "2".to_string(),
            observation_id: observation_b_id,
            result: ComparisonOutcome {
                agreement: false,
                divergence: Some(DivergenceMeasure::AbsoluteDifference(2.0)),
                detail: "prediction B falsified".to_string(),
            },
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 4,
        })
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(observation_a)
        .add_event(observation_b)
        .add_event(comparison_a)
        .add_event(comparison_b)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let comparisons = overlay.compare_predictions(&log);
    assert_eq!(comparisons.len(), 2);
    assert_eq!(comparisons[0].prediction_id, Some(SpecElementId(1)));
    assert_eq!(comparisons[0].variable, "var_a");
    assert!(!comparisons[0].is_falsified);
    assert_eq!(comparisons[1].prediction_id, Some(SpecElementId(2)));
    assert_eq!(comparisons[1].variable, "var_b");
    assert!(comparisons[1].is_falsified);
}

#[test]
fn test_compare_predictions_with_dag_node_ref() {
    setup();
    let spec = test_spec_with_predictions(vec![PredictionRecord {
        id: SpecElementId(7),
        variable: "temperature".to_string(),
        predicted_value: Value::Known(300.0, "K".to_string()),
        tolerance: Some(1.0),
    }]);

    let observation = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::ObservableMeasurement {
            variable_name: "temperature".to_string(),
            measurement_method: "synthetic".to_string(),
            value: Value::Known(300.5, "K".to_string()),
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
    let observation_id = observation.id;

    let comparison = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ComparisonResult {
            prediction_id: "7".to_string(),
            observation_id,
            result: ComparisonOutcome {
                agreement: true,
                divergence: None,
                detail: "matched".to_string(),
            },
        })
        .temporal(TemporalCoord {
            simulation_step: 1,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .dag_node_ref("dag.compare.temperature".to_string())
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), spec)
        .add_event(observation)
        .add_event(comparison)
        .build();
    let overlay = CausalOverlay::from_log(&log);

    let comparisons = overlay.compare_predictions(&log);
    assert_eq!(comparisons.len(), 1);
    assert_eq!(
        comparisons[0].dag_node,
        Some("dag.compare.temperature".to_string())
    );
}

#[test]
fn test_implicate_no_ancestors() {
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
        .build();
    let overlay = CausalOverlay::from_log(&log);
    let comparison = PredictionComparison {
        comparison_event_idx: 0,
        prediction_id: None,
        variable: "unknown".to_string(),
        outcome: ComparisonOutcome {
            agreement: false,
            divergence: None,
            detail: "synthetic".to_string(),
        },
        is_falsified: true,
        dag_node: None,
    };

    let implicated = overlay.implicate_causal_nodes(&log, &comparison);
    assert!(implicated.is_empty());
}

#[test]
fn test_implicate_theory_layer() {
    setup();
    let theory_ancestor = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "theory".to_string(),
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
        .dag_node_ref("theory_ancestor".to_string())
        .build();
    let theory_ancestor_id = theory_ancestor.id;

    let comparison_event = TraceEventBuilder::new()
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
        .causal_refs(vec![theory_ancestor_id])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(theory_ancestor)
        .add_event(comparison_event)
        .build();
    let overlay = CausalOverlay::from_log(&log);
    let comparison = PredictionComparison {
        comparison_event_idx: 1,
        prediction_id: None,
        variable: "unknown".to_string(),
        outcome: ComparisonOutcome {
            agreement: false,
            divergence: None,
            detail: "synthetic".to_string(),
        },
        is_falsified: true,
        dag_node: None,
    };

    let implicated = overlay.implicate_causal_nodes(&log, &comparison);
    assert_eq!(implicated.len(), 1);
    assert_eq!(implicated[0].dag_node, "theory_ancestor");
    assert_eq!(implicated[0].layer, Layer::Theory);
    assert_eq!(implicated[0].causal_distance, 1);
    assert_eq!(implicated[0].ancestor_event_indices, vec![0]);
}

#[test]
fn test_implicate_implementation_layer() {
    setup();
    let impl_ancestor = TraceEventBuilder::new()
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
            logical_sequence: 1,
        })
        .dag_node_ref("impl_ancestor".to_string())
        .build();
    let impl_ancestor_id = impl_ancestor.id;

    let comparison_event = TraceEventBuilder::new()
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
        .causal_refs(vec![impl_ancestor_id])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(impl_ancestor)
        .add_event(comparison_event)
        .build();
    let overlay = CausalOverlay::from_log(&log);
    let comparison = PredictionComparison {
        comparison_event_idx: 1,
        prediction_id: None,
        variable: "unknown".to_string(),
        outcome: ComparisonOutcome {
            agreement: false,
            divergence: None,
            detail: "synthetic".to_string(),
        },
        is_falsified: true,
        dag_node: None,
    };

    let implicated = overlay.implicate_causal_nodes(&log, &comparison);
    assert_eq!(implicated.len(), 1);
    assert_eq!(implicated[0].dag_node, "impl_ancestor");
    assert_eq!(implicated[0].layer, Layer::Implementation);
    assert_eq!(implicated[0].causal_distance, 1);
}

#[test]
fn test_implicate_methodology_layer() {
    setup();
    let methodology_ancestor = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "method".to_string(),
            specified_value: None,
            actual_value: Value::Known(2.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .dag_node_ref("methodology_ancestor".to_string())
        .build();
    let methodology_ancestor_id = methodology_ancestor.id;

    let comparison_event = TraceEventBuilder::new()
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
        .causal_refs(vec![methodology_ancestor_id])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(methodology_ancestor)
        .add_event(comparison_event)
        .build();
    let overlay = CausalOverlay::from_log(&log);
    let comparison = PredictionComparison {
        comparison_event_idx: 1,
        prediction_id: None,
        variable: "unknown".to_string(),
        outcome: ComparisonOutcome {
            agreement: false,
            divergence: None,
            detail: "synthetic".to_string(),
        },
        is_falsified: true,
        dag_node: None,
    };

    let implicated = overlay.implicate_causal_nodes(&log, &comparison);
    assert_eq!(implicated.len(), 1);
    assert_eq!(implicated[0].dag_node, "methodology_ancestor");
    assert_eq!(implicated[0].layer, Layer::Methodology);
    assert_eq!(implicated[0].causal_distance, 1);
}

#[test]
fn test_implicate_mixed_layers() {
    setup();
    let methodology_ancestor = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "method".to_string(),
            specified_value: None,
            actual_value: Value::Known(2.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 1,
        })
        .dag_node_ref("method_node".to_string())
        .build();
    let methodology_ancestor_id = methodology_ancestor.id;

    let theory_ancestor = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "theory".to_string(),
            specified_value: None,
            actual_value: Value::Known(1.0, "arb".to_string()),
            units: Some("arb".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 0,
            wall_clock_ns: None,
            logical_sequence: 2,
        })
        .dag_node_ref("theory_node".to_string())
        .build();
    let theory_ancestor_id = theory_ancestor.id;

    let comparison_event = TraceEventBuilder::new()
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
        .causal_refs(vec![methodology_ancestor_id, theory_ancestor_id])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(methodology_ancestor)
        .add_event(theory_ancestor)
        .add_event(comparison_event)
        .build();
    let overlay = CausalOverlay::from_log(&log);
    let comparison = PredictionComparison {
        comparison_event_idx: 2,
        prediction_id: None,
        variable: "unknown".to_string(),
        outcome: ComparisonOutcome {
            agreement: false,
            divergence: None,
            detail: "synthetic".to_string(),
        },
        is_falsified: true,
        dag_node: None,
    };

    let implicated = overlay.implicate_causal_nodes(&log, &comparison);
    assert_eq!(implicated.len(), 2);
    assert_eq!(implicated[0].dag_node, "theory_node");
    assert_eq!(implicated[0].layer, Layer::Theory);
    assert_eq!(implicated[1].dag_node, "method_node");
    assert_eq!(implicated[1].layer, Layer::Methodology);
}

#[test]
fn test_implicate_depth_ordering() {
    setup();
    let far = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "far".to_string(),
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
        .dag_node_ref("far_node".to_string())
        .build();
    let far_id = far.id;

    let near = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "near".to_string(),
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
        .causal_refs(vec![far_id])
        .dag_node_ref("near_node".to_string())
        .build();
    let near_id = near.id;

    let comparison_event = TraceEventBuilder::new()
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
        .causal_refs(vec![near_id])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(far)
        .add_event(near)
        .add_event(comparison_event)
        .build();
    let overlay = CausalOverlay::from_log(&log);
    let comparison = PredictionComparison {
        comparison_event_idx: 2,
        prediction_id: None,
        variable: "unknown".to_string(),
        outcome: ComparisonOutcome {
            agreement: false,
            divergence: None,
            detail: "synthetic".to_string(),
        },
        is_falsified: true,
        dag_node: None,
    };

    let implicated = overlay.implicate_causal_nodes(&log, &comparison);
    assert_eq!(implicated.len(), 2);
    assert_eq!(implicated[0].dag_node, "near_node");
    assert_eq!(implicated[0].causal_distance, 1);
    assert_eq!(implicated[1].dag_node, "far_node");
    assert_eq!(implicated[1].causal_distance, 2);
}

#[test]
fn test_implicate_ancestor_without_dag_node() {
    setup();
    let ancestor_without_dag = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "unnamed".to_string(),
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
    let ancestor_without_dag_id = ancestor_without_dag.id;

    let comparison_event = TraceEventBuilder::new()
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
        .causal_refs(vec![ancestor_without_dag_id])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(ancestor_without_dag)
        .add_event(comparison_event)
        .build();
    let overlay = CausalOverlay::from_log(&log);
    let comparison = PredictionComparison {
        comparison_event_idx: 1,
        prediction_id: None,
        variable: "unknown".to_string(),
        outcome: ComparisonOutcome {
            agreement: false,
            divergence: None,
            detail: "synthetic".to_string(),
        },
        is_falsified: true,
        dag_node: None,
    };

    let implicated = overlay.implicate_causal_nodes(&log, &comparison);
    assert!(implicated.is_empty());
}

#[test]
fn test_implicate_multiple_events_same_dag_node() {
    setup();
    let far_shared = TraceEventBuilder::new()
        .layer(Layer::Theory)
        .kind(EventKind::ParameterRecord {
            name: "shared_far".to_string(),
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
        .dag_node_ref("shared_node".to_string())
        .build();
    let far_shared_id = far_shared.id;

    let near_shared = TraceEventBuilder::new()
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
        .causal_refs(vec![far_shared_id])
        .dag_node_ref("shared_node".to_string())
        .build();
    let near_shared_id = near_shared.id;

    let comparison_event = TraceEventBuilder::new()
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
        .causal_refs(vec![near_shared_id])
        .build();

    let log = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec())
        .add_event(far_shared)
        .add_event(near_shared)
        .add_event(comparison_event)
        .build();
    let overlay = CausalOverlay::from_log(&log);
    let comparison = PredictionComparison {
        comparison_event_idx: 2,
        prediction_id: None,
        variable: "unknown".to_string(),
        outcome: ComparisonOutcome {
            agreement: false,
            divergence: None,
            detail: "synthetic".to_string(),
        },
        is_falsified: true,
        dag_node: None,
    };

    let implicated = overlay.implicate_causal_nodes(&log, &comparison);
    assert_eq!(implicated.len(), 1);
    assert_eq!(implicated[0].dag_node, "shared_node");
    assert_eq!(implicated[0].layer, Layer::Implementation);
    assert_eq!(implicated[0].causal_distance, 1);
    assert_eq!(implicated[0].ancestor_event_indices.len(), 2);
    assert!(implicated[0].ancestor_event_indices.contains(&0));
    assert!(implicated[0].ancestor_event_indices.contains(&1));
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

const OPENMM_STABLE_ENERGY_SERIES: &str = r#"
0 -45023.7000
1000 -45023.9000
2000 -45024.0100
3000 -45024.0500
"#;

const OPENMM_OSCILLATING_ENERGY_SERIES: &str = r#"
0 -1000.0
100 -999.0
200 -1001.0
300 -998.0
400 -1002.0
"#;

const OPENMM_SHORT_ENERGY_SERIES: &str = r#"
0 -1000.0
100 -1000.3
200 -1000.4
"#;

const OPENMM_CSV_STABLE: &str = r#"
#"Step","Time (ps)","Potential Energy (kJ/mol)","Temperature (K)"
0,0.000,-45023.7000,300.00
1000,2.000,-45023.9000,300.10
2000,4.000,-45024.0100,299.90
3000,6.000,-45024.0500,300.20
"#;

const OPENMM_CSV_OSCILLATING: &str = r#"
#"Step","Time (ps)","Potential Energy (kJ/mol)","Temperature (K)"
0,0.000,-1000.0,300.00
100,0.200,-998.0,300.10
200,0.400,-1001.0,299.90
300,0.600,-997.5,300.20
400,0.800,-1002.0,300.00
"#;

const OPENMM_CSV_DRIFTING: &str = r#"
#"Step","Time (ps)","Potential Energy (kJ/mol)","Temperature (K)"
0,0.000,0.000,300.00
100,0.200,0.010,300.10
200,0.400,0.020,299.90
300,0.600,0.030,300.20
"#;

const OPENMM_CSV_DIVERGENT_NAN: &str = r#"
#"Step","Time (ps)","Potential Energy (kJ/mol)","Temperature (K)"
0,0.000,-1000.0,300.00
100,0.200,-999.9,300.10
200,0.400,nan,299.90
300,0.600,-999.7,300.20
"#;

const OPENMM_CSV_BOUNDARY: &str = r#"
#"Step","Time (ps)","Potential Energy (kJ/mol)","Temperature (K)"
0,0.000,0.0000,300.00
100,0.200,0.0001,300.10
200,0.400,0.0002,299.90
300,0.600,0.0003,300.20
"#;

const OPENMM_REAL_CSV_DEFAULT_KJ: &str =
    include_str!("../../testdata/openmm_state_datareporter/openmm84_default_kj.csv");
const OPENMM_REAL_CSV_REORDERED_EXTRA_COLUMNS: &str = include_str!(
    "../../testdata/openmm_state_datareporter/openmm84_progress_volume_density_speed.csv"
);
const OPENMM_REAL_CSV_MINIMAL_COLUMNS: &str =
    include_str!("../../testdata/openmm_state_datareporter/openmm84_minimal_step_potential.csv");
const OPENMM_REAL_CSV_WINDOWS_CRLF: &str =
    include_str!("../../testdata/openmm_state_datareporter/openmm84_crlf.csv");
const OPENMM_REAL_CSV_BOM_PREFIX: &str =
    include_str!("../../testdata/openmm_state_datareporter/openmm84_bom.csv");

const OPENMM_SOURCE_DERIVED_CSV_KCAL_UNITS: &str = r#"
#"Step","Time (ps)","Potential Energy (kcal/mol)","Temperature (K)"
100,0.200,0.0,300.0
200,0.400,0.0,300.0
300,0.600,0.0,300.0
400,0.800,0.0,300.0
"#;

const OPENMM_SOURCE_DERIVED_CSV_UNQUOTED_HEADER: &str = r#"
Step,Time (ps),Potential Energy (kJ/mole),Temperature (K)
100,0.200,0.0,300.0
200,0.400,0.0,300.0
300,0.600,0.0,300.0
400,0.800,0.0,300.0
"#;

const OPENMM_SOURCE_DERIVED_CSV_EMPTY_TRAILING_COLUMNS: &str = r#"
#"Step","Time (ps)","Potential Energy (kJ/mole)","Temperature (K)",
100,0.200,0.0,300.0,
200,0.400,0.0,300.0,
300,0.600,0.0,300.0,
400,0.800,0.0,300.0,
"#;

const OPENMM_EXPECTED_REAL_ZERO_SERIES_5: [(u64, f64); 5] = [
    (100, 0.0),
    (200, 0.0),
    (300, 0.0),
    (400, 0.0),
    (500, 0.0),
];
const OPENMM_EXPECTED_ZERO_SERIES_4: [(u64, f64); 4] =
    [(100, 0.0), (200, 0.0), (300, 0.0), (400, 0.0)];

fn assert_openmm_csv_variant(
    raw: &str,
    expected_pairs: &[(u64, f64)],
    expected_pattern: ConvergencePattern,
) {
    let parsed = parse_openmm_energy_series(raw);
    assert_eq!(parsed.len(), expected_pairs.len());
    for ((actual_step, actual_energy), (expected_step, expected_energy)) in
        parsed.iter().zip(expected_pairs.iter())
    {
        assert_eq!(*actual_step, *expected_step);
        assert!((*actual_energy - *expected_energy).abs() < 1e-12);
    }

    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace(raw).unwrap();
    let canonical = classify_all_convergence(&log, "openmm");
    let first_pattern = canonical
        .first()
        .map(|entry| entry.pattern.clone())
        .unwrap_or(ConvergencePattern::InsufficientData);
    assert_eq!(first_pattern, expected_pattern);
}

fn parse_gromacs_log_energy_pairs(log_content: &str) -> Vec<(u64, f64)> {
    parse_log(log_content, 0)
        .unwrap()
        .into_iter()
        .filter_map(|event| match event.kind {
            EventKind::EnergyRecord {
                total: Value::Known(total, _),
                ..
            } => Some((event.temporal.simulation_step, total)),
            _ => None,
        })
        .collect()
}

fn assert_gromacs_log_variant(
    log: &str,
    expected_pairs: &[(u64, f64)],
    expected_pattern: ConvergencePattern,
) {
    let parsed_pairs = parse_gromacs_log_energy_pairs(log);
    assert_eq!(parsed_pairs.len(), expected_pairs.len());
    for ((actual_step, actual_energy), (expected_step, expected_energy)) in
        parsed_pairs.iter().zip(expected_pairs.iter())
    {
        assert_eq!(*actual_step, *expected_step);
        assert!((*actual_energy - *expected_energy).abs() < 1e-6);
    }

    let adapter = GromacsAdapter;
    let parsed_log = adapter.parse_trace(log).unwrap();
    let first_pattern = classify_all_convergence(&parsed_log, "gromacs")
        .into_iter()
        .map(|entry| entry.pattern)
        .next()
        .unwrap_or(ConvergencePattern::InsufficientData);
    assert_eq!(first_pattern, expected_pattern);
}

fn assert_gromacs_log_parses_energy_count(log: &str, count: usize) {
    let energy_count = parse_log(log, 0)
        .unwrap()
        .into_iter()
        .filter(|event| matches!(event.kind, EventKind::EnergyRecord { .. }))
        .count();
    assert_eq!(energy_count, count);
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
fn test_mock_adapter_derives_convergence_summary_for_stable_series() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace(OPENMM_STABLE_ENERGY_SERIES).unwrap();

    let convergence = log
        .events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ConvergencePoint {
                metric_name,
                converged,
                ..
            } => Some((metric_name, converged)),
            _ => None,
        })
        .expect("Expected derived ConvergencePoint event");

    assert_eq!(convergence.0, "derived_convergence_rel_delta_max");
    assert_eq!(convergence.1, &Some(true));
}

#[test]
fn test_mock_adapter_derives_oscillation_summary_for_non_converging_series() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace(OPENMM_OSCILLATING_ENERGY_SERIES).unwrap();

    let convergence = log
        .events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ConvergencePoint {
                metric_name,
                converged,
                ..
            } => Some((metric_name, converged)),
            _ => None,
        })
        .expect("Expected derived ConvergencePoint event");

    assert_eq!(convergence.0, "derived_oscillation_rel_delta_mean");
    assert_eq!(convergence.1, &Some(false));
}

#[test]
fn test_mock_adapter_no_convergence_summary_below_min_window() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace(OPENMM_SHORT_ENERGY_SERIES).unwrap();

    assert!(!log
        .events
        .iter()
        .any(|event| matches!(event.kind, EventKind::ConvergencePoint { .. })));
}

#[test]
fn test_mock_adapter_convergence_summary_provenance_refs() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace(OPENMM_STABLE_ENERGY_SERIES).unwrap();

    let convergence_event = log
        .events
        .iter()
        .find(|event| matches!(event.kind, EventKind::ConvergencePoint { .. }))
        .expect("Expected derived ConvergencePoint event");
    let energy_ids: Vec<EventId> = log
        .events
        .iter()
        .filter(|event| matches!(event.kind, EventKind::EnergyRecord { .. }))
        .map(|event| event.id)
        .collect();
    let execution_id = log
        .events
        .iter()
        .find(|event| matches!(event.kind, EventKind::ExecutionStatus { .. }))
        .map(|event| event.id)
        .expect("Expected ExecutionStatus event");

    assert_eq!(energy_ids.len(), 4);
    for expected_id in &energy_ids {
        assert!(
            convergence_event.causal_refs.contains(expected_id),
            "ConvergencePoint missing source EnergyRecord {:?}",
            expected_id
        );
    }
    assert!(
        convergence_event.causal_refs.contains(&execution_id),
        "ConvergencePoint missing source ExecutionStatus {:?}",
        execution_id
    );
}

#[test]
fn test_mock_adapter_parses_openmm_statedatareporter_csv_pairs() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace(OPENMM_CSV_STABLE).unwrap();

    let energy_records: Vec<(u64, f64)> = log
        .events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::EnergyRecord {
                total: Value::Known(total, _),
                ..
            } => Some((event.temporal.simulation_step, *total)),
            _ => None,
        })
        .collect();

    assert_eq!(energy_records.len(), 4);
    assert_eq!(energy_records[0].0, 0);
    assert_eq!(energy_records[1].0, 1000);
    assert_eq!(energy_records[2].0, 2000);
    assert_eq!(energy_records[3].0, 3000);
    assert!((energy_records[0].1 + 45023.7000).abs() < 1e-9);
    assert!((energy_records[3].1 + 45024.0500).abs() < 1e-9);
}

#[test]
fn test_mock_adapter_csv_derives_convergence_summary_for_stable_series() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace(OPENMM_CSV_STABLE).unwrap();

    let convergence = log
        .events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ConvergencePoint {
                metric_name,
                converged,
                ..
            } => Some((metric_name, converged)),
            _ => None,
        })
        .expect("Expected derived ConvergencePoint event");

    assert_eq!(convergence.0, "derived_convergence_rel_delta_max");
    assert_eq!(convergence.1, &Some(true));
}

#[test]
fn test_mock_adapter_csv_derives_oscillation_summary_for_non_converging_series() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace(OPENMM_CSV_OSCILLATING).unwrap();

    let convergence = log
        .events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ConvergencePoint {
                metric_name,
                converged,
                ..
            } => Some((metric_name, converged)),
            _ => None,
        })
        .expect("Expected derived ConvergencePoint event");

    assert_eq!(convergence.0, "derived_oscillation_rel_delta_mean");
    assert_eq!(convergence.1, &Some(false));
}

#[test]
fn test_mock_adapter_whitespace_parser_backward_compat() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace(OPENMM_STABLE_ENERGY_SERIES).unwrap();

    let energy_records: Vec<(u64, f64)> = log
        .events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::EnergyRecord {
                total: Value::Known(total, _),
                ..
            } => Some((event.temporal.simulation_step, *total)),
            _ => None,
        })
        .collect();

    assert_eq!(energy_records.len(), 4);
    assert_eq!(energy_records[0].0, 0);
    assert_eq!(energy_records[3].0, 3000);
    assert!((energy_records[0].1 + 45023.7000).abs() < 1e-9);
    assert!((energy_records[3].1 + 45024.0500).abs() < 1e-9);
}

#[test]
fn test_openmm_csv_divergent_fixture_emits_nan_status() {
    setup();
    let adapter = MockOpenMmAdapter;
    let log = adapter.parse_trace(OPENMM_CSV_DIVERGENT_NAN).unwrap();

    assert!(log.events.iter().any(|event| {
        matches!(
            event.kind,
            EventKind::NumericalStatus {
                event_type: NumericalEventType::NaNDetected,
                ..
            }
        )
    }));
}

#[test]
fn test_openmm_csv_variant_reordered_columns() {
    setup();
    assert_openmm_csv_variant(
        OPENMM_REAL_CSV_REORDERED_EXTRA_COLUMNS,
        &OPENMM_EXPECTED_REAL_ZERO_SERIES_5,
        ConvergencePattern::Converged,
    );
}

#[test]
fn test_openmm_csv_variant_optional_extra_columns() {
    setup();
    assert_openmm_csv_variant(
        OPENMM_REAL_CSV_REORDERED_EXTRA_COLUMNS,
        &OPENMM_EXPECTED_REAL_ZERO_SERIES_5,
        ConvergencePattern::Converged,
    );
}

#[test]
fn test_openmm_csv_variant_minimal_columns() {
    setup();
    assert_openmm_csv_variant(
        OPENMM_REAL_CSV_MINIMAL_COLUMNS,
        &OPENMM_EXPECTED_REAL_ZERO_SERIES_5,
        ConvergencePattern::Converged,
    );
}

#[test]
fn test_openmm_csv_variant_kcal_units() {
    setup();
    assert_openmm_csv_variant(
        OPENMM_SOURCE_DERIVED_CSV_KCAL_UNITS,
        &OPENMM_EXPECTED_ZERO_SERIES_4,
        ConvergencePattern::Converged,
    );
}

#[test]
fn test_openmm_csv_variant_quoted_header() {
    setup();
    assert_openmm_csv_variant(
        OPENMM_REAL_CSV_DEFAULT_KJ,
        &OPENMM_EXPECTED_REAL_ZERO_SERIES_5,
        ConvergencePattern::Converged,
    );
}

#[test]
fn test_openmm_csv_variant_unquoted_header() {
    setup();
    assert_openmm_csv_variant(
        OPENMM_SOURCE_DERIVED_CSV_UNQUOTED_HEADER,
        &OPENMM_EXPECTED_ZERO_SERIES_4,
        ConvergencePattern::Converged,
    );
}

#[test]
fn test_openmm_csv_variant_empty_trailing_columns() {
    setup();
    assert_openmm_csv_variant(
        OPENMM_SOURCE_DERIVED_CSV_EMPTY_TRAILING_COLUMNS,
        &OPENMM_EXPECTED_ZERO_SERIES_4,
        ConvergencePattern::Converged,
    );
}

#[test]
fn test_openmm_csv_variant_windows_crlf() {
    setup();
    assert_openmm_csv_variant(
        OPENMM_REAL_CSV_WINDOWS_CRLF,
        &OPENMM_EXPECTED_REAL_ZERO_SERIES_5,
        ConvergencePattern::Converged,
    );
}

#[test]
fn test_openmm_csv_variant_bom_prefix() {
    setup();
    assert_openmm_csv_variant(
        OPENMM_REAL_CSV_BOM_PREFIX,
        &OPENMM_EXPECTED_REAL_ZERO_SERIES_5,
        ConvergencePattern::Converged,
    );
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

const GROMACS_MDP_SAMPLE: &str = r#"
; GROMACS mdp file
integrator = md
dt = 0.002 ; ps
nsteps = 500000
coulombtype = PME
rcoulomb = 1.0
ref_t = 300
ref_p = 1.0
tcoupl = V-rescale
tau_t = 0.1
pcoupl = Parrinello-Rahman
nstlog = 1000
nstxout = 1000
"#;

const GROMACS_LOG_SAMPLE: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:

Using 1 GPU

   Step           Time
      0        0.00000

Energies (kJ/mol)
   Bond          Angle    Proper Dih.          LJ-14     Coulomb-14
1234.56       2345.67        345.678       456.789       567.890
   LJ (SR)   Coulomb (SR)   Coul. recip.      Potential    Kinetic En.
-12345.6      -54321.0        1234.56      -45678.9       12345.6
   Total Energy   Pressure (bar)
-33333.3            1.013

Finished mdrun on rank 0
"#;

const GROMACS_LOG_NAN: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 GPU
   Step           Time
      10        0.02000
Energies (kJ/mol)
   Bond          Angle
100.0          NaN
   Total Energy
200.0
Finished mdrun on rank 0
"#;

const GROMACS_LOG_TRUNCATED: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      50        0.10000
Energies (kJ/mol)
   Bond          Angle
10.0           20.0
   Total Energy
30.0
"#;

const GROMACS_LOG_FATAL_ERROR: &str = "\
             :-) GROMACS - gmx mdrun, 2023.3 (-:

Using 1 GPU

   Step           Time
      0        0.00000

Energies (kJ/mol)
   Kinetic En.   Total Energy
      1234.56       -5678.90

Fatal error: Step 100: The total potential energy is -1e+14
";

const GROMACS_LOG_STABLE_SERIES: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 GPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Total Energy
-1000.000
   Step           Time
      100        0.20000
Energies (kJ/mol)
   Total Energy
-1000.050
   Step           Time
      200        0.40000
Energies (kJ/mol)
   Total Energy
-1000.080
   Step           Time
      300        0.60000
Energies (kJ/mol)
   Total Energy
-1000.090
Finished mdrun on rank 0
"#;

const GROMACS_LOG_OSCILLATING_SERIES: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 GPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Total Energy
-1000.000
   Step           Time
      100        0.20000
Energies (kJ/mol)
   Total Energy
-998.000
   Step           Time
      200        0.40000
Energies (kJ/mol)
   Total Energy
-1001.000
   Step           Time
      300        0.60000
Energies (kJ/mol)
   Total Energy
-997.500
   Step           Time
      400        0.80000
Energies (kJ/mol)
   Total Energy
-1002.000
Finished mdrun on rank 0
"#;

const GROMACS_LOG_SHORT_SERIES: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Total Energy
-1000.000
   Step           Time
      100        0.20000
Energies (kJ/mol)
   Total Energy
-1000.050
   Step           Time
      200        0.40000
Energies (kJ/mol)
   Total Energy
-1000.080
Finished mdrun on rank 0
"#;

const GROMACS_LOG_DRIFTING_SERIES: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Total Energy
0.000
   Step           Time
      100        0.20000
Energies (kJ/mol)
   Total Energy
0.010
   Step           Time
      200        0.40000
Energies (kJ/mol)
   Total Energy
0.020
   Step           Time
      300        0.60000
Energies (kJ/mol)
   Total Energy
0.030
Finished mdrun on rank 0
"#;

const GROMACS_LOG_DIVERGENT_NAN_SERIES: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Bond          Angle
1.0            2.0
   Total Energy
-1000.0
   Step           Time
      100        0.20000
Energies (kJ/mol)
   Bond          Angle
1.1            2.1
   Total Energy
-999.9
   Step           Time
      200        0.40000
Energies (kJ/mol)
   Bond          Angle
1.2            NaN
   Total Energy
-999.8
   Step           Time
      300        0.60000
Energies (kJ/mol)
   Bond          Angle
1.3            2.3
   Total Energy
-999.7
Finished mdrun on rank 0
"#;

const GROMACS_LOG_BOUNDARY_SERIES: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Total Energy
0.0000
   Step           Time
      100        0.20000
Energies (kJ/mol)
   Total Energy
0.0001
   Step           Time
      200        0.40000
Energies (kJ/mol)
   Total Energy
0.0002
   Step           Time
      300        0.60000
Energies (kJ/mol)
   Total Energy
0.0003
Finished mdrun on rank 0
"#;

const GROMACS_FILE_NVT_MD_LOG: &str =
    include_str!("../../testdata/gromacs_md_log/gromacs2023_nvt_md.log");
const GROMACS_FILE_NPT_EQUILIBRATION_LOG: &str =
    include_str!("../../testdata/gromacs_md_log/gromacs2023_npt_equilibration.log");
const GROMACS_FILE_ENERGY_MINIMIZATION_LOG: &str =
    include_str!("../../testdata/gromacs_md_log/gromacs2023_energy_minimization.log");

const GROMACS_LOG_COMPACT_BLOCK: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Potential   Total Energy
-9000.0      -10000.0000
   Step           Time
      100        0.20000
Energies (kJ/mol)
   Potential   Total Energy
-9000.3      -10000.5000
   Step           Time
      200        0.40000
Energies (kJ/mol)
   Potential   Total Energy
-9000.5      -10000.8000
   Step           Time
      300        0.60000
Energies (kJ/mol)
   Potential   Total Energy
-9000.6      -10000.9000
Finished mdrun on rank 0
"#;

const GROMACS_LOG_WIDE_BLOCK: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Bond      Angle   Proper Dih.      LJ-14   Coulomb-14      LJ (SR)
100.0      200.0      10.0      20.0      30.0      -400.0
   Coulomb (SR)   Coul. recip.   Potential   Kinetic En.   Total Energy   Pressure (bar)
-500.0      50.0      -800.0      300.0      -5000.0      1.0000
Finished mdrun on rank 0
"#;

const GROMACS_LOG_SCIENTIFIC_NOTATION: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Potential   Total Energy
-1.10000e+05   -1.234560e+05
   Step           Time
      100        0.20000
Energies (kJ/mol)
   Potential   Total Energy
-1.10001e+05   -1.234570e+05
   Step           Time
      200        0.40000
Energies (kJ/mol)
   Potential   Total Energy
-1.10002e+05   -1.234575e+05
   Step           Time
      300        0.60000
Energies (kJ/mol)
   Potential   Total Energy
-1.10003e+05   -1.234578e+05
Finished mdrun on rank 0
"#;

const GROMACS_LOG_TRUNCATED_MID_BLOCK: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Potential   Total Energy
-2000.0      -3000.0
   Step           Time
      100        0.20000
Energies (kJ/mol)
   Potential   Total Energy
"#;

const GROMACS_LOG_TAB_WHITESPACE: &str = "             :-) GROMACS - gmx mdrun, 2023.3 (-:\n\
Using 1 CPU\n\
   Step           Time\n\
      0        0.00000\n\
Energies (kJ/mol)\n\
\tPotential\tTotal Energy\n\
-5000.0\t-10000.0000\n\
   Step           Time\n\
      100        0.20000\n\
Energies (kJ/mol)\n\
\tPotential\tTotal Energy\n\
-5000.1\t-10000.5000\n\
   Step           Time\n\
      200        0.40000\n\
Energies (kJ/mol)\n\
\tPotential\tTotal Energy\n\
-5000.2\t-10000.8000\n\
   Step           Time\n\
      300        0.60000\n\
Energies (kJ/mol)\n\
\tPotential\tTotal Energy\n\
-5000.3\t-10000.9000\n\
Finished mdrun on rank 0\n";

const GROMACS_LOG_DOUBLE_PRECISION: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Potential   Total Energy
-11111.123456789   -12345.123456789
   Step           Time
      100        0.20000
Energies (kJ/mol)
   Potential   Total Energy
-11111.123556789   -12345.123956789
   Step           Time
      200        0.40000
Energies (kJ/mol)
   Potential   Total Energy
-11111.123606789   -12345.124156789
   Step           Time
      300        0.60000
Energies (kJ/mol)
   Potential   Total Energy
-11111.123626789   -12345.124256789
Finished mdrun on rank 0
"#;

const GROMACS_LOG_EM_NO_TOTAL_ENERGY: &str = r#"
             :-) GROMACS - gmx mdrun, 2023.3 (-:
Using 1 CPU
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Potential
-180000.0
   Step           Time
      1        0.00000
Energies (kJ/mol)
   Potential
-180050.0
   Step           Time
      2        0.00000
Energies (kJ/mol)
   Potential
-180080.0
Finished mdrun on rank 0
"#;

#[test]
fn test_classify_theory_params() {
    setup();
    let (layer, boundary, units) = classify_mdp_parameter("coulombtype", "PME");
    assert_eq!(layer, Layer::Theory);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);

    let (layer, boundary, units) = classify_mdp_parameter("rcoulomb", "1.0");
    assert_eq!(layer, Layer::Theory);
    match boundary {
        BoundaryClassification::DualAnnotated { secondary_layer, .. } => {
            assert_eq!(secondary_layer, Layer::Methodology);
        }
        other => panic!("Expected DualAnnotated, got {:?}", other),
    }
    assert_eq!(units, Some("nm"));
}

#[test]
fn test_classify_methodology_params() {
    setup();
    let (layer, boundary, units) = classify_mdp_parameter("integrator", "md");
    assert_eq!(layer, Layer::Methodology);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);

    let (layer, _, units) = classify_mdp_parameter("dt", "0.002");
    assert_eq!(layer, Layer::Methodology);
    assert_eq!(units, Some("ps"));

    let (layer, boundary, units) = classify_mdp_parameter("tcoupl", "V-rescale");
    assert_eq!(layer, Layer::Methodology);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);
}

#[test]
fn test_classify_implementation_params() {
    setup();
    let (layer, boundary, units) = classify_mdp_parameter("nstlog", "1000");
    assert_eq!(layer, Layer::Implementation);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);

    let (layer, boundary, units) = classify_mdp_parameter("nstxout", "1000");
    assert_eq!(layer, Layer::Implementation);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);
}

#[test]
fn test_classify_dual_annotated() {
    setup();
    let (_, boundary, _) = classify_mdp_parameter("dt", "0.002");
    match boundary {
        BoundaryClassification::DualAnnotated {
            secondary_layer,
            rationale,
        } => {
            assert_eq!(secondary_layer, Layer::Implementation);
            assert!(rationale.contains("Timestep affects both sampling methodology"));
        }
        other => panic!("Expected DualAnnotated for dt, got {:?}", other),
    }

    let (_, boundary, _) = classify_mdp_parameter("rcoulomb", "1.0");
    match boundary {
        BoundaryClassification::DualAnnotated {
            secondary_layer,
            rationale,
        } => {
            assert_eq!(secondary_layer, Layer::Methodology);
            assert!(rationale.contains("Cutoff radius affects both force field accuracy"));
        }
        other => panic!("Expected DualAnnotated for rcoulomb, got {:?}", other),
    }
}

#[test]
fn test_classify_unknown_param() {
    setup();
    let (layer, boundary, units) = classify_mdp_parameter("mystery_param", "42");
    assert_eq!(layer, Layer::Implementation);
    assert_eq!(units, None);
    match boundary {
        BoundaryClassification::ContextDependent {
            default_layer,
            context_note,
        } => {
            assert_eq!(default_layer, Layer::Implementation);
            assert!(context_note.contains("classification table"));
        }
        other => panic!("Expected ContextDependent, got {:?}", other),
    }
}

#[test]
fn test_parse_mdp_basic() {
    setup();
    let mdp = r#"
integrator = md
dt = 0.002
nsteps = 500000
coulombtype = PME
ref_t = 300
"#;
    let events = parse_mdp(mdp).unwrap();
    assert_eq!(events.len(), 5);

    let names: Vec<String> = events
        .iter()
        .map(|event| match &event.kind {
            EventKind::ParameterRecord { name, .. } => name.clone(),
            other => panic!("Expected ParameterRecord, got {:?}", other),
        })
        .collect();
    assert_eq!(
        names,
        vec!["integrator", "dt", "nsteps", "coulombtype", "ref_t"]
    );

    match &events[1].kind {
        EventKind::ParameterRecord { actual_value, .. } => {
            assert_eq!(actual_value, &Value::Known(0.002, "ps".to_string()));
        }
        other => panic!("Expected ParameterRecord for dt, got {:?}", other),
    }
}

#[test]
fn test_parse_mdp_comments_stripped() {
    setup();
    let mdp = "dt = 0.002 ; ps\nintegrator = md ; leapfrog\n";
    let events = parse_mdp(mdp).unwrap();
    assert_eq!(events.len(), 2);

    match &events[0].kind {
        EventKind::ParameterRecord { actual_value, .. } => {
            assert_eq!(actual_value, &Value::Known(0.002, "ps".to_string()));
        }
        other => panic!("Expected ParameterRecord for dt, got {:?}", other),
    }

    match &events[1].kind {
        EventKind::ParameterRecord { actual_value, .. } => {
            assert_eq!(actual_value, &Value::KnownCat("md".to_string()));
        }
        other => panic!("Expected ParameterRecord for integrator, got {:?}", other),
    }
}

#[test]
fn test_parse_mdp_empty() {
    setup();
    let events = parse_mdp("").unwrap();
    assert!(events.is_empty());
}

#[test]
fn test_parse_mdp_layer_distribution() {
    setup();
    let events = parse_mdp(GROMACS_MDP_SAMPLE).unwrap();
    assert!(!events.is_empty());

    let layers: std::collections::HashSet<Layer> =
        events.iter().map(|event| event.layer).collect();
    assert!(layers.contains(&Layer::Theory));
    assert!(layers.contains(&Layer::Methodology));
    assert!(layers.contains(&Layer::Implementation));
}

#[test]
fn test_parse_mdp_provenance_lines() {
    setup();
    let mdp = "; comment\nintegrator = md\n\n dt = 0.002 ; ps\n; another comment\nnstlog = 1000\n";
    let events = parse_mdp(mdp).unwrap();
    assert_eq!(events.len(), 3);

    let line_starts: Vec<u32> = events
        .iter()
        .map(|event| match &event.provenance.source_location {
            SourceLocation::LineRange { start, end } => {
                assert_eq!(start, end);
                *start
            }
            other => panic!("Expected LineRange, got {:?}", other),
        })
        .collect();

    assert_eq!(line_starts, vec![2, 4, 6]);
}

#[test]
fn test_parse_energy_block() {
    setup();
    let log = r#"
   Step           Time
      0        0.00000
Energies (kJ/mol)
   Bond          Angle
1.0           2.0
   Total Energy
3.0
Finished mdrun on rank 0
"#;
    let events = parse_log(log, 0).unwrap();
    let energy = events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::EnergyRecord { total, components } => Some((total, components)),
            _ => None,
        })
        .expect("Expected at least one EnergyRecord");

    assert_eq!(energy.0, &Value::Known(3.0, "kJ/mol".to_string()));
    assert!(energy.1.iter().any(|(name, _)| name == "Bond"));
    assert!(energy.1.iter().any(|(name, _)| name == "Angle"));
}

#[test]
fn test_parse_log_header() {
    setup();
    let events = parse_log(GROMACS_LOG_SAMPLE, 0).unwrap();
    let resource = events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ResourceStatus {
                platform_type,
                device_ids,
                ..
            } => Some((platform_type, device_ids)),
            _ => None,
        })
        .expect("Expected ResourceStatus event");

    assert_eq!(resource.0, "GPU");
    assert!(resource.1.iter().any(|entry| entry.contains("GROMACS")));
}

#[test]
fn test_parse_log_energy_record() {
    setup();
    let events = parse_log(GROMACS_LOG_SAMPLE, 0).unwrap();
    let energy = events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::EnergyRecord { total, components } => Some((total, components)),
            _ => None,
        })
        .expect("Expected EnergyRecord event");

    match energy.0 {
        Value::Known(total, unit) => {
            assert!((*total + 33333.3).abs() < 1e-6);
            assert_eq!(unit, "kJ/mol");
        }
        other => panic!("Expected scalar energy total, got {:?}", other),
    }
    assert!(energy.1.iter().any(|(name, _)| name == "Bond"));
    assert!(energy.1.iter().any(|(name, _)| name == "Kinetic_En."));
}

#[test]
fn test_parse_log_nan_detection() {
    setup();
    let events = parse_log(GROMACS_LOG_NAN, 0).unwrap();
    assert!(events.iter().any(|event| matches!(
        event.kind,
        EventKind::EnergyRecord { .. }
    )));
    assert!(events.iter().any(|event| matches!(
        event.kind,
        EventKind::NumericalStatus {
            event_type: NumericalEventType::NaNDetected,
            ..
        }
    )));
}

#[test]
fn test_parse_log_success() {
    setup();
    let events = parse_log(GROMACS_LOG_SAMPLE, 0).unwrap();
    assert!(events.iter().any(|event| matches!(
        event.kind,
        EventKind::ExecutionStatus {
            status: ExecutionOutcome::Success,
            ..
        }
    )));
}

#[test]
fn test_parse_log_fatal_error() {
    setup();
    let events = crate::gromacs_adapter::parse_log(GROMACS_LOG_FATAL_ERROR, 0).unwrap();
    let last = events.last().expect("Expected at least one event");
    match &last.kind {
        EventKind::ExecutionStatus {
            status: ExecutionOutcome::CrashDivergent,
            ..
        } => {}
        other => panic!("Expected CrashDivergent, got {:?}", other),
    }
}

#[test]
fn test_parse_log_truncated() {
    setup();
    let events = parse_log(GROMACS_LOG_TRUNCATED, 0).unwrap();
    let timeout_event = events
        .iter()
        .find(|event| {
            matches!(
                event.kind,
                EventKind::ExecutionStatus {
                    status: ExecutionOutcome::Timeout,
                    ..
                }
            )
        })
        .expect("Expected timeout completion event");

    match &timeout_event.confidence.completeness {
        Completeness::PartiallyInferred { inference_method } => {
            assert_eq!(inference_method, "no completion marker in log");
        }
        other => panic!("Expected PartiallyInferred completeness, got {:?}", other),
    }
    assert!((timeout_event.confidence.field_coverage - 0.5).abs() < f32::EPSILON);
}

/// Tier 2 (source-derived): no local Tier 1 md.log files were available; fixture follows
/// GROMACS 2023.x documented block shape and parser normalization table conventions.
#[test]
fn test_gromacs_log_variant_standard_nvt_md() {
    setup();
    let expected_pairs = [
        (0, -10000.0000),
        (100, -10000.5000),
        (200, -10000.8000),
        (300, -10000.9000),
    ];
    assert_gromacs_log_variant(
        GROMACS_FILE_NVT_MD_LOG,
        &expected_pairs,
        ConvergencePattern::Converged,
    );

    let components = parse_log(GROMACS_FILE_NVT_MD_LOG, 0)
        .unwrap()
        .into_iter()
        .find_map(|event| match event.kind {
            EventKind::EnergyRecord { components, .. } => Some(components),
            _ => None,
        })
        .expect("Expected at least one EnergyRecord in NVT fixture");
    assert_eq!(components.len(), 11);
    for expected_name in [
        "Bond",
        "Angle",
        "Proper_Dih.",
        "LJ-14",
        "Coulomb-14",
        "LJ_(SR)",
        "Coulomb_(SR)",
        "Coul._recip.",
        "Potential",
        "Kinetic_En.",
        "Pressure_(bar)",
    ] {
        assert!(
            components.iter().any(|(name, _)| name == expected_name),
            "Missing NVT component header: {}",
            expected_name
        );
    }
}

/// Tier 2 (source-derived): no local Tier 1 md.log files were available; NPT terms are
/// hyphenated single-token headers consistent with GROMACS 2023.x log formatting.
#[test]
fn test_gromacs_log_variant_npt_equilibration() {
    setup();
    let expected_pairs = [
        (0, -9000.0000),
        (100, -9000.4500),
        (200, -9000.7000),
        (300, -9000.8500),
    ];
    assert_gromacs_log_variant(
        GROMACS_FILE_NPT_EQUILIBRATION_LOG,
        &expected_pairs,
        ConvergencePattern::Converged,
    );

    let events = parse_log(GROMACS_FILE_NPT_EQUILIBRATION_LOG, 0).unwrap();
    let components = events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::EnergyRecord { components, .. } => Some(components),
            _ => None,
        })
        .expect("Expected at least one EnergyRecord in NPT fixture");

    for expected_name in [
        "Volume", "Density", "Pres-XX", "Pres-YY", "Pres-ZZ", "Pres-XY", "Pres-XZ", "Pres-YZ",
        "Box-XX", "Box-YY", "Box-ZZ",
    ] {
        assert!(
            components.iter().any(|(name, _)| name == expected_name),
            "Missing NPT component header: {}",
            expected_name
        );
    }
}

/// Tier 2 (source-derived): no local Tier 1 md.log files were available; fixture models
/// steep minimization blocks where Potential is present and Total Energy is absent.
#[test]
fn test_gromacs_log_variant_energy_minimization() {
    setup();
    assert_gromacs_log_parses_energy_count(GROMACS_FILE_ENERGY_MINIMIZATION_LOG, 0);
    assert_gromacs_log_variant(
        GROMACS_FILE_ENERGY_MINIMIZATION_LOG,
        &[],
        ConvergencePattern::InsufficientData,
    );

    let warning_count = parse_log(GROMACS_FILE_ENERGY_MINIMIZATION_LOG, 0)
        .unwrap()
        .into_iter()
        .filter(|event| {
            matches!(
                event.kind,
                EventKind::NumericalStatus {
                    event_type: NumericalEventType::ConvergenceFailure,
                    severity: Severity::Warning,
                    ..
                }
            )
        })
        .count();
    assert!(warning_count >= 1);
}

/// Tier 2 (source-derived): compact two-column energy block variant synthesized from
/// canonical GROMACS 2023.x energy block structure.
#[test]
fn test_gromacs_log_variant_compact_block() {
    setup();
    let expected_pairs = [
        (0, -10000.0000),
        (100, -10000.5000),
        (200, -10000.8000),
        (300, -10000.9000),
    ];
    assert_gromacs_log_variant(
        GROMACS_LOG_COMPACT_BLOCK,
        &expected_pairs,
        ConvergencePattern::Converged,
    );
}

/// Tier 2 (source-derived): wide multi-row energy block synthesized from canonical
/// GROMACS 2023.x block formatting to stress header/value alignment.
#[test]
fn test_gromacs_log_variant_wide_block() {
    setup();
    assert_gromacs_log_parses_energy_count(GROMACS_LOG_WIDE_BLOCK, 1);
    assert_gromacs_log_variant(
        GROMACS_LOG_WIDE_BLOCK,
        &[(0, -5000.0)],
        ConvergencePattern::InsufficientData,
    );

    let components = parse_log(GROMACS_LOG_WIDE_BLOCK, 0)
        .unwrap()
        .into_iter()
        .find_map(|event| match event.kind {
            EventKind::EnergyRecord { components, .. } => Some(components),
            _ => None,
        })
        .expect("Expected wide-block EnergyRecord");
    assert_eq!(components.len(), 11);
    for expected_name in [
        "Bond",
        "Angle",
        "Proper_Dih.",
        "LJ-14",
        "Coulomb-14",
        "LJ_(SR)",
        "Coulomb_(SR)",
        "Coul._recip.",
        "Potential",
        "Kinetic_En.",
        "Pressure_(bar)",
    ] {
        assert!(
            components.iter().any(|(name, _)| name == expected_name),
            "Missing component {} in wide block",
            expected_name
        );
    }
}

/// Tier 2 (source-derived): scientific notation values synthesized to match GROMACS
/// numeric formatting and verify f64 parsing behavior.
#[test]
fn test_gromacs_log_variant_scientific_notation() {
    setup();
    let expected_pairs = [
        (0, -1.234560e+05),
        (100, -1.234570e+05),
        (200, -1.234575e+05),
        (300, -1.234578e+05),
    ];
    assert_gromacs_log_variant(
        GROMACS_LOG_SCIENTIFIC_NOTATION,
        &expected_pairs,
        ConvergencePattern::Converged,
    );
}

/// Tier 2 (source-derived): truncated block fixture synthesized to verify parser recovery
/// when EOF occurs after headers but before value rows.
#[test]
fn test_gromacs_log_variant_truncated_mid_block() {
    setup();
    assert_gromacs_log_parses_energy_count(GROMACS_LOG_TRUNCATED_MID_BLOCK, 1);
    assert_gromacs_log_variant(
        GROMACS_LOG_TRUNCATED_MID_BLOCK,
        &[(0, -3000.0)],
        ConvergencePattern::InsufficientData,
    );
}

/// Tier 2 (source-derived): tab-delimited header spacing synthesized from canonical
/// GROMACS headers to verify non-space whitespace tokenization.
#[test]
fn test_gromacs_log_variant_tab_whitespace() {
    setup();
    let expected_pairs = [
        (0, -10000.0000),
        (100, -10000.5000),
        (200, -10000.8000),
        (300, -10000.9000),
    ];
    assert_gromacs_log_variant(
        GROMACS_LOG_TAB_WHITESPACE,
        &expected_pairs,
        ConvergencePattern::Converged,
    );
}

/// Tier 2 (source-derived): double-precision decimal fixture synthesized from canonical
/// GROMACS block formatting to verify 9+ decimal-place parsing.
#[test]
fn test_gromacs_log_variant_double_precision() {
    setup();
    let expected_pairs = [
        (0, -12345.123456789),
        (100, -12345.123956789),
        (200, -12345.124156789),
        (300, -12345.124256789),
    ];
    assert_gromacs_log_variant(
        GROMACS_LOG_DOUBLE_PRECISION,
        &expected_pairs,
        ConvergencePattern::Converged,
    );
}

/// Tier 2 (source-derived): EM-style no-total-energy fixture synthesized from canonical
/// energy block format; documents current prototype limitation for EM convergence.
#[test]
fn test_gromacs_log_variant_em_no_total_energy() {
    setup();
    assert_gromacs_log_parses_energy_count(GROMACS_LOG_EM_NO_TOTAL_ENERGY, 0);
    assert_gromacs_log_variant(
        GROMACS_LOG_EM_NO_TOTAL_ENERGY,
        &[],
        ConvergencePattern::InsufficientData,
    );

    let warning_count = parse_log(GROMACS_LOG_EM_NO_TOTAL_ENERGY, 0)
        .unwrap()
        .into_iter()
        .filter(|event| {
            matches!(
                event.kind,
                EventKind::NumericalStatus {
                    event_type: NumericalEventType::ConvergenceFailure,
                    severity: Severity::Warning,
                    ..
                }
            )
        })
        .count();
    assert!(warning_count >= 1);
}

#[test]
fn test_gromacs_adapter_combined() {
    setup();
    let adapter = GromacsAdapter;
    let raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_SAMPLE
    );

    let log = adapter.parse_trace(&raw).unwrap();
    assert!(!log.events.is_empty());
    assert!(
        log.events
            .iter()
            .any(|event| matches!(event.kind, EventKind::ParameterRecord { .. }))
    );
    assert!(
        log.events
            .iter()
            .any(|event| matches!(event.kind, EventKind::EnergyRecord { .. }))
    );
}

#[test]
fn test_gromacs_adapter_derives_convergence_summary_for_stable_series() {
    setup();
    let adapter = GromacsAdapter;
    let raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_STABLE_SERIES
    );
    let log = adapter.parse_trace(&raw).unwrap();

    let convergence = log
        .events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ConvergencePoint {
                metric_name,
                converged,
                ..
            } => Some((metric_name, converged)),
            _ => None,
        })
        .expect("Expected derived ConvergencePoint event");

    assert_eq!(convergence.0, "derived_convergence_rel_delta_max");
    assert_eq!(convergence.1, &Some(true));
}

#[test]
fn test_gromacs_adapter_derives_oscillation_summary_for_non_converging_series() {
    setup();
    let adapter = GromacsAdapter;
    let raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_OSCILLATING_SERIES
    );
    let log = adapter.parse_trace(&raw).unwrap();

    let convergence = log
        .events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ConvergencePoint {
                metric_name,
                converged,
                ..
            } => Some((metric_name, converged)),
            _ => None,
        })
        .expect("Expected derived ConvergencePoint event");

    assert_eq!(convergence.0, "derived_oscillation_rel_delta_mean");
    assert_eq!(convergence.1, &Some(false));
}

#[test]
fn test_gromacs_adapter_no_convergence_summary_below_min_window() {
    setup();
    let adapter = GromacsAdapter;
    let raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_SHORT_SERIES
    );
    let log = adapter.parse_trace(&raw).unwrap();

    assert!(!log
        .events
        .iter()
        .any(|event| matches!(event.kind, EventKind::ConvergencePoint { .. })));
}

#[test]
fn test_gromacs_adapter_convergence_summary_provenance_refs() {
    setup();
    let adapter = GromacsAdapter;
    let raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_STABLE_SERIES
    );
    let log = adapter.parse_trace(&raw).unwrap();

    let convergence_event = log
        .events
        .iter()
        .find(|event| matches!(event.kind, EventKind::ConvergencePoint { .. }))
        .expect("Expected derived ConvergencePoint event");
    let energy_ids: Vec<EventId> = log
        .events
        .iter()
        .filter(|event| matches!(event.kind, EventKind::EnergyRecord { .. }))
        .map(|event| event.id)
        .collect();
    let execution_id = log
        .events
        .iter()
        .find(|event| matches!(event.kind, EventKind::ExecutionStatus { .. }))
        .map(|event| event.id)
        .expect("Expected ExecutionStatus event");

    assert_eq!(energy_ids.len(), 4);
    for expected_id in &energy_ids {
        assert!(
            convergence_event.causal_refs.contains(expected_id),
            "ConvergencePoint missing source EnergyRecord {:?}",
            expected_id
        );
    }
    assert!(
        convergence_event.causal_refs.contains(&execution_id),
        "ConvergencePoint missing source ExecutionStatus {:?}",
        execution_id
    );
}

#[test]
fn test_gromacs_adapter_mdp_only() {
    setup();
    let adapter = GromacsAdapter;
    let raw = format!("--- MDP ---\n{}", GROMACS_MDP_SAMPLE);
    let log = adapter.parse_trace(&raw).unwrap();

    assert!(!log.events.is_empty());
    assert!(
        log.events
            .iter()
            .all(|event| matches!(event.kind, EventKind::ParameterRecord { .. }))
    );
}

#[test]
fn test_gromacs_adapter_controlled_vars() {
    setup();
    let adapter = GromacsAdapter;
    let raw = format!("--- MDP ---\n{}", GROMACS_MDP_SAMPLE);
    let log = adapter.parse_trace(&raw).unwrap();

    assert_eq!(log.spec.controlled_variables.len(), 2);
    let mut parameters: Vec<String> = log
        .spec
        .controlled_variables
        .iter()
        .map(|var| var.parameter.clone())
        .collect();
    parameters.sort();
    assert_eq!(parameters, vec!["pressure".to_string(), "temperature".to_string()]);
}

#[test]
fn test_gromacs_overlay_construction() {
    setup();
    let adapter = GromacsAdapter;
    let raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_SAMPLE
    );
    let log = adapter.parse_trace(&raw).unwrap();
    let overlay = CausalOverlay::from_log(&log);

    assert_eq!(overlay.len(), log.events.len());
}

#[test]
fn test_gromacs_overlay_layer_span() {
    setup();
    let adapter = GromacsAdapter;
    let raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_SAMPLE
    );
    let log = adapter.parse_trace(&raw).unwrap();

    assert!(log.indexes.by_layer.contains_key(&Layer::Theory));
    assert!(log.indexes.by_layer.contains_key(&Layer::Methodology));
    assert!(log.indexes.by_layer.contains_key(&Layer::Implementation));
}

#[test]
fn test_gromacs_e2e_confounder_detection() {
    setup();
    let adapter = GromacsAdapter;
    let raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_SAMPLE
    );
    let mut log = adapter.parse_trace(&raw).unwrap();

    let mut tau_t_id: Option<EventId> = None;
    let mut dt_idx: Option<usize> = None;
    let mut ref_t_idx: Option<usize> = None;

    for (idx, event) in log.events.iter().enumerate() {
        if let EventKind::ParameterRecord { name, .. } = &event.kind {
            if name == "tau_t" {
                tau_t_id = Some(event.id);
            } else if name == "dt" {
                dt_idx = Some(idx);
            } else if name == "ref_t" {
                ref_t_idx = Some(idx);
            }
        }
    }

    let tau_t_id = tau_t_id.expect("tau_t parameter event must exist");
    let dt_idx = dt_idx.expect("dt parameter event must exist");
    let ref_t_idx = ref_t_idx.expect("ref_t parameter event must exist");

    log.events[dt_idx].causal_refs = vec![tau_t_id];
    log.events[ref_t_idx].causal_refs = vec![tau_t_id];

    let overlay = CausalOverlay::from_log(&log);
    let candidates = overlay.detect_confounders(&log, "ref_t", "dt");
    assert!(!candidates.is_empty());
    assert!(candidates.iter().any(|candidate| candidate.dag_node == "tau_t"));
}

const VASP_INCAR_SAMPLE: &str = r#"
GGA = PE
ENCUT = 520 ! cutoff energy
PREC = Accurate
SIGMA = 0.05 # smearing width
ISMEAR = 0
IBRION = 2
NSW = 50
ISIF = 3
EDIFF = 1E-6
EDIFFG = -0.01
NCORE = 4
KPAR = 2
ALGO = Fast
"#;

const VASP_OSZICAR_SAMPLE: &str = r#"
DAV:   1    0.400E+03    0.400E+03   -0.500E+00   200   0.200E+02
DAV:   2   -0.200E+02   -0.100E+01   -0.200E+00   220   0.120E+02
DAV:   3   -0.100E+01   -0.500E+00   -0.100E-01   240   0.800E+01
   1 F= -.11401725E+03 E0= -.11400000E+03  d E =-.11401725E+03
RMM:   1   -0.300E+01   -0.300E+00   -0.300E-01   260   0.600E+01
RMM:   2   -0.200E+00   -0.100E+00   -0.100E-02   280   0.400E+01
DAV:   3   -0.100E+00   -0.500E-01   -0.500E-03   300   0.200E+01
   2 F= -.11411725E+03 E0= -.11410000E+03  dE = -.10000000E+00
"#;

const VASP_OUTCAR_SAMPLE: &str = r#"
vasp.6.4.2 18Apr23 complex
running on    16 total cores
free  energy   TOTEN  =      -114.50000000 eV
POSITION                                       TOTAL-FORCE (eV/Angst)
General timing and accounting
"#;

const VASP_OUTCAR_TRUNCATED: &str = r#"
vasp.6.4.2 18Apr23 complex
free  energy   TOTEN  =      -113.75000000 eV
"#;

const VASP_OUTCAR_ERROR: &str = r#"
vasp.6.4.2 18Apr23 complex
VERY BAD NEWS
"#;

const VASP_COMBINED_SAMPLE: &str = r#"--- INCAR ---
GGA = PE
ENCUT = 520 ! cutoff energy
PREC = Accurate
SIGMA = 0.05 # smearing width
ISMEAR = 0
IBRION = 2
NSW = 50
ISIF = 3
EDIFF = 1E-6
EDIFFG = -0.01
NCORE = 4
KPAR = 2
ALGO = Fast
--- OSZICAR ---
DAV:   1    0.400E+03    0.400E+03   -0.500E+00   200   0.200E+02
DAV:   2   -0.200E+02   -0.100E+01   -0.200E+00   220   0.120E+02
DAV:   3   -0.100E+01   -0.500E+00   -0.100E-01   240   0.800E+01
   1 F= -.11401725E+03 E0= -.11400000E+03  d E =-.11401725E+03
RMM:   1   -0.300E+01   -0.300E+00   -0.300E-01   260   0.600E+01
RMM:   2   -0.200E+00   -0.100E+00   -0.100E-02   280   0.400E+01
DAV:   3   -0.100E+00   -0.500E-01   -0.500E-03   300   0.200E+01
   2 F= -.11411725E+03 E0= -.11410000E+03  dE = -.10000000E+00
--- OUTCAR ---
vasp.6.4.2 18Apr23 complex
running on    16 total cores
free  energy   TOTEN  =      -114.50000000 eV
POSITION                                       TOTAL-FORCE (eV/Angst)
General timing and accounting
"#;

const VASP_OSZICAR_NO_F_SAMPLE: &str = r#"
DAV:   1    0.400E+03    0.400E+03   -0.500E+00   200   0.200E+02
DAV:   2   -0.200E+02   -0.100E+01   -0.200E+00   220   0.120E+02
DAV:   3   -0.100E+01   -0.500E+00   -0.100E-01   240   0.800E+01
"#;

const VASP_COMBINED_DIVERGENT_SAMPLE: &str = r#"--- INCAR ---
GGA = PE
ENCUT = 520 ! cutoff energy
PREC = Accurate
SIGMA = 0.05 # smearing width
ISMEAR = 0
IBRION = 2
NSW = 50
ISIF = 3
EDIFF = 1E-6
EDIFFG = -0.01
NCORE = 4
KPAR = 2
ALGO = Fast
--- OSZICAR ---
DAV:   1    0.400E+03    0.400E+03   -0.500E+00   200   0.200E+02
DAV:   2   -0.200E+02   -0.100E+01   -0.200E+00   220   0.120E+02
DAV:   3   -0.100E+01   -0.500E+00   -0.100E-01   240   0.800E+01
   1 F= -.11401725E+03 E0= -.11400000E+03  d E =-.11401725E+03
--- OUTCAR ---
vasp.6.4.2 18Apr23 complex
VERY BAD NEWS
"#;

const VASP_FILE_CONVERGED_RELAXATION: &str =
    include_str!("../../testdata/vasp/converged_relaxation.vasp");
const VASP_FILE_NONCONVERGED_SCF: &str =
    include_str!("../../testdata/vasp/nonconverged_scf.vasp");
const VASP_FILE_OSCILLATING_SCF: &str =
    include_str!("../../testdata/vasp/oscillating_scf.vasp");
const VASP_FILE_MIXED_SCF_DAV_RMM: &str =
    include_str!("../../testdata/vasp/mixed_scf_dav_rmm.vasp");
const VASP_FILE_T1_HONEYCOMB_PT52: &str = include_str!("../../testdata/vasp/t1_honeycomb_pt52.vasp");
const VASP_FILE_T1_LARGE_APPROX: &str = include_str!("../../testdata/vasp/t1_large_approx.vasp");
const VASP_FILE_T1_SIGMA_PT56_SUBSTRATE: &str =
    include_str!("../../testdata/vasp/t1_sigma_pt56_substrate.vasp");

const VASP_VARIANT_ERROR_EDDDAV: &str = r#"--- INCAR ---
GGA = PE
ENCUT = 520
PREC = Accurate
ISMEAR = 0
IBRION = 2
NSW = 2
EDIFF = 1E-6
ALGO = Fast
--- OSZICAR ---
DAV:   1    0.100E+02    0.100E+02   -0.500E+00   200   0.200E+02
   1 F= -.10000000E+03 E0= -.99900000E+02  d E =-.10000000E-01
--- OUTCAR ---
vasp.6.4.2 18Apr23 complex
EDDDAV: Call to ZHEGV failed
"#;

const VASP_VARIANT_LARGE_ENCUT_SCINOTATION: &str = r#"--- INCAR ---
GGA = PE
ENCUT = 1.200E+03
PREC = Accurate
ISMEAR = 0
IBRION = 2
NSW = 1
EDIFF = 1E-6
ALGO = Fast
--- OSZICAR ---
DAV:   1    0.200E+02    0.200E+02   -0.100E+00   220   0.120E+02
   1 F= -.20000000E+03 E0= -.19990000E+03  dE = -.10000000E-01
--- OUTCAR ---
vasp.6.4.2 18Apr23 complex
running on     4 total cores
free  energy   TOTEN  =     -200.00000000 eV
General timing and accounting
"#;

const VASP_VARIANT_STATIC_CALC: &str = r#"--- INCAR ---
GGA = PE
ENCUT = 520
PREC = Accurate
ISMEAR = 0
IBRION = -1
NSW = 0
EDIFF = 1E-6
ALGO = Normal
--- OSZICAR ---
DAV:   1    0.150E+02    0.150E+02   -0.700E-01   200   0.900E+01
   1 F= -.50000000E+02 E0= -.49950000E+02  d E =-.50000000E-02
--- OUTCAR ---
vasp.6.4.2 18Apr23 complex
free  energy   TOTEN  =      -50.00000000 eV
General timing and accounting
"#;

const VASP_VARIANT_V5_VERSION: &str = r#"--- INCAR ---
GGA = PE
ENCUT = 520
PREC = Accurate
ISMEAR = 0
IBRION = 2
NSW = 1
EDIFF = 1E-6
ALGO = Fast
--- OSZICAR ---
DAV:   1    0.100E+02    0.100E+02   -0.500E-01   200   0.800E+01
   1 F= -.75000000E+02 E0= -.74990000E+02  dE = -.10000000E-02
--- OUTCAR ---
vasp.5.4.4.18Apr17-6-g9f103f2a35
running on     2 total cores
free  energy   TOTEN  =      -75.00000000 eV
General timing and accounting
"#;

fn test_vasp_rebuild_log(log: &LayeredEventLog) -> LayeredEventLog {
    let mut builder = LayeredEventLogBuilder::new(log.experiment_ref.clone(), log.spec.clone());
    for event in log.events.clone() {
        builder = builder.add_event(event);
    }
    builder.build()
}

fn parse_vasp_oszicar_energy_pairs(oszicar: &str) -> Vec<(u64, f64)> {
    parse_oszicar(oszicar, 0)
        .unwrap()
        .into_iter()
        .filter_map(|event| match event.kind {
            EventKind::EnergyRecord {
                total: Value::Known(total, _),
                ..
            } => Some((event.temporal.simulation_step, total)),
            _ => None,
        })
        .collect()
}

fn parse_vasp_oszicar_convergence_pairs(oszicar: &str) -> Vec<(u64, f64)> {
    parse_oszicar(oszicar, 0)
        .unwrap()
        .into_iter()
        .filter_map(|event| match event.kind {
            EventKind::ConvergencePoint {
                iteration,
                metric_value: Value::Known(metric_value, _),
                ..
            } => Some((iteration, metric_value)),
            _ => None,
        })
        .collect()
}

fn assert_vasp_variant(
    combined: &str,
    expected_energies: &[(u64, f64)],
    expected_patterns: &[ConvergencePattern],
) {
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(combined).unwrap();

    let oszicar_pairs: Vec<(u64, f64)> = log
        .events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::EnergyRecord {
                total: Value::Known(total, _),
                ..
            } if event.provenance.source_file == "OSZICAR" => {
                Some((event.temporal.simulation_step, *total))
            }
            _ => None,
        })
        .collect();

    assert_eq!(oszicar_pairs.len(), expected_energies.len());
    for ((actual_step, actual_energy), (expected_step, expected_energy)) in
        oszicar_pairs.iter().zip(expected_energies.iter())
    {
        assert_eq!(*actual_step, *expected_step);
        assert!((*actual_energy - *expected_energy).abs() < 1e-6);
    }

    let actual_patterns: Vec<ConvergencePattern> = classify_all_convergence(&log, "vasp")
        .into_iter()
        .map(|canonical| canonical.pattern)
        .collect();
    for expected_pattern in expected_patterns {
        assert!(actual_patterns.contains(expected_pattern));
    }
}

fn assert_vasp_parses_energy_count(combined: &str, count: usize) {
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(combined).unwrap();
    let energy_count = log
        .events
        .iter()
        .filter(|event| matches!(event.kind, EventKind::EnergyRecord { .. }))
        .count();
    assert_eq!(energy_count, count);
}

fn assert_vasp_execution_status(combined: &str, expected: ExecutionOutcome) {
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(combined).unwrap();
    let status = log
        .events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ExecutionStatus { status, .. } => Some(status),
            _ => None,
        })
        .expect("Expected ExecutionStatus event");
    assert_eq!(status, &expected);
}

#[test]
fn test_vasp_classify_theory_params() {
    setup();
    let (layer, boundary, units) = classify_incar_parameter("GGA", "PE");
    assert_eq!(layer, Layer::Theory);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);

    let (layer, boundary, units) = classify_incar_parameter("ISMEAR", "0");
    assert_eq!(layer, Layer::Theory);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);
}

#[test]
fn test_vasp_classify_methodology_params() {
    setup();
    let (layer, boundary, units) = classify_incar_parameter("IBRION", "2");
    assert_eq!(layer, Layer::Methodology);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);

    let (layer, boundary, units) = classify_incar_parameter("EDIFF", "1E-6");
    assert_eq!(layer, Layer::Methodology);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, Some("eV"));

    let (layer, boundary, units) = classify_incar_parameter("NSW", "50");
    assert_eq!(layer, Layer::Methodology);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);
}

#[test]
fn test_vasp_classify_implementation_params() {
    setup();
    let (layer, boundary, units) = classify_incar_parameter("NCORE", "4");
    assert_eq!(layer, Layer::Implementation);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);

    let (layer, boundary, units) = classify_incar_parameter("KPAR", "2");
    assert_eq!(layer, Layer::Implementation);
    assert_eq!(boundary, BoundaryClassification::PrimaryLayer);
    assert_eq!(units, None);
}

#[test]
fn test_vasp_classify_dual_annotated() {
    setup();

    let (_, boundary, _) = classify_incar_parameter("ENCUT", "520");
    match boundary {
        BoundaryClassification::DualAnnotated {
            secondary_layer,
            rationale,
        } => {
            assert_eq!(secondary_layer, Layer::Implementation);
            assert_eq!(
                rationale,
                "cutoff determines both physics accuracy and memory/compute cost"
            );
        }
        other => panic!("Expected DualAnnotated for ENCUT, got {:?}", other),
    }

    let (_, boundary, _) = classify_incar_parameter("ALGO", "Fast");
    match boundary {
        BoundaryClassification::DualAnnotated {
            secondary_layer,
            rationale,
        } => {
            assert_eq!(secondary_layer, Layer::Methodology);
            assert_eq!(
                rationale,
                "algorithm can affect which SCF minimum is found"
            );
        }
        other => panic!("Expected DualAnnotated for ALGO, got {:?}", other),
    }

    let (_, boundary, _) = classify_incar_parameter("LREAL", "Auto");
    match boundary {
        BoundaryClassification::DualAnnotated {
            secondary_layer,
            rationale,
        } => {
            assert_eq!(secondary_layer, Layer::Theory);
            assert_eq!(rationale, "real-space projection trades accuracy for speed");
        }
        other => panic!("Expected DualAnnotated for LREAL, got {:?}", other),
    }

    let (_, boundary, _) = classify_incar_parameter("SIGMA", "0.05");
    match boundary {
        BoundaryClassification::DualAnnotated {
            secondary_layer,
            rationale,
        } => {
            assert_eq!(secondary_layer, Layer::Methodology);
            assert_eq!(
                rationale,
                "smearing width affects both electronic structure accuracy and BZ integration convergence"
            );
        }
        other => panic!("Expected DualAnnotated for SIGMA, got {:?}", other),
    }

    let (_, boundary, _) = classify_incar_parameter("PREC", "Accurate");
    match boundary {
        BoundaryClassification::DualAnnotated {
            secondary_layer,
            rationale,
        } => {
            assert_eq!(secondary_layer, Layer::Implementation);
            assert_eq!(
                rationale,
                "precision affects both physical accuracy and FFT grid resources"
            );
        }
        other => panic!("Expected DualAnnotated for PREC, got {:?}", other),
    }
}

#[test]
fn test_vasp_classify_unknown_param() {
    setup();
    let (layer, boundary, units) = classify_incar_parameter("MYSTERY", "42");
    assert_eq!(layer, Layer::Implementation);
    assert_eq!(units, None);

    match boundary {
        BoundaryClassification::ContextDependent {
            default_layer,
            context_note,
        } => {
            assert_eq!(default_layer, Layer::Implementation);
            assert_eq!(context_note, "VASP parameter not in classification table");
        }
        other => panic!("Expected ContextDependent, got {:?}", other),
    }
}

#[test]
fn test_vasp_parse_incar_basic() {
    setup();
    let events = parse_incar(VASP_INCAR_SAMPLE).unwrap();
    assert_eq!(events.len(), 13);
    assert!(events
        .iter()
        .all(|event| matches!(event.kind, EventKind::ParameterRecord { .. })));

    let names: Vec<String> = events
        .iter()
        .map(|event| match &event.kind {
            EventKind::ParameterRecord { name, .. } => name.clone(),
            other => panic!("Expected ParameterRecord, got {:?}", other),
        })
        .collect();

    assert_eq!(
        names,
        vec![
            "GGA", "ENCUT", "PREC", "SIGMA", "ISMEAR", "IBRION", "NSW", "ISIF", "EDIFF",
            "EDIFFG", "NCORE", "KPAR", "ALGO"
        ]
    );

    let gga = events
        .iter()
        .find(|event| matches!(&event.kind, EventKind::ParameterRecord { name, .. } if name == "GGA"))
        .expect("Expected GGA parameter");
    match &gga.kind {
        EventKind::ParameterRecord { actual_value, .. } => {
            assert_eq!(actual_value, &Value::KnownCat("PE".to_string()));
        }
        other => panic!("Expected ParameterRecord, got {:?}", other),
    }
}

#[test]
fn test_vasp_parse_incar_comments_stripped() {
    setup();
    let events = parse_incar(VASP_INCAR_SAMPLE).unwrap();

    let encut = events
        .iter()
        .find(|event| {
            matches!(
                &event.kind,
                EventKind::ParameterRecord { name, .. } if name == "ENCUT"
            )
        })
        .expect("Expected ENCUT");
    match &encut.kind {
        EventKind::ParameterRecord { actual_value, .. } => {
            assert_eq!(actual_value, &Value::Known(520.0, "eV".to_string()));
        }
        other => panic!("Expected ParameterRecord for ENCUT, got {:?}", other),
    }

    let sigma = events
        .iter()
        .find(|event| {
            matches!(
                &event.kind,
                EventKind::ParameterRecord { name, .. } if name == "SIGMA"
            )
        })
        .expect("Expected SIGMA");
    match &sigma.kind {
        EventKind::ParameterRecord { actual_value, .. } => {
            assert_eq!(actual_value, &Value::Known(0.05, "eV".to_string()));
        }
        other => panic!("Expected ParameterRecord for SIGMA, got {:?}", other),
    }
}

#[test]
fn test_vasp_parse_incar_empty() {
    setup();
    let events = parse_incar("").unwrap();
    assert!(events.is_empty());
}

#[test]
fn test_vasp_parse_incar_layer_distribution() {
    setup();
    let events = parse_incar(VASP_INCAR_SAMPLE).unwrap();
    let layers: std::collections::HashSet<Layer> =
        events.iter().map(|event| event.layer).collect();

    assert!(layers.contains(&Layer::Theory));
    assert!(layers.contains(&Layer::Methodology));
    assert!(layers.contains(&Layer::Implementation));
}

#[test]
fn test_vasp_parse_oszicar_convergence_points() {
    setup();
    let events = parse_oszicar(VASP_OSZICAR_SAMPLE, 0).unwrap();
    let convergence_count = events
        .iter()
        .filter(|event| matches!(event.kind, EventKind::ConvergencePoint { .. }))
        .count();
    assert_eq!(convergence_count, 6);
}

#[test]
fn test_vasp_parse_oszicar_energy_records() {
    setup();
    let events = parse_oszicar(VASP_OSZICAR_SAMPLE, 0).unwrap();
    let totals: Vec<f64> = events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::EnergyRecord {
                total: Value::Known(value, _),
                ..
            } => Some(*value),
            _ => None,
        })
        .collect();

    assert_eq!(totals.len(), 2);
    assert!((totals[0] + 114.01725).abs() < 1e-6);
    assert!((totals[1] + 114.11725).abs() < 1e-6);
}

#[test]
fn test_vasp_parse_oszicar_convergence_flagged() {
    setup();
    let events = parse_oszicar(VASP_OSZICAR_SAMPLE, 0).unwrap();
    let converged_count = events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                EventKind::ConvergencePoint {
                    converged: Some(true),
                    ..
                }
            )
        })
        .count();
    assert_eq!(converged_count, 2);
}

#[test]
fn test_vasp_parse_oszicar_single_step() {
    setup();
    let sample = r#"
DAV:   1    0.100E+02    0.100E+02   -0.500E+00   200   0.200E+02
DAV:   2   -0.200E+01   -0.100E+01   -0.100E+00   220   0.120E+02
   1 F= -.10000000E+02 E0= -.99900000E+01  d E =-.10000000E-01
"#;
    let events = parse_oszicar(sample, 0).unwrap();

    let convergence_count = events
        .iter()
        .filter(|event| matches!(event.kind, EventKind::ConvergencePoint { .. }))
        .count();
    let energy_count = events
        .iter()
        .filter(|event| matches!(event.kind, EventKind::EnergyRecord { .. }))
        .count();

    assert_eq!(convergence_count, 2);
    assert_eq!(energy_count, 1);
}

#[test]
fn test_vasp_parse_outcar_resource_status() {
    setup();
    let events = parse_outcar(VASP_OUTCAR_SAMPLE, 0).unwrap();
    let resource = events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ResourceStatus {
                platform_type,
                parallelization,
                device_ids,
                ..
            } => Some((platform_type, parallelization, device_ids)),
            _ => None,
        })
        .expect("Expected ResourceStatus");

    assert!(resource.0.to_ascii_lowercase().contains("vasp"));
    assert_eq!(resource.1, &Some("16 cores".to_string()));
    assert_eq!(resource.2.len(), 1);
}

#[test]
fn test_vasp_parse_outcar_energy_and_forces() {
    setup();
    let events = parse_outcar(VASP_OUTCAR_SAMPLE, 0).unwrap();

    assert!(events
        .iter()
        .any(|event| matches!(event.kind, EventKind::EnergyRecord { .. })));
    assert!(events.iter().any(|event| matches!(
        event.kind,
        EventKind::StateSnapshot {
            snapshot_type: SnapshotType::Forces,
            ..
        }
    )));
}

#[test]
fn test_vasp_parse_outcar_success() {
    setup();
    let events = parse_outcar(VASP_OUTCAR_SAMPLE, 0).unwrap();
    assert!(events.iter().any(|event| matches!(
        event.kind,
        EventKind::ExecutionStatus {
            status: ExecutionOutcome::Success,
            ..
        }
    )));
}

#[test]
fn test_vasp_parse_outcar_truncated() {
    setup();
    let events = parse_outcar(VASP_OUTCAR_TRUNCATED, 0).unwrap();
    let timeout = events
        .iter()
        .find(|event| {
            matches!(
                event.kind,
                EventKind::ExecutionStatus {
                    status: ExecutionOutcome::Timeout,
                    ..
                }
            )
        })
        .expect("Expected timeout event");

    match &timeout.confidence.completeness {
        Completeness::PartiallyInferred { inference_method } => {
            assert_eq!(inference_method, "no completion marker in OUTCAR");
        }
        other => panic!("Expected PartiallyInferred, got {:?}", other),
    }
    assert!((timeout.confidence.field_coverage - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_vasp_adapter_combined() {
    setup();
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(VASP_COMBINED_SAMPLE).unwrap();

    assert!(log
        .events
        .iter()
        .any(|event| matches!(event.kind, EventKind::ParameterRecord { .. })));
    assert!(log
        .events
        .iter()
        .any(|event| matches!(event.kind, EventKind::ConvergencePoint { .. })));
    assert!(log
        .events
        .iter()
        .any(|event| matches!(event.kind, EventKind::EnergyRecord { .. })));
    assert!(log
        .events
        .iter()
        .any(|event| matches!(event.kind, EventKind::StateSnapshot { .. })));
    assert!(log
        .events
        .iter()
        .any(|event| matches!(event.kind, EventKind::ResourceStatus { .. })));
    assert!(log
        .events
        .iter()
        .any(|event| matches!(event.kind, EventKind::ExecutionStatus { .. })));
}

#[test]
fn test_vasp_adapter_incar_only() {
    setup();
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(VASP_INCAR_SAMPLE).unwrap();

    assert!(!log.events.is_empty());
    assert!(log
        .events
        .iter()
        .all(|event| matches!(event.kind, EventKind::ParameterRecord { .. })));
}

#[test]
fn test_vasp_adapter_controlled_vars_empty() {
    setup();
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(VASP_COMBINED_SAMPLE).unwrap();
    assert!(log.spec.controlled_variables.is_empty());
}

#[test]
fn test_vasp_overlay_construction() {
    setup();
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(VASP_COMBINED_SAMPLE).unwrap();
    let overlay = CausalOverlay::from_log(&log);
    assert_eq!(overlay.len(), log.events.len());
}

#[test]
fn test_vasp_overlay_layer_span() {
    setup();
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(VASP_COMBINED_SAMPLE).unwrap();
    assert!(log.indexes.by_layer.contains_key(&Layer::Theory));
    assert!(log.indexes.by_layer.contains_key(&Layer::Methodology));
    assert!(log.indexes.by_layer.contains_key(&Layer::Implementation));
}

#[test]
fn test_vasp_hidden_confounder_litmus() {
    setup();
    let adapter = VaspAdapter;
    let mut log = adapter.parse_trace(VASP_COMBINED_SAMPLE).unwrap();

    let resolve_position = |variable: &str, log: &LayeredEventLog| -> usize {
        let event_id = *log
            .indexes
            .by_variable
            .get(variable)
            .and_then(|ids| ids.first())
            .expect("Expected variable to be indexed");
        *log.indexes
            .by_id
            .get(&event_id)
            .expect("Expected EventId to resolve to event position")
    };

    let prec_pos = resolve_position("PREC", &log);
    let sigma_pos = resolve_position("SIGMA", &log);
    let ibrion_pos = resolve_position("IBRION", &log);
    let prec_event_id = log.events[prec_pos].id;

    log.events[sigma_pos].causal_refs = vec![prec_event_id];
    log.events[ibrion_pos].causal_refs = vec![prec_event_id];

    log = test_vasp_rebuild_log(&log);

    let overlay = CausalOverlay::from_log(&log);
    let confounders = overlay.detect_confounders(&log, "SIGMA", "IBRION");
    assert!(!confounders.is_empty());
    assert!(confounders
        .iter()
        .any(|candidate| candidate.dag_node == "PREC"));
}

#[test]
fn test_vasp_hidden_confounder_controlled_excluded() {
    setup();
    let adapter = VaspAdapter;
    let mut log = adapter.parse_trace(VASP_COMBINED_SAMPLE).unwrap();

    let resolve_position = |variable: &str, log: &LayeredEventLog| -> usize {
        let event_id = *log
            .indexes
            .by_variable
            .get(variable)
            .and_then(|ids| ids.first())
            .expect("Expected variable to be indexed");
        *log.indexes
            .by_id
            .get(&event_id)
            .expect("Expected EventId to resolve to event position")
    };

    let prec_pos = resolve_position("PREC", &log);
    let sigma_pos = resolve_position("SIGMA", &log);
    let ibrion_pos = resolve_position("IBRION", &log);
    let prec_event_id = log.events[prec_pos].id;

    log.events[sigma_pos].causal_refs = vec![prec_event_id];
    log.events[ibrion_pos].causal_refs = vec![prec_event_id];
    log.spec.controlled_variables.push(ControlledVariable {
        id: SpecElementId(9_999),
        parameter: "PREC".to_string(),
        held_value: Value::KnownCat("Accurate".to_string()),
    });

    log = test_vasp_rebuild_log(&log);

    let overlay = CausalOverlay::from_log(&log);
    let confounders = overlay.detect_confounders(&log, "SIGMA", "IBRION");
    assert!(confounders.is_empty());
}

#[test]
fn test_vasp_adapter_error_execution() {
    setup();
    let events = parse_outcar(VASP_OUTCAR_ERROR, 0).unwrap();
    let execution = events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ExecutionStatus { status, .. } => Some(status),
            _ => None,
        })
        .expect("Expected ExecutionStatus event");

    assert_eq!(*execution, ExecutionOutcome::CrashDivergent);
}

#[test]
fn test_vasp_variant_converged_relaxation() {
    setup();
    let oszicar = VASP_FILE_CONVERGED_RELAXATION
        .split("--- OSZICAR ---")
        .nth(1)
        .and_then(|section| section.split("--- OUTCAR ---").next())
        .expect("Expected OSZICAR section");
    let expected_pairs = vec![(1, -100.0), (2, -100.1), (3, -100.12)];
    let parsed_pairs = parse_vasp_oszicar_energy_pairs(oszicar);
    assert_eq!(parsed_pairs, expected_pairs);

    let oszicar_events = parse_oszicar(oszicar, 0).unwrap();
    let component_names: Vec<Vec<String>> = oszicar_events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::EnergyRecord { components, .. } => Some(
                components
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<Vec<String>>(),
            ),
            _ => None,
        })
        .collect();
    assert_eq!(component_names.len(), 3);
    assert!(component_names
        .iter()
        .all(|names| names == &vec!["E0".to_string(), "dE".to_string()]));

    assert_vasp_variant(
        VASP_FILE_CONVERGED_RELAXATION,
        &expected_pairs,
        &[ConvergencePattern::Converged],
    );
    assert_vasp_execution_status(
        VASP_FILE_CONVERGED_RELAXATION,
        ExecutionOutcome::Success,
    );
}

#[test]
fn test_vasp_variant_nonconverged_scf() {
    setup();
    let oszicar = VASP_FILE_NONCONVERGED_SCF
        .split("--- OSZICAR ---")
        .nth(1)
        .and_then(|section| section.split("--- OUTCAR ---").next())
        .expect("Expected OSZICAR section");

    let energy_pairs = parse_vasp_oszicar_energy_pairs(oszicar);
    assert!(energy_pairs.is_empty());

    let convergence_pairs = parse_vasp_oszicar_convergence_pairs(oszicar);
    let expected_pairs = vec![(1, 50.0), (2, 20.0), (3, 10.0), (4, 5.0), (5, 2.0)];
    assert_eq!(convergence_pairs.len(), expected_pairs.len());
    for ((actual_iteration, actual_value), (expected_iteration, expected_value)) in
        convergence_pairs.iter().zip(expected_pairs.iter())
    {
        assert_eq!(*actual_iteration, *expected_iteration);
        assert!((*actual_value - *expected_value).abs() < 1e-6);
    }

    let oszicar_events = parse_oszicar(oszicar, 0).unwrap();
    let unconverged_count = oszicar_events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                EventKind::ConvergencePoint {
                    converged: None,
                    ..
                }
            )
        })
        .count();
    assert_eq!(unconverged_count, 5);

    assert_vasp_variant(
        VASP_FILE_NONCONVERGED_SCF,
        &[],
        &[ConvergencePattern::Stalled],
    );
    assert_vasp_execution_status(VASP_FILE_NONCONVERGED_SCF, ExecutionOutcome::Timeout);
}

#[test]
fn test_vasp_variant_mixed_scf_dav_rmm() {
    setup();
    let oszicar = VASP_FILE_MIXED_SCF_DAV_RMM
        .split("--- OSZICAR ---")
        .nth(1)
        .and_then(|section| section.split("--- OUTCAR ---").next())
        .expect("Expected OSZICAR section");

    let expected_energy_pairs = vec![(1, -21.0), (2, -21.1)];
    let parsed_energy_pairs = parse_vasp_oszicar_energy_pairs(oszicar);
    assert_eq!(parsed_energy_pairs, expected_energy_pairs);

    let convergence_pairs = parse_vasp_oszicar_convergence_pairs(oszicar);
    let expected_convergence_pairs = vec![
        (1, 6.0),
        (2, 3.0),
        (3, 2.0),
        (4, 1.0),
        (1, 5.0),
        (2, 2.5),
        (3, 1.2),
        (4, 0.6),
    ];
    assert_eq!(convergence_pairs.len(), expected_convergence_pairs.len());
    for ((actual_iteration, actual_value), (expected_iteration, expected_value)) in
        convergence_pairs.iter().zip(expected_convergence_pairs.iter())
    {
        assert_eq!(*actual_iteration, *expected_iteration);
        assert!((*actual_value - *expected_value).abs() < 1e-6);
    }

    assert_vasp_variant(
        VASP_FILE_MIXED_SCF_DAV_RMM,
        &expected_energy_pairs,
        &[ConvergencePattern::Converged],
    );
    assert_vasp_execution_status(VASP_FILE_MIXED_SCF_DAV_RMM, ExecutionOutcome::Success);
}

#[test]
fn test_vasp_variant_oscillating_scf() {
    setup();
    let oszicar = VASP_FILE_OSCILLATING_SCF
        .split("--- OSZICAR ---")
        .nth(1)
        .and_then(|section| section.split("--- OUTCAR ---").next())
        .expect("Expected OSZICAR section");

    let energy_pairs = parse_vasp_oszicar_energy_pairs(oszicar);
    assert!(energy_pairs.is_empty());

    let oszicar_events = parse_oszicar(oszicar, 0).unwrap();
    let unconverged_count = oszicar_events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                EventKind::ConvergencePoint {
                    converged: None,
                    ..
                }
            )
        })
        .count();
    assert_eq!(unconverged_count, 6);

    assert_vasp_variant(
        VASP_FILE_OSCILLATING_SCF,
        &[],
        &[ConvergencePattern::Oscillating],
    );
    assert_vasp_execution_status(VASP_FILE_OSCILLATING_SCF, ExecutionOutcome::Timeout);
}

#[test]
fn test_vasp_adapter_derives_oscillation_summary_for_non_converging_scf() {
    setup();
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(VASP_FILE_OSCILLATING_SCF).unwrap();

    let convergence = log
        .events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ConvergencePoint {
                metric_name,
                converged,
                ..
            } if metric_name == "derived_vasp_scf_oscillation_dE" => {
                Some((metric_name, converged))
            }
            _ => None,
        })
        .expect("Expected derived oscillation ConvergencePoint event");

    assert_eq!(convergence.0, "derived_vasp_scf_oscillation_dE");
    assert_eq!(convergence.1, &Some(false));
}

#[test]
fn test_vasp_adapter_derives_stall_summary_for_non_converging_scf() {
    setup();
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(VASP_FILE_NONCONVERGED_SCF).unwrap();

    let convergence = log
        .events
        .iter()
        .find_map(|event| match &event.kind {
            EventKind::ConvergencePoint {
                metric_name,
                converged,
                ..
            } if metric_name == "derived_vasp_scf_stall_dE" => Some((metric_name, converged)),
            _ => None,
        })
        .expect("Expected derived stall ConvergencePoint event");

    assert_eq!(convergence.0, "derived_vasp_scf_stall_dE");
    assert_eq!(convergence.1, &Some(false));
}

#[test]
fn test_vasp_adapter_no_scf_convergence_summary_below_min_window() {
    setup();
    let adapter = VaspAdapter;
    let raw = format!(
        "--- INCAR ---\n{}\n--- OSZICAR ---\n{}\n--- OUTCAR ---\n{}",
        VASP_INCAR_SAMPLE, VASP_OSZICAR_NO_F_SAMPLE, VASP_OUTCAR_SAMPLE
    );
    let log = adapter.parse_trace(&raw).unwrap();

    assert!(!log.events.iter().any(|event| {
        matches!(
            &event.kind,
            EventKind::ConvergencePoint { metric_name, .. } if metric_name.starts_with("derived_vasp_scf_")
        )
    }));
}

#[test]
fn test_vasp_adapter_scf_convergence_summary_provenance_refs() {
    setup();
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(VASP_FILE_NONCONVERGED_SCF).unwrap();

    let convergence_event = log
        .events
        .iter()
        .find(|event| {
            matches!(
                &event.kind,
                EventKind::ConvergencePoint { metric_name, .. } if metric_name == "derived_vasp_scf_stall_dE"
            )
        })
        .expect("Expected derived stall ConvergencePoint event");

    assert!(convergence_event.causal_refs.len() >= 4);
    assert!(matches!(
        convergence_event.confidence.completeness,
        Completeness::Derived { .. }
    ));
}

#[test]
fn test_vasp_adapter_no_scf_convergence_summary_for_converged_run() {
    setup();
    let adapter = VaspAdapter;
    let log = adapter.parse_trace(VASP_FILE_CONVERGED_RELAXATION).unwrap();

    assert!(!log.events.iter().any(|event| {
        matches!(
            &event.kind,
            EventKind::ConvergencePoint { metric_name, .. } if metric_name.starts_with("derived_vasp_scf_")
        )
    }));
}

#[test]
fn test_vasp_variant_error_edddav() {
    setup();
    assert_vasp_variant(
        VASP_VARIANT_ERROR_EDDDAV,
        &[(1, -100.0)],
        &[ConvergencePattern::Divergent],
    );
    assert_vasp_execution_status(
        VASP_VARIANT_ERROR_EDDDAV,
        ExecutionOutcome::CrashDivergent,
    );
}

#[test]
fn test_vasp_variant_large_encut_scinotation() {
    setup();
    assert_vasp_variant(
        VASP_VARIANT_LARGE_ENCUT_SCINOTATION,
        &[(1, -200.0)],
        &[ConvergencePattern::Converged],
    );
    assert_vasp_execution_status(
        VASP_VARIANT_LARGE_ENCUT_SCINOTATION,
        ExecutionOutcome::Success,
    );

    let adapter = VaspAdapter;
    let log = adapter
        .parse_trace(VASP_VARIANT_LARGE_ENCUT_SCINOTATION)
        .unwrap();
    let encut = log
        .events
        .iter()
        .find(|event| {
            matches!(
                &event.kind,
                EventKind::ParameterRecord { name, .. } if name == "ENCUT"
            )
        })
        .expect("Expected ENCUT parameter");
    match &encut.kind {
        EventKind::ParameterRecord { actual_value, .. } => {
            assert_eq!(actual_value, &Value::Known(1200.0, "eV".to_string()));
        }
        other => panic!("Expected ParameterRecord for ENCUT, got {:?}", other),
    }
}

#[test]
fn test_vasp_variant_static_calc() {
    setup();
    assert_vasp_variant(
        VASP_VARIANT_STATIC_CALC,
        &[(1, -50.0)],
        &[ConvergencePattern::Converged],
    );
    assert_vasp_execution_status(VASP_VARIANT_STATIC_CALC, ExecutionOutcome::Success);
}

#[test]
fn test_vasp_variant_v5_version() {
    setup();
    assert_vasp_variant(
        VASP_VARIANT_V5_VERSION,
        &[(1, -75.0)],
        &[ConvergencePattern::Converged],
    );

    let adapter = VaspAdapter;
    let log = adapter.parse_trace(VASP_VARIANT_V5_VERSION).unwrap();
    let has_v5 = log.events.iter().any(|event| {
        matches!(
            &event.kind,
            EventKind::ResourceStatus { platform_type, .. }
                if platform_type.to_ascii_lowercase().contains("vasp.5")
        )
    });
    assert!(has_v5);
}

#[test]
fn test_vasp_variant_truncated_outcar() {
    setup();
    let combined = format!(
        "--- INCAR ---\n{}\n--- OSZICAR ---\n{}\n--- OUTCAR ---\n{}",
        VASP_INCAR_SAMPLE, VASP_OSZICAR_SAMPLE, VASP_OUTCAR_TRUNCATED
    );
    assert_vasp_execution_status(&combined, ExecutionOutcome::Timeout);

    let adapter = VaspAdapter;
    let log = adapter.parse_trace(&combined).unwrap();
    let timeout = log
        .events
        .iter()
        .find(|event| {
            matches!(
                event.kind,
                EventKind::ExecutionStatus {
                    status: ExecutionOutcome::Timeout,
                    ..
                }
            )
        })
        .expect("Expected timeout execution status");
    match &timeout.confidence.completeness {
        Completeness::PartiallyInferred { inference_method } => {
            assert_eq!(inference_method, "no completion marker in OUTCAR");
        }
        other => panic!("Expected PartiallyInferred completeness, got {:?}", other),
    }
}

#[test]
fn test_vasp_variant_error_very_bad_news() {
    setup();
    let combined = format!(
        "--- INCAR ---\n{}\n--- OSZICAR ---\n{}\n--- OUTCAR ---\n{}",
        VASP_INCAR_SAMPLE, VASP_OSZICAR_SAMPLE, VASP_OUTCAR_ERROR
    );
    assert_vasp_variant(
        &combined,
        &[(1, -114.01725), (2, -114.11725)],
        &[ConvergencePattern::Divergent],
    );
    assert_vasp_execution_status(&combined, ExecutionOutcome::CrashDivergent);
}

#[test]
fn test_vasp_variant_energy_count_cross_source() {
    setup();
    assert_vasp_parses_energy_count(VASP_FILE_CONVERGED_RELAXATION, 4);
    assert_vasp_parses_energy_count(VASP_FILE_NONCONVERGED_SCF, 1);
    assert_vasp_parses_energy_count(VASP_FILE_MIXED_SCF_DAV_RMM, 3);
}

#[test]
fn test_vasp_t1_honeycomb_pt52() {
    setup();
    assert_vasp_variant(
        VASP_FILE_T1_HONEYCOMB_PT52,
        &[(1, -957.02531)],
        &[ConvergencePattern::Converged],
    );
    assert_vasp_execution_status(VASP_FILE_T1_HONEYCOMB_PT52, ExecutionOutcome::Success);
    assert_vasp_parses_energy_count(VASP_FILE_T1_HONEYCOMB_PT52, 2);
}

#[test]
fn test_vasp_t1_large_approx() {
    setup();
    assert_vasp_variant(
        VASP_FILE_T1_LARGE_APPROX,
        &[(1, -3279.8853)],
        &[ConvergencePattern::Converged],
    );
    assert_vasp_execution_status(VASP_FILE_T1_LARGE_APPROX, ExecutionOutcome::Success);
    assert_vasp_parses_energy_count(VASP_FILE_T1_LARGE_APPROX, 2);
}

#[test]
fn test_vasp_t1_sigma_pt56_substrate() {
    setup();
    assert_vasp_variant(
        VASP_FILE_T1_SIGMA_PT56_SUBSTRATE,
        &[(1, -561.68546)],
        &[ConvergencePattern::Converged],
    );
    assert_vasp_execution_status(
        VASP_FILE_T1_SIGMA_PT56_SUBSTRATE,
        ExecutionOutcome::Success,
    );
    assert_vasp_parses_energy_count(VASP_FILE_T1_SIGMA_PT56_SUBSTRATE, 2);
}

fn test_convergence_event(
    metric_name: &str,
    converged: Option<bool>,
    completeness: Completeness,
    logical_sequence: u64,
) -> TraceEvent {
    TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ConvergencePoint {
            iteration: logical_sequence,
            metric_name: metric_name.to_string(),
            metric_value: Value::Known(1.0, "relative".to_string()),
            converged,
        })
        .temporal(TemporalCoord {
            simulation_step: logical_sequence,
            wall_clock_ns: None,
            logical_sequence,
        })
        .provenance(test_provenance())
        .confidence(ConfidenceMeta {
            completeness,
            field_coverage: 1.0,
            notes: vec![],
        })
        .build()
}

fn test_log_from_events(events: Vec<TraceEvent>) -> LayeredEventLog {
    let mut builder = LayeredEventLogBuilder::new(test_experiment_ref(), test_spec());
    for event in events {
        builder = builder.add_event(event);
    }
    builder.build()
}

#[test]
fn test_classify_convergence_converged_derived() {
    setup();
    let convergence = test_convergence_event(
        "derived_convergence_rel_delta_max",
        Some(true),
        Completeness::Derived {
            from_elements: vec![ElementId(1)],
        },
        1,
    );
    let log = test_log_from_events(vec![convergence]);
    let event = log.events.first().expect("Expected ConvergencePoint");

    let canonical = classify_convergence(event, "openmm", &log);
    assert_eq!(canonical.pattern, ConvergencePattern::Converged);
    assert_eq!(canonical.confidence, ConvergenceConfidence::Derived);
}

#[test]
fn test_classify_convergence_oscillating_derived() {
    setup();
    let convergence = test_convergence_event(
        "derived_oscillation_rel_delta_mean",
        Some(false),
        Completeness::Derived {
            from_elements: vec![ElementId(2)],
        },
        2,
    );
    let log = test_log_from_events(vec![convergence]);
    let event = log.events.first().expect("Expected ConvergencePoint");

    let canonical = classify_convergence(event, "gromacs", &log);
    assert_eq!(canonical.pattern, ConvergencePattern::Oscillating);
    assert_eq!(canonical.confidence, ConvergenceConfidence::Derived);
}

#[test]
fn test_classify_convergence_stalled_derived() {
    setup();
    let convergence = test_convergence_event(
        "derived_stall_rel_delta_mean",
        Some(false),
        Completeness::Derived {
            from_elements: vec![ElementId(3)],
        },
        3,
    );
    let log = test_log_from_events(vec![convergence]);
    let event = log.events.first().expect("Expected ConvergencePoint");

    let canonical = classify_convergence(event, "openmm", &log);
    assert_eq!(canonical.pattern, ConvergencePattern::Stalled);
    assert_eq!(canonical.confidence, ConvergenceConfidence::Derived);
}

#[test]
fn test_classify_convergence_vasp_de_converged_direct() {
    setup();
    let convergence =
        test_convergence_event("dE", Some(true), Completeness::FullyObserved, 4);
    let log = test_log_from_events(vec![convergence]);
    let event = log.events.first().expect("Expected ConvergencePoint");

    let canonical = classify_convergence(event, "vasp", &log);
    assert_eq!(canonical.pattern, ConvergencePattern::Converged);
    assert_eq!(canonical.confidence, ConvergenceConfidence::Direct);
}

#[test]
fn test_classify_convergence_vasp_de_insufficient_direct() {
    setup();
    let convergence = test_convergence_event("dE", None, Completeness::FullyObserved, 5);
    let log = test_log_from_events(vec![convergence]);
    let event = log.events.first().expect("Expected ConvergencePoint");

    let canonical = classify_convergence(event, "vasp", &log);
    assert_eq!(canonical.pattern, ConvergencePattern::InsufficientData);
    assert_eq!(canonical.confidence, ConvergenceConfidence::Direct);
}

#[test]
fn test_classify_convergence_vasp_scf_oscillation_derived() {
    setup();
    let convergence = test_convergence_event(
        "derived_vasp_scf_oscillation_dE",
        Some(false),
        Completeness::Derived {
            from_elements: vec![ElementId(12)],
        },
        12,
    );
    let log = test_log_from_events(vec![convergence]);
    let event = log.events.first().expect("Expected ConvergencePoint");

    let canonical = classify_convergence(event, "vasp", &log);
    assert_eq!(canonical.pattern, ConvergencePattern::Oscillating);
    assert_eq!(canonical.confidence, ConvergenceConfidence::Derived);
}

#[test]
fn test_classify_convergence_vasp_scf_stall_derived() {
    setup();
    let convergence = test_convergence_event(
        "derived_vasp_scf_stall_dE",
        Some(false),
        Completeness::Derived {
            from_elements: vec![ElementId(13)],
        },
        13,
    );
    let log = test_log_from_events(vec![convergence]);
    let event = log.events.first().expect("Expected ConvergencePoint");

    let canonical = classify_convergence(event, "vasp", &log);
    assert_eq!(canonical.pattern, ConvergencePattern::Stalled);
    assert_eq!(canonical.confidence, ConvergenceConfidence::Derived);
}

#[test]
fn test_classify_convergence_divergent_override_priority() {
    setup();
    let convergence = test_convergence_event(
        "derived_convergence_rel_delta_max",
        Some(true),
        Completeness::Derived {
            from_elements: vec![ElementId(6)],
        },
        6,
    );
    let numerical = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(EventKind::NumericalStatus {
            event_type: NumericalEventType::NaNDetected,
            affected_quantity: "potential_energy".to_string(),
            severity: Severity::Error,
            detail: Value::KnownCat("NaN in simulation".to_string()),
        })
        .temporal(TemporalCoord {
            simulation_step: 6,
            wall_clock_ns: None,
            logical_sequence: 7,
        })
        .provenance(test_provenance())
        .confidence(ConfidenceMeta {
            completeness: Completeness::FullyObserved,
            field_coverage: 1.0,
            notes: vec![],
        })
        .build();
    let log = test_log_from_events(vec![convergence, numerical]);
    let event = log
        .events
        .iter()
        .find(|entry| matches!(entry.kind, EventKind::ConvergencePoint { .. }))
        .expect("Expected ConvergencePoint");

    let canonical = classify_convergence(event, "openmm", &log);
    assert_eq!(canonical.pattern, ConvergencePattern::Divergent);
    assert_eq!(canonical.confidence, ConvergenceConfidence::Derived);
}

#[test]
fn test_classify_convergence_unknown_metric_absent() {
    setup();
    let convergence = test_convergence_event(
        "unknown_metric",
        Some(false),
        Completeness::FullyObserved,
        8,
    );
    let log = test_log_from_events(vec![convergence]);
    let event = log.events.first().expect("Expected ConvergencePoint");

    let canonical = classify_convergence(event, "gromacs", &log);
    assert_eq!(canonical.pattern, ConvergencePattern::InsufficientData);
    assert_eq!(canonical.confidence, ConvergenceConfidence::Absent);
}

#[test]
fn test_classify_all_convergence_filters_convergence_points() {
    setup();
    let convergence_one = test_convergence_event(
        "derived_convergence_rel_delta_max",
        Some(true),
        Completeness::Derived {
            from_elements: vec![ElementId(9)],
        },
        9,
    );
    let parameter = TraceEventBuilder::new()
        .layer(Layer::Methodology)
        .kind(EventKind::ParameterRecord {
            name: "dt".to_string(),
            specified_value: None,
            actual_value: Value::Known(0.002, "ps".to_string()),
            units: Some("ps".to_string()),
            observation_mode: ObservationMode::Observational,
        })
        .temporal(TemporalCoord {
            simulation_step: 9,
            wall_clock_ns: None,
            logical_sequence: 10,
        })
        .provenance(test_provenance())
        .confidence(ConfidenceMeta {
            completeness: Completeness::FullyObserved,
            field_coverage: 1.0,
            notes: vec![],
        })
        .build();
    let convergence_two = test_convergence_event(
        "derived_stall_rel_delta_mean",
        Some(false),
        Completeness::Derived {
            from_elements: vec![ElementId(10)],
        },
        11,
    );
    let log = test_log_from_events(vec![convergence_one, parameter, convergence_two]);

    let all = classify_all_convergence(&log, "openmm");
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].pattern, ConvergencePattern::Converged);
    assert_eq!(all[1].pattern, ConvergencePattern::Stalled);
}

fn first_pattern_or_insufficient(
    log: &LayeredEventLog,
    framework: &str,
) -> ConvergencePattern {
    classify_all_convergence(log, framework)
        .into_iter()
        .map(|canonical| canonical.pattern)
        .next()
        .unwrap_or(ConvergencePattern::InsufficientData)
}

#[test]
fn test_equivalence_scenario_a_steady_state_converged() {
    setup();
    let gromacs_adapter = GromacsAdapter;
    let gromacs_raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_STABLE_SERIES
    );
    let gromacs_log = gromacs_adapter.parse_trace(&gromacs_raw).unwrap();

    let openmm_adapter = MockOpenMmAdapter;
    let openmm_log = openmm_adapter.parse_trace(OPENMM_CSV_STABLE).unwrap();

    let vasp_adapter = VaspAdapter;
    let vasp_log = vasp_adapter.parse_trace(VASP_COMBINED_SAMPLE).unwrap();

    assert_eq!(
        first_pattern_or_insufficient(&gromacs_log, "gromacs"),
        ConvergencePattern::Converged
    );
    assert_eq!(
        first_pattern_or_insufficient(&openmm_log, "openmm"),
        ConvergencePattern::Converged
    );
    let vasp_patterns: Vec<ConvergencePattern> =
        classify_all_convergence(&vasp_log, "vasp")
            .into_iter()
            .map(|canonical| canonical.pattern)
            .collect();
    assert!(vasp_patterns.contains(&ConvergencePattern::Converged));
}

#[test]
fn test_equivalence_scenario_b_oscillating() {
    setup();
    let gromacs_adapter = GromacsAdapter;
    let gromacs_raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_OSCILLATING_SERIES
    );
    let gromacs_log = gromacs_adapter.parse_trace(&gromacs_raw).unwrap();

    let openmm_adapter = MockOpenMmAdapter;
    let openmm_log = openmm_adapter.parse_trace(OPENMM_CSV_OSCILLATING).unwrap();

    let vasp_adapter = VaspAdapter;
    let vasp_log = vasp_adapter.parse_trace(VASP_FILE_OSCILLATING_SCF).unwrap();

    assert_eq!(
        first_pattern_or_insufficient(&gromacs_log, "gromacs"),
        ConvergencePattern::Oscillating
    );
    assert_eq!(
        first_pattern_or_insufficient(&openmm_log, "openmm"),
        ConvergencePattern::Oscillating
    );
    let vasp_patterns: Vec<ConvergencePattern> = classify_all_convergence(&vasp_log, "vasp")
        .into_iter()
        .map(|canonical| canonical.pattern)
        .collect();
    assert!(vasp_patterns.contains(&ConvergencePattern::Oscillating));
}

#[test]
fn test_equivalence_scenario_c_stalled() {
    setup();
    let gromacs_adapter = GromacsAdapter;
    let gromacs_raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_DRIFTING_SERIES
    );
    let gromacs_log = gromacs_adapter.parse_trace(&gromacs_raw).unwrap();

    let openmm_adapter = MockOpenMmAdapter;
    let openmm_log = openmm_adapter.parse_trace(OPENMM_CSV_DRIFTING).unwrap();

    let vasp_adapter = VaspAdapter;
    let vasp_log = vasp_adapter.parse_trace(VASP_FILE_NONCONVERGED_SCF).unwrap();

    assert_eq!(
        first_pattern_or_insufficient(&gromacs_log, "gromacs"),
        ConvergencePattern::Stalled
    );
    assert_eq!(
        first_pattern_or_insufficient(&openmm_log, "openmm"),
        ConvergencePattern::Stalled
    );
    let vasp_patterns: Vec<ConvergencePattern> = classify_all_convergence(&vasp_log, "vasp")
        .into_iter()
        .map(|canonical| canonical.pattern)
        .collect();
    assert!(vasp_patterns.contains(&ConvergencePattern::Stalled));
}

#[test]
fn test_equivalence_scenario_d_divergent_nan() {
    setup();
    let gromacs_adapter = GromacsAdapter;
    let gromacs_raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_DIVERGENT_NAN_SERIES
    );
    let gromacs_log = gromacs_adapter.parse_trace(&gromacs_raw).unwrap();

    let openmm_adapter = MockOpenMmAdapter;
    let openmm_log = openmm_adapter.parse_trace(OPENMM_CSV_DIVERGENT_NAN).unwrap();

    let vasp_adapter = VaspAdapter;
    let vasp_log = vasp_adapter
        .parse_trace(VASP_COMBINED_DIVERGENT_SAMPLE)
        .unwrap();

    assert_eq!(
        first_pattern_or_insufficient(&gromacs_log, "gromacs"),
        ConvergencePattern::Divergent
    );
    assert_eq!(
        first_pattern_or_insufficient(&openmm_log, "openmm"),
        ConvergencePattern::Divergent
    );
    let vasp_patterns: Vec<ConvergencePattern> =
        classify_all_convergence(&vasp_log, "vasp")
            .into_iter()
            .map(|canonical| canonical.pattern)
            .collect();
    assert!(vasp_patterns.contains(&ConvergencePattern::Divergent));
}

#[test]
fn test_equivalence_scenario_e_insufficient_data() {
    setup();
    let gromacs_adapter = GromacsAdapter;
    let gromacs_raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_SHORT_SERIES
    );
    let gromacs_log = gromacs_adapter.parse_trace(&gromacs_raw).unwrap();

    let openmm_adapter = MockOpenMmAdapter;
    let openmm_log = openmm_adapter.parse_trace(OPENMM_SHORT_ENERGY_SERIES).unwrap();

    let vasp_adapter = VaspAdapter;
    let vasp_raw = format!(
        "--- INCAR ---\n{}\n--- OSZICAR ---\n{}\n--- OUTCAR ---\n{}",
        VASP_INCAR_SAMPLE, VASP_OSZICAR_NO_F_SAMPLE, VASP_OUTCAR_SAMPLE
    );
    let vasp_log = vasp_adapter.parse_trace(&vasp_raw).unwrap();

    assert_eq!(
        first_pattern_or_insufficient(&gromacs_log, "gromacs"),
        ConvergencePattern::InsufficientData
    );
    assert_eq!(
        first_pattern_or_insufficient(&openmm_log, "openmm"),
        ConvergencePattern::InsufficientData
    );
    assert_eq!(
        first_pattern_or_insufficient(&vasp_log, "vasp"),
        ConvergencePattern::InsufficientData
    );
}

#[test]
fn test_equivalence_scenario_f_threshold_boundary() {
    setup();
    let gromacs_adapter = GromacsAdapter;
    let gromacs_raw = format!(
        "--- MDP ---\n{}\n--- LOG ---\n{}",
        GROMACS_MDP_SAMPLE, GROMACS_LOG_BOUNDARY_SERIES
    );
    let gromacs_log = gromacs_adapter.parse_trace(&gromacs_raw).unwrap();

    let openmm_adapter = MockOpenMmAdapter;
    let openmm_log = openmm_adapter.parse_trace(OPENMM_CSV_BOUNDARY).unwrap();

    assert_eq!(
        first_pattern_or_insufficient(&gromacs_log, "gromacs"),
        ConvergencePattern::Converged
    );
    assert_eq!(
        first_pattern_or_insufficient(&openmm_log, "openmm"),
        ConvergencePattern::Converged
    );
}
