# Candidate IR Schemas for the Trace Semantics Engine

**Investigation:** Trace Semantics Engine — IR Design (Step 5a)
**Date:** 2026-02-20
**Input Documents:**
- `requirements-coverage-matrix.md`: R1-R29 definitions, per-framework coverage codes, gap analysis, Decision Gate 4
- `ir-pattern-catalog.md`: 7 transferable patterns (P1-P7), 9 anti-patterns (AP1-AP9), 3 candidate previews (LEL/DGR/TAL), unified architecture, Decision Gate 2
- `cross-framework-synthesis.md`: Trace capability matrix, failure mode taxonomy (49 modes), adapter contract, Decision Gate 1
- `FINDINGS.md`: Accumulated findings (40 items), 29 requirements (R1-R29), investigation logs
- `decisions/001-python-rust-core.md`: Rust for Trace Semantics Engine, PyO3/maturin interop

---

## Section 0: Preamble

### Purpose

This document presents concrete IR schema designs for the Trace Semantics Engine, synthesizing the full body of evidence from Steps 1-4 and synthesis steps 1d, 2c, 3b into evaluable candidates with a recommendation for Step 5b prototyping.

### Three-Input Data Flow Architecture

The requirements coverage matrix (§8) established that the IR is not a pure trace-log derivative. It is a composite structure joining three data sources:

```
External Inputs (NT — 31% of R1-R29)     Trace Data (DA/DI)           Domain Knowledge (ER)
├── Experiment spec (R3,R10,R13,R22)      ├── Execution events (R1,R2)  ├── GROMACS param table (R19)
├── Hypothesis (R15,R23)                  ├── Numerical metrics (R6)    └── VASP INCAR table (R19)
├── DAG (R9,R11,R18)                      ├── Resource state (R7)
├── Cycle ID (R29)                        ├── Parameter echo (R3,R4)
└── Observation mode (R28)                ├── Energy/observables (R8,R16)
                                          └── Temporal markers (R21)
```

This three-input architecture is the organizing principle for all candidate evaluations. Candidates are assessed by how naturally they accommodate multi-source composition, not just trace parsing efficiency.

### Candidate Selection Rationale

**Two candidates + one hybrid, not three standalone.**

The pattern catalog (ir-pattern-catalog.md §6) previewed three candidates: LEL, DGR, TAL. The coverage matrix analysis (requirements-coverage-matrix.md §8) refined this:

- **TAL deferred to query layer.** The coverage matrix concluded TAL "works better as a query interface layer than a data representation" — it aligns well with the LFI's assertion-checking workflow but imposes the highest barrier for data extraction (converting DA/DI trace data into structured assertions). TAL has the highest novelty risk (no close precedent), the weakest causal graph traversal, and its core strength (sequential assertion checking ordered by audit stage) functions identically as a query interface over LEL or DGR substrates. TAL's assertion-checking pattern is preserved as a recommended query-layer interface for the LFI over whichever IR substrate is chosen.

- **LEL-DGR Hybrid added.** Open question #2 from the pattern catalog ("Can the unified architecture be incrementally implemented?") and open question #5 from the coverage matrix ("LEL→DGR incremental path") are architecturally informative and were unanswered. The Hybrid candidate addresses both: LEL as streaming storage with DGR-style graph overlay built on-demand for Stages 2-3. This is more informative than a high-risk novelty candidate (TAL standalone) because it tests the incremental adoption path.

---

## Section 1: Common Structural Foundation

All three candidates share these types. They are defined once and used by all candidates to ensure consistent semantics.

### Layer Classification

```rust
/// R19: The load-bearing structural distinction.
/// Every IR element is tagged with exactly one primary layer.
/// DSL API separation makes this possible (ARCHITECTURE.md §3.1).
enum Layer {
    Theory,         // User specifications, hypothesis predictions, causal claims
    Methodology,    // Experiment design, sampling config, variable control
    Implementation, // Framework execution, resource management, numerical computation
}
```

### Boundary Classification

```rust
/// Resolves Open Question #4: How should boundary parameters
/// (GROMACS dt, VASP PREC) be represented?
///
/// Rather than creating a "boundary" sub-dialect or duplicating entities,
/// the primary layer is assigned based on where the parameter's failure
/// would have the most diagnostic impact, with an explicit annotation
/// for dual-nature parameters.
enum BoundaryClassification {
    /// Unambiguously belongs to one layer.
    /// Examples: OpenMM ForceField (Theory), OpenMM Platform (Implementation)
    PrimaryLayer,

    /// Serves dual roles. Assigned a primary layer for routing but
    /// carries a cross-reference annotation to the secondary layer.
    /// Examples: GROMACS dt (primary: Methodology, secondary: Implementation),
    /// VASP PREC (primary: Theory, secondary: Implementation)
    DualAnnotated {
        secondary_layer: Layer,
        rationale: &'static str,
    },

    /// Classification depends on the specific system being simulated.
    /// Examples: VASP ALGO (Implementation for standard systems,
    /// Theory-adjacent for pathological systems where ALGO choice
    /// affects which SCF minimum is found)
    ContextDependent {
        default_layer: Layer,
        context_note: &'static str,
    },
}
```

### Observation Mode

```rust
/// R28: Interventional vs. observational distinction.
/// Critical for the hidden confounder litmus test —
/// confounders invisible in observational data become
/// visible under intervention.
enum ObservationMode {
    /// Parameter was actively varied as an experimental intervention
    Interventional,
    /// Parameter varied naturally or was passively observed
    Observational,
}
```

### Value Representation

```rust
/// R26: Observability gap representation.
/// From Boogie pattern P6 — explicitly unknown state is represented
/// structurally rather than silently omitted.
enum Value {
    /// Known value with units
    Known(f64, Unit),
    /// Known vector (e.g., forces, coordinates)
    KnownVec(Vec<f64>, Unit),
    /// Known categorical value (e.g., ensemble type, functional name)
    KnownCat(String),
    /// Explicitly unknown — the trace does not contain this value.
    /// Prevents the LFI from making assumptions about unobserved state.
    /// The reason field enables the LFI to distinguish "not logged"
    /// from "framework limitation" from "configuration omission."
    Havoc {
        expected_type: ValueType,
        reason: HavocReason,
    },
}

enum HavocReason {
    NotLogged,              // Reporter/output not configured
    FrameworkLimitation,    // Closed source, no API access
    ConfigurationOmission,  // User did not enable this output
    CrashStateGap,         // Lost due to crash before checkpoint
    TemporalGap {           // Value not captured in this time window
        last_known_step: u64,
        gap_steps: u64,
    },
}
```

### Temporal Coordinate

```rust
/// R21: Temporal ordering. Three coordinate systems to accommodate
/// both MD (step-based) and DFT (iteration-based) frameworks,
/// plus wall-clock for cross-framework comparison.
struct TemporalCoord {
    /// Simulation step (MD) or ionic/SCF iteration (DFT).
    /// Total ordering within a single experiment.
    simulation_step: u64,

    /// Wall clock time in nanoseconds since experiment start.
    /// Used for cross-framework comparison and performance analysis.
    wall_clock_ns: Option<u64>,

    /// Monotonic sequence number assigned by the IR during construction.
    /// Guarantees total ordering even when simulation_step is ambiguous
    /// (e.g., VASP ionic step 3 SCF step 7 vs. ionic step 4 SCF step 1).
    logical_sequence: u64,
}
```

### Provenance Anchor

