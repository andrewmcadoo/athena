# Survey: Intermediate Representations in RCA and Formal Verification Systems

## Purpose

Identify IR design patterns from root cause analysis (RCA) tools and formal verification systems that transfer to ATHENA's trace semantics problem: translating raw DSL trace logs into a structured representation suitable for deterministic three-stage fault classification (implementation audit, methodological audit, theoretical evaluation).

**Key evaluation criterion for each IR:** Does it cleanly separate "what was specified" from "what executed"?

**Implementation constraint:** The IR will be implemented in Rust for parsing throughput (ADR 001). Patterns must be compatible with zero-copy/streaming parsing of megabyte-scale traces.

---

## 1. RCA Intermediate Representations

### 1.1 LLM-Based RCA

#### "Exploring LLM-based Agents for Root Cause Analysis" (arxiv:2403.04123)

**System overview.** This paper evaluates LLM-based agents for root cause analysis in cloud/microservice environments. The agents consume system telemetry (logs, metrics, traces) and attempt to identify the root cause of incidents.

**Implicit IR.** The system does not define a formal IR. Instead, it relies on:
- **Chain-of-thought reasoning** as an implicit intermediate representation: the LLM's internal reasoning steps serve as an unstructured, non-queryable "representation" of causal chains.
- **Event logs** consumed as raw text, with the LLM performing ad-hoc extraction of relevant events.
- **Dependency graphs** of services/components provided as contextual input, but not formally integrated into a queryable structure.

**Data structures.** No typed data structures for causal chains. The "IR" is natural language embedded in prompt context windows.

**Causal ordering.** Temporal ordering is implicit in log timestamps. Causal ordering (A caused B) is inferred by the LLM rather than structurally represented.

**Root cause ranking.** LLMs produce ranked lists of candidate root causes, but ranking is based on LLM confidence (a calibration-questionable metric) rather than structural evidence.

**Queryability.** Extremely low. The chain-of-thought is a write-once artifact; there is no mechanism to query "which implementation-layer events preceded this failure?" or "was this variable controlled for?"

**Reported accuracy.** The paper reports accuracy metrics for root cause identification; the approximately 21% Top@1 figure for unstructured traces is consistent with findings across similar LLM-based RCA evaluations. [Note: The exact source of the 21% figure cited in ARCHITECTURE.md 4.5 may originate from this paper or a related evaluation; see Section 1.1.3 below.]

**Transferability assessment.** The absence of a formal IR is itself the finding. This paper demonstrates what happens when causal analysis operates on unstructured representations: low accuracy, non-reproducible reasoning, and inability to systematically distinguish failure categories. This is the negative baseline ATHENA must exceed.

#### "Stalled, Biased, and Confused: Impact of Missing Context, Misinformation, and Bias on LLM-Powered Root Cause Analysis" (arxiv:2601.22208)

**System overview.** This paper analyzes failure modes in LLM-based RCA systems, identifying specific categories of reasoning failure that stem from IR deficiencies.

**Key failure categories identified:**
1. **Stalled:** LLMs fail to progress when critical context is missing from the input representation. The implicit IR (raw logs + prompts) cannot signal what information is absent.
2. **Biased:** LLMs exhibit systematic biases toward certain root cause categories, regardless of evidence. The lack of structured evidence weighting means prior biases dominate over trace evidence.
3. **Confused:** When multiple potential causal chains are present, LLMs conflate them. The unstructured representation provides no mechanism to track and separate parallel causal paths.

**IR deficiencies identified:**
- No separation between observational evidence and inferred causation.
- No mechanism to represent "unknown" or "unobserved" states -- the LLM treats absence of evidence as evidence of absence.
- No structural distinction between correlation and causation in the input representation.
- Context window limitations force lossy compression of trace data, with no principled selection of what to retain.

**Structural improvements proposed:**
- Structured context provision (providing dependency graphs, not just logs).
- Explicit uncertainty representation in inputs.
- Decomposed reasoning with intermediate verification checkpoints.

**Transferability assessment.** HIGH. This paper's failure taxonomy maps directly to ATHENA's design requirements:
- "Stalled" -> the IR must explicitly represent observability boundaries (what was and was not captured in the trace).
- "Biased" -> the IR must separate evidence from inference, preventing the LFI from inheriting LLM biases.
- "Confused" -> the IR must support isolation of parallel causal chains for independent evaluation.

These three deficiencies correspond to requirements for ATHENA's three audit stages: implementation audit needs complete observability representation, methodological audit needs evidence/inference separation, and theoretical evaluation needs isolated causal chain tracking.

#### "Empowering Practical Root Cause Analysis" and the 21% Top@1 Baseline

**Status.** The specific paper sourcing the 21% Top@1 accuracy figure cited in ARCHITECTURE.md 4.5 was not individually located as a standalone publication. The figure is consistent with reported accuracy ranges across multiple LLM-based RCA evaluations on unstructured traces (including the ReAct-based agent evaluations in arxiv:2403.04123 and benchmark results reported in the AIOps literature). The claim that "accuracy improves substantially within constrained DSL environments" is an architectural assertion grounded in the structural argument that DSL API separation reduces the three-way ambiguity problem, but has not been empirically validated within ATHENA's specific context.

**Implication for this survey.** The 21% figure serves as a baseline motivation rather than a precise benchmark. The relevant finding is the mechanism: unstructured traces produce low RCA accuracy because the representation does not encode the structural distinctions needed for fault classification. DSL environments improve this by providing API-level separation between theory and implementation layers, which can be encoded into the IR.

### 1.2 Structured RCA

#### "Chain-of-Event: Interpretable Root Cause Analysis for Microservices" (FSE 2024)

**System overview.** Chain-of-Event (CoE) introduces a structured event chain IR for root cause analysis in microservice architectures. Rather than consuming raw logs, it constructs typed, linked event chains that represent the causal propagation of failures across service boundaries.

**Data structures:**
- **Event node:** A typed record containing: event type (enum: error, latency_spike, resource_exhaustion, timeout, etc.), source service, timestamp, severity, and a set of observable attributes (key-value pairs drawn from structured logs and metrics).
- **Causal link:** A directed edge between event nodes representing hypothesized causal dependency. Each link carries: link type (direct_call, resource_contention, cascading_failure), confidence score, and evidence references (which log entries or metric correlations support this link).
- **Event chain:** An ordered sequence of causally linked event nodes forming a path from a symptom back to its root cause. Chains are the primary unit of analysis.

