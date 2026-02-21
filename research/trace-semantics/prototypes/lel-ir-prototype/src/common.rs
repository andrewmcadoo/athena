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