```rust
/// R20: Every IR element is traceable to its source in raw trace data.
/// Enables LFI verification against raw evidence, enhanced logging re-runs
/// (ARCHITECTURE.md §5.3), and human escalation (ARCHITECTURE.md §6.3).
struct ProvenanceAnchor {
    /// Source file identifier (e.g., "vasprun.xml", "simulation.log")
    source_file: String,
    /// Line range or structural path (e.g., XPath for XML, line range for text)
    source_location: SourceLocation,
    /// Hash of the raw text/bytes for integrity verification
    raw_hash: u64,
}

enum SourceLocation {
    LineRange { start: u32, end: u32 },
    XPath(String),
    BinaryOffset { start: u64, length: u64 },
    ApiQuery(String), // For OpenMM runtime API queries
    ExternalInput,    // For NT elements (experiment spec, hypothesis, DAG)
}
```

### Experiment Reference

```rust
/// R22, R29: Links every IR element to its experiment and cycle context.
/// Enables cross-experiment queryability for the 50-cycle litmus test.
struct ExperimentRef {
    /// Unique experiment identifier
    experiment_id: String,
    /// Cycle number within the ATHENA falsification loop (0-indexed)
    cycle_id: u32,
    /// Hypothesis under test in this cycle
    hypothesis_id: String,
}
```

### Confidence Metadata

```rust
/// R25: Classification confidence for each IR element.
/// Enables the LFI to compute classification confidence
/// and trigger escalation when evidence is insufficient.
struct ConfidenceMeta {
    /// Was this element fully observed or partially inferred?
    completeness: Completeness,
    /// What fraction of expected fields are populated (vs. Havoc)?
    field_coverage: f32,
    /// Framework-specific confidence notes
    notes: Vec<String>,
}

enum Completeness {
    FullyObserved,
    PartiallyInferred { inference_method: String },
    ExternallyProvided, // For NT elements
    Derived { from_elements: Vec<ElementId> },
}
```

---

## Section 2: Candidate 1 — Layered Event Log (LEL)

### Design Philosophy

LEL represents the IR as a flat, append-only sequence of typed, layer-tagged events. It is the lowest-impedance translation of raw DSL trace output into a structured representation. The design prioritizes Stage 1 (implementation audit) efficiency and streaming compatibility.

### Pattern Sources

- Pattern 3 (Typed event chains) — primary structure
- Pattern 5 (Dialect system) — layer tags on events
- Pattern 4 (SSA data flow) — unique event IDs with version tracking
- Pattern 1 (Counter-example traces) — state-transition sequences

### Core Structures

```rust
/// The top-level LEL IR container.
struct LayeredEventLog {
    /// The experiment this log belongs to
    experiment_ref: ExperimentRef,

    /// The experiment specification, parsed before trace processing.
    /// First-class entity per Pattern 6 — prevents AP1 (spec-impl conflation).
    spec: ExperimentSpec,

    /// The ordered event stream. Append-only during construction.
    events: Vec<TraceEvent>,

    /// Secondary indexes built during construction for R24 queryability.
    /// These are optional acceleration structures, not primary data.
    indexes: EventIndexes,
}

/// Experiment specification as a first-class entity.
/// Prevents AP1 (specification-implementation conflation) by structurally
/// separating "what was intended" from "what happened."
struct ExperimentSpec {
    /// Pre-conditions: what must hold before execution
    /// (valid inputs, correct precision mode, hardware requirements)
    preconditions: Vec<ContractTerm>,

    /// Post-conditions: what must hold after execution
    /// (predicted observable values, expected relationships)
    postconditions: Vec<ContractTerm>,

    /// Predictions: hypothesis-derived quantitative predictions (R15)
    predictions: Vec<PredictionRecord>,

    /// Interventions: which parameters were varied and how (R10)
    interventions: Vec<InterventionRecord>,

    /// Controlled variables: what was held constant (R13)
    controlled_variables: Vec<ControlledVariable>,

    /// DAG references for cross-referencing (R9, R11)
    dag_refs: Vec<DagReference>,

    provenance: ProvenanceAnchor,
}

/// Each trace event has an SSA-like unique ID, a layer tag,
/// a typed event kind, temporal coordinates, and optional
/// causal references to prior events.
struct TraceEvent {
    /// Unique, immutable identifier. Monotonically increasing.
    /// SSA-like: each event is "assigned once" and never modified.
    id: EventId,

    /// Primary layer classification
    layer: Layer,

    /// Boundary classification for dual-nature events
    boundary: BoundaryClassification,

    /// The typed event payload
    kind: EventKind,

    /// When this event occurred
    temporal: TemporalCoord,

    /// Optional references to causally prior events.
    /// These are "best effort" — LEL does not require exhaustive
    /// causal annotation. Events without causal_refs are causally
    /// ordered only by temporal sequence.
    causal_refs: Vec<EventId>,

    /// Optional reference to a DAG node this event relates to (R9, R11)
    dag_node_ref: Option<String>,

    /// Optional reference to the spec element this event realizes
    spec_ref: Option<SpecElementId>,

    /// Source traceability
    provenance: ProvenanceAnchor,

    /// Confidence metadata
    confidence: ConfidenceMeta,
}

/// Event types mapped to requirements R1-R7, R8, R12, R16.
/// One variant per requirement class, ensuring exhaustive coverage.
enum EventKind {
    // === Stage 1: Implementation Audit (R1-R7) ===

    /// R1: Execution completed or terminated abnormally
    ExecutionStatus {
        status: ExecutionOutcome,
        framework_error_id: Option<String>,
    },

    /// R2: Exception or error event from the DSL framework
    ExceptionEvent {
        exception_type: String,
        component: String,
        dsl_call_path: Vec<String>,
        message: String,
        severity: Severity,
    },

    /// R3/R4: Input parameter — specification value and/or actual value
    ParameterRecord {
        name: String,
        specified_value: Option<Value>,
        actual_value: Value,
        units: Option<String>,
        observation_mode: ObservationMode,
    },

    /// R5: Input validation result (derived from R3+R4 comparison)
    ValidationResult {
        parameter_name: String,
        match_status: MatchStatus,
        deviation_detail: Option<String>,
    },

    /// R6: Numerical health status
    NumericalStatus {
        event_type: NumericalEventType,
        affected_quantity: String,
        severity: Severity,
        detail: Value,
    },

    /// R7: Resource and environment status
    ResourceStatus {
        platform_type: String,
        device_ids: Vec<String>,
        memory_allocated: Option<Value>,
        memory_peak: Option<Value>,
        parallelization: Option<String>,
        warnings: Vec<String>,
    },

    // === Stage 2: Methodology Audit (R8, R12) ===

    /// R8/R16: Observable measurement
    ObservableMeasurement {
        variable_name: String,
        measurement_method: String,
        value: Value,
        uncertainty: Option<Value>,
        conditions: String,
        observation_mode: ObservationMode,
    },

    /// R12: Sampling metadata
    SamplingMetadata {
        sample_count: u64,
        sampling_method: String,
        equilibration_steps: Option<u64>,
        autocorrelation_time: Option<Value>,
        statistical_power: Option<Value>,
    },

    // === Stage 3: Theory Evaluation (R17) ===

    /// R17: Prediction-observation comparison result (derived)
    ComparisonResult {
        prediction_id: String,
        observation_id: EventId,
        result: ComparisonOutcome,
    },

    // === Convergence and State ===

    /// Convergence trajectory point (SCF, ionic, constraint)
    ConvergencePoint {
        iteration: u64,
        metric_name: String,
        metric_value: Value,
        converged: Option<bool>,
    },

    /// State snapshot (coordinates, velocities, forces at a timestep)
    StateSnapshot {
        snapshot_type: SnapshotType,
        data_ref: String, // Reference to external binary data
    },

    /// Energy decomposition at a timestep
    EnergyRecord {
        total: Value,
        components: Vec<(String, Value)>,
    },
}

/// Secondary indexes for R24 queryability.
struct EventIndexes {
    by_layer: HashMap<Layer, Vec<EventId>>,
    by_kind: HashMap<EventKindTag, Vec<EventId>>,
    by_time_range: BTreeMap<u64, EventId>, // simulation_step -> first event at that step
    by_variable: HashMap<String, Vec<EventId>>,
    by_dag_node: HashMap<String, Vec<EventId>>,
}
```

