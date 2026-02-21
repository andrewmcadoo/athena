# Codex Prompt: LEL IR Prototype (Step 5b)

You are implementing a research prototype for ATHENA, a falsification-driven AI co-scientist. This prototype validates the Layered Event Log (LEL) — the foundation layer of a Hybrid LEL+DGR intermediate representation for translating DSL simulation traces into semantic failure representations. The prototype targets OpenMM molecular dynamics traces, demonstrating event typing, layer tagging, and specification separation.

Produce a compilable, fully tested Rust crate at the path `research/trace-semantics/prototypes/lel-ir-prototype/` with the exact structure and contents specified below. Every type must be fully defined, every import must resolve, and `rust-analyzer` must report zero errors or warnings.

---

## 1. Crate Layout

```
lel-ir-prototype/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── common.rs
│   ├── lel.rs
│   ├── event_kinds.rs
│   ├── adapter.rs
│   └── tests/
│       └── mod.rs
```

---

## 2. Cargo.toml

```toml
[package]
name = "lel-ir-prototype"
version = "0.1.0"
edition = "2021"
publish = false
description = "ATHENA Trace Semantics Engine — LEL IR prototype (Step 5b research artifact)"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## 3. src/lib.rs

Declare modules and re-export all public types.

```rust
pub mod common;
pub mod lel;
pub mod event_kinds;
pub mod adapter;

#[cfg(test)]
mod tests;
```

---

## 4. src/common.rs — Section 1 Common Foundation Types

These types are defined verbatim from the candidate IR schemas document (Section 1). They form the shared structural foundation. Additionally, several supporting types referenced but not formally defined in the schemas document are specified here.

```rust
use serde::{Deserialize, Serialize};

// ============================================================
// Supporting types (referenced in §1/§2 but not formally defined)
// ============================================================

/// Unique, monotonic event identifier. SSA-like: each event is assigned once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub u64);

/// Identifies a spec element (precondition, prediction, intervention, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpecElementId(pub u64);

/// Generic element reference for Completeness::Derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId(pub u64);

/// Physical units — string alias for prototype flexibility.
pub type Unit = String;

/// Expected type for Value::Havoc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueType {
    Scalar,
    Vector,
    Categorical,
}

/// R1 completion/failure states.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionOutcome {
    Success,
    CrashDivergent,
    Timeout,
    FrameworkError,
}

/// Standard severity ladder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// R6 numerical health events from OpenMM context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NumericalEventType {
    NaNDetected,
    InfDetected,
    LargeForce,
    EnergyDrift,
    ConvergenceFailure,
}

/// R5 parameter validation status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchStatus {
    Exact,
    WithinTolerance { deviation: f64 },
    Mismatch { deviation: f64 },
}

/// State snapshot variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SnapshotType {
    Coordinates,
    Velocities,
    Forces,
    Full,
}

/// Discriminant tag mirroring EventKind variant names (no payload).
/// Used as a key in EventIndexes::by_kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventKindTag {
    ExecutionStatus,
    ExceptionEvent,
    ParameterRecord,
    ValidationResult,
    NumericalStatus,
    ResourceStatus,
    ObservableMeasurement,
    SamplingMetadata,
    ComparisonResult,
    ConvergencePoint,
    StateSnapshot,
    EnergyRecord,
}

/// Minimal precondition/postcondition contract term.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractTerm {
    pub id: SpecElementId,
    pub description: String,
    pub layer: Layer,
}

/// R15 prediction record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionRecord {
    pub id: SpecElementId,
    pub variable: String,
    pub predicted_value: Value,
    pub tolerance: Option<f64>,
}

/// R10 intervention record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterventionRecord {
    pub id: SpecElementId,
    pub parameter: String,
    pub values: Vec<Value>,
}

/// R13 controlled variable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlledVariable {
    pub id: SpecElementId,
    pub parameter: String,
    pub held_value: Value,
}

/// R9/R11 DAG cross-reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagReference {
    pub node_id: String,
    pub edge_ids: Vec<String>,
}

/// R17 comparison outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonOutcome {
    pub agreement: bool,
    pub divergence: Option<DivergenceMeasure>,
    pub detail: String,
}