**Causal ordering representation:**
- Temporal ordering is baseline (all events are timestamped).
- Causal ordering is layered on top via typed links. The system distinguishes "A happened before B" (temporal) from "A caused B" (causal, with evidence).
- Multiple causal chains can coexist for a single incident, representing alternative hypotheses about root cause.

**Root cause ranking:** Event chains are ranked by a composite score incorporating: chain length (shorter chains preferred, Occam's razor), aggregate link confidence, and coverage (how many observed symptoms the chain explains). The ranking is transparent and decomposable -- each factor can be inspected independently.

**Queryability:** HIGH. The typed event chain structure supports structured queries:
- "What is the root cause of symptom X?" (trace back through causal links)
- "What events of type Y occurred in service Z before time T?" (filtered traversal)
- "Are there alternative causal explanations for this symptom?" (enumerate chains)

**Reported accuracy improvement.** CoE reports substantial accuracy improvements over unstructured LLM-based RCA approaches, with the improvement attributed to the structured event chain IR enabling systematic causal reasoning rather than ad-hoc LLM inference.

**Spec-vs-execution separation.** PARTIAL. CoE represents what executed (the event chain) but does not explicitly represent what was specified (the intended service behavior). Service-level objectives (SLOs) are implicitly referenced in event typing (e.g., a latency_spike event implies a latency threshold was exceeded) but are not first-class entities in the IR.

**Transferability assessment.** HIGH for the event chain structure, MEDIUM for direct adoption.

Transferable elements:
- **Typed event nodes** map to ATHENA's need for typed trace events (implementation-layer vs. theory-layer events in DSL traces).
- **Causal links with evidence references** map to ATHENA's need for traceable causal attribution (every fault classification must reference specific trace evidence).
- **Multiple coexisting chains** map to ATHENA's need to evaluate alternative fault hypotheses before committing to a classification.
- **Decomposable ranking** provides transparency the LFI needs for deterministic, auditable fault classification.

Non-transferable elements:
- CoE's event taxonomy is microservice-specific (error, latency_spike, timeout). ATHENA needs a DSL-specific taxonomy (framework_exception, numerical_overflow, parameter_violation, theory_prediction_mismatch).
- CoE operates post-mortem on completed incidents. ATHENA needs streaming IR construction during experiment execution.

#### Distributed Tracing Systems: Jaeger and Zipkin

**System overview.** Jaeger (originally Uber) and Zipkin (originally Twitter) are distributed tracing systems implementing the OpenTelemetry/OpenTracing span model. They represent request execution across distributed services as hierarchical span trees.

**Data structures:**
- **Span:** The fundamental unit. Contains: operation name, service name, start time, duration, span ID, parent span ID (forming the tree), status (OK/ERROR), and a bag of key-value tags and structured log entries.
- **Trace:** A tree of spans sharing a trace ID, representing the complete execution path of a single request across services.
- **Span context:** Propagated metadata (trace ID, span ID, baggage items) that links spans across process boundaries.

**Causal ordering representation:**
- Parent-child span relationships encode invocation causality (service A called service B).
- Sibling span ordering encodes temporal sequencing within a parent scope.
- "Follows-from" relationships (OpenTracing) encode causality without parent-child hierarchy (e.g., message queue consumer follows producer).

**Root cause ranking.** Not built into the tracing model itself. Jaeger and Zipkin provide the data structure; analysis tools (built on top) perform root cause identification. The span tree provides the structural foundation for RCA but does not perform it.

**Queryability:** HIGH for structural queries (find all spans in trace X, find all error spans in service Y, compute latency breakdown by service). MEDIUM for causal queries (the span tree encodes invocation structure, not domain-level causality).

**Spec-vs-execution separation.** MINIMAL. Spans represent what executed. The specification (what was intended) is not represented in the tracing model. Service-level expectations must be provided externally.

**Transferability assessment.** MEDIUM.

Transferable elements:
- **Hierarchical span model** maps to ATHENA's need for hierarchical trace structure. DSL traces have natural hierarchy: experiment -> simulation step -> force calculation -> numerical kernel. A span-like model can represent this hierarchy with parent-child relationships.
- **Context propagation** maps to ATHENA's need to track causal context across DSL framework boundaries. When a theory-layer parameter choice propagates through implementation-layer execution, the span context model provides a mechanism to link the two.
- **Tag/log annotation model** provides a flexible mechanism for attaching domain-specific metadata to trace events without modifying the core data structure.
- **Zero-copy friendliness.** Span-based models are naturally streaming-compatible: spans can be emitted and parsed individually without buffering the entire trace. This aligns with the Rust implementation constraint.

Non-transferable elements:
- The span model is optimized for request-response patterns, not batch simulation execution. DSL traces may contain millions of timesteps without clear request-response boundaries.
- No built-in mechanism for representing theoretical predictions alongside execution observations.

---

## 2. Formal Verification IRs

### 2.1 Compiler/Program Analysis IRs

#### LLVM IR (SSA Form)

**System overview.** LLVM IR is a typed, SSA-form (Static Single Assignment) intermediate representation used as the core abstraction in the LLVM compiler infrastructure. It sits between source languages and target machine code, providing a language-independent representation of program semantics.

Reference: llvm.org/docs/LangRef.html

**Key structural properties:**
- **SSA form:** Every variable is assigned exactly once. This makes data flow explicit: the definition of every value is unambiguous, and use-def chains are trivially computable.
- **Type system:** Strong typing with explicit type annotations on all values. Types include integers (arbitrary width), floating point, vectors, arrays, structures, pointers, and function types.
- **Basic blocks:** Code is organized into basic blocks (sequences of instructions with a single entry point and single exit). Control flow is represented as edges between basic blocks, forming a control flow graph (CFG).
- **Metadata:** LLVM IR supports arbitrary metadata attachments on instructions. Debug information, optimization hints, and analysis results are all expressed as metadata without modifying the core instruction semantics.
- **Intrinsics:** Domain-specific operations (e.g., math functions, memory operations, exception handling) are represented as intrinsic function calls, providing extensibility without modifying the instruction set.

**Data flow representation.** SSA form makes data flow analysis trivial: every value has exactly one definition, and all uses are direct references to that definition. This enables efficient reaching-definition analysis, dead code elimination, and constant propagation -- all of which are analogous to trace analysis operations (which trace events influenced this outcome? which parameter values are dead/irrelevant?).

**Error path representation.** LLVM represents exceptional control flow through invoke/landingpad instructions and personality functions. Error paths are first-class in the CFG -- they are not special cases but regular control flow edges to exception-handling basic blocks. This means error analysis uses the same infrastructure as normal program analysis.

**Spec-vs-execution separation.** NONE in the base IR. LLVM IR represents only the implementation. Specifications (pre/postconditions, invariants) are not part of the IR; they exist in higher-level tools (e.g., Clang annotations, KLEE symbolic execution constraints) that operate on LLVM IR but are not expressed in it.

**Transferability assessment.** MEDIUM-HIGH for specific patterns.

Transferable elements:
- **SSA-like event identification.** Each trace event could receive a unique, immutable identifier with explicit data-flow links to prior events. This makes "which prior events influenced this outcome?" a structural query rather than a search problem. Directly supports Stage 1 (implementation audit): tracing data pipeline correctness is a data-flow query.
- **Typed instructions with metadata.** DSL trace events can be represented as typed operations (force_calculation, parameter_update, data_load) with metadata bags for domain-specific annotations. The type system enables stage-appropriate filtering: implementation-layer types are checked in Stage 1, methodology-layer types in Stage 2.
- **Basic block structure for trace segmentation.** Trace logs can be segmented into "basic blocks" -- contiguous sequences of same-layer operations with single entry/exit points. This enables hierarchical analysis: check block-level properties first, then drill into individual events only when a block fails.

Non-transferable elements:
- LLVM IR is designed for compilation, not diagnosis. It does not carry the "why" of a computation, only the "what." ATHENA's IR must carry both (what was specified and what happened).
- The SSA restriction (single assignment) may be too strict for trace events where a parameter is legitimately updated multiple times during a simulation.

#### MLIR (Multi-Level Intermediate Representation)

**System overview.** MLIR is an extensible IR framework within the LLVM project designed to support multiple levels of abstraction within a single IR infrastructure. Rather than committing to a single abstraction level (as LLVM IR does), MLIR allows coexistence of high-level domain operations and low-level implementation details.

Reference: mlir.llvm.org

**Key structural properties:**
- **Dialect system:** MLIR organizes operations into dialects -- namespaced collections of operations, types, and attributes. Each dialect represents a specific abstraction level or domain. Standard dialects include `affine` (loop analysis), `linalg` (linear algebra), `scf` (structured control flow), `arith` (arithmetic), and `func` (functions). Custom dialects can be defined for any domain.
- **Multi-level abstraction.** A single MLIR module can contain operations from multiple dialects simultaneously. A high-level `linalg.matmul` operation can coexist with low-level `arith.addf` operations in the same IR. Progressive lowering transforms high-level operations into lower-level ones, but the IR framework does not require all operations to be at the same level.
- **Regions and blocks.** Operations can contain nested regions, which contain blocks, which contain operations. This recursive structure supports arbitrary nesting of abstraction levels.
- **Extensible type system.** Each dialect defines its own types. The `tensor` dialect has tensor types; the `memref` dialect has memory reference types. Types from different dialects can interact through explicit conversion operations.
- **Attributes.** Compile-time constant metadata attached to operations. Used for configuration, optimization hints, and analysis annotations.

**Spec-vs-execution separation.** ARCHITECTURALLY SUPPORTED but not enforced. MLIR's dialect system can represent specifications and implementations as separate dialects within the same IR. A "specification dialect" could define operations like `requires(precondition)` and `ensures(postcondition)`, coexisting with an "implementation dialect" defining actual computations. The framework supports this structurally but does not mandate it.

**Transferability assessment.** HIGH -- this is the most structurally relevant pattern in the survey.

Transferable elements:
- **Dialect system maps to ATHENA's layer separation.** Define a "theory dialect" (operations representing the user's scientific specification: set_force_field, define_ensemble, specify_observable) and an "implementation dialect" (operations representing framework execution: allocate_memory, load_data, compute_forces, write_checkpoint). The DSL's API separation maps directly to dialect separation. This is the most promising structural pattern for encoding the theory-implementation distinction that the LFI's three-stage audit requires.
- **Multi-level coexistence.** A single trace IR can contain both high-level theory operations and low-level implementation events, linked by explicit "lowering" relationships. When the user specifies `set_temperature(300K)`, the IR records both the theory-level operation and the implementation-level events it generated (thermostat initialization, velocity rescaling, etc.). The LFI can then check: did the implementation events faithfully realize the theory-level operation?
- **Progressive lowering as audit structure.** The three-stage audit maps to three levels of abstraction:
  - Level 0 (implementation): raw framework events, exceptions, resource states.
  - Level 1 (methodology): experiment design operations, variable control, sampling configuration.
  - Level 2 (theory): hypothesis predictions, causal claims, expected observations.
  The LFI audits from Level 0 upward, checking each level's consistency before proceeding to the next.