### Strengths

- **Streaming (AP2 avoidance):** Pure append-only construction. Events are parsed, typed, and appended as they arrive from the adapter. No forward references, no graph construction overhead. Ideal for megabyte-scale traces under Rust zero-copy parsing (ADR 001).
- **Stage 1 efficiency:** Implementation-layer events are directly filterable via the `by_layer` index. R1-R7 are answered by scanning implementation-tagged events — a single-pass operation.
- **Low complexity:** `Vec<TraceEvent>` is the most cache-friendly representation. O(1) append, O(n) sequential scan with early termination for Stage 1 (if implementation fault found, no need to continue).
- **Minimal impedance mismatch:** DSL frameworks emit event streams; LEL stores event streams. The adapter's job is classification (layer tagging, event typing), not structural transformation.

### Weaknesses

- **Causal reasoning requires sequential search:** Without graph structure, "which prior events influenced this outcome?" requires scanning causal_refs chains sequentially. This is O(n) per query vs. O(path_length) for graph traversal. The optional causal_refs field provides some structure but is not exhaustive.
- **Stage 2/3 cross-referencing is awkward:** R14 (confounder query support) requires joining observables, interventions, controlled variables, and DAG structure. In LEL, this requires multiple index lookups and manual correlation — the log does not naturally support these multi-way joins.
- **Specification is a separate entity:** The ExperimentSpec is structurally separated from the event stream (by design, to prevent AP1), but this means spec-vs-event queries require cross-referencing two structures.

---

## Section 3: Candidate 2 — Dual-Graph IR (DGR)

### Design Philosophy

DGR represents the IR as a pair of graphs: a prospective graph (the experiment specification and hypothesis predictions) and a retrospective graph (the actual execution trace). Both graphs use the Entity-Activity-Agent model from PROV-DM (Pattern 2), organized by layer/dialect (Pattern 5), with contract terms (Pattern 6) bridging the two graphs. The design prioritizes causal reasoning for Stages 2-3 and natural multi-source entity representation.

### Pattern Sources

- Pattern 5 (Dialect system) — primary organization into three-layer graphs
- Pattern 2 (Entity-Activity-Agent) — PROV-DM-like causal graph structure
- Pattern 6 (Contracts) — specification entities with requires/ensures
- Pattern 7 (Conformance checking) — expected-vs-actual via graph alignment

### Core Structures

```rust
/// The top-level DGR IR container.
struct DualGraphIR {
    /// The experiment this IR belongs to
    experiment_ref: ExperimentRef,

    /// Prospective graph: what was specified, predicted, intended.
    /// Built before trace processing begins from experiment spec + hypothesis.
    spec_graph: ProvenanceGraph,

    /// Retrospective graph: what actually happened during execution.
    /// Built incrementally during trace processing.
    exec_graph: ProvenanceGraph,

    /// Lowering relations connecting spec → exec entities.
    /// "This theory-level specification was realized by these
    /// implementation-level events." Carries diagnostic payload:
    /// divergence between specification and realization.
    lowering_relations: Vec<LoweringRelation>,

    /// Ghost variables: methodology metadata from external sources.
    /// From Why3 pattern P4 — properties not in the execution trace
    /// but needed for Stage 2 (sampling sufficiency, confounder control).
    ghost_vars: Vec<GhostVar>,
}

/// A provenance graph (used for both spec_graph and exec_graph).
/// Arena-allocated for cache-friendly traversal.
struct ProvenanceGraph {
    /// All entities in this graph
    entities: Arena<Entity>,

    /// All activities in this graph
    activities: Arena<Activity>,

    /// All agents in this graph
    agents: Arena<TypedAgent>,

    /// Derivation edges: entity wasDerivedFrom entity
    /// The primary causal reasoning substrate.
    derivations: Vec<Derivation>,

    /// Usage edges: activity used entity (with role)
    usages: Vec<Usage>,

    /// Generation edges: entity wasGeneratedBy activity
    generations: Vec<Generation>,

    /// Attribution edges: entity wasAttributedTo agent
    attributions: Vec<Attribution>,

    /// Indexes for R24 queryability
    entity_by_layer: HashMap<Layer, Vec<EntityId>>,
    entity_by_dag_node: HashMap<String, Vec<EntityId>>,
    entity_by_variable: HashMap<String, Vec<EntityId>>,
}

/// An entity in the provenance graph.
/// Entities are things with fixed aspects — data, parameters, results.
struct Entity {
    id: EntityId,

    /// Primary layer classification
    layer: Layer,

    /// Boundary classification
    boundary: BoundaryClassification,

    /// The kind of entity
    kind: EntityKind,

    /// Optional reference to a DAG node (R9, R11)
    dag_node_ref: Option<String>,

    /// The entity's value
    value: Value,

    /// Temporal coordinate (when this entity came into existence)
    temporal: TemporalCoord,

    /// Observation mode (R28)
    observation_mode: ObservationMode,

    /// Source traceability
    provenance: ProvenanceAnchor,

    /// Confidence metadata
    confidence: ConfidenceMeta,
}

/// Entity kinds mapped to R1-R29 requirements.
enum EntityKind {
    // Stage 1
    ExecutionStatus(ExecutionOutcome),      // R1
    ExceptionRecord(ExceptionDetail),       // R2
    ParameterSpec(ParameterDetail),         // R3
    ParameterActual(ParameterDetail),       // R4
    ValidationResult(MatchStatus),          // R5
    NumericalMetric(NumericalDetail),       // R6
    ResourceState(ResourceDetail),          // R7

    // Stage 2
    Observable(ObservableDetail),           // R8, R16
    Intervention(InterventionDetail),       // R10
    ControlledVariable(ControlDetail),      // R13
    SamplingRecord(SamplingDetail),         // R12

    // Stage 3
    Prediction(PredictionDetail),           // R15
    Comparison(ComparisonDetail),           // R17
    CausalImplication(ImplicationDetail),   // R18

    // Structural
    EnergyValue(EnergyDetail),
    ConvergencePoint(ConvergenceDetail),
    StateSnapshot(SnapshotDetail),
}

/// An activity in the provenance graph.
/// Activities are time-bounded operations — simulation steps, computations.
struct Activity {
    id: ActivityId,

    /// Primary layer classification
    layer: Layer,

    /// The kind of activity
    kind: ActivityKind,

    /// Contract terms for this activity (Pattern 6).
    /// Preconditions that must hold before, postconditions after.
    contract: ContractTerms,

    /// Temporal bounds
    started: TemporalCoord,
    ended: Option<TemporalCoord>,

    /// Source traceability
    provenance: ProvenanceAnchor,
}

/// Contract terms attached to activities (Pattern 6).
struct ContractTerms {
    /// What must hold before this activity
    preconditions: Vec<ContractTerm>,
    /// What must hold after this activity
    postconditions: Vec<ContractTerm>,
    /// What this activity may change
    modifies: Vec<String>,
}

/// Activity kinds
enum ActivityKind {
    // Theory layer
    SpecifyHypothesis,
    PredictObservable,
    DefineEnsemble,

    // Methodology layer
    ConfigureExperiment,
    Sample,
    Intervene,
    Measure,

    // Implementation layer
    LoadData,
    ComputeForces,
    Integrate,
    AllocateMemory,
    Checkpoint,
    Validate,
}

/// A typed agent — avoids AP4 (untyped agent proliferation).
/// Each agent is typed by layer, enabling the LFI to query
/// "all implementation-layer agents" directly.
struct TypedAgent {
    id: AgentId,

    /// Layer determines which audit stage is responsible for this agent
    layer: Layer,

    /// Agent description
    kind: AgentKind,
}

enum AgentKind {
    TheoryAgent(String),        // User's scientific specification
    MethodologyAgent(String),   // Experiment design system
    ImplementationAgent(String), // DSL framework, hardware platform
}

/// Lowering relation connecting spec_graph entities to exec_graph entities.
/// "This specification element was realized by these execution elements."
struct LoweringRelation {
    /// Entity in the spec_graph
    spec_entity: EntityId,

    /// Entity/entities in the exec_graph that realize the spec
    exec_entities: Vec<EntityId>,

    /// The nature of the realization
    relation_type: LoweringType,

    /// Divergence measure when realization differs from specification
    divergence: Option<DivergenceMeasure>,
}

enum LoweringType {
    /// Spec faithfully realized by execution
    Realization,
    /// Execution deviated from spec
    Deviation { detail: String },
    /// Spec only partially realized (some elements missing/havoc)
    Partial { coverage: f32 },
}

/// R17 structural slot — resolves Open Question #1.
/// The comparison method is pluggable, not fixed in the IR.
/// The IR provides the structural container; the comparison logic
/// is supplied by the LFI at Stage 3 evaluation time.
struct ComparisonResult {
    /// Which prediction was being tested
    prediction_entity: EntityId,
    /// Which observation was compared
    observation_entity: EntityId,
    /// The divergence measure used
    divergence: DivergenceMeasure,
    /// Whether the observation falls within the prediction's tolerance
    within_tolerance: bool,
    /// The comparison method that produced this result
    method: String,
}

/// Divergence measures for prediction-observation comparison.
/// This enum provides structural slots; the actual computation
/// is external to the IR. Each variant carries a numeric value
/// and the method used to compute it.
enum DivergenceMeasure {
    /// Raw difference (observation - prediction)
    AbsoluteDifference { value: f64, units: String },
    /// Normalized difference relative to prediction uncertainty
    ZScore { value: f64 },
    /// Bayes factor for divergence
    BayesFactor { value: f64 },
    /// KL divergence between predicted and observed distributions
    KLDivergence { value: f64 },
    /// Effect size (Cohen's d or similar)
    EffectSize { value: f64, measure: String },
    /// Custom comparison with method description
    Custom { value: f64, method: String },
}

/// Ghost variable for methodology metadata (Why3 Pattern P4).
/// Properties not in the execution trace but needed for Stage 2.
struct GhostVar {
    name: String,
    layer: Layer, // Always Methodology for ghost vars
    value: Value,
    source: GhostVarSource,
    dag_node_ref: Option<String>,
}

enum GhostVarSource {
    ExperimentSpec,
    DagDerived,
    DomainRule(String),
}
```