/// Pluggable divergence measure for prediction-observation comparison (from §3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DivergenceMeasure {
    AbsoluteDifference(f64),
    ZScore(f64),
    BayesFactor(f64),
    KLDivergence(f64),
    EffectSize(f64),
    Custom { name: String, value: f64 },
}

// ============================================================
// Section 1: Common Structural Foundation (verbatim from schemas)
// ============================================================

/// R19: The load-bearing structural distinction.
/// Every IR element is tagged with exactly one primary layer.
/// DSL API separation makes this possible (ARCHITECTURE.md §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Layer {
    Theory,
    Methodology,
    Implementation,
}

/// Resolves Open Question #4: How should boundary parameters be represented?
///
/// The primary layer is assigned based on where the parameter's failure
/// would have the most diagnostic impact, with an explicit annotation
/// for dual-nature parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BoundaryClassification {
    /// Unambiguously belongs to one layer.
    PrimaryLayer,

    /// Serves dual roles. Assigned a primary layer for routing but
    /// carries a cross-reference annotation to the secondary layer.
    DualAnnotated {
        secondary_layer: Layer,
        rationale: String,
    },

    /// Classification depends on the specific system being simulated.
    ContextDependent {
        default_layer: Layer,
        context_note: String,
    },
}

/// R28: Interventional vs. observational distinction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObservationMode {
    Interventional,
    Observational,
}

/// R26: Observability gap representation.
/// From Boogie pattern P6 — explicitly unknown state is represented
/// structurally rather than silently omitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Known value with units.
    Known(f64, Unit),
    /// Known vector (e.g., forces, coordinates).
    KnownVec(Vec<f64>, Unit),
    /// Known categorical value (e.g., ensemble type, functional name).
    KnownCat(String),
    /// Explicitly unknown — the trace does not contain this value.
    Havoc {
        expected_type: ValueType,
        reason: HavocReason,
    },
}

/// Reason a value is unknown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HavocReason {
    NotLogged,
    FrameworkLimitation,
    ConfigurationOmission,
    CrashStateGap,
    TemporalGap {
        last_known_step: u64,
        gap_steps: u64,
    },
}

/// R21: Temporal ordering. Three coordinate systems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalCoord {
    /// Simulation step (MD) or ionic/SCF iteration (DFT).
    pub simulation_step: u64,
    /// Wall clock time in nanoseconds since experiment start.
    pub wall_clock_ns: Option<u64>,
    /// Monotonic sequence number assigned by the IR during construction.
    pub logical_sequence: u64,
}

/// R20: Every IR element is traceable to its source in raw trace data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceAnchor {
    pub source_file: String,
    pub source_location: SourceLocation,
    pub raw_hash: u64,
}

/// Source location variant within a trace file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SourceLocation {
    LineRange { start: u32, end: u32 },
    XPath(String),
    BinaryOffset { start: u64, length: u64 },
    ApiQuery(String),
    ExternalInput,
}

/// R22, R29: Links every IR element to its experiment and cycle context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExperimentRef {
    pub experiment_id: String,
    pub cycle_id: u32,
    pub hypothesis_id: String,
}

/// R25: Classification confidence for each IR element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceMeta {
    pub completeness: Completeness,
    pub field_coverage: f32,
    pub notes: Vec<String>,
}

/// Completeness classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Completeness {
    FullyObserved,
    PartiallyInferred { inference_method: String },
    ExternallyProvided,
    Derived { from_elements: Vec<ElementId> },
}
```

---

## 5. src/event_kinds.rs — EventKind Enum

All 12 variants from §2. Stage 2/3 variants are defined but not constructed in mock/tests (avoids future refactor).

```rust
use serde::{Deserialize, Serialize};

use crate::common::{
    ComparisonOutcome, EventId, EventKindTag, ExecutionOutcome, MatchStatus,
    NumericalEventType, ObservationMode, Severity, SnapshotType, Value,
};

