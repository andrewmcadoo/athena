# ADR 001: Python + Rust Core with PyO3/Maturin Interop

## Status

Accepted — 2026-02-20

## Context

ATHENA's architecture (ARCHITECTURE.md) identifies eleven components across five functional groups. Three components contain performance-critical inner loops that will dominate execution time at scale:

1. **Causal Graph Manager** (Section 4.2) — The single most connected component in the architecture. It receives update directives from four components and is queried by four others. Every analysis decision depends on its response time. Graph traversal, edge update, cycle detection, and versioned snapshotting are all latency-sensitive operations that scale with graph size. Identified as the single point of failure (Section 8.2).

2. **Trace Semantics Engine** (Section 4.5) — Translates raw DSL trace logs into structured semantic IR. Trace log size scales linearly with experiment complexity and simulation duration. Parsing throughput directly gates the falsification loop's cycle time.

3. **Bayesian Surprise Evaluator** (Section 4.5) — Computes KL divergence between prior and posterior beliefs. The pre-experiment forward simulation sub-responsibility (estimating predicted information gain across candidate experiments) is dense numerical work over DAG structures. This is the computational bottleneck for the Adversarial Experiment Designer's selection process.

The remaining eight components — Structural Prior Generator, Epistemic Explorer, Hypothesis Generator, Adversarial Experiment Designer, Experiment Executor, Lakatosian Fault Isolator, DSL Environment Interface, and Mode Controller — are orchestration-heavy. They coordinate LLM calls, manage state machines, interface with external DSL frameworks, and compose results from the core components. These benefit from rapid prototyping, extensive library ecosystems, and flexibility over raw throughput.

## Decision

**Rust** for the three performance-critical components: Causal Graph Manager, Trace Semantics Engine, Bayesian Surprise Evaluator.

**Python** for everything else: orchestration, LLM integration, DSL framework bindings, mode control, hypothesis generation, experiment execution.

**PyO3/maturin** for the interop boundary: Rust components are compiled as Python-importable native modules via maturin, exposing a Python API through PyO3 bindings.

## Rationale

- **Graph as bottleneck.** The Causal Graph Manager is queried on every cycle by four downstream components. Rust's ownership model provides safe concurrent reads without GC pauses, and its graph libraries (petgraph) offer cache-friendly adjacency representations. Python graph libraries (NetworkX) are adequate for small graphs but degrade at the scale where versioned snapshotting and concurrent querying become relevant.

- **Trace parsing scales with log size.** DSL trace logs from frameworks like OpenMM or GROMACS can produce megabytes of structured output per experiment. Parsing this into a semantic IR is a throughput problem well-suited to Rust's zero-copy parsing patterns.

- **KL divergence is dense numerical work.** Forward simulation of candidate experiments against a DAG to estimate predicted information gain involves repeated numerical computation over graph structures. While NumPy/SciPy handle array math well, the graph-structured nature of the computation (traversing DAG paths, marginalizing over edge subsets) benefits from Rust's ability to fuse graph traversal with numerical computation without Python interpreter overhead.

- **Orchestration needs flexibility.** LLM API calls, DSL framework bindings, and experiment orchestration change frequently during research. Python's ecosystem (LangChain/litellm, existing DSL Python bindings, rapid iteration in notebooks) makes it the natural choice for these layers.

- **PyO3/maturin is mature.** The interop boundary is well-established: maturin handles build/packaging, PyO3 provides ergonomic type conversions. The Rust components appear as normal Python imports to the orchestration layer.

## Consequences

- **Build complexity.** The project requires both a Python package manager and a Rust toolchain. Maturin handles most of the friction, but CI/CD must build for multiple platforms.

- **Research-phase flexibility.** During the research phase, prototypes for all components (including the three Rust targets) may be written in Python first. The Rust implementation follows once the component's interface and behavior are validated through research. This ADR does not mandate Rust for prototypes.

- **Scope boundary.** This decision covers the core language split only. It does not determine: web frameworks, deployment targets, database choices, specific ML frameworks, or infrastructure. Those decisions require their own research and ADRs.

## References

- ARCHITECTURE.md Sections 4.2, 4.5, 8.2 (graph as hub/bottleneck)
- ARCHITECTURE.md Section 8.6 (computational overhead concerns)
- ARCHITECTURE.md Appendix (open research dependencies affecting Trace Semantics and Bayesian Surprise)