### Strengths

- **Causal reasoning is structural:** `wasDerivedFrom` chains enable "which prior entities influenced this outcome?" as a graph traversal — O(path_length) vs. O(n) sequential search in LEL. This is the primary advantage for Stages 2-3, where the LFI must trace causal paths through specifications, interventions, observables, and confounders.
- **Natural multi-source representation:** The three-input data flow (trace + spec + hypothesis + DAG) maps naturally to graph entities with typed edges. NT elements (31% of requirements) are entities in the spec_graph. DA/DI elements are entities in the exec_graph. Cross-references are edges between graphs.
- **Specification-execution structural separation:** The spec_graph/exec_graph split directly prevents AP1 (specification-implementation conflation). The two are separate graph structures connected only by explicit lowering relations.
- **R17 comparison is a structural slot:** ComparisonResult + DivergenceMeasure provides the container for prediction-observation comparison. The comparison method is pluggable — the IR stores the result, not the logic. This resolves Open Question #1 structurally without requiring the comparison formalization to be complete.
- **Cross-experiment analysis:** Graph structure supports cross-cycle entity linking via shared variable names and DAG node references. R29 (cross-experiment queryability) is naturally supported.

### Weaknesses

- **Higher construction complexity:** Converting raw trace events into graph entities requires entity resolution (identifying which trace entries correspond to the same logical entity), relationship extraction (determining derivation chains from temporal co-occurrence and API structure), and graph construction (node/edge creation with proper typing). This is substantially more complex than LEL's append-and-classify approach.
- **Streaming challenges:** The spec_graph must be pre-built before trace processing begins (it comes from the experiment spec + hypothesis, both available before execution). The exec_graph can be built incrementally for forward-flowing traces, but iterative workflows (convergence loops with back-references) may require buffering. Forward references are managed by pre-building the spec_graph and resolving exec_graph references as entities are created.
- **Memory overhead:** Each entity carries Layer, BoundaryClassification, Value, TemporalCoord, ProvenanceAnchor, and ConfidenceMeta. Graph edges add per-edge storage. Estimated ~200-800 bytes per entity + edges vs. ~100-500 bytes per LEL event.
- **Over-engineering risk for Stage 1:** The full Entity-Activity-Agent model with qualified relations may be more structure than the LFI needs for Stage 1 (implementation audit), where a simple event scan suffices.

---

## Section 4: Candidate 3 — LEL-DGR Hybrid

### Design Philosophy

The Hybrid uses LEL as the primary storage layer (streaming-compatible, efficient for Stage 1) with a DGR-style causal overlay built on demand when the LFI advances to Stages 2-3. This addresses Open Question #2 (incremental LEL→DGR path) and provides the best combined stage coverage.

### Architecture

```rust
/// The top-level Hybrid IR container.
struct HybridIR {
    /// The experiment this IR belongs to
    experiment_ref: ExperimentRef,

    /// The experiment specification (shared with both layers).
    spec: ExperimentSpec,

    /// Primary storage: LEL event stream.
    /// Always available. Used directly for Stage 1.
    events: Vec<TraceEvent>,

    /// LEL indexes for Stage 1 queryability
    indexes: EventIndexes,

    /// On-demand causal overlay for Stages 2-3.
    /// None until Stage 2 entry triggers construction.
    overlay: Option<CausalOverlay>,
}

/// The causal overlay built from LEL events + spec when Stage 2 begins.
/// Construction cost: O(n) pass over events to build graph edges.
struct CausalOverlay {
    /// Entities derived from LEL events (references back to EventIds)
    entities: Arena<OverlayEntity>,

    /// Derivation edges between entities
    derivations: Vec<Derivation>,

    /// Lowering relations from spec elements to trace events
    lowering_relations: Vec<LoweringRelation>,

    /// Ghost variables for methodology metadata
    ghost_vars: Vec<GhostVar>,

    /// Graph indexes for causal queries
    entity_by_dag_node: HashMap<String, Vec<OverlayEntityId>>,
    causal_ancestors: HashMap<OverlayEntityId, Vec<OverlayEntityId>>,
}

/// An entity in the overlay. Wraps an LEL event with graph relationships.
struct OverlayEntity {
    id: OverlayEntityId,

    /// Reference back to the source LEL event
    event_id: EventId,

    /// Layer (copied from the LEL event for graph-local queries)
    layer: Layer,

    /// DAG node reference (copied from LEL event)
    dag_node_ref: Option<String>,

    /// Spec element reference (copied from LEL event)
    spec_ref: Option<SpecElementId>,
}
```

### Operational Modes

**Stage 1 (pure LEL operation):**
- Adapter produces `TraceEvent`s with layer tags, event kinds, temporal coordinates.
- Events appended to `events` vec. Indexes updated incrementally.
- LFI Stage 1 queries use `indexes.by_layer[Implementation]` to find implementation events.
- Stage 1 checks R1-R7 by scanning implementation-tagged events.
- If Stage 1 finds a fault → classification complete. No overlay needed. **Early termination.**