- **Region nesting for trace structure.** Simulation traces have natural nesting: experiment contains phases, phases contain timesteps, timesteps contain force calculations. MLIR's region model maps to this hierarchy.
- **Extensibility per DSL.** Each target DSL (OpenMM, GROMACS, VASP) can define its own dialect within a shared framework, enabling DSL-specific trace parsing while maintaining a common audit interface.

Non-transferable elements:
- MLIR is a compile-time framework; it does not natively support streaming or runtime trace construction. The dialect pattern transfers, but the implementation machinery does not.
- MLIR's operation semantics are computational (they represent transformations). ATHENA's trace events are observational (they represent what happened). The semantic interpretation differs even if the structural patterns align.

### 2.2 Verification-Specific IRs

#### Boogie (from Dafny, VCC, Corral)

**System overview.** Boogie is an intermediate verification language used as the target for multiple source-level verifiers. Dafny compiles to Boogie; VCC (C verifier) compiles to Boogie; Corral (concurrency verifier) operates on Boogie. It serves as a verification-specific IR that separates program logic from verification conditions.

Reference: github.com/boogie-org/boogie

**Key structural properties:**
- **Specification-implementation separation is first-class.** Boogie programs contain both:
  - `requires` clauses (preconditions): what must hold before execution.
  - `ensures` clauses (postconditions): what must hold after execution.
  - `invariant` clauses: what must hold throughout execution.
  - `modifies` clauses: what the implementation is permitted to change.
  - Implementation bodies: the actual computation.
