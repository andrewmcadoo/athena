# Provenance Data Models and Scientific Workflow IRs: Survey for ATHENA Trace Semantics

**Date:** 2026-02-20
**Research question:** Can W3C PROV-DM (or extensions like ProvONE) represent the theory-implementation distinction central to ATHENA's trace semantics problem?
**Architecture references:** ARCHITECTURE.md §4.5 (Trace Semantics Engine), §5.3 (Fault Isolation Decision Tree)
**Key assessment criterion:** "Can this represent the theory-implementation distinction deterministically?"

---

## W3C PROV-DM Analysis

### Core Types

W3C PROV-DM (W3C Recommendation, 2013-04-30, Section 2) defines three core types that form the foundation of all provenance statements:

**Entity** (PROV-DM §2.1): A physical, digital, conceptual, or other kind of thing with some fixed aspects. Entities may be real or imaginary. An entity has a fixed set of attributes that were determined at its creation or that describe its nature. Examples: a document, a dataset, a value, an image.

**Activity** (PROV-DM §2.2): Something that occurs over a period of time and acts upon or with entities. Activities may include consuming, processing, transforming, modifying, relocating, using, or generating entities. An activity has a start time and an end time.

**Agent** (PROV-DM §2.3): Something that bears some form of responsibility for an activity taking place, for the existence of an entity, or for another agent's activity. An agent may be a software agent, a person, or an organization.

### Core Relations

Six core relations connect the three types (PROV-DM §2.1-2.5):

| Relation | Domain -> Range | Semantics | PROV-DM Section |
|:---|:---|:---|:---|
| `wasGeneratedBy` | Entity -> Activity | The entity was created by (or during) the activity | §2.1.2 |
| `used` | Activity -> Entity | The activity utilized the entity | §2.1.1 |
| `wasAttributedTo` | Entity -> Agent | The entity is ascribed to the agent | §2.3.1 |
| `wasDerivedFrom` | Entity -> Entity | The entity was transformed from, or constructed using, the source entity | §2.1.3 |
| `wasAssociatedWith` | Activity -> Agent | The activity was associated with (controlled by) the agent | §2.3.2 |
| `actedOnBehalfOf` | Agent -> Agent | The delegating agent was responsible for the activity of the delegate agent | §2.3.3 |

### Mapping to ATHENA Concepts

The critical question: does Entity-Activity-Agent map to theory-implementation-methodology?

**Proposed mapping (steel-manned):**

| PROV-DM Concept | ATHENA Concept | Justification | Fit Quality |
|:---|:---|:---|:---|
| Entity | Simulation state, parameter values, result datasets | Entities are "things with fixed aspects" -- parameter snapshots and result vectors fit naturally | Good |
| Activity | Simulation step, computation phase, experiment execution | Activities are time-bounded operations -- simulation steps fit naturally | Good |
| Agent | DSL framework (implementation), user specification (theory), experiment design (methodology) | Agents bear responsibility -- the framework, the user's theory, and the experimental design each bear different kinds of responsibility | Strained |

**The fundamental problem with this mapping:** PROV-DM's Agent concept is not designed to capture the *kind* of responsibility an agent bears. Two agents (the DSL framework and the user's theory specification) can both be `wasAssociatedWith` the same Activity, but the model does not structurally distinguish *implementation-agent responsibility* from *theory-agent responsibility*. The distinction is semantic, not structural.