/// Event types mapped to requirements R1-R7, R8, R12, R16, R17.
/// One variant per requirement class, ensuring exhaustive coverage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    // === Stage 1: Implementation Audit (R1-R7) ===

    /// R1: Execution completed or terminated abnormally.
    ExecutionStatus {
        status: ExecutionOutcome,
        framework_error_id: Option<String>,
    },

    /// R2: Exception or error event from the DSL framework.
    ExceptionEvent {
        exception_type: String,
        component: String,
        dsl_call_path: Vec<String>,
        message: String,
        severity: Severity,
    },

    /// R3/R4: Input parameter — specification value and/or actual value.
    ParameterRecord {
        name: String,
        specified_value: Option<Value>,
        actual_value: Value,
        units: Option<String>,
        observation_mode: ObservationMode,
    },

    /// R5: Input validation result (derived from R3+R4 comparison).
    ValidationResult {
        parameter_name: String,
        match_status: MatchStatus,
        deviation_detail: Option<String>,
    },

    /// R6: Numerical health status.
    NumericalStatus {
        event_type: NumericalEventType,
        affected_quantity: String,
        severity: Severity,
        detail: Value,
    },

    /// R7: Resource and environment status.
    ResourceStatus {
        platform_type: String,
        device_ids: Vec<String>,
        memory_allocated: Option<Value>,
        memory_peak: Option<Value>,
        parallelization: Option<String>,
        warnings: Vec<String>,
    },

    // === Stage 2: Methodology Audit (R8, R12) ===

    /// R8/R16: Observable measurement.
    ObservableMeasurement {
        variable_name: String,
        measurement_method: String,
        value: Value,
        uncertainty: Option<Value>,
        conditions: String,
        observation_mode: ObservationMode,
    },

    /// R12: Sampling metadata.
    SamplingMetadata {
        sample_count: u64,
        sampling_method: String,
        equilibration_steps: Option<u64>,
        autocorrelation_time: Option<Value>,
        statistical_power: Option<Value>,
    },

    // === Stage 3: Theory Evaluation (R17) ===

    /// R17: Prediction-observation comparison result (derived).
    ComparisonResult {
        prediction_id: String,
        observation_id: EventId,
        result: ComparisonOutcome,
    },

    // === Convergence and State ===

    /// Convergence trajectory point (SCF, ionic, constraint).
    ConvergencePoint {
        iteration: u64,
        metric_name: String,
        metric_value: Value,
        converged: Option<bool>,
    },

    /// State snapshot (coordinates, velocities, forces at a timestep).
    StateSnapshot {
        snapshot_type: SnapshotType,
        data_ref: String,
    },

    /// Energy decomposition at a timestep.
    EnergyRecord {
        total: Value,
        components: Vec<(String, Value)>,
    },
}