- **Assertion structure.** `assert` statements embed verification conditions within the implementation. Each assertion is a point where the verifier checks that the implementation's state satisfies a property. Failed assertions produce counter-examples.
- **Procedure contracts.** Each procedure has a contract (requires/ensures/modifies) and a body. The verifier checks the body against the contract. This is structurally analogous to checking whether an experiment's execution matched its specification.
- **Havoc statements.** `havoc x` non-deterministically assigns a value to x, representing unknown or uncontrolled state. This is the mechanism for modeling incomplete information.

**Spec-vs-execution separation.** EXPLICIT AND FIRST-CLASS. This is Boogie's raison d'etre. The contract (requires/ensures/modifies) is the specification; the body is the implementation. The verifier's entire purpose is to determine whether the implementation satisfies the specification.

**Counter-example generation.** When verification fails, Boogie produces a counter-example: a concrete execution trace that violates an assertion. The counter-example includes:
- The specific assertion that failed.
- The values of relevant variables at the failure point.
- The execution path leading to the failure.

**Transferability assessment.** HIGH for the specification-implementation contract pattern.

Transferable elements:
- **Procedure contracts map to experiment specifications.** An ATHENA experiment has a specification (hypothesis predictions, controlled variables, expected observables) and an execution (what actually happened in the DSL framework). The Boogie pattern of attaching `requires`/`ensures`/`modifies` to procedures maps directly:
  - `requires`: preconditions the experiment assumes (correct data loading, valid parameter ranges, hardware availability).
  - `ensures`: what the experiment should produce if the hypothesis is correct (predicted observable values, expected relationships).
  - `modifies`: what the experiment is permitted to change (which state variables the simulation updates).
- **Assertion-based fault localization.** Embed checkable assertions at layer boundaries in the trace IR. At the implementation/methodology boundary: "assert(all data pipelines produced valid output)". At the methodology/theory boundary: "assert(experiment measured the hypothesized causal variables)". Failed assertions pinpoint the layer where the fault occurred, directly supporting the LFI's sequential audit.
- **Havoc for unobserved state.** When the trace does not capture a variable's value (incomplete observability, per ARCHITECTURE.md 8.4), represent it as `havoc` -- explicitly unknown. This prevents the LFI from making assumptions about unobserved state, addressing the "Stalled" failure mode from arxiv:2601.22208.

Non-transferable elements:
- Boogie verification is static (pre-execution). ATHENA's trace analysis is post-execution. The verification conditions must be checked against actual trace data, not against all possible executions.
- Boogie's logic is first-order. ATHENA's causal reasoning involves graph structures and probabilistic relationships that exceed first-order expressiveness.

#### Why3 (from Frama-C, SPARK)

**System overview.** Why3 is a platform for deductive program verification. It provides an intermediate language (WhyML) with built-in specification constructs and dispatches verification conditions to multiple automated provers (Alt-Ergo, Z3, CVC4). Frama-C (C verification) and SPARK (Ada verification) generate Why3 verification conditions.

Reference: why3.lri.fr

**Key structural properties:**
- **Pre/postcondition logic.** Like Boogie, but with richer specification constructs: ghost variables (specification-only state not present in the implementation), abstract specifications (specification without implementation), and refinement (progressively adding implementation detail to an abstract specification).
- **Ghost state.** Why3 allows specification-only variables that exist for verification purposes but have no runtime counterpart. Ghost state tracks properties the implementation does not explicitly compute but that the specification requires.
- **Theories.** Why3 organizes mathematical definitions and axioms into theories -- reusable collections of types, functions, and axioms. Theories provide the mathematical vocabulary for specifications.
- **Counterexample extraction.** When a verification condition fails, Why3 extracts counter-examples from the SMT solver, presenting them as concrete variable assignments that violate the specification.

**Spec-vs-execution separation.** EXPLICIT AND MULTI-LAYERED. Why3 distinguishes:
- Mathematical theories (pure specifications with no computational content).
- Abstract programs (specifications with logical contracts but no implementation).
- Concrete programs (implementations that refine abstract programs).

**Transferability assessment.** MEDIUM-HIGH.

Transferable elements:
- **Ghost state for methodological metadata.** ATHENA's methodology audit (Stage 2) requires information that may not appear in the raw trace: Was sampling sufficient? Were confounders controlled? These are "ghost variables" -- they are properties of the experiment design, not of the execution trace. The IR can represent them as ghost state, populated from the experiment specification rather than the trace log.
- **Theory libraries for domain axioms.** Each DSL domain has axioms (conservation laws in molecular dynamics, thermodynamic constraints in materials science). These can be represented as Why3-style theories -- reusable specification vocabularies that the LFI references when evaluating whether results contradict theoretical predictions (Stage 3).
- **Refinement for theory-to-implementation tracing.** The abstract specification (hypothesis) is refined into a concrete specification (experiment design), which is refined into an implementation (DSL framework execution). The refinement chain provides a structural mechanism for the LFI to trace backwards from a failure to the layer where the specification was violated.

Non-transferable elements:
- Why3's verification is deductive and exhaustive. ATHENA's trace analysis is empirical and sample-based. The logical infrastructure is relevant, but the verification methodology is not.
- Why3 targets full functional correctness. ATHENA targets fault classification (a much weaker property than correctness).

### 2.3 Proof and Counter-Example Traces

#### DRAT Proofs (Deletion Resolution Asymmetric Tautology)

**System overview.** DRAT is the standard format for machine-checkable unsatisfiability proofs produced by SAT solvers. When a solver determines that a formula is unsatisfiable, it emits a DRAT proof: a sequence of clause additions and deletions that, when verified, confirms that the formula has no satisfying assignment.

Reference: satcompetition.github.io (SAT Competition proof format specifications)

**Key structural properties:**
- **Refutation structure.** A DRAT proof does not demonstrate that a solution exists; it demonstrates that no solution exists. Each step either adds a clause (provably implied by existing clauses) or deletes a clause (no longer needed). The proof terminates with the empty clause, establishing unsatisfiability.
- **Machine-checkable.** DRAT proofs are designed for independent verification. A DRAT checker can verify the proof without trusting the solver. This is relevant to ATHENA's requirement for deterministic, auditable fault classification.
- **Streaming format.** DRAT proofs are emitted and checked incrementally. The checker processes one clause operation at a time without needing the entire proof in memory. This aligns with the Rust streaming parsing constraint.
- **Compact representation.** Clause operations are represented as sequences of literal indices. The format is extremely space-efficient for its purpose.