In PROV-DM, if an OpenMM simulation step (Activity) fails, we can record that it `wasAssociatedWith` both an "OpenMM-framework" Agent and a "user-forcefield-spec" Agent. But the model does not encode that these agents operate at different ontological layers -- that one is implementation and the other is theory. That distinction would need to be imposed through:
1. A typing/classification system on Agents (using PROV-DM's extensibility attributes), or
2. An extension ontology that subclasses Agent into TheoryAgent, ImplementationAgent, MethodologyAgent.

Neither is natively available. PROV-DM is layer-agnostic by design.

**Assessment of the Entity mapping:** Strong. DSL trace events produce Entities (parameter snapshots, force values, trajectory frames, error codes) that are naturally representable. The `wasDerivedFrom` chain captures the causal lineage of how one state led to another.

**Assessment of the Activity mapping:** Strong with caveats. Simulation steps, energy minimizations, integration cycles are Activities with clear start/end times. However, PROV-DM Activities are flat -- they do not natively distinguish between a "theory-layer activity" (evaluating a force field equation) and an "implementation-layer activity" (allocating GPU memory). This distinction must be imposed externally.

**Assessment of the Agent mapping:** Weak for ATHENA's purposes. The theory-implementation-methodology trichotomy requires that the *source of causal responsibility* be structurally typed, not just named. PROV-DM Agents are untyped containers for responsibility attribution.

### Qualified Relations

PROV-DM §3 defines qualified relations that add structured detail to the core binary relations. Instead of a simple `Activity used Entity`, a qualified relation interposes an intermediate node:

- **Usage** (§3.1): Qualifies the `used` relation. Records *how* the entity was used (role, time, attributes).
- **Generation** (§3.2): Qualifies `wasGeneratedBy`. Records *how* the entity came into existence.
- **Derivation** (§3.3): Qualifies `wasDerivedFrom`. Records the activity and usage/generation involved.
- **Association** (§3.4): Qualifies `wasAssociatedWith`. Can attach a **Plan** to describe what the agent was supposed to do.
- **Delegation** (§3.5): Qualifies `actedOnBehalfOf`.
- **Attribution** (§3.6): Qualifies `wasAttributedTo`.
- **Influence** (§3.7): Generic qualified relation superclass.

**Relevance for fault classification:**

Qualified Usage and Generation provide the mechanism to record *what role* an entity played in an activity. For example, a qualified Usage could record that entity X was used "as-force-field-parameters" (theory-layer) while entity Y was used "as-gpu-memory-allocation" (implementation-layer). This is the closest PROV-DM gets to structurally encoding the theory-implementation distinction.

Qualified Association with Plans is architecturally significant. A Plan (PROV-DM §3.4, PROV-O §3.1.2) describes the set of actions or steps that the agent intended to follow. If we encode the *expected* simulation behavior as a Plan, and the *actual* execution as the Activity's trace, the Association's Plan becomes a representation of *predicted behavior* that can be compared against *observed behavior*. This is directly relevant to ATHENA's expected-vs-actual comparison in the LFI.

**Assessment:** Qualified relations substantially improve resolution. They allow encoding *how* entities participate in activities and *what plan* agents followed. However, the resolution is still attribute-based (key-value pairs on the qualification node), not structurally typed. The difference between "this parameter was used as theory input" and "this memory buffer was used as implementation resource" is recorded in attributes, not in the graph topology. This means fault classification queries must inspect attribute values rather than follow structural paths. For LFI Stage 1 (implementation audit), this is sufficient -- one can query for all activities with implementation-typed usage qualifiers and check their outputs. For LFI Stage 3 (theory evaluation), this is less clean -- the theory-layer derivation chain is mixed into the same graph as implementation-layer derivations, distinguishable only by attributes.

### PROV-CONSTRAINTS and LFI Audit Preconditions

W3C PROV-CONSTRAINTS (W3C Recommendation, 2013-04-30) defines a set of validity constraints and inferences on PROV documents. Key constraints relevant to ATHENA:

**Ordering constraints (§5):**
- **event-ordering-constraint**: All events within an activity occur between its start and end events. Provenance records must respect temporal ordering.
- **generation-generation-ordering**: An entity's generation event must precede any of its usage events.
- **usage-within-activity**: Usage events of an entity by an activity occur within that activity's time bounds.

These constraints enforce a consistent temporal ordering on the provenance graph. For ATHENA's LFI, this means the IR would inherit built-in consistency checks: if a trace claims that a result Entity was generated *before* the Activity that created it started, the constraint system flags an inconsistency. This is directly useful for LFI Stage 1 (implementation audit) -- temporal inconsistencies in the trace indicate implementation-layer failures (race conditions, incorrect logging, out-of-order execution).

**Typing constraints (§6):**
- **typing-constraint**: An identifier used as an Entity cannot also be used as an Activity (unless it is explicitly both, which is rare). Type consistency is enforced.

This is relevant but insufficient. It prevents type confusion within PROV's own types, but does not enforce the theory-implementation typing that ATHENA needs.

**Derivation constraints (§7):**
- **derivation-usage-generation-ordering**: If Entity e2 was derived from Entity e1 via Activity a, then a used e1 and generated e2, and usage(e1) precedes generation(e2).

This is directly useful for constructing causal chains. If the LFI needs to trace "was this output causally dependent on implementation parameter X or theory parameter Y?", the derivation chain provides the structural backbone for that query.

**Uniqueness constraints (§8):**
- **entity-uniqueness**: An entity's generation event is unique.
- **activity-uniqueness**: Start and end events are unique per activity.

These prevent ambiguous provenance records where the same entity appears to have been generated by multiple activities, which would make fault classification underdetermined.

**Assessment for LFI audit preconditions:** PROV-CONSTRAINTS can encode *some* LFI preconditions -- specifically, temporal consistency, derivation chain integrity, and uniqueness of causal origins. They cannot encode *domain-specific* preconditions like "this force calculation must use double precision" or "this parameter must be within physical bounds." Those require a domain constraint layer that PROV-CONSTRAINTS does not provide. PROV-CONSTRAINTS validates the *structure* of the provenance graph but not the *domain semantics* of the entities and activities within it.

### Strengths as IR Foundation

1. **Mature W3C standard** with formal semantics, multiple serialization formats (PROV-N, PROV-JSON, PROV-XML), and a validation framework.
2. **Derivation chains** naturally represent causal lineage of data transformations -- who produced what from what.
3. **Qualified relations** provide extensible resolution for recording roles, plans, and usage details.
4. **Temporal ordering** is built-in and formally constrained.
5. **Plans** provide a mechanism for expected-vs-actual comparison.
6. **Agent model** provides a starting point for responsibility attribution, even if it requires extension for ATHENA's needs.
7. **Interoperability**: PROV-based IRs would be queryable with standard SPARQL and compatible with provenance ecosystems in science.

### Limitations as IR Foundation

1. **No native theory-implementation distinction.** The most critical gap. PROV-DM is ontologically flat with respect to the *kind* of causal responsibility. ATHENA's three-layer model (theory, methodology, implementation) has no structural representation in base PROV-DM. It must be imposed via extension.
2. **Graph-based overhead.** PROV-DM is designed for RDF/linked data ecosystems. A faithful implementation involves triple stores, RDF serialization, SPARQL query engines. This is antithetical to the Rust-for-throughput requirement in ARCHITECTURE.md (ADR 001). The data model can be adopted without the RDF stack, but this means forfeiting the standard tooling advantage.
3. **No native failure semantics.** PROV-DM records *what happened* (provenance), not *what should have happened* (expectations) or *what went wrong* (fault). Plans partially address expectations, but there is no standard pattern for encoding "this activity failed" or "this entity diverges from its expected value."
4. **Granularity mismatch.** PROV-DM was designed for document/dataset provenance, not for fine-grained trace-level provenance of numerical computations. Recording every floating-point operation as an Entity-Activity pair would be prohibitively verbose. An abstraction layer is needed.
5. **No built-in fault classification.** The LFI's three-way output (implementation / methodology / theory) has no PROV-DM representation. Classification results would need a custom extension vocabulary.

---

## ProvONE (Scientific Workflow Extension)

### Additional Concepts

ProvONE (Provenance for scientific workflows, Belhajjame et al., 2015; derived from earlier work in Kepler/COMAD) extends PROV-DM with concepts specific to scientific workflows:

| ProvONE Concept | Definition | PROV-DM Superclass |
|:---|:---|:---|
| **Program** | A computational step or method in a workflow (prospective) | Entity (Plan subclass) |
| **Port** | An input or output interface of a Program | Entity |
| **Channel** | A data flow connection between Ports | Entity |
| **Controller** | A control-flow link between Programs (conditional, iterative) | Entity |
| **Workflow** | A composite Program containing sub-Programs | Program (subclass) |
| **Execution** | A particular run of a Workflow | Activity |
| **User** | The person or system initiating the workflow | Agent |
| **Document** | A data item consumed or produced | Entity |

ProvONE's key innovation is separating **prospective provenance** (the workflow definition: what was *supposed* to happen) from **retrospective provenance** (the execution trace: what *actually* happened). This separation is implemented through:
- **Prospective layer:** Program, Port, Channel, Controller, Workflow -- the template.
- **Retrospective layer:** Execution (of Program), Data artifacts (at Ports), actual data transfers (through Channels).

### Mapping to DSL Simulation Concepts

| ProvONE Concept | DSL Simulation Equivalent | Fit |
|:---|:---|:---|
| Workflow | Complete simulation pipeline (setup -> minimize -> equilibrate -> produce) | Good |
| Program | Individual simulation step (e.g., energy minimization) | Good |
| Port | Input parameter (force field, temperature) / output observable (energy, trajectory) | Good |
| Channel | Data flow between steps (minimized structure -> equilibration input) | Good |
| Controller | Conditional logic (convergence checks, timestep adaptors) | Moderate |
| Execution | Actual run of a simulation step | Good |
| User | Human researcher or ATHENA system | Adequate |

### Does ProvONE Address PROV-DM Gaps?

**Prospective/retrospective separation: YES, partially.** This is ProvONE's strongest contribution to the ATHENA problem. The prospective layer (workflow definition) corresponds to the *user's theoretical specification* -- the intended computation. The retrospective layer (execution trace) corresponds to *what the framework actually did*. This is structurally close to the theory-implementation distinction, but not identical:

- The prospective layer captures *what the user specified* (theory + methodology), not just theory.
- The retrospective layer captures *what executed* (implementation), including both correct execution of the specification and any implementation failures.
- The distinction is specification-vs-execution, not theory-vs-implementation. A methodological error (wrong metric, insufficient sampling) lives in the *prospective* layer -- it is part of the specification, not an implementation failure.

This means ProvONE provides a two-way split (specification vs. execution) rather than ATHENA's three-way split (theory vs. methodology vs. implementation). The methodology layer is collapsed into the specification layer.

**Program/Port structure: YES, helpful.** Programs with typed Ports provide a natural representation for DSL API boundaries. A Program that accepts "force-field parameters" through one Port (theory) and "GPU device ID" through another Port (implementation) structurally separates the two kinds of input. The Port typing is where the theory-implementation distinction can be encoded -- if Ports are classified as theory-Ports or implementation-Ports.

**Controllers: PARTIALLY.** Controllers capture control flow (loops, conditionals), which is relevant for methodology representation (e.g., "run until convergence" is a methodological choice). But Controllers in ProvONE are coarse -- they link Programs, not fine-grained operations within Programs.

**Assessment:** ProvONE moves closer to ATHENA's needs than base PROV-DM by providing: (a) prospective/retrospective separation, (b) typed Ports for parameter classification, and (c) a workflow structure for simulation pipelines. It does not solve the three-way distinction, but it provides a two-way structural backbone that could be extended with a third layer.

---

## Provenance Query Languages

### SPARQL over PROV

PROV-O (W3C Recommendation, 2013-04-30) is the OWL ontology for PROV, enabling SPARQL queries over provenance graphs stored as RDF triples. SPARQL is the standard query language for RDF data.

**Expressiveness for causal reasoning:**

The core causal query for ATHENA -- "was this output causally dependent on implementation parameter X or theory parameter Y?" -- can be expressed in SPARQL, but with significant caveats:

```sparql
# Find all entities in the causal ancestry of result R
# that were classified as theory-layer inputs
SELECT ?theoryInput ?activity
WHERE {
  ?result prov:wasDerivedFrom+ ?intermediate .
  ?intermediate prov:wasGeneratedBy ?activity .
  ?activity prov:qualifiedUsage ?usage .
  ?usage prov:entity ?theoryInput .
  ?usage athena:parameterLayer athena:Theory .
}
```

This query works if:
1. The `wasDerivedFrom` chain is complete (every derivation step is recorded).
2. Each usage is qualified with an ATHENA-specific `parameterLayer` attribute.
3. The graph is stored in a triple store with SPARQL support.

**Limitations of SPARQL for this purpose:**

- **Property paths** (`wasDerivedFrom+` for transitive closure) are supported in SPARQL 1.1 but can be expensive on large graphs.
- **Negation** (checking that no implementation-layer entity appears in the causal chain) requires `NOT EXISTS` patterns, which are supported but can be slow.
- **Aggregation over causal chains** (e.g., "what fraction of this result's ancestry is theory-layer vs. implementation-layer?") requires combining property paths with aggregation, which is complex and potentially slow.
- **Temporal reasoning** is not native to SPARQL. Queries like "did the implementation failure occur before the theory evaluation?" require explicit timestamp comparisons, not structural graph traversal.

### Can the Key Causal Query Be Expressed?

The query "was this output causally dependent on implementation parameter X or theory parameter Y?" can be decomposed:

1. **Causal dependency**: Follow `wasDerivedFrom` chains backward from the output. This is a graph reachability query, expressible in SPARQL 1.1 with property paths.
2. **Layer classification**: Each entity/activity in the chain must be tagged with its layer (theory/methodology/implementation). This requires the ATHENA extension vocabulary on every node.
3. **Deterministic answer**: The answer must be unambiguous -- "yes, this output depends on X" or "no, it does not." PROV-DM derivation chains provide this determinism *if* the chain is complete. Incomplete chains (missing intermediate derivations) make the answer indeterminate.

**Assessment:** The query is expressible in principle but requires: (a) complete derivation chains, (b) layer-tagged nodes, and (c) potentially expensive graph traversals. It is not a simple lookup.

### Performance for Megabyte-Scale Provenance Graphs

Megabyte-scale provenance graphs (the scale referenced in ARCHITECTURE.md for DSL traces) translate to roughly 10^4 to 10^6 triples, depending on encoding density.

| Storage Backend | Insert Rate (triples/sec) | Simple Query (ms) | Path Query (ms) | Notes |
|:---|:---|:---|:---|:---|
| Apache Jena TDB2 | ~50K | 1-10 | 10-1000 | Mature, Java-based |
| Oxigraph (Rust) | ~100K | <5 | 5-500 | Rust-native, but limited features |
| In-memory graph (petgraph/Rust) | >500K | <1 | 1-100 | No SPARQL, custom query needed |
| SQLite-backed (custom) | ~200K | 1-5 | 5-200 | Requires custom schema |

At megabyte scale, triple-store performance is adequate. The bottleneck is not storage or simple queries but transitive closure queries over long derivation chains, which can degrade on graphs with high branching factors. For a typical simulation trace (linear pipeline with moderate branching), path queries should complete in milliseconds.

**However:** The question is whether the triple-store overhead is justified. If ATHENA's IR only needs causal chain traversal and layer classification (not arbitrary SPARQL), a custom Rust graph structure (e.g., petgraph with typed edges) would be 10-100x faster for the specific queries needed.

---

## Scientific Workflow Provenance Systems

### Kepler

**Provenance model:** Kepler (Altintas et al., 2006) uses a process-oriented provenance model based on the actor-oriented dataflow paradigm. Each actor (processing step) has typed input/output ports. Provenance tracks data dependencies between actors.

**Relevance to ATHENA:** Kepler's typed-port model directly inspired ProvONE's Port concept. The distinction between actor definition (prospective) and actor execution (retrospective) maps to specification-vs-execution. Kepler records detailed token-level provenance: every data token flowing through a port is tracked.

**Fault analysis relevance:** Kepler's provenance can answer "which actor produced this erroneous output?" but cannot answer "was the error in the actor's implementation or its specification?" The provenance is execution-level, not semantic-level.

### Taverna

**Provenance model:** Taverna (Wolstencroft et al., 2013) records data provenance as a directed graph of data artifacts linked by processor invocations. It uses the OPM (Open Provenance Model, predecessor to PROV) and later adopted PROV-DM.

**Relevance to ATHENA:** Taverna's provenance model is coarser than Kepler's -- it tracks service invocations and data flow but not internal processor state. Its strength is in recording intermediate data products, which is useful for "where did this value come from?" queries.

**Fault analysis relevance:** Limited. Taverna provenance identifies *which service* produced unexpected output but does not provide internal execution traces within services. For DSL simulations, where the interesting failures occur *within* a simulation step (not between steps), Taverna-level provenance is too coarse.

### VisTrails

**Provenance model:** VisTrails (Callahan et al., 2006) uniquely tracks workflow *evolution* provenance -- not just data lineage within a single run, but how the workflow itself changed between runs. It records a version tree of workflow definitions.

**Relevance to ATHENA:** VisTrails' workflow evolution model is directly relevant to ATHENA's iterative hypothesis refinement. Each ATHENA cycle produces a modified experiment specification. VisTrails' approach of recording the *diff* between workflow versions (rather than full copies) could reduce storage requirements for ATHENA's audit trail. The version tree also supports "why did the user change this parameter?" queries.

**Fault analysis relevance:** Moderate. VisTrails can answer "what changed between the working run and the failing run?" -- a classic debugging question that maps to differential fault isolation. However, it does not classify the nature of the change (theory vs. implementation).

### Galaxy

**Provenance model:** Galaxy (Afgan et al., 2018) records dataset provenance as a DAG of tools and datasets. Each dataset has a complete lineage back to input data. Galaxy's provenance is stored in a relational database, not as RDF triples.

**Relevance to ATHENA:** Galaxy's relational approach to provenance storage is interesting as an alternative to RDF triple stores. The relational schema is simpler, faster for known query patterns, and more natural for Rust implementation. Galaxy also records tool versions and parameter values, which directly supports "which version of the implementation was used?" queries.

**Fault analysis relevance:** Galaxy's tool versioning provides a mechanism for LFI Stage 1: if the tool version changed between a working run and a failing run, this is evidence of an implementation-layer issue. Galaxy does not natively distinguish theory parameters from implementation parameters.

### Common Workflow Language (CWL)

**Provenance model:** CWL (Amstutz et al., 2016) defines a portable workflow description standard. CWL workflows are executed by various engines (cwltool, Toil, Arvados), each of which may produce different provenance formats. The CWL community has adopted Research Object Crates (RO-Crate) as the standard provenance container, which embeds PROV-DM provenance alongside workflow definitions and data artifacts.

**Relevance to ATHENA:** CWL's RO-Crate approach bundles prospective provenance (the CWL workflow definition), retrospective provenance (PROV-DM execution trace), and the actual data artifacts into a single package. This is the most mature implementation of the prospective/retrospective separation relevant to ATHENA. The CWL workflow definition serves as the "specification" against which execution can be compared.

**Fault analysis relevance:** CWL RO-Crates provide the structural backbone for expected-vs-actual comparison. The CWL definition says "step X should produce output Y given input Z." The PROV-DM trace records what actually happened. Divergence between the two is a fault signal. However, CWL operates at the tool level (coarse) not at the DSL-internal level (fine).

### Summary Table

| System | Provenance Model | Prospective/Retrospective? | Granularity | Theory-Impl Distinction? | Fault Classification? |
|:---|:---|:---|:---|:---|:---|
| Kepler | Actor dataflow | Yes | Token-level | No | Actor-level only |
| Taverna | OPM/PROV-DM | Partial | Service-level | No | Service-level only |
| VisTrails | Version tree + data | Yes (evolution) | Workflow-level | No | Differential only |
| Galaxy | Relational DAG | Partial | Tool-level | No | Version-based only |
| CWL/RO-Crate | PROV-DM + CWL def | Yes | Tool-level | No | Spec-vs-execution |

None of these systems natively support the theory-implementation distinction. All provide provenance at the workflow/tool level, not at the DSL-internal semantic level that ATHENA requires.

---

## Process Mining on Workflow Logs

### XES Event Log Standard

XES (eXtensible Event Stream, IEEE 1849-2016) is the standard format for event logs used in process mining. An XES log consists of:

- **Log**: Container with global attributes.
- **Trace**: A sequence of events belonging to a single case (process instance).
- **Event**: A single occurrence with attributes (activity name, timestamp, resource, lifecycle transition).

XES defines standard extensions:
- **Concept extension**: Activity names.
- **Time extension**: Timestamps.
- **Lifecycle extension**: Event lifecycle (start, complete, suspend, resume).
- **Organizational extension**: Resource (who performed the event).

**Mapping to DSL traces:**

| XES Concept | DSL Trace Equivalent | Notes |
|:---|:---|:---|
| Trace | Single simulation run | Natural mapping |
| Event | Simulation step, parameter change, error | Natural mapping |
| Activity name | Step type (minimize, equilibrate, etc.) | Natural mapping |
| Timestamp | Simulation time or wall-clock time | Depends on logging |
| Resource | DSL framework component | Partial -- does not distinguish theory/impl |
| Lifecycle | Step start/complete/fail | Useful for fault detection |

**Assessment:** XES provides a flat, sequential event representation that is simpler than PROV-DM's graph model. It is well-suited for recording *what happened* in temporal order but does not natively represent causal dependencies (wasDerivedFrom) or responsibility attribution (wasAssociatedWith). XES is a *log format*, not a *causal model*.

### Conformance Checking

Conformance checking is a process mining technique that compares an expected process model (normative) against an actual execution log (descriptive) to identify deviations. This is structurally parallel to ATHENA's expected-vs-actual comparison.

**Key techniques:**

1. **Token-based replay**: "Replay" the event log on the process model. Count missing tokens (the model expected an event that did not occur) and remaining tokens (events occurred that the model did not expect). Produces a fitness score.

2. **Alignment-based conformance**: Find the optimal alignment between the log trace and the model trace. Each deviation is explicitly identified as either a "move on log" (unexpected event) or "move on model" (missing event). This provides fine-grained deviation analysis.

3. **Declarative conformance**: Check the log against declarative constraints (e.g., "activity A always precedes activity B", "activity C occurs at most once"). Violations are enumerated.

**Relevance to ATHENA's LFI:**

Alignment-based conformance checking is directly relevant to LFI Stages 1 and 2:

- **Stage 1 (implementation audit)**: The expected process model encodes the correct implementation behavior. "Moves on log" that are not in the model represent unexpected implementation events (crashes, exceptions, unexpected state transitions). "Moves on model" that are not in the log represent missing implementation steps (incomplete execution).

- **Stage 2 (methodology audit)**: The expected process model encodes the experimental methodology. Deviations between the methodology specification and the actual execution indicate methodological failures (wrong metric computed, insufficient sampling performed).

The limitation is that conformance checking requires two separate models -- one for correct implementation and one for sound methodology. These models must exist before the check can be performed, and their construction is not trivial.

### Discovery Algorithms for Causal Structure

Process mining discovery algorithms (Alpha Miner, Inductive Miner, Heuristic Miner, Split Miner) automatically extract process models from event logs. These are relevant to ATHENA's Epistemic Exploration Phase (constructing causal structure from observations) more than to the Trace Semantics Engine directly.

**Alpha Miner** discovers Petri net models from logs using ordering relations (direct succession, causality, parallelism, choice). It assumes complete logs and produces sound workflow nets.

**Inductive Miner** recursively decomposes logs and guarantees soundness (no deadlocks). More robust to noise than Alpha Miner.

**Heuristic Miner** uses frequency-based heuristics to discover models, handling noise and incomplete logs better than Alpha Miner but without formal guarantees.

**Assessment:** Discovery algorithms could augment the Epistemic Explorer by automatically extracting process models from DSL execution traces, which could then be used as the "expected model" for conformance checking. However, these algorithms discover *control flow* structure, not *causal* structure. The process model they produce says "A happens before B" but not "A causes B." ATHENA's causal DAG requires the stronger causal claim.

---

## Scalability Assessment

### PROV Graphs for Megabyte-Scale Traces

A typical DSL simulation trace (e.g., an OpenMM molecular dynamics run) at megabyte scale contains:
- 10^3 to 10^5 timesteps
- 10^1 to 10^3 parameters per timestep
- 10^1 to 10^2 observable values per timestep
- Total: 10^4 to 10^7 data points

Encoding each data point as a PROV Entity with Generation, Usage, and Derivation relations produces approximately 3-5 triples per data point, yielding 10^4 to 5x10^7 triples.

| Scale | Triples | Storage (N-Triples) | Storage (binary) | Load Time | Path Query |
|:---|:---|:---|:---|:---|:---|
| Small (KB trace) | ~10^4 | ~1 MB | ~200 KB | <1s | <10ms |
| Medium (MB trace) | ~10^6 | ~100 MB | ~20 MB | 5-30s | 10-500ms |
| Large (10 MB trace) | ~10^7 | ~1 GB | ~200 MB | 30-300s | 100ms-10s |

At the medium scale (megabyte traces), PROV graph construction and basic queries are feasible but not fast. Path queries over long derivation chains (which is what the LFI needs) can take hundreds of milliseconds -- acceptable for single queries but problematic if the LFI needs hundreds of such queries per fault classification.

### Storage and Query Performance

**Triple store approach (RDF):**
- Pros: Standard SPARQL queries, interoperability, inference support.
- Cons: High storage overhead (3-5x the raw data), slow for custom queries, Java/C++ implementations do not integrate well with Rust.
- Performance: Adequate at megabyte scale; prohibitive at gigabyte scale.

**Custom graph approach (Rust):**
- Pros: Minimal storage overhead, custom indexing for common query patterns, native Rust performance.
- Cons: No standard query language, must implement all traversal algorithms, no free interoperability.
- Performance: 10-100x faster than triple stores for targeted queries. Petgraph or custom adjacency list structures handle 10^6-node graphs efficiently.

**Relational approach (SQLite/similar):**
- Pros: Moderate storage overhead, SQL queries, good Rust bindings (rusqlite).
- Cons: Graph traversal (transitive closure) is awkward in SQL, requires recursive CTEs.
- Performance: Between triple store and custom graph. Good for point queries, adequate for short path queries, poor for deep transitive closure.

### Streaming/Incremental Construction

DSL traces are generated sequentially -- events arrive in temporal order as the simulation runs. The IR must support incremental construction: adding nodes and edges as events arrive, not requiring the full trace to be loaded before analysis begins.

**PROV-DM compatibility with streaming:**
- Generation events create new Entity nodes and wasGeneratedBy edges. These are naturally incremental -- each generation event adds to the graph independently.
- Usage events add edges to existing nodes. These require the used Entity to already exist in the graph. For forward-flowing simulation traces, this is satisfied (inputs are generated before they are used).
- Derivation events may reference entities generated earlier. For linear pipelines, derivation chains grow monotonically. For iterative workflows (loops), back-references may require updating existing graph regions.

**Rust implementation considerations:**
- Arena-based allocation (e.g., typed-arena or bumpalo) is well-suited for append-mostly provenance graphs.
- A streaming parser that converts DSL log lines to PROV-like events can operate in O(1) memory per event if only forward references exist.
- Iterative DSL workflows (e.g., convergence loops) require back-references and may need O(n) memory for the current iteration's graph fragment.

**Assessment:** Streaming construction is feasible for PROV-like graphs in Rust, with the caveat that iterative workflows require careful handling. The PROV data model does not preclude incremental construction, but it was designed for post-hoc provenance recording, not real-time streaming. An ATHENA-specific adaptation would need to define the streaming protocol explicitly.

### Comparison with Flat Event Log Approaches

| Dimension | PROV Graph | Flat Event Log (XES-like) |
|:---|:---|:---|
| **Causal dependency** | Explicit (wasDerivedFrom edges) | Implicit (must be inferred from temporal order) |
| **Storage overhead** | 3-5x raw data | 1-2x raw data |
| **Query for "what caused X?"** | Graph traversal (fast, direct) | Full log scan + inference (slow, uncertain) |
| **Query for "what happened at time T?"** | Index on temporal attributes (moderate) | Direct scan (fast) |
| **Incremental construction** | Feasible, needs forward-reference management | Trivial (append-only) |
| **Rust implementation cost** | Moderate (custom graph library) | Low (Vec of structs) |
| **Fault classification support** | Structural (follow derivation chains) | Analytical (pattern match on event sequences) |

**Key tradeoff:** Flat event logs are simpler, faster, and cheaper, but they require *inference* to determine causal dependencies. PROV graphs are more expensive but make causal dependencies *explicit*. For ATHENA's LFI, which needs to deterministically trace causal chains to classify faults, explicit causal dependencies are strongly preferred. The cost of causal inference from flat logs is both computational (must run discovery algorithms) and epistemic (the inferred causal structure may be wrong).

**Hybrid approach consideration:** A staged approach where raw DSL logs are captured as flat events (fast, cheap), then incrementally converted to a PROV-like causal graph during analysis (when the LFI needs to trace a specific failure). This amortizes the graph construction cost to only the regions the LFI actually inspects, rather than building the full graph for every trace.

---

## Expected vs. Actual Outcome Representation

### How Provenance Models Represent Predictions vs. Observations

Standard PROV-DM does not natively distinguish predictions from observations. Both are Entities. However, several patterns exist for encoding the expected-vs-actual distinction:

**Pattern 1: Dual-entity with specialization.**
Encode the expected result as one Entity and the actual result as another. Link them with `prov:alternateOf` (PROV-DM §4.1) -- both are alternate representations of "the result of this experiment." The expected entity is generated by a "prediction Activity" (running the hypothesis through the causal model); the actual entity is generated by the "execution Activity" (running the real experiment).

```
expected_result  prov:alternateOf  actual_result .
expected_result  prov:wasGeneratedBy  prediction_activity .
actual_result    prov:wasGeneratedBy  execution_activity .
```

This pattern preserves both values and their provenance. A query for "does the prediction match the observation?" compares the two entities' attribute values.

**Limitation:** `alternateOf` in PROV-DM means "both describe the same thing" -- it does not inherently carry the semantics of "one is predicted, the other is observed." That must be encoded in type attributes.

**Pattern 2: Plans as expected behavior.**
PROV-DM Plans (§3.4) describe the intended steps of an Activity. If the Plan encodes specific expected outputs (e.g., "energy should decrease monotonically during minimization"), divergence between the Plan's expectations and the Activity's actual outputs signals a potential failure.

```
experiment  prov:qualifiedAssociation  [
    prov:agent      athena_system ;
    prov:hadPlan    expected_behavior_plan
] .
```

The Plan Entity can carry structured attributes encoding expected output ranges, monotonicity constraints, convergence criteria, etc. Post-execution, comparing the actual Entity attributes against the Plan's expected attributes produces a divergence signal.

**Limitation:** Plans in PROV-DM are informal -- they are Entities with no prescribed internal structure. ATHENA would need to define a formal Plan vocabulary for expected outcomes.

**Pattern 3: Invalidation for failed expectations.**
PROV-DM defines `wasInvalidatedBy` (§2.4) -- an Entity can be invalidated by an Activity. If an expected-result Entity is invalidated by the comparison Activity (because the actual result diverges), this provides a structural record of the expectation failure.

```
expected_result  prov:wasInvalidatedBy  comparison_activity .
comparison_activity  prov:generated  divergence_report .
```

This pattern records *that* the expectation failed and links it to a divergence report Entity. However, it does not classify the failure (theory/methodology/implementation). That classification is ATHENA-specific.

### Standard Patterns for Expected-vs-Actual Divergence

Across provenance and workflow systems, three general approaches exist:

1. **Provenance diff** (VisTrails-inspired): Compare the provenance graphs of two runs -- one expected (a reference run or a prediction) and one actual. Structural differences in the graphs indicate where execution diverged from expectation. This is powerful for identifying *where* divergence occurred but does not classify *why*.

2. **Constraint-based checking** (PROV-CONSTRAINTS-inspired): Encode expected properties as constraints on the provenance graph. Validate the actual graph against these constraints. Violations identify where expectations were not met. This is closer to ATHENA's needs -- constraints can be typed as "theory constraints" (expected by the hypothesis), "methodology constraints" (required by the experimental design), or "implementation constraints" (required by the DSL framework).

3. **Oracle comparison** (process mining-inspired): Define an oracle process model (the expected behavior) and compare the actual event log against it using conformance checking. Deviations are classified as "moves on log only" (unexpected events) or "moves on model only" (missing events). This is the process mining equivalent of expected-vs-actual comparison.

**Assessment for ATHENA:** Pattern 2 (Plans with formal constraint vocabulary) combined with Pattern 3 (constraint-based checking with typed constraints) provides the best foundation for ATHENA's expected-vs-actual representation. Plans encode what *should* happen; typed constraints classify *which layer* each expectation belongs to; constraint violations identify *where and what kind* of divergence occurred.

---

## Transferable Patterns Catalog

### Pattern 1: Derivation Chain Traversal

- **Source:** W3C PROV-DM §2.1.3 (wasDerivedFrom), §3.3 (qualified Derivation)
- **Mechanism:** Entities linked by wasDerivedFrom form directed acyclic chains. Traversing backward from an output Entity to its causal ancestors identifies all entities that contributed to the output's value.
- **Transferability:** HIGH. DSL simulation traces produce sequential state transformations that are naturally expressible as derivation chains. The mechanism is domain-agnostic.
- **LFI stage supported:** Stage 3 (Theory Evaluation) -- tracing which theory parameters are in the causal ancestry of a failed prediction. Also Stage 1 (Implementation Audit) -- tracing whether an implementation entity is in the causal chain.
- **Limitations:** Requires complete derivation recording. Missing intermediate derivations break the chain and make causal queries underdetermined. For high-frequency simulation data (nanosecond timesteps), recording every derivation may be prohibitively expensive.

### Pattern 2: Prospective/Retrospective Separation

- **Source:** ProvONE (Belhajjame et al., 2015); CWL RO-Crate provenance
- **Mechanism:** Separate the workflow definition (what should happen) from the execution trace (what did happen). The prospective layer defines Programs, Ports, and data flow topology. The retrospective layer records Executions, actual data values, and timing.
- **Transferability:** HIGH. DSL frameworks inherently separate the user specification (Python script defining the simulation) from the execution (framework running the simulation). This maps directly to prospective/retrospective.
- **LFI stage supported:** Stage 1 (Implementation Audit) -- compare prospective specification against retrospective execution to identify implementation deviations. Stage 2 (Methodology Audit) -- the prospective layer encodes the experimental methodology, which can be audited independently.
- **Limitations:** The mapping is two-way (specification vs. execution), not three-way (theory vs. methodology vs. implementation). ATHENA would need to further split the prospective layer into theory and methodology sub-layers.

### Pattern 3: Typed Ports for Parameter Classification

- **Source:** ProvONE Port concept; Kepler actor model
- **Mechanism:** Each computational step has typed input/output Ports. Each Port is classified by the kind of data it handles. In ATHENA's adaptation, Ports would be classified as theory-layer (force field parameters, equation coefficients), methodology-layer (sampling frequency, convergence criteria), or implementation-layer (GPU device, memory buffer, precision mode).
- **Transferability:** HIGH. DSL frameworks already expose typed APIs. OpenMM's `System`, `ForceField`, `Integrator` classes have clearly typed parameters. Port classification maps to API parameter classification.
- **LFI stage supported:** All three stages. Port typing determines which parameters are audited at each stage. Theory-Ports are only examined at Stage 3. Implementation-Ports are examined at Stage 1.
- **Limitations:** Requires domain-specific Port classification for each DSL. The classification is not always clean -- some parameters (e.g., integration timestep) straddle theory and methodology.

### Pattern 4: Conformance Checking for Fault Detection

- **Source:** Process mining (van der Aalst, 2016); alignment-based conformance
- **Mechanism:** Compare an expected process model (normative) against an actual execution trace. Identify deviations as "unexpected events" or "missing events." Classify deviations by location in the process model.
- **Transferability:** MODERATE. DSL simulations have a well-defined expected process flow (setup -> minimize -> equilibrate -> produce). Deviations from this flow (steps skipped, unexpected errors, out-of-order execution) are detectable via conformance checking.
- **LFI stage supported:** Stage 1 (Implementation Audit) -- deviations from the expected implementation process indicate implementation failures. Stage 2 (Methodology Audit) -- deviations from the expected experimental protocol indicate methodology failures.
- **Limitations:** Requires a pre-defined expected process model. The model must be constructed per experiment, which adds complexity. Conformance checking operates on control flow, not data values -- it detects *structural* deviations but not *value* deviations (e.g., energy too high).

### Pattern 5: Plans as Hypothesis Predictions

- **Source:** W3C PROV-DM §3.4 (qualified Association with Plan); PROV-O §3.1.2
- **Mechanism:** Encode the hypothesis's expected behavior as a Plan Entity attached to the experiment's Association. After execution, compare the Plan's expected outcomes against actual outcomes. Divergence is a fault signal.
- **Transferability:** MODERATE. Requires formalizing the hypothesis's predictions into a structured Plan format. For quantitative predictions (expected energy, expected trajectory), this is feasible. For qualitative predictions (expected phase transition, expected binding mode), formalization is harder.
- **LFI stage supported:** Stage 3 (Theory Evaluation) -- the Plan encodes the theory's predictions. Divergence between Plan and actual (after Stages 1 and 2 pass) indicates theoretical falsification.
- **Limitations:** Plan formalization is non-trivial. Plans in PROV-DM are unstructured Entities; ATHENA would need a formal Plan vocabulary. Also, some hypotheses make fuzzy predictions that are difficult to encode as precise Plans.

### Pattern 6: Provenance Graph Versioning for Iterative Refinement

- **Source:** VisTrails version tree; PROV-DM Bundle mechanism (§5)
- **Mechanism:** Each ATHENA cycle produces a provenance graph for one experiment. The version tree links graphs across cycles, recording how the experimental specification evolved. Bundles (named provenance sub-graphs) group provenance records by cycle.
- **Transferability:** HIGH. ATHENA's iterative loop naturally produces a sequence of provenance graphs. Bundles provide a standard mechanism for grouping and linking them.
- **LFI stage supported:** Cross-cycle analysis -- comparing provenance across cycles to identify persistent failure patterns or regression.
- **Limitations:** Bundle management adds storage overhead. For 50 cycles (the litmus test limit), this is manageable. For longer runs, pruning strategies are needed.

### Pattern 7: Agent Delegation for Multi-Layer Responsibility

- **Source:** W3C PROV-DM §2.3.3 (actedOnBehalfOf)
- **Mechanism:** Model the DSL framework as an Agent that `actedOnBehalfOf` the user's specification Agent. The user specifies the theory; the framework implements it. Delegation captures the relationship "the framework acted on behalf of the user's instructions."
- **Transferability:** MODERATE. The delegation model captures the *relationship* between theory and implementation agents but does not distinguish their *failure modes*. Knowing that the framework acted on behalf of the user does not tell you which agent caused a failure.
- **LFI stage supported:** Contextual -- provides the structural relationship needed to attribute failures to the correct agent. If the framework (delegate) failed, it is an implementation fault. If the user's specification (delegator) was wrong, it is a theory fault.
- **Limitations:** The delegation model is binary (delegator/delegate). ATHENA's three-way model requires at least two delegation levels: user specifies theory, methodology translates theory to experiment, framework implements experiment. This is representable but cumbersome.

---

## Anti-Patterns

### Anti-Pattern 1: Full-Granularity Entity Recording

**Description:** Recording every floating-point value, every memory allocation, and every intermediate computational step as a separate PROV Entity with full derivation chains.

**Why harmful:** At DSL simulation scale (millions of timesteps, thousands of atoms), this produces provenance graphs with 10^8+ nodes. Storage becomes gigabytes; queries become seconds to minutes; incremental construction requires sustained high write throughput. The graph becomes too large to analyze efficiently, defeating the purpose of the IR.

**ATHENA-specific harm:** The LFI needs to trace causal chains for a *specific failure*, not analyze the entire simulation provenance. Over-recording creates a "needle in a haystack" problem where the relevant causal chain is buried in irrelevant provenance.

**Alternative:** Selective recording. Record provenance at the granularity of DSL API calls (simulation steps, parameter changes, observable outputs), not at the granularity of internal computations. Record fine-grained provenance only in regions flagged as potentially faulty (adaptive granularity).

### Anti-Pattern 2: Untyped Agent Proliferation

**Description:** Creating a separate Agent for every software component involved in a simulation (the Python interpreter, the CUDA driver, the OpenMM kernel, the file system) without a classification hierarchy.

**Why harmful:** With dozens of untyped agents associated with every activity, determining *which agent's failure caused the outcome* requires exhaustive inspection of all agents. The flat Agent model provides no structural guidance for the outside-in audit order that the LFI requires.

**ATHENA-specific harm:** LFI Stage 1 must check implementation agents before Stage 2 checks methodology agents before Stage 3 checks theory agents. Without agent typing, the LFI cannot determine the audit order from the graph structure. It must rely on external metadata.

**Alternative:** Strict agent hierarchy. Classify agents into exactly three layers (theory, methodology, implementation) and record the classification as a structural type, not an attribute. This enables the LFI to query "all implementation-layer agents" directly.

### Anti-Pattern 3: Treating PROV-DM as a Complete IR

**Description:** Adopting PROV-DM as-is, without extension, and attempting to encode all of ATHENA's trace semantics within the base vocabulary.

**Why harmful:** PROV-DM lacks: (a) the theory-implementation-methodology trichotomy, (b) failure/fault semantics, (c) expected-vs-actual comparison primitives, (d) domain constraint representation. Forcing these concepts into PROV-DM's generic Entity-Activity-Agent model produces an IR that is technically valid PROV but semantically impoverished. Queries become complex attribute-matching operations rather than simple graph traversals.

**ATHENA-specific harm:** The LFI's three-stage audit requires the IR to structurally separate theory, methodology, and implementation layers. If these are encoded only as attributes on generic PROV nodes, every LFI query must filter by attributes, adding complexity and reducing performance.

**Alternative:** Use PROV-DM as a *foundation*, not a *complete IR*. Define an ATHENA-specific extension vocabulary that adds: (a) typed agents (TheoryAgent, MethodologyAgent, ImplementationAgent), (b) fault entities (FaultClassification with theory/methodology/implementation value), (c) expectation entities (Prediction, linked to Plan), (d) domain constraint entities.

### Anti-Pattern 4: RDF Triple Store as Primary Storage

**Description:** Storing the ATHENA IR in a standard RDF triple store (Jena, Virtuoso, etc.) and querying it with SPARQL.

**Why harmful for ATHENA:** Triple stores are optimized for schema-flexible, interoperable data. ATHENA's IR has a known, fixed schema (theory/methodology/implementation layers, derivation chains, agent hierarchies). Using a schema-flexible store for fixed-schema data forfeits performance. Additionally, standard triple stores are Java or C++ implementations that do not integrate with the Rust core requirement (ADR 001).

**ATHENA-specific harm:** The Rust-for-throughput decision means the IR engine must be a native Rust component. No mature Rust RDF triple stores exist at production quality (Oxigraph is closest but lacks key features). Building a custom triple store in Rust to gain SPARQL compatibility is high-cost engineering with uncertain payoff.

**Alternative:** Adopt PROV-DM's *data model* (Entity-Activity-Agent, derivation chains, qualified relations) without adopting its *serialization format* (RDF triples) or *query language* (SPARQL). Implement the data model in a Rust-native graph structure (petgraph or custom) with purpose-built query functions for LFI operations.

### Anti-Pattern 5: Ignoring Temporal Ordering in Favor of Pure Causal Structure

**Description:** Recording only causal dependencies (wasDerivedFrom) without temporal information (timestamps, event ordering).

**Why harmful:** The LFI's outside-in audit requires temporal reasoning. "Did the implementation failure occur before the theory was evaluated?" is a temporal question that cannot be answered from causal structure alone. Two entities may be causally independent but temporally ordered in a way that determines the fault classification.

**ATHENA-specific harm:** Without temporal ordering, the LFI cannot determine whether an implementation anomaly *preceded* the unexpected result (suggesting the anomaly caused it) or *followed* it (suggesting the anomaly is a consequence, not a cause). This ambiguity undermines fault classification.

**Alternative:** Always record both causal dependencies and temporal ordering. PROV-DM's event model (generation time, usage time, activity start/end time) provides the temporal backbone. The Rust IR should store timestamps alongside graph edges.

---

## Overall Assessment: PROV-DM as IR Foundation for ATHENA

### Verdict: Viable foundation with mandatory extensions. Not usable as-is.

PROV-DM provides approximately 60-70% of what ATHENA's Trace Semantics Engine needs. Its Entity-Activity-Agent model, derivation chains, qualified relations, and temporal constraints form a solid structural backbone for provenance-based causal reasoning. The W3C standardization provides formal semantics, validation rules, and a shared vocabulary.

However, PROV-DM has three critical gaps that must be addressed through extension:

**Gap 1: No theory-implementation-methodology trichotomy.** This is the central requirement that PROV-DM does not meet. Agents, Activities, and Entities are all untyped with respect to the theory/methodology/implementation distinction. This must be added as an extension ontology that classifies PROV nodes into the three layers. The extension is structurally straightforward (subclass Agent into TheoryAgent, MethodologyAgent, ImplementationAgent; similarly for Activities and Entities) but domain-dependent (the classification rules vary per DSL).

**Gap 2: No failure/fault semantics.** PROV-DM records provenance (what happened and why), not diagnostics (what went wrong and whose fault it was). The LFI's output -- a fault classification with causal attribution -- has no PROV-DM representation. An ATHENA fault vocabulary is needed.

**Gap 3: No expected-vs-actual primitives.** Plans partially address this, but PROV-DM Plans are unstructured entities. A formal vocabulary for encoding predictions, comparing them against observations, and recording divergence is required.

### Decision Gate 2 Implications

If PROV-DM is adopted as the foundation (with the three extensions above), the risk profile is:

- **Lower risk:** The core data model is mature, formally specified, and battle-tested. ATHENA does not need to invent a new provenance model from scratch.
- **Moderate risk:** The extensions are non-trivial but well-understood. Subclassing PROV types is a standard OWL/ontology engineering task. The domain-specific classification rules require per-DSL research.
- **The Rust implementation risk is the highest residual risk.** PROV-DM is designed for RDF/linked data ecosystems. Implementing a PROV-DM-compatible data model in Rust without RDF infrastructure requires building custom graph structures, query functions, and validation logic. The alternative (using an RDF store from Rust via FFI) adds complexity and latency.

If PROV-DM is rejected, the alternative is a novel IR design. This is higher risk (no existing specification to build on) but offers the opportunity to design specifically for ATHENA's needs (three-layer typing built into the core model, fault semantics as primitives, Rust-native from the start).

**Recommendation:** Adopt the PROV-DM *data model* (concepts, relations, qualified patterns) as the conceptual foundation. Do not adopt the PROV-DM *technology stack* (RDF, SPARQL, OWL). Implement a Rust-native graph structure that is *PROV-DM-compatible* (the same concepts, representable as PROV if needed) but *ATHENA-specific* (three-layer typing, fault semantics, expected-vs-actual primitives built in). This captures the maturity benefits of PROV-DM without the performance costs of the RDF stack.

### Comparison Matrix: IR Approaches

| Approach | Theory-Impl Distinction | Fault Classification | Performance (Rust) | Maturity | Interoperability |
|:---|:---|:---|:---|:---|:---|
| PROV-DM as-is | No (attribute-only) | No | Poor (RDF overhead) | High | High |
| PROV-DM + extensions | Yes (via subclasses) | Yes (via extension) | Moderate (custom impl) | Moderate | Moderate |
| ProvONE | Partial (2-way split) | No | Poor (RDF overhead) | Moderate | Moderate |
| Flat event log (XES-like) | No | No | Excellent | High | Low |
| Novel ATHENA IR | Yes (native) | Yes (native) | Excellent | Low | Low |
| **Hybrid: PROV-DM model, Rust impl** | **Yes (native + compatible)** | **Yes (native + compatible)** | **Good** | **Moderate** | **Moderate** |

The hybrid approach (last row) offers the best balance for ATHENA's requirements. It captures the conceptual maturity of PROV-DM, adds the missing ATHENA-specific semantics as first-class concepts, and implements everything in Rust-native structures for throughput.

---

## References

- W3C PROV-DM: The PROV Data Model. W3C Recommendation, 2013-04-30. https://www.w3.org/TR/prov-dm/
- W3C PROV-O: The PROV Ontology. W3C Recommendation, 2013-04-30. https://www.w3.org/TR/prov-o/
- W3C PROV-CONSTRAINTS: Constraints of the PROV Data Model. W3C Recommendation, 2013-04-30. https://www.w3.org/TR/prov-constraints/
- W3C PROV-N: The Provenance Notation. W3C Recommendation, 2013-04-30. https://www.w3.org/TR/prov-n/
- Belhajjame, K., et al. "Using a suite of ontologies for preserving workflow-centric research objects." Journal of Web Semantics, 32, 2015. (ProvONE specification)
- Altintas, I., et al. "Provenance Collection Support in the Kepler Scientific Workflow System." IPAW, 2006.
- Callahan, S. P., et al. "VisTrails: Visualization meets Data Management." SIGMOD, 2006.
- Wolstencroft, K., et al. "The Taverna workflow suite." Nucleic Acids Research, 41(W1), 2013.
- Afgan, E., et al. "The Galaxy platform for accessible, reproducible and collaborative biomedical analyses." Nucleic Acids Research, 46(W1), 2018.
- Amstutz, P., et al. "Common Workflow Language, v1.0." Specification, 2016. https://www.commonwl.org/
- IEEE 1849-2016: XES Standard Definition. IEEE, 2016. https://xes-standard.org/
- van der Aalst, W. M. P. "Process Mining: Data Science in Action." Springer, 2016.
- Günther, C. W. and Verbeek, H. M. W. "XES Standard Definition." BPM Center Report, 2014.
- Soares, E., et al. "ProvenanceRO-Crate: A lightweight approach to packaging and sharing provenance in scientific workflows." Future Generation Computer Systems, 2022.
