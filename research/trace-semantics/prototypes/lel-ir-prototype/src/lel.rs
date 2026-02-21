use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::common::{
    BoundaryClassification, Completeness, ConfidenceMeta, ContractTerm, ControlledVariable,
    DagReference, EventId, EventKindTag, ExperimentRef, InterventionRecord, Layer,
    PredictionRecord, ProvenanceAnchor, SourceLocation, SpecElementId, TemporalCoord,
};
use crate::event_kinds::EventKind;

// ============================================================
// Core LEL Structures (from §2)
// ============================================================

/// The top-level LEL IR container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeredEventLog {
    /// The experiment this log belongs to.
    pub experiment_ref: ExperimentRef,

    /// The experiment specification, parsed before trace processing.
    /// First-class entity per Pattern 6 — prevents AP1 (spec-impl conflation).
    pub spec: ExperimentSpec,

    /// The ordered event stream. Append-only during construction.
    pub events: Vec<TraceEvent>,

    /// Secondary indexes built during construction for R24 queryability.
    pub indexes: EventIndexes,
}

/// Experiment specification as a first-class entity.
/// Prevents AP1 (specification-implementation conflation) by structurally
/// separating "what was intended" from "what happened."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSpec {
    pub preconditions: Vec<ContractTerm>,
    pub postconditions: Vec<ContractTerm>,
    pub predictions: Vec<PredictionRecord>,
    pub interventions: Vec<InterventionRecord>,
    pub controlled_variables: Vec<ControlledVariable>,
    pub dag_refs: Vec<DagReference>,
    pub provenance: ProvenanceAnchor,
}

/// Each trace event has an SSA-like unique ID, a layer tag,
/// a typed event kind, temporal coordinates, and optional
/// causal references to prior events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Unique, immutable identifier. Monotonically increasing.
    pub id: EventId,

    /// Primary layer classification.
    pub layer: Layer,

    /// Boundary classification for dual-nature events.
    pub boundary: BoundaryClassification,

    /// The typed event payload.
    pub kind: EventKind,

    /// When this event occurred.
    pub temporal: TemporalCoord,

    /// Optional references to causally prior events.
    /// Best-effort — LEL does not require exhaustive causal annotation.
    pub causal_refs: Vec<EventId>,

    /// Optional reference to a DAG node this event relates to (R9, R11).
    /// Preserved for Hybrid upgrade path.
    pub dag_node_ref: Option<String>,

    /// Optional reference to the spec element this event realizes.
    /// Preserved for Hybrid upgrade path.
    pub spec_ref: Option<SpecElementId>,

    /// Source traceability.
    pub provenance: ProvenanceAnchor,

    /// Confidence metadata.
    pub confidence: ConfidenceMeta,
}

/// Secondary indexes for R24 queryability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventIndexes {
    pub by_id: HashMap<EventId, usize>,
    pub by_layer: HashMap<Layer, Vec<EventId>>,
    pub by_kind: HashMap<EventKindTag, Vec<EventId>>,
    /// simulation_step -> first event at that step.
    pub by_time_range: BTreeMap<u64, EventId>,
    pub by_variable: HashMap<String, Vec<EventId>>,
    pub by_dag_node: HashMap<String, Vec<EventId>>,
}

impl EventIndexes {
    /// Create empty indexes.
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            by_layer: HashMap::new(),
            by_kind: HashMap::new(),
            by_time_range: BTreeMap::new(),
            by_variable: HashMap::new(),
            by_dag_node: HashMap::new(),
        }
    }

    /// Index a single event. Called during log construction.
    pub fn index_event(&mut self, event: &TraceEvent, position: usize) {
        self.by_id.insert(event.id, position);

        // By layer
        self.by_layer.entry(event.layer).or_default().push(event.id);

        // By kind tag
        self.by_kind
            .entry(event.kind.tag())
            .or_default()
            .push(event.id);

        // By time range (first event at each simulation step)
        self.by_time_range
            .entry(event.temporal.simulation_step)
            .or_insert(event.id);

        // By variable (for ParameterRecord and ObservableMeasurement)
        match &event.kind {
            EventKind::ParameterRecord { name, .. } => {
                self.by_variable.entry(name.clone()).or_default().push(event.id);
            }
            EventKind::ObservableMeasurement { variable_name, .. } => {
                self.by_variable
                    .entry(variable_name.clone())
                    .or_default()
                    .push(event.id);
            }
            _ => {}
        }

        // By DAG node
        if let Some(dag_ref) = &event.dag_node_ref {
            self.by_dag_node
                .entry(dag_ref.clone())
                .or_default()
                .push(event.id);
        }
    }
}