**Spec-vs-execution separation.** YES, structurally. The original formula is the specification ("these constraints must be simultaneously satisfied"). The DRAT proof is the evidence that the specification is unsatisfiable. The two are cleanly separated.

**Transferability assessment.** MEDIUM for the refutation pattern, LOW for the specific format.

Transferable elements:
- **Refutation-as-evidence pattern.** ATHENA's Stage 3 (theoretical evaluation) asks: does the evidence refute the hypothesis under clean conditions? A DRAT-like structure could represent the refutation evidence as a chain of inference steps from observations to contradiction. Each step is independently verifiable, and the chain constitutes a machine-checkable refutation of the theoretical prediction. This provides auditability: a human (or automated checker) can verify the LFI's theoretical falsification by replaying the refutation chain.
- **Streaming verification.** DRAT's incremental checking model aligns with the Rust streaming parsing requirement. Trace events can be processed and verified incrementally, with fault detection occurring as events are consumed rather than requiring full trace buffering.

Non-transferable elements:
- DRAT operates on propositional logic. ATHENA's domain involves quantitative, continuous-valued scientific data. The refutation logic is conceptually useful but the propositional machinery is irrelevant.
- DRAT proofs are solver-generated artifacts. ATHENA's "refutation evidence" must be constructed from empirical trace data, which is fundamentally different from logical derivation.

#### AIGER (And-Inverter Graph Exchange Format)

**System overview.** AIGER is the standard format for representing hardware circuits and their properties in model checking. When a model checker finds that a property is violated, it produces a counter-example trace: a sequence of input assignments that drive the circuit from an initial state to a state violating the property.

Reference: fmv.jku.at/aiger (AIGER format specification)

**Key structural properties:**
- **State-transition model.** The circuit is represented as a state machine. Each clock cycle defines a transition from the current state (latches) to the next state, driven by inputs. The counter-example trace is a sequence of input vectors, one per clock cycle.
- **Property specification.** Safety properties are expressed as output signals that must never be asserted. A counter-example trace is a finite sequence of inputs that causes the bad-state output to be asserted.
- **Witness traces.** The counter-example trace includes the values of all state variables at each step, providing complete observability of the system state as the violation develops.

**Spec-vs-execution separation.** YES. The property (output signal constraint) is the specification. The counter-example trace is the execution that violates it. The two are structurally distinct.

**Transferability assessment.** MEDIUM.

Transferable elements:
- **State-transition trace model.** DSL simulation traces are naturally state-transition sequences: at each timestep, the system transitions from one state to another based on force calculations, integrator steps, etc. Representing the trace as a sequence of (state, transition, next_state) tuples provides a clean foundation for checking invariants at each step. Implementation audit (Stage 1) can check that each transition was valid (no numerical overflow, no resource exhaustion). Methodological audit (Stage 2) can check that the state trajectory covers the intended experimental conditions.
- **Witness trace completeness.** AIGER counter-examples include all state variable values at each step. This sets a standard for trace completeness: ATHENA's trace IR should capture sufficient state at each transition point for the LFI to determine whether the transition was valid. Incomplete state capture should be explicitly flagged (cf. Boogie's `havoc`), not silently ignored.

Non-transferable elements:
- AIGER is for hardware circuits with discrete, finite state. Scientific simulations have continuous, high-dimensional state. The format is irrelevant; the pattern of state-transition traces with completeness guarantees is relevant.
- AIGER counter-examples are minimal witnesses (shortest path to violation). ATHENA's traces are complete execution records. Different information-density tradeoffs apply.

---

## 3. Program Analysis Error Path Representations

### 3.1 Clang Static Analyzer

**System overview.** The Clang Static Analyzer performs path-sensitive, inter-procedural analysis of C/C++/Objective-C code. It detects bugs by exploring possible execution paths through the program's control flow graph and tracking symbolic state along each path.

Reference: clang-analyzer.llvm.org

**Error path representation:**
- **Bug reports with path notes.** Each detected bug includes a sequence of "path notes" -- annotations on specific program points along the execution path leading to the bug. Path notes include: the specific statement, the symbolic state at that point, and a human-readable explanation of why this point is relevant to the bug.
- **Exploded graph.** Internally, the analyzer maintains an "exploded graph" -- a product of the control flow graph and the abstract state space. Each node is a (program point, abstract state) pair. Bug paths are paths through this exploded graph from an entry point to a bug-triggering state.
- **Path constraints.** Along each path, the analyzer tracks constraints on symbolic values (e.g., "x > 0", "ptr != NULL"). The bug report includes the constraints that must hold for the bug to manifest.

**Spec-vs-execution separation.** IMPLICIT. The "specification" is the set of correctness properties the analyzer checks (no null dereference, no use-after-free, no division by zero). These are built into the analyzer, not represented in the IR. The "execution" is the symbolic path through the exploded graph.

**Transferability assessment.** MEDIUM.

Transferable elements:
- **Path notes as audit trail.** ATHENA's LFI audit trail could use a path-notes-like structure: for each fault classification, provide a sequence of trace events (with explanations) showing the causal chain from root cause to observed symptom. This makes the classification auditable and debuggable.
- **Path constraints as failure conditions.** The constraints under which a failure manifests can be explicitly tracked in the IR. "This numerical overflow occurs when force_magnitude > 1e15 AND timestep > 2fs" -- these are path constraints that the LFI can report alongside its classification.

### 3.2 Soot and WALA (Java/JVM Analysis)

**System overview.** Soot and WALA are program analysis frameworks for Java/JVM bytecode. Soot provides multiple IR levels (Baf, Jimple, Shimple, Grimp) with progressively more analysis-friendly representations. WALA provides SSA-based IR with hierarchical scope structure.

**Key IR patterns:**
- **Soot's Jimple:** A typed, three-address-code IR that simplifies JVM bytecode. Each statement has at most three operands, making data flow analysis straightforward. Jimple is explicitly designed for analysis, not compilation -- it sacrifices code generation convenience for analysis convenience.
- **WALA's SSA IR:** Similar to LLVM IR but with call graph construction and pointer analysis integrated into the IR construction process. Interprocedural data flow is represented through call graph edges.

