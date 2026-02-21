use serde::{Deserialize, Serialize};

use crate::common::{
    ComparisonOutcome, EventId, EventKindTag, ExecutionOutcome, MatchStatus, NumericalEventType,
    ObservationMode, Severity, SnapshotType, Value,
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