**Stage 1→2 boundary (overlay construction):**
- Stage 1 passes → LFI advances to Stage 2.
- Overlay construction triggered: single O(n) pass over events.
- For each event with `dag_node_ref` or `spec_ref`: create overlay entity, extract derivation edges from causal_refs.
- Build lowering_relations from spec elements to their realizing events.
- Populate ghost_vars from experiment spec + DAG context.
- **Key constraint:** LEL events must carry `dag_node_ref` and `spec_ref` from the start. These fields enable overlay construction without re-parsing raw trace data. This means the adapter must classify references during initial event construction, not defer them.

**Stages 2-3 (graph traversal via overlay):**
- LFI Stage 2/3 queries use the overlay's graph structure.
- R14 (confounder query): traverse derivation chains to find common ancestors of intervention and observable entities, check against controlled variable set.
- R17 (comparison): locate prediction entities in spec, corresponding observation entities in overlay, produce ComparisonResult.
- R18 (causal implication): traverse overlay derivation chains from falsified prediction back to implicated DAG edges.

### Strengths

- **Best combined stage coverage:** Stage 1 operates at LEL efficiency (pure append-only, sequential scan). Stages 2-3 operate at DGR graph-traversal efficiency (structural causal queries).
- **Streaming compatible for Stage 1:** The most common path (implementation fault detected at Stage 1) never builds a graph. Only when Stage 1 passes and the analysis continues to Stage 2 is the overlay constructed.
- **Incremental adoption path:** Validates LEL's viability for Stage 1 first (the simplest, most tractable audit). Graph structure added only when needed, proving the incremental path from LEL to DGR-like capability.
- **Memory-efficient for common case:** Most experiment traces that contain faults will be classified at Stage 1 (if Sakana V2's 41% implementation error rate is indicative). These traces never incur graph construction or storage overhead.

### Weaknesses

- **Dual representation at Stages 2-3:** After overlay construction, both the LEL event stream and the overlay entities exist in memory. The overlay entities are lightweight (references back to LEL events), but the graph edges and indexes add memory overhead. Estimated: LEL base + ~50-100 bytes per entity in overlay + edge storage.
- **Stage 1/2 boundary is a synchronization point:** The overlay must be fully constructed before Stage 2 queries begin. This is an O(n) pass — fast for typical traces (milliseconds for 10^4-10^5 events) but creates a latency boundary.
- **Adapter must classify references upfront:** The `dag_node_ref` and `spec_ref` fields on LEL events must be populated during initial event construction. This pushes some of DGR's entity resolution complexity into the adapter, even during Stage 1 when these references are not yet needed.
- **Overlay correctness depends on LEL event quality:** If causal_refs, dag_node_ref, or spec_ref are incomplete or incorrect in the LEL events, the overlay will be incomplete. Unlike pure DGR, where entity resolution is done during graph construction with full context, the Hybrid defers resolution to the adapter.

---

## Section 5: R1-R29 Coverage Matrix

Each row maps a requirement to specific structural elements in each candidate. Coverage ratings: **STRONG** (native structural support, no workaround needed), **MODERATE** (supported but requires additional logic or cross-referencing), **WEAK** (possible but awkward, requires external joins or sequential search).

### Stage 1: Implementation Audit (R1-R7)

| Req | LEL | DGR | Hybrid |
|:---|:---|:---|:---|
| **R1** Execution status | **STRONG** — `EventKind::ExecutionStatus` variant, directly filterable by layer index | **STRONG** — `EntityKind::ExecutionStatus` in exec_graph, filterable by layer | **STRONG** — LEL path: same as LEL |
| **R2** Exception event | **STRONG** — `EventKind::ExceptionEvent` variant | **STRONG** — `EntityKind::ExceptionRecord` entity | **STRONG** — LEL path |
| **R3** Input spec | **STRONG** — `ExperimentSpec.preconditions` + `EventKind::ParameterRecord.specified_value` | **STRONG** — `ParameterSpec` entities in spec_graph | **STRONG** — LEL path (spec is shared) |
| **R4** Actual input | **STRONG** — `EventKind::ParameterRecord.actual_value` | **STRONG** — `ParameterActual` entities in exec_graph | **STRONG** — LEL path |
| **R5** Validation result | **STRONG** — `EventKind::ValidationResult` (derived from R3+R4) | **STRONG** — `ValidationResult` entity (derived) | **STRONG** — LEL path |
| **R6** Numerical status | **STRONG** — `EventKind::NumericalStatus` variant | **STRONG** — `NumericalMetric` entity | **STRONG** — LEL path |
| **R7** Resource status | **STRONG** — `EventKind::ResourceStatus` variant | **STRONG** — `ResourceState` entity | **STRONG** — LEL path |

**Stage 1 summary:** All candidates are STRONG for all R1-R7 requirements. Stage 1 is the most tractable stage and all designs handle it well. LEL and Hybrid have slight efficiency advantages due to simpler append-only construction.

### Stage 2: Methodology Audit (R8-R14)

| Req | LEL | DGR | Hybrid |
|:---|:---|:---|:---|
| **R8** Observable measurement | **STRONG** — `EventKind::ObservableMeasurement` | **STRONG** — `Observable` entity | **STRONG** — LEL event, accessible via overlay |
| **R9** Observable-to-DAG linkage | **MODERATE** — `dag_node_ref` on events, requires index lookup + manual join to DAG | **STRONG** — `dag_node_ref` on entities, graph traversal enables structural join | **STRONG** — overlay entity `dag_node_ref` with graph traversal |
| **R10** Intervention spec | **STRONG** — `ExperimentSpec.interventions` | **STRONG** — `Intervention` entity in spec_graph | **STRONG** — via shared ExperimentSpec |
| **R11** Intervention-to-DAG linkage | **MODERATE** — index lookup + manual join | **STRONG** — graph traversal from intervention entity to DAG node | **STRONG** — overlay graph traversal |
| **R12** Sampling metadata | **STRONG** — `EventKind::SamplingMetadata` | **STRONG** — `SamplingRecord` entity | **STRONG** — LEL event |
| **R13** Controlled variable set | **STRONG** — `ExperimentSpec.controlled_variables` | **STRONG** — `ControlledVariable` entities in spec_graph | **STRONG** — via shared ExperimentSpec |
| **R14** Confounder query | **WEAK** — requires multi-index lookup across interventions, observables, controlled vars, and DAG; no structural join | **STRONG** — graph traversal: find common ancestors of intervention and observable entities, check against controlled variable entities | **STRONG** — overlay graph traversal |

**Stage 2 summary:** LEL is WEAK for R14 (the core confounder detection query) because confounder detection requires multi-way structural joins that a flat log cannot natively support. DGR and Hybrid are STRONG because graph traversal supports these joins structurally.

### Stage 3: Theory Evaluation (R15-R18)

| Req | LEL | DGR | Hybrid |
|:---|:---|:---|:---|
| **R15** Prediction record | **STRONG** — `ExperimentSpec.predictions` | **STRONG** — `Prediction` entity in spec_graph | **STRONG** — via shared ExperimentSpec |
| **R16** Observation record | **STRONG** — `EventKind::ObservableMeasurement` (subset relevant to predictions) | **STRONG** — `Observable` entity linked to prediction via derivation | **STRONG** — overlay links observation to prediction |
| **R17** Comparison result | **MODERATE** — `EventKind::ComparisonResult` stores the result; computing it requires cross-referencing spec predictions with observation events | **STRONG** — `ComparisonResult` struct with `DivergenceMeasure`; graph structure links prediction→observation→comparison | **STRONG** — overlay enables structural comparison |
| **R18** Causal implication | **WEAK** — requires manual backward traversal through causal_refs to find implicated DAG edges; no structural support for transitive causal ancestry | **STRONG** — `CausalImplication` entity; graph traversal from falsified prediction through derivation chains to implicated DAG edges | **STRONG** — overlay graph traversal |

**Stage 3 summary:** LEL is WEAK for R18 (causal implication mapping) because transitive causal ancestry requires graph traversal. DGR and Hybrid are STRONG due to structural derivation chains.

### Cross-Cutting Requirements (R19-R29)

| Req | LEL | DGR | Hybrid |
|:---|:---|:---|:---|
| **R19** Layer tag | **STRONG** — `TraceEvent.layer` enum | **STRONG** — `Entity.layer` enum | **STRONG** — both LEL event and overlay entity carry layer |
| **R20** Provenance chain | **STRONG** — `ProvenanceAnchor` on every event | **STRONG** — `ProvenanceAnchor` on every entity | **STRONG** — ProvenanceAnchor on LEL events |
| **R21** Temporal ordering | **STRONG** — `TemporalCoord` on every event; total ordering by logical_sequence | **STRONG** — `TemporalCoord` on every entity | **STRONG** — TemporalCoord on LEL events |
| **R22** Experiment spec linkage | **STRONG** — `ExperimentSpec` is first-class | **STRONG** — spec_graph IS the specification | **STRONG** — shared ExperimentSpec |
| **R23** Hypothesis linkage | **MODERATE** — `ExperimentRef.hypothesis_id` provides linkage; spec predictions reference hypothesis elements | **STRONG** — hypothesis represented as entities in spec_graph with qualified relationships | **STRONG** — overlay inherits spec linkage |
| **R24** Queryability | **STRONG** — `EventIndexes` provides multi-index lookup | **STRONG** — graph indexes + traversal | **STRONG** — LEL indexes for Stage 1; overlay indexes for Stages 2-3 |
| **R25** Confidence metadata | **STRONG** — `ConfidenceMeta` on every event | **STRONG** — `ConfidenceMeta` on every entity | **STRONG** — on LEL events |
| **R26** Observability gap | **STRONG** — `Value::Havoc` on every value field | **STRONG** — `Value::Havoc` on every entity value | **STRONG** — Value::Havoc on LEL events |
| **R27** Confounder classification | **MODERATE** — requires combining R8+R13+R14; R14 is WEAK | **STRONG** — graph structure supports classification | **STRONG** — overlay supports classification |
| **R28** Observation mode | **STRONG** — `ObservationMode` on relevant events | **STRONG** — `ObservationMode` on relevant entities | **STRONG** — on LEL events |
| **R29** Cross-experiment query | **MODERATE** — `ExperimentRef.cycle_id` enables filtering; cross-experiment joins require external aggregation | **STRONG** — graph entities carry experiment_ref; cross-graph queries supported | **MODERATE** — overlay is per-experiment; cross-experiment requires aggregation |

### Coverage Summary

| Stage | LEL | DGR | Hybrid |
|:---|:---|:---|:---|
| Stage 1 (R1-R7) | **STRONG** (7/7) | **STRONG** (7/7) | **STRONG** (7/7) |
| Stage 2 (R8-R14) | MODERATE (5 STRONG, 2 MOD, 1 WEAK) | **STRONG** (7/7) | **STRONG** (7/7) |
| Stage 3 (R15-R18) | MODERATE (2 STRONG, 1 MOD, 1 WEAK) | **STRONG** (4/4) | **STRONG** (4/4) |
| Cross-cutting (R19-R29) | MODERATE (7 STRONG, 3 MOD) | **STRONG** (10/10 STRONG, 1 near-STRONG) | STRONG (8 STRONG, 2 MOD, 1 near-STRONG) |

---

## Section 6: Anti-Pattern Compliance Matrix

Each candidate evaluated against the 9 anti-patterns from ir-pattern-catalog.md §3.

| Anti-Pattern | Severity | LEL | DGR | Hybrid |
|:---|:---|:---|:---|:---|
| **AP1: Spec-Impl Conflation** | CRITICAL | **PASS** — ExperimentSpec is structurally separate from events. Spec elements are in a dedicated struct, not in the event stream. | **PASS** — spec_graph and exec_graph are separate graph structures. Lowering relations are the only bridge. | **PASS** — ExperimentSpec shared, separate from event stream. Overlay lowering_relations bridge spec→events. |
| **AP2: Post-Mortem Only** | HIGH | **PASS** — Pure append-only, streaming from first event. No buffering required for Stage 1. | **PARTIAL** — spec_graph pre-built (acceptable: from experiment spec, available before execution). exec_graph streaming-safe for forward-flowing traces. Cross-graph queries deferred until needed. | **PASS** — Stage 1 is pure streaming. Overlay construction deferred until Stage 2 entry. |
| **AP3: Full-Granularity** | HIGH | **PASS** — Events recorded at DSL API-call granularity, not internal computation granularity. Adapter controls granularity. | **PASS** — Entities at API-call granularity. Arena allocation prevents per-entity heap overhead. | **PASS** — Same granularity as LEL. |
| **AP4: Untyped Agent Proliferation** | MEDIUM | **N/A** — LEL does not use an Agent model. Layer tags on events provide equivalent routing. | **PASS** — TypedAgent enum enforces three-layer classification. Every agent typed as TheoryAgent/MethodologyAgent/ImplementationAgent. | **PASS** — Overlay inherits layer typing from LEL events. No untyped agents. |
| **AP5: Flat Event Namespace** | MEDIUM | **PASS** — EventKind enum provides typed event hierarchy. Layer tag enables structural routing to audit stages. | **PASS** — EntityKind + Layer provide typed, routed entities. | **PASS** — LEL EventKind + Layer for Stage 1; overlay entity types for Stages 2-3. |
| **AP6: Binary Pass/Fail** | HIGH | **PASS** — EventKind variants carry structured failure records (exception type, numerical event type, severity, detail). ComparisonResult carries DivergenceMeasure. | **PASS** — ComparisonResult + DivergenceMeasure provide structured three-way classification support. | **PASS** — Both LEL events and overlay provide structured failure representation. |
| **AP7: Implicit Causal Ordering** | MEDIUM | **PARTIAL** — causal_refs on events provide optional explicit causal links, but they are not exhaustive. Temporal ordering is always present; structural causal ordering is best-effort. | **PASS** — Derivation edges (wasDerivedFrom) provide structural causal ordering. Causal relationships are explicit graph edges, not inferred from temporal proximity. | **PASS** — LEL causal_refs during Stage 1 (partial); overlay derivation edges during Stages 2-3 (structural). Overall: PASS because Stages 2-3, where causal reasoning matters most, use structural edges. |
| **AP8: Lossy Compression** | HIGH | **PASS** — Value::Havoc explicitly marks omitted data. ConfidenceMeta tracks field_coverage. No silent omission. | **PASS** — Value::Havoc on entities. Ghost variables for methodology metadata. No silent omission. | **PASS** — Same as LEL (Havoc + ConfidenceMeta). |
| **AP9: RDF Triple Store** | MEDIUM | **PASS** — Rust-native Vec + HashMap. No RDF. | **PASS** — Rust-native arena-allocated graph. PROV-DM concepts adopted without RDF technology stack. | **PASS** — Rust-native for both LEL and overlay. |

### Anti-Pattern Summary

| Candidate | PASS | PARTIAL | FAIL |
|:---|:---|:---|:---|
| LEL | 7 | 1 (AP7) | 0 |
| DGR | 8 | 1 (AP2) | 0 |
| Hybrid | 9 | 0 | 0 |

**Key differentiator:** AP7 (implicit causal ordering). LEL is PARTIAL because causal_refs are optional best-effort. DGR and Hybrid are PASS because structural derivation edges provide explicit causal relationships where they matter most (Stages 2-3).

All three candidates PASS on AP1 (CRITICAL), which is the most important result.

---

## Section 7: Streaming Compatibility

Per ADR 001, the Trace Semantics Engine must support Rust zero-copy/streaming parsing of megabyte-scale traces.

### LEL: EXCELLENT

- **Construction model:** Pure append-only. Events parsed from trace data, typed, layer-tagged, and appended to `Vec<TraceEvent>`. Indexes updated incrementally. No forward references, no graph construction.
- **Memory model:** Each event is self-contained (~100-500 bytes depending on payload). No inter-event references required for storage (causal_refs are optional links, not structural dependencies).
- **Stage 1 analysis:** Can begin before trace processing is complete. The LFI can inspect events as they are appended, enabling real-time Stage 1 fault detection.
- **Serde compatibility:** All structures are `#[derive(Serialize, Deserialize)]` compatible. PyO3 interop via serde_json or custom FromPyObject implementations.

### DGR: MODERATE

- **Construction model:** Two-phase. Phase 1: spec_graph built from experiment specification and hypothesis (before trace processing). Phase 2: exec_graph built incrementally during trace processing. Entity resolution (determining which trace entries correspond to the same logical entity) adds complexity.
- **Forward reference management:** The spec_graph is pre-built, so exec_graph entities that reference spec elements can resolve references immediately. Iterative workflows (convergence loops) may create entities that reference not-yet-created entities; these require deferred resolution via placeholder IDs resolved in a post-processing pass.
- **Memory model:** ~200-800 bytes per entity (entity + edges + indexes). Arena allocation amortizes per-entity allocation cost. Graph structure is less cache-friendly than LEL's Vec for sequential access but more efficient for graph traversal.
- **Cross-graph queries:** Lowering relations between spec_graph and exec_graph require both graphs to be available. This is satisfied by pre-building the spec_graph.
- **Serde compatibility:** Arena-allocated graphs require custom serialization. petgraph provides serde support; custom graph structures need explicit implementations.

### Hybrid: EXCELLENT (Stage 1), MODERATE (Stages 2-3)

- **Stage 1:** Identical to LEL. Pure append-only, streaming, real-time analysis possible.
- **Overlay construction:** Triggered at Stage 1→2 boundary. Single O(n) pass over events. For 10^4-10^5 events, this is milliseconds. For 10^6 events (megabyte-scale), this is tens of milliseconds.
- **Stages 2-3:** After overlay construction, behavior matches DGR for graph traversal queries.
- **Memory model:** LEL base (~100-500 bytes/event) + overlay overhead (~50-100 bytes/entity in overlay + edge storage). The overlay references LEL events rather than duplicating data, keeping additional memory modest.

### Memory Estimates

| Scale | LEL | DGR | Hybrid (Stage 1 only) | Hybrid (with overlay) |
|:---|:---|:---|:---|:---|
| 10^3 events (small experiment) | ~100-500 KB | ~200-800 KB | ~100-500 KB | ~150-600 KB |
| 10^4 events (typical MD run) | ~1-5 MB | ~2-8 MB | ~1-5 MB | ~1.5-6 MB |
| 10^5 events (long simulation) | ~10-50 MB | ~20-80 MB | ~10-50 MB | ~15-60 MB |
| 10^6 events (megabyte-scale) | ~100-500 MB | ~200-800 MB | ~100-500 MB | ~150-600 MB |

All estimates are within Rust's ability to handle efficiently per ADR 001. The primary concern at 10^6 scale is DGR's graph construction overhead, not memory.

### PyO3 Serialization

All candidates use Rust-native types (enums, structs, Vecs, HashMaps) that are serde-compatible. PyO3 interop options:
- **serde_json:** Serialize to JSON for flexible Python consumption. Highest compatibility, moderate performance.
- **Custom FromPyObject/IntoPyObject:** Direct Rust↔Python type conversion for zero-copy access to hot paths.
- **numpy interop (pyo3-numpy):** For bulk numerical data (energy time series, convergence trajectories).

---

## Section 8: Open Question Resolution

### OQ1: How should R17 (prediction-observation comparison) be formalized?

**Resolution: Structural slot with pluggable comparison method.**

The `ComparisonResult` struct + `DivergenceMeasure` enum provides the IR's structural container for comparison results. The IR stores the result of a comparison (which prediction, which observation, the divergence value, whether within tolerance) but does not fix the comparison logic.

The `DivergenceMeasure` enum provides six variants (AbsoluteDifference, ZScore, BayesFactor, KLDivergence, EffectSize, Custom) that cover the standard statistical comparison methods. The LFI selects the appropriate measure based on the prediction's type (point prediction → ZScore, distribution prediction → KLDivergence, directional prediction → EffectSize).

**What remains open:** The specific rules for selecting comparison methods and tolerance thresholds per prediction type. This is LFI logic, not IR structure — the IR provides the container, the LFI fills it.

### OQ2: Is the LEL→DGR incremental path viable?

**Resolution: Yes, the Hybrid candidate demonstrates viability.**

The Hybrid design proves the LEL→DGR incremental path by construction: LEL events carry `dag_node_ref` and `spec_ref` fields that enable overlay construction without re-parsing raw trace data. The Stage 1→2 boundary triggers a single O(n) pass that builds the overlay graph from existing LEL events.

**Key constraint identified:** LEL events must carry DGR-compatible references (`dag_node_ref`, `spec_ref`, `causal_refs`) from day one. If these fields are omitted during initial LEL implementation, the overlay cannot be constructed without re-parsing. This means the adapter must perform reference classification during initial event construction, even though these references are not used during Stage 1.

**Implication for Step 5b:** The LEL prototype must include `dag_node_ref`, `spec_ref`, and `causal_refs` fields on TraceEvent, even if the Stage 1 prototype does not use them. This ensures the incremental path remains viable.

### OQ3: Which causal reasoning substrate best matches the LFI's query patterns?

**Resolution: Per-stage answer.**

- **Stage 1:** Sequential search is sufficient. Stage 1 queries ("did execution complete?", "any exceptions?", "numerical health?") are filter-and-inspect operations on implementation-tagged events. These are O(n) scans with early termination — no causal reasoning required.
- **Stages 2-3:** Graph traversal is required. Stage 2 queries ("is the intervention on the hypothesized cause?", "are there uncontrolled confounders?") and Stage 3 queries ("which causal edges does the contradiction implicate?") require transitive causal ancestry, multi-way joins, and structural path finding. Sequential search over a flat log would be O(n) per query with complex join logic; graph traversal is O(path_length) with structural support.

This per-stage answer directly motivates the Hybrid design: LEL for Stage 1 (search-sufficient), overlay for Stages 2-3 (traversal-required).

### OQ4: How should boundary parameters be represented?

**Resolution: `BoundaryClassification` enum with three variants.**

- `PrimaryLayer`: Unambiguous. Most parameters fall here.
- `DualAnnotated`: For parameters like GROMACS `dt` (primary: Methodology, secondary: Implementation) and VASP `PREC` (primary: Theory, secondary: Implementation). The primary layer determines routing; the secondary layer is recorded as annotation for diagnostic context.
- `ContextDependent`: For parameters like VASP `ALGO` where classification depends on the system. The default layer is used for routing; the context note is available for the LFI to override when system-specific information is available.

This avoids both a "boundary" sub-dialect (which would create a fourth layer, breaking the three-stage audit structure) and entity duplication (which would complicate identity).

---

## Section 9: Evaluation Framework

Seven criteria, weighted by their importance to the LFI's effectiveness and ATHENA's architectural constraints.

| # | Criterion | Weight | Rationale |
|:---|:---|:---|:---|
| 1 | R1-R29 coverage | 25% | The IR's primary purpose is satisfying LFI requirements. Incomplete coverage means the LFI cannot function. |
| 2 | Anti-pattern compliance | 20% | Anti-patterns represent known failure modes. AP1 (CRITICAL) violation is disqualifying. |
| 3 | Streaming compatibility | 15% | Per ADR 001, megabyte-scale traces under Rust zero-copy parsing. Non-streaming designs are not viable for production. |
| 4 | Stage 1 efficiency | 15% | Stage 1 is the most common classification path (most failures are implementation-layer). Efficiency here dominates overall system performance. |
| 5 | Stage 2-3 causal reasoning | 15% | The LFI's value proposition (distinguishing methodology from theory failures) depends on causal reasoning quality at Stages 2-3. |
| 6 | Implementation complexity | 5% | Lower complexity reduces research-phase prototyping cost. Not weighted heavily because this is a research artifact. |
| 7 | Incremental adoptability | 5% | Can we validate Stage 1 first and add Stage 2-3 capability later? Reduces risk. |

### Candidate Scores

| Criterion | Weight | LEL | DGR | Hybrid |
|:---|:---|:---|:---|:---|
| R1-R29 coverage | 25% | 20/25 (2 WEAK: R14, R18) | 25/25 | 24/25 (minor: R29 cross-experiment) |
| Anti-pattern compliance | 20% | 18/20 (PARTIAL: AP7) | 18/20 (PARTIAL: AP2) | 20/20 |
| Streaming | 15% | 15/15 | 10/15 | 14/15 |
| Stage 1 efficiency | 15% | 15/15 | 10/15 | 15/15 |
| Stage 2-3 causal reasoning | 15% | 5/15 | 15/15 | 13/15 |
| Implementation complexity | 5% | 5/5 | 2/5 | 3/5 |
| Incremental adoptability | 5% | 4/5 | 2/5 | 5/5 |
| **Total** | **100%** | **82/100** | **82/100** | **94/100** |

### Score Rationale

**LEL (82):** Full marks for streaming and Stage 1 efficiency. Loses significantly on Stage 2-3 causal reasoning (WEAK R14 and R18 are the confounder detection and causal implication queries — the core of ATHENA's value proposition). Strong on simplicity.

**DGR (82):** Full marks for R1-R29 coverage and causal reasoning. Loses on streaming (moderate — spec_graph pre-built, exec_graph iterative, cross-graph deferred) and Stage 1 efficiency (graph construction overhead for the most common classification path). Same total as LEL but with inverted strengths and weaknesses.

**Hybrid (94):** Captures LEL's strengths (streaming, Stage 1 efficiency) and DGR's strengths (causal reasoning, coverage) simultaneously. Loses only marginally on Stage 2-3 causal reasoning (overlay is slightly less expressive than full DGR due to reference-back-to-LEL indirection) and implementation complexity (dual representation).

---

## Section 10: Recommendation

### Primary: Hybrid (LEL core + DGR overlay)

The Hybrid achieves the highest combined score (94/100) by providing per-stage optimized operation: LEL efficiency for Stage 1 (the common path) and DGR-like causal reasoning for Stages 2-3 (the differentiating path). It avoids the core weaknesses of both standalone candidates:
- LEL's inability to support structural causal queries for R14 and R18
- DGR's unnecessary graph construction overhead for Stage 1 classification

The Hybrid also demonstrates the incremental adoption path: validate Stage 1 (LEL only) first, then add Stage 2-3 capability (overlay) as the research matures.

### Step 5b Prototype: LEL First

The Step 5b prototype should implement the LEL core with the following scope:
- **Target:** OpenMM traces (best structural foundation: clean API boundary, R19=DA)
- **Scope:** R1-R7 (Stage 1 requirements) + R19 (layer tag) + R20 (provenance) + R21 (temporal ordering)
- **Key validation:** Demonstrate event typing, layer tagging, and specification separation on real OpenMM adapter output
- **Critical fields:** Include `dag_node_ref`, `spec_ref`, and `causal_refs` on TraceEvent even though Stage 1 does not use them. These fields preserve the Hybrid upgrade path.
- **Not in scope for 5b:** CausalOverlay construction, Stage 2-3 queries, DivergenceMeasure computation

This minimal prototype validates the LEL foundation with minimal complexity while preserving the evolution path toward the full Hybrid.

### Evolution Path: Toward Full DGR as Stages 2-3 Mature

As the LFI's Stage 2 and Stage 3 logic matures, the Hybrid's CausalOverlay can be progressively enriched:
1. **Phase 1 (5b):** LEL only. Stage 1 validation on OpenMM traces.
2. **Phase 2:** CausalOverlay construction from LEL events. Stage 2 confounder detection queries.
3. **Phase 3:** Full DGR-like overlay with ghost variables, lowering relations, and ComparisonResult production. Stage 3 prediction-observation comparison.
4. **Phase 4:** If the overlay proves insufficient for complex causal queries, evolve to full DGR by replacing the LEL core with a graph-native representation. The overlay's overlay entities and derivation edges are already DGR-compatible.

### TAL as Query Layer

TAL's assertion-checking pattern (sequential assertions ordered by audit stage, each carrying evidence chains) is adopted as the recommended query interface for the LFI:
- The LFI formulates its audit as a sequence of typed assertions over whichever IR substrate is in use.
- Each assertion references specific IR elements (LEL events or overlay entities) as evidence.
- Assertion evaluation results are structured records with classification, confidence, and evidence chains.
- This is an LFI-side interface, not an IR storage format.

---

## Section 11: Evidence Traceability

Every design decision in this document traces to specific evidence from the input documents:

| Decision | Evidence Source |
|:---|:---|
| Three-input data flow architecture | requirements-coverage-matrix.md §8 (31% NT, three-input diagram) |
| TAL deferred to query layer | requirements-coverage-matrix.md §8 ("better as query interface"), ir-pattern-catalog.md §6 (highest novelty risk) |
| LEL-DGR Hybrid added | ir-pattern-catalog.md §7 OQ5, requirements-coverage-matrix.md §8 OQ2 |
| Layer enum (three variants) | R19, ARCHITECTURE.md §3.1, ir-pattern-catalog.md §1 Pattern 5 |
| BoundaryClassification enum | cross-framework-synthesis.md §2 (boundary params), ir-pattern-catalog.md §7 OQ2 |
| Value::Havoc | ir-pattern-catalog.md §1 Pattern 6 (Boogie), R26 |
| TemporalCoord (three coordinates) | R21, cross-framework-synthesis.md §1 (MD step vs. DFT iteration) |
| ExperimentSpec as first-class | ir-pattern-catalog.md §1 Pattern 6, AP1 avoidance |
| ComparisonResult + DivergenceMeasure | R17, ir-pattern-catalog.md §2 Stage 3 gaps, §7 OQ1 |
| TypedAgent (three variants) | ir-pattern-catalog.md §3 AP4, Pattern 2 |
| Overlay construction at Stage 1/2 boundary | Per-stage causal substrate analysis (OQ3 resolution) |
| LEL events carry dag_node_ref/spec_ref | Hybrid viability constraint (OQ2 resolution) |
| Streaming compatibility model | ADR 001, ir-pattern-catalog.md §1 Patterns 1,3 |
| Ghost variables for methodology | ir-pattern-catalog.md §1 Pattern 6 (Why3), Stage 2 gap analysis |
| ObservationMode enum | R28, evaluation/hidden-confounder/README.md |

---

**Sources:** All citations reference the following documents:
- [ir-pattern-catalog.md §N] = ir-pattern-catalog.md, Section N
- [requirements-coverage-matrix.md §N] = requirements-coverage-matrix.md, Section N
- [cross-framework-synthesis.md §N] = cross-framework-synthesis.md, Section N
- [FINDINGS.md] = FINDINGS.md, Investigation logs and Accumulated Findings
- [ADR 001] = decisions/001-python-rust-core.md
- [ARCHITECTURE.md §N] = ARCHITECTURE.md, Section N
- [hidden-confounder §N] = evaluation/hidden-confounder/README.md, Section N