impl EventKind {
    /// Returns the discriminant tag for this event kind (no payload).
    /// Used for indexing in EventIndexes::by_kind.
    pub fn tag(&self) -> EventKindTag {
        match self {
            EventKind::ExecutionStatus { .. } => EventKindTag::ExecutionStatus,
            EventKind::ExceptionEvent { .. } => EventKindTag::ExceptionEvent,
            EventKind::ParameterRecord { .. } => EventKindTag::ParameterRecord,
            EventKind::ValidationResult { .. } => EventKindTag::ValidationResult,
            EventKind::NumericalStatus { .. } => EventKindTag::NumericalStatus,
            EventKind::ResourceStatus { .. } => EventKindTag::ResourceStatus,
            EventKind::ObservableMeasurement { .. } => EventKindTag::ObservableMeasurement,
            EventKind::SamplingMetadata { .. } => EventKindTag::SamplingMetadata,
            EventKind::ComparisonResult { .. } => EventKindTag::ComparisonResult,
            EventKind::ConvergencePoint { .. } => EventKindTag::ConvergencePoint,
            EventKind::StateSnapshot { .. } => EventKindTag::StateSnapshot,
            EventKind::EnergyRecord { .. } => EventKindTag::EnergyRecord,
        }
    }
}
```

---

## 6. src/lel.rs — LEL Core Structures

Section 2 structures from the schemas document, plus builder helpers.

```rust
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::common::{
    BoundaryClassification, ConfidenceMeta, Completeness, ContractTerm,
    ControlledVariable, DagReference, EventId, EventKindTag, ExperimentRef,
    InterventionRecord, Layer, PredictionRecord, ProvenanceAnchor,
    SourceLocation, SpecElementId, TemporalCoord,
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
            by_layer: HashMap::new(),
            by_kind: HashMap::new(),
            by_time_range: BTreeMap::new(),
            by_variable: HashMap::new(),
            by_dag_node: HashMap::new(),
        }
    }

    /// Index a single event. Called during log construction.
    pub fn index_event(&mut self, event: &TraceEvent) {
        // By layer
        self.by_layer
            .entry(event.layer)
            .or_default()
            .push(event.id);

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
        if let Some(ref dag_ref) = event.dag_node_ref {
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
            temporal: self.temporal.expect("TraceEventBuilder: temporal is required"),
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
        self.indexes.index_event(&event);
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
```

---

## 7. src/adapter.rs — OpenMM Adapter Trait and Mock

```rust
use std::fmt;

use crate::common::{
    BoundaryClassification, Completeness, ConfidenceMeta, ControlledVariable,
    EventId, ExecutionOutcome, ExperimentRef, Layer, ObservationMode,
    ProvenanceAnchor, SourceLocation, SpecElementId, TemporalCoord, Value,
};
use crate::event_kinds::EventKind;
use crate::lel::{
    ExperimentSpec, LayeredEventLog, LayeredEventLogBuilder, TraceEvent,
    TraceEventBuilder,
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
                rationale: "Timestep affects both sampling methodology and numerical stability".to_string(),
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
                    ("kinetic".to_string(), Value::Known(12500.3, "kJ/mol".to_string())),
                    ("potential".to_string(), Value::Known(-57524.0, "kJ/mol".to_string())),
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
```

---

## 8. src/tests/mod.rs — Unit Tests

All 11 tests specified below. Import everything needed from the crate.

```rust
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

    let exec_ids = log
        .indexes
        .by_kind
        .get(&EventKindTag::ExecutionStatus)
        .unwrap();
    assert_eq!(exec_ids.len(), 1);

    let resource_ids = log
        .indexes
        .by_kind
        .get(&EventKindTag::ResourceStatus)
        .unwrap();
    assert_eq!(resource_ids.len(), 1);

    let energy_ids = log
        .indexes
        .by_kind
        .get(&EventKindTag::EnergyRecord)
        .unwrap();
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
            reason: HavocReason::TemporalGap {
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
        assert_eq!(event.dag_node_ref, None);
        // spec_ref exists (as None for mock data)
        assert_eq!(event.spec_ref, None);
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
    assert!(
        !events.is_empty(),
        "Event stream should contain trace events"
    );

    // Spec provenance is different from event provenance (different sources)
    assert_ne!(
        spec.provenance.source_file,
        events[0].provenance.source_file,
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
```

---

## 9. Verification

After producing all files, run these commands and confirm each passes:

```bash
cd research/trace-semantics/prototypes/lel-ir-prototype/
cargo build 2>&1          # Must succeed with no errors
cargo test 2>&1           # All 11 tests must pass
cargo clippy -- -D warnings 2>&1  # Zero warnings
```

Additionally, verify that `rust-analyzer` reports zero errors or warnings. This means:
- All types are fully defined (no forward references to undefined types).
- All imports resolve (no unresolved `use` statements).
- No dead code warnings (all types and functions are either `pub` or used in tests).
- No unused import warnings.

---

## 10. Constraints — What NOT To Do

- **No CausalOverlay / HybridIR / DGR structures.** This prototype is LEL-only. The `dag_node_ref`, `spec_ref`, and `causal_refs` fields are preserved for the Hybrid upgrade path but no overlay types are defined.
- **No real OpenMM trace parsing.** `MockOpenMmAdapter` uses hardcoded data. No file I/O, no XML/JSON parsing of real traces.
- **No PyO3 bindings.** Pure Rust prototype per ADR 001 research-phase flexibility.
- **No Stage 2/3 query logic.** Stage 2/3 `EventKind` variants (`ObservableMeasurement`, `SamplingMetadata`, `ComparisonResult`) are defined as types but not constructed in mock data or tested beyond type-level existence.
- **No `DivergenceMeasure` computation logic.** Only the type definition exists. No statistical computation.
- **No async/streaming.** Batch construction via builders only.
- **No production code.** This is a throwaway research artifact. It will be discarded when the research question is resolved.
