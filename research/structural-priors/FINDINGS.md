# Structural Prior Generator: LLM-Generated Initial DAG Quality

## Research Question

What quality thresholds must LLM-generated initial causal DAGs meet for the Epistemic Explorer to correct them into usable graphs, and where is the boundary between correctable imprecision and uncorrectable misspecification? The Structural Prior Generator is the bootstrap entry point for the entire system — its output propagates through every downstream component. Success criteria: a characterized boundary between "imprecise but correctable" and "misspecified beyond recovery" initial DAGs, with empirical data on where current LLM causal graph generation falls relative to that boundary.

**Priority-vs-severity tension.** This component is rated Critical severity in Section 8.1 — its failure mode is catastrophic silent corruption that propagates through every downstream component. It ranks #4 in resolution priority because the warm-start approach provides a usable starting point that defers the failure rather than preventing it. Items 1-2 (trace semantics, adversarial reward) block the system from functioning at all; this item determines whether the functioning system produces correct results. A collaborator should read this as: investigate items 1-2 first, but do not underestimate the severity here.

## Architecture References

| Reference | Section | Relevance |
| :--- | :--- | :--- |
| ARCHITECTURE.md | 4.2 (Structural Prior Generator) | Component definition — zero internal dependencies, highest downstream impact |
| ARCHITECTURE.md | 5.2 (Exploration Phase) | Explorer refines initial DAG; quality determines LFI reliability |
| ARCHITECTURE.md | 8.1 (Per-Component Risks) | Severity: Critical. Widest gap between importance and design maturity |
| ARCHITECTURE.md | 8.3 (Bootstrapping Error Propagation) | Structural errors at initialization that self-correction cannot reach |
| VISION.md | Section 6.1 (Causal Bootstrapping Paradox) | Most severe theoretical threat to the architecture |
| Constraint | Warm-Started Causal Priors | Non-negotiable: no zero-knowledge bootstrapping |

## Status

NOT STARTED

## Key Definitions

- **Structural Hamming distance (SHD)**: The number of edge insertions, deletions, and reversals required to transform one DAG into another. Primary metric for comparing generated vs. ground-truth causal graphs.
- **Correctable imprecision**: Errors in the initial DAG (wrong edge weights, missing weak edges, slightly wrong directions) that the Epistemic Explorer can detect and correct through targeted probes within a reasonable budget.
- **Uncorrectable misspecification**: Fundamental structural errors (missing critical variables entirely, inverted major causal pathways, self-consistent but wrong subgraphs) that the Explorer's probing strategy cannot detect because the errors do not produce observable inconsistencies during exploration.
- **Self-reinforcing loop**: A failure mode where an incorrect graph structure causes the LFI to misclassify failures, producing graph updates that further entrench the incorrect structure.
- **Domain seed**: An alternative to LLM-generated priors: a hand-crafted initial DAG from domain expertise, potentially more accurate but less scalable.

## Investigation Log

*No entries yet.*

## Accumulated Findings

### What We Know

*No findings yet.*

### What We Suspect

*No findings yet.*

### What We Don't Know

*No findings yet.*

## Next Steps

1. **Survey LLM causal graph generation benchmarks** — What is the current state of the art for LLM-generated causal DAGs? What domains have been tested? What SHD values are typical? Are there systematic error patterns (e.g., LLMs consistently miss certain edge types)? Scope: 2-3 sessions.

2. **Characterize the correctable vs. uncorrectable misspecification boundary** — Formally define what makes an initial DAG error correctable by the Epistemic Explorer. What properties of the error (locality, consistency, observability) determine whether targeted probes can detect it? Scope: 2-3 sessions.

3. **Investigate domain-seed alternatives** — When LLM-generated priors are insufficient, what alternative bootstrapping strategies exist? Domain-expert seeds, literature-extracted graphs, hybrid LLM+expert approaches. What is the quality-scalability tradeoff? Scope: 1-2 sessions.

4. **Analyze self-reinforcing loop conditions** — Under what conditions does an initial DAG error become self-reinforcing through the LFI feedback loop? Is there a formal characterization of "self-consistent but wrong" subgraphs that resist correction? Scope: 2-3 sessions.

5. **Design a testable quality threshold experiment** — Propose a synthetic experiment where an LLM generates a DAG for a known causal structure, the Explorer attempts correction, and we measure the relationship between initial SHD and post-exploration accuracy. Scope: 2-3 sessions.