**Transferability assessment.** LOW-MEDIUM. The general pattern of having an IR designed for analysis rather than compilation/execution is relevant (ATHENA's trace IR is designed for fault classification, not for replaying or compiling traces), but the specific Java/JVM representations are not transferable.

### 3.3 Facebook Infer

**System overview.** Infer is a static analysis tool for Java, C, C++, and Objective-C. It uses bi-abductive reasoning to perform compositional, inter-procedural analysis. Infer can analyze large codebases incrementally by analyzing each procedure independently using procedure contracts inferred by bi-abduction.

Reference: fbinfer.com

**Key structural properties:**
- **Separation logic assertions.** Infer uses separation logic to reason about heap-manipulating programs. Pre and postconditions are expressed as separation logic formulas describing the heap structure.
- **Compositional analysis.** Each procedure is analyzed independently. The analysis infers a procedure summary (pre/postcondition pair) that describes the procedure's effect on memory. Callers use the summary instead of re-analyzing the callee. This enables incremental analysis of large codebases.
- **Bi-abduction.** Given a postcondition and an implementation, Infer infers the weakest precondition (abduction) and the tightest postcondition (deduction). The inferred preconditions represent assumptions the procedure makes about its calling context.

**Spec-vs-execution separation.** INFERRED. Unlike Boogie/Why3 where specifications are manually written, Infer infers specifications from the implementation. The specification is a derived artifact, not a primary one.

**Transferability assessment.** MEDIUM.

Transferable elements:
- **Compositional/incremental analysis.** For megabyte-scale traces, analyzing the entire trace monolithically is impractical. Infer's compositional approach -- analyze each "procedure" (trace segment) independently, produce a summary, compose summaries -- maps to ATHENA's need for scalable trace analysis. Each simulation phase can be summarized independently, and the LFI can compose phase summaries to determine whether the overall experiment succeeded or failed.
- **Inferred preconditions as implicit assumptions.** When a DSL framework function executes, it makes implicit assumptions about its inputs (valid parameter ranges, allocated memory, initialized state). Infer's bi-abductive approach of inferring these assumptions from the implementation pattern suggests that ATHENA's trace IR could infer preconditions from DSL API documentation or prior trace analysis, then check them against actual trace data.

Non-transferable elements:
- Separation logic is specific to heap-manipulating programs. Scientific simulations operate on numerical arrays and structured data, not arbitrary heap structures. The logical framework does not transfer.
- Infer's analysis is static (pre-execution). ATHENA's analysis is post-execution on actual traces.

---

## 4. Transferable Patterns Catalog

| # | Pattern Name | Source | Core Mechanism | LFI Stage Supported | Transferability | Reasoning | Limitations |
|---|---|---|---|---|---|---|---|
| P1 | **Dialect-Based Layer Separation** | MLIR | Organize operations into namespaced dialects representing different abstraction levels. Theory operations and implementation operations coexist in the same IR but are structurally distinguished. | All three stages (primary enabler of stage routing) | HIGH | Maps directly to DSL API separation. The theory dialect captures user specifications; the implementation dialect captures framework execution. The LFI routes to the correct audit stage by checking which dialect's operations failed. | Requires defining dialect boundaries per DSL. Not all trace events will cleanly map to exactly one dialect. |
| P2 | **Specification-Implementation Contracts** | Boogie, Why3 | Attach requires/ensures/modifies contracts to procedures. Verification checks implementation against contract. | Stage 1 (implementation audit), Stage 2 (methodology audit) | HIGH | Experiment specifications become contracts. Implementation audit checks that execution satisfied preconditions and postconditions. Methodology audit checks that the experiment specification (the "contract") was adequate to test the hypothesis. | Contracts must be derived from experiment specifications and DSL API semantics; not available in raw traces. Requires a specification extraction step. |
| P3 | **Typed Event Chains with Evidence Links** | Chain-of-Event (FSE 2024) | Typed event nodes connected by causal links carrying confidence scores and evidence references. | Stage 1 (tracing failure propagation), Stage 3 (building refutation evidence) | HIGH | Event typing enables stage-appropriate filtering. Evidence links ensure every fault classification is traceable to specific trace data. Multiple coexisting chains support evaluation of alternative fault hypotheses. | Event taxonomy must be defined per DSL domain. Causal link confidence requires a scoring model. |
| P4 | **Ghost State for Methodological Metadata** | Why3 | Specification-only variables that exist for verification but have no runtime counterpart. | Stage 2 (methodology audit) | HIGH | Methodological properties (sampling sufficiency, confounder control, variable coverage) are not in the execution trace but are needed for Stage 2. Representing them as ghost state, populated from the experiment specification, makes them queryable alongside trace events. | Ghost state must be populated from external sources (experiment spec, DAG). If the external source is wrong, ghost state inherits the error. |
| P5 | **SSA-Style Event Identification** | LLVM IR | Each value defined exactly once with explicit use-def chains. | Stage 1 (data pipeline audit) | MEDIUM-HIGH | Each trace event gets a unique immutable ID. Data-flow queries ("which events influenced this outcome?") become structural traversals rather than search problems. Directly supports checking data pipeline correctness. | SSA is restrictive for parameters that are legitimately updated during simulation. Requires versioned identifiers (parameter_v1, parameter_v2) rather than strict SSA. |
| P6 | **Progressive Lowering as Audit Hierarchy** | MLIR | Transform high-level operations into lower-level ones through a sequence of lowering passes. | All three stages (audit ordering) | MEDIUM-HIGH | Maps the three audit stages to three abstraction levels. Level 0 (implementation events) is audited first; Level 1 (methodology) second; Level 2 (theory) last. Each level is a "lowered" view of the level above. The LFI progresses through levels only after the lower level passes audit. | The lowering relationship must be defined per DSL. Not all theory-level operations have clear implementation-level counterparts in the trace. |
| P7 | **Havoc for Unobserved State** | Boogie | Non-deterministic assignment representing unknown state. | Stage 1 (incomplete observability), Stage 2 (uncontrolled confounders) | MEDIUM-HIGH | When the trace does not capture a variable, represent it as `havoc` (explicitly unknown) rather than leaving it absent. The LFI can then distinguish "this variable was checked and is fine" from "this variable was not observed." Addresses the incomplete observability risk (ARCHITECTURE.md 8.4). | Overuse of `havoc` makes the IR non-informative. The system must be calibrated to distinguish "genuinely unobserved" from "not logged because uninteresting." |
| P8 | **Refutation Chain Structure** | DRAT proofs | Sequence of independently verifiable inference steps building to a contradiction. | Stage 3 (theoretical evaluation) | MEDIUM | When the LFI classifies a failure as theoretical falsification, the refutation can be represented as a chain of steps: (1) hypothesis predicted X, (2) clean implementation produced Y, (3) X and Y contradict under conditions Z. Each step is independently checkable. | Scientific refutation is quantitative and probabilistic, not propositional. The chain structure transfers, but each step requires statistical rather than logical verification. |
| P9 | **State-Transition Traces with Invariant Checking** | AIGER model checking | Represent execution as state transitions, check invariants at each transition. | Stage 1 (numerical precision, resource state) | MEDIUM | DSL simulation traces are state-transition sequences. Invariants (energy conservation, mass conservation, numerical stability bounds) can be checked at each transition. Violations localize the implementation fault to a specific timestep. | Scientific simulations may have "soft" invariants (quantities that drift slowly rather than violating discretely). Requires threshold-based rather than binary invariant checking. |
| P10 | **Compositional Trace Summaries** | Infer | Analyze each procedure independently, produce a summary, compose summaries for interprocedural analysis. | Stage 1 (scalability), Stage 2 (phase-level methodology checks) | MEDIUM | For megabyte-scale traces, analyze each simulation phase independently (equilibration, production, analysis). Produce phase summaries (succeeded/failed, key metrics, invariant violations). Compose summaries for experiment-level fault classification. Enables streaming analysis without full-trace buffering. | Summary quality determines fault classification quality. Over-summarization loses causal detail needed for precise fault localization. |
| P11 | **Hierarchical Span Model** | Jaeger/Zipkin (OpenTelemetry) | Tree of spans with parent-child relationships, tags, and logs. | Stage 1 (execution structure), Stage 2 (experiment design structure) | MEDIUM | Natural hierarchy in DSL traces: experiment -> phase -> timestep -> kernel call. Span model captures this hierarchy with parent-child links. Tags carry domain metadata. Streaming-compatible (spans emitted individually). | Optimized for request-response, not batch simulation. May need adaptation for million-timestep simulations. |
| P12 | **Path Constraints as Failure Conditions** | Clang Static Analyzer | Track constraints along execution paths; report conditions under which bugs manifest. | Stage 1 (error path analysis), Stage 3 (conditions of falsification) | MEDIUM | Report not just "what failed" but "under what conditions it failed." A numerical overflow is more informative as "overflow when force > 1e15 AND timestep > 2fs" than as bare "overflow at step 47291." Supports the LFI in determining whether the failure conditions are theoretically relevant or implementation-specific. | Constraint extraction from numerical simulation traces requires domain-specific analysis. Not all failure conditions are expressible as simple constraints. |
| P13 | **Exploded Graph for Trace Analysis** | Clang Static Analyzer | Product of control flow and abstract state; bug paths are paths through this product graph. | Stage 1 (root cause localization within implementation) | LOW-MEDIUM | For complex implementation failures involving multiple interacting components, the exploded graph provides a systematic way to explore alternative causal paths through the trace. | Exploded graphs suffer state explosion. For high-dimensional simulation state, this is impractical without aggressive abstraction. |

---

## 5. Anti-Patterns

### AP1: Post-Mortem-Only Representation

**Description.** IRs designed exclusively for post-mortem analysis (after the full execution completes) that cannot be constructed or queried incrementally during execution.

**Sources where observed.** Traditional RCA event correlation systems; many incident management platforms.

**Why harmful for ATHENA.** ATHENA's trace IR must support streaming construction (Rust zero-copy parsing of megabyte-scale traces, per ADR 001). A post-mortem-only IR requires buffering the entire trace before analysis begins, creating memory pressure and latency. Additionally, early fault detection (identifying an implementation crash at timestep 100 of a million-timestep simulation) is only possible with incremental IR construction.

**Mitigation.** Adopt streaming-compatible patterns (P10 compositional summaries, P11 hierarchical spans, P8 DRAT-style incremental verification).

### AP2: Specification-Implementation Conflation

**Description.** Representing specification and implementation in the same namespace without structural distinction. Treating "what was intended" and "what happened" as the same kind of entity.

**Sources where observed.** LLM-based RCA systems (arxiv:2403.04123) where specifications and observations are both embedded in unstructured prompts. Also present in systems that log only execution events without recording the experiment specification.

**Why harmful for ATHENA.** The entire three-stage audit depends on separating specification from execution. If the IR conflates them, the LFI cannot determine whether a discrepancy indicates a specification violation (implementation bug) or a specification satisfaction that contradicts predictions (theoretical falsification). This is the single most harmful anti-pattern for ATHENA's design.

**Mitigation.** Adopt pattern P1 (dialect-based layer separation) or P2 (specification-implementation contracts) to enforce structural distinction.

### AP3: Implicit Causal Ordering

**Description.** Representing events in temporal order only, with causal relationships inferred ad-hoc rather than structurally encoded.

**Sources where observed.** Raw log files; most distributed tracing systems (Jaeger/Zipkin encode invocation structure but not domain-level causality); LLM-based RCA systems that rely on the LLM to infer causality from temporal sequences.

**Why harmful for ATHENA.** Temporal ordering is necessary but not sufficient for fault classification. "Event A happened before event B" does not establish that A caused B. ATHENA's IR must encode causal relationships (this parameter choice caused this force calculation result) structurally, so the LFI can traverse causal chains rather than correlating timestamps.

**Mitigation.** Adopt pattern P3 (typed event chains with evidence links) to make causal ordering explicit. Use the DSL's API structure to derive causal links: when a theory-layer operation triggers implementation-layer events through a known API call, that call constitutes a structural causal link.

### AP4: Lossy Compression Without Principled Selection

**Description.** Reducing trace size by discarding events without a principled strategy for what to keep. Context window limitations in LLM-based systems force this; some log aggregation systems also apply it.

**Sources where observed.** All LLM-based RCA systems (forced by context window limits). The "Confused" failure mode in arxiv:2601.22208 is partly caused by important events being dropped during context compression.

**Why harmful for ATHENA.** If the IR discards events that are critical for distinguishing implementation faults from methodological faults from theoretical contradictions, the LFI's classification becomes unreliable. Worse, the LFI may not know that critical information is missing, leading to confident but wrong classifications.

**Mitigation.** Adopt pattern P7 (havoc for unobserved state) to explicitly mark what was not captured. Adopt pattern P10 (compositional summaries) to compress traces while preserving layer-level information. If compression is necessary, it must be layer-aware: implementation-layer events can be summarized for methodology audit, but must be preserved in full for implementation audit.

### AP5: Flat Event Namespace

**Description.** Treating all trace events as the same kind of entity, differing only in their attributes. No structural distinction between an "out of memory" error, a "force field not converged" warning, and a "temperature reached equilibrium" observation.

**Sources where observed.** Generic log aggregation systems (ELK stack, Splunk) that index all log lines uniformly. Some structured logging frameworks that provide fields but not type hierarchies.

**Why harmful for ATHENA.** The three-stage audit requires routing events to the correct audit stage. If all events are flat, the routing decision must be made per-event by inspecting attributes, which is fragile (an event's attributes may not clearly indicate its layer) and slow (every event is examined by every stage). A typed hierarchy enables structural routing: implementation-layer events are checked only in Stage 1, methodology-layer events only in Stage 2.

**Mitigation.** Adopt pattern P1 (dialect-based layer separation) with P3 (typed event nodes). Define a type hierarchy rooted in the three audit stages.

### AP6: Binary Pass/Fail Without Failure Characterization

**Description.** Representing outcomes as binary (pass/fail) or scalar (accuracy score) without characterizing the nature of the failure.

**Sources where observed.** Generation-first AI co-scientist systems (Sakana V2, Google Co-Scientist, AI2 CodeScientist) that use scalar reward signals. Also present in CI/CD test result reporting.

**Why harmful for ATHENA.** This is the fundamental anti-pattern ATHENA's architecture is designed to avoid (ARCHITECTURE.md Section 2.1). A binary/scalar outcome collapses the three-way failure ambiguity that the LFI exists to resolve. If the IR represents outcomes as pass/fail, it has already lost the information needed for fault classification.

**Mitigation.** The IR must represent outcomes as structured failure records containing: the observation, the prediction, the discrepancy, and the execution context sufficient for the LFI to classify the discrepancy's cause. Pattern P2 (contracts) and P3 (event chains) together provide this structure.

---

## 6. Synthesis: Recommended IR Design Patterns for ATHENA

### 6.1 Primary Structural Patterns (Must-Have)

**L1: Dialect-based layer separation (P1).** Define three dialects: `theory` (user specifications, hypothesis predictions, causal claims), `methodology` (experiment design, variable control, sampling), `implementation` (framework execution, resource management, numerical computation). Every trace event belongs to exactly one dialect. The LFI audits dialects in order: implementation -> methodology -> theory.

**L2: Specification-implementation contracts (P2).** Each experiment carries a contract derived from its specification. The contract is a first-class entity in the IR, not embedded in the trace events. Implementation audit (Stage 1) checks trace events against implementation-level contract terms (preconditions on data, postconditions on execution). Methodology audit (Stage 2) checks the experiment specification against the hypothesis (is the contract adequate to test the claim?). Theory audit (Stage 3) checks results against theoretical predictions (did the implementation satisfying its contract produce results contradicting the theory?).

**L3: Typed event chains with evidence links (P3).** Every fault classification produced by the LFI must be backed by a chain of typed events linked by causal relationships with explicit evidence references. This ensures auditability and prevents the "Biased" failure mode (classifications unsupported by trace evidence).

### 6.2 Secondary Structural Patterns (Should-Have)

**L4: Ghost state for methodological metadata (P4).** Represent experiment-design properties (sampling sufficiency, confounder control) as ghost state, populated from the experiment specification and current DAG, not from the trace.

**L5: Havoc for unobserved state (P7).** Explicitly mark unobserved variables rather than treating their absence as irrelevant.

**L6: Compositional trace summaries (P10).** Enable scalable analysis of megabyte-scale traces by summarizing simulation phases independently and composing summaries.

### 6.3 Tertiary Patterns (Nice-to-Have)

**L7: SSA-style event identification (P5).** Unique, immutable event identifiers with explicit data-flow links.

**L8: Refutation chain structure (P8).** Machine-checkable refutation evidence for Stage 3 theoretical falsification.

**L9: Path constraints as failure conditions (P12).** Report conditions under which failures manifest, not just that they occurred.

### 6.4 Rust Implementation Compatibility

All primary and secondary patterns are compatible with Rust zero-copy/streaming parsing:
- Dialect tags are enum variants (zero-cost abstraction).
- Contracts are structured records parsed once from the experiment specification.
- Event chains can be constructed incrementally as trace events are parsed.
- Ghost state is populated before trace parsing begins (from experiment spec).
- Havoc markers are a variant in the value representation enum.
- Compositional summaries are produced per-phase, enabling streaming.

The tertiary patterns are also compatible but add implementation complexity:
- SSA-style IDs require a monotonic counter (trivial in Rust).
- Refutation chains require a proof-step data structure.
- Path constraints require symbolic expression representation.

---

## 7. Open Questions for Subsequent Investigations

1. **Dialect boundary definition per DSL.** How do we determine which trace events belong to the theory, methodology, or implementation dialect for each target DSL (OpenMM, GROMACS, VASP)? This likely requires the DSL trace format survey (Next Step 1 in FINDINGS.md).

2. **Causal link derivation.** How are causal links between events derived from DSL trace data? The DSL API structure provides some links (API call -> framework execution), but methodology-level and theory-level causality requires the causal DAG.

3. **Contract extraction automation.** Can experiment specification contracts be automatically extracted from DSL experiment scripts, or must they be manually specified? This determines the practicality of pattern P2.

4. **Ghost state validation.** How is methodological ghost state validated? If the experiment specification claims "confounders controlled" but the actual experiment does not control them, the ghost state is wrong. This connects to the DAG quality issue (ARCHITECTURE.md 8.3).

5. **Streaming completeness trade-off.** How much trace data must be buffered vs. streamed for each audit stage? Stage 1 (implementation) may be fully streaming; Stage 3 (theory) may require the complete trace to evaluate predictions.

6. **Quantitative refutation logic.** Pattern P8 (refutation chains) needs adaptation from propositional to quantitative/statistical reasoning. What does a "step" in a scientific refutation chain look like?