impl Default for EventIndexes {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Builder Helpers
// ============================================================

/// Global atomic counter for auto-assigning EventIds.
static EVENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Reset the global event ID counter (for test isolation).
pub fn reset_event_id_counter() {
    EVENT_ID_COUNTER.store(1, Ordering::SeqCst);
}

/// Fluent builder for constructing TraceEvent instances.
pub struct TraceEventBuilder {
    layer: Option<Layer>,
    boundary: BoundaryClassification,
    kind: Option<EventKind>,
    temporal: Option<TemporalCoord>,
    causal_refs: Vec<EventId>,
    dag_node_ref: Option<String>,
    spec_ref: Option<SpecElementId>,
    provenance: Option<ProvenanceAnchor>,
    confidence: Option<ConfidenceMeta>,
}

impl TraceEventBuilder {
    pub fn new() -> Self {
        Self {
            layer: None,
            boundary: BoundaryClassification::PrimaryLayer,
            kind: None,
            temporal: None,
            causal_refs: Vec::new(),
            dag_node_ref: None,
            spec_ref: None,
            provenance: None,
            confidence: None,
        }
    }

    /// Required: set the primary layer.
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = Some(layer);
        self
    }

    /// Optional: set the boundary classification (defaults to PrimaryLayer).
    pub fn boundary(mut self, boundary: BoundaryClassification) -> Self {
        self.boundary = boundary;
        self
    }

    /// Required: set the typed event payload.
    pub fn kind(mut self, kind: EventKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Required: set when this event occurred.
    pub fn temporal(mut self, temporal: TemporalCoord) -> Self {
        self.temporal = Some(temporal);
        self
    }

    /// Optional: add causal references to prior events.
    pub fn causal_refs(mut self, refs: Vec<EventId>) -> Self {
        self.causal_refs = refs;
        self
    }

    /// Optional: set DAG node reference (Hybrid upgrade path).
    pub fn dag_node_ref(mut self, dag_ref: String) -> Self {
        self.dag_node_ref = Some(dag_ref);
        self
    }

    /// Optional: set spec element reference (Hybrid upgrade path).
    pub fn spec_ref(mut self, spec_ref: SpecElementId) -> Self {
        self.spec_ref = Some(spec_ref);
        self
    }

    /// Optional: set provenance (defaults to a synthetic anchor).
    pub fn provenance(mut self, provenance: ProvenanceAnchor) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Optional: set confidence metadata (defaults to FullyObserved, 1.0 coverage).
    pub fn confidence(mut self, confidence: ConfidenceMeta) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Build the TraceEvent. Panics if required fields (layer, kind, temporal) are missing.
    pub fn build(self) -> TraceEvent {
        let id = EventId(EVENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst));

        TraceEvent {
            id,
            layer: self.layer.expect("TraceEventBuilder: layer is required"),
            boundary: self.boundary,
            kind: self.kind.expect("TraceEventBuilder: kind is required"),
            temporal: self
                .temporal
                .expect("TraceEventBuilder: temporal is required"),
            causal_refs: self.causal_refs,
            dag_node_ref: self.dag_node_ref,
            spec_ref: self.spec_ref,
            provenance: self.provenance.unwrap_or_else(|| ProvenanceAnchor {
                source_file: "synthetic".to_string(),
                source_location: SourceLocation::ExternalInput,
                raw_hash: 0,
            }),
            confidence: self.confidence.unwrap_or_else(|| ConfidenceMeta {
                completeness: Completeness::FullyObserved,
                field_coverage: 1.0,
                notes: Vec::new(),
            }),
        }
    }
}

impl Default for TraceEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Fluent builder for constructing LayeredEventLog instances.
pub struct LayeredEventLogBuilder {
    experiment_ref: ExperimentRef,
    spec: ExperimentSpec,
    events: Vec<TraceEvent>,
    indexes: EventIndexes,
}

impl LayeredEventLogBuilder {
    pub fn new(experiment_ref: ExperimentRef, spec: ExperimentSpec) -> Self {
        Self {
            experiment_ref,
            spec,
            events: Vec::new(),
            indexes: EventIndexes::new(),
        }
    }

    /// Add an event and update indexes.
    pub fn add_event(mut self, event: TraceEvent) -> Self {
        self.indexes.index_event(&event, self.events.len());
        self.events.push(event);
        self
    }

    /// Build the LayeredEventLog.
    pub fn build(self) -> LayeredEventLog {
        LayeredEventLog {
            experiment_ref: self.experiment_ref,
            spec: self.spec,
            events: self.events,
            indexes: self.indexes,
        }
    }
}
