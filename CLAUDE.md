# ATHENA

Falsification-driven AI co-scientist. Structured failure analysis over generation volume.

## Project Phase: Research (Active Investigation)

Five open research dependencies identified in ARCHITECTURE.md Appendix are under investigation. Prototypes are research artifacts only — no production code exists.

## Core Thesis

In domains with asymmetric verification costs, establishing the causal locus of experimental failure accelerates convergence exponentially faster than stochastic hypothesis generation.

## Three Architectural Constraints (Non-Negotiable)

These survived two adversarial review passes. Do not weaken, remove, or work around them.

1. **DSL-only environments** — ATHENA operates within structured frameworks where theory and implementation are API-separated (e.g., OpenMM, GROMACS, CESM, VASP). It cannot parse arbitrary Python.
2. **Warm-started causal priors** — No zero-knowledge bootstrapping. Initial causal graphs come from LLM-generated structural priors or domain seeds, refined through an Epistemic Exploration Phase.
3. **Bounded adversarial design** — The adversarial agent maximizes epistemic information gain within deterministic, domain-valid subspaces. Unbounded surprise-maximization causes Noisy TV degeneration.

## Key Artifacts

- `VISION.md` — Stress-tested conceptual vision (8 sections). Source of truth for all claims, limitations, and open questions. Read this first for any ATHENA work.
- `ARCHITECTURE.md` — Component architecture, information flow, mode transitions, evaluation strategy, and risk analysis. Source of truth for structural decisions.
- `decisions/` — Architecture Decision Records. `001-python-rust-core.md` records the Python+Rust language split.
- `research/*/FINDINGS.md` — Active research investigations. Each contains a research question, architecture references, investigation log, accumulated findings, and next steps.
- `evaluation/hidden-confounder/README.md` — Specification anchor for the end-to-end litmus test.

## Directory Structure

```
athena/
├── CLAUDE.md                           # Project governance (this file)
├── VISION.md                           # Conceptual vision, source of truth
├── ARCHITECTURE.md                     # Component architecture and risks
├── decisions/
│   └── 001-python-rust-core.md         # ADR: Rust for perf-critical, Python for orchestration
├── research/
│   ├── trace-semantics/                # Priority 1: IR design for DSL traces
│   │   ├── FINDINGS.md
│   │   ├── dsl-evaluation/             # DSL trace format surveys
│   │   └── prototypes/                 # Throwaway IR prototypes
│   ├── adversarial-reward/             # Priority 2: Epistemic info gain formalization
│   │   ├── FINDINGS.md
│   │   └── prototypes/                 # Throwaway reward function prototypes
│   ├── exploration-convergence/        # Priority 3: Exploration-to-falsification criteria
│   │   └── FINDINGS.md
│   ├── structural-priors/              # Priority 4: LLM DAG quality thresholds
│   │   └── FINDINGS.md
│   └── surprise-over-dags/             # Priority 5: KL divergence over graph structures
│       └── FINDINGS.md
└── evaluation/
    └── hidden-confounder/              # End-to-end litmus test specification
        └── README.md
```

## Research Workflow

### Session Scoping

1. **Read first.** Before working on any research question, read its FINDINGS.md. Do not start from scratch.
2. **Pick a next step.** Choose one item from the Next Steps section. Do not work on multiple investigations in a single session unless they are explicitly coordinated.
3. **Write before ending.** Before ending a session, write an investigation log entry in the relevant FINDINGS.md with: Scope, Method, Findings, Implications, Open Threads.

### FINDINGS.md Protocol

- **Append-only log.** The Investigation Log is reverse chronological. New entries go at the top. Do not edit or delete previous entries.
- **Living synthesis.** The Accumulated Findings section (What We Know / What We Suspect / What We Don't Know) is updated as evidence accumulates. Move items between categories as confidence changes.
- **Cite evidence for claims.** Every statement in Accumulated Findings must reference either a log entry, an external source, or a prototype result. No unsupported assertions.

### Prototype Code Rules

- Prototypes live in `prototypes/` subdirectories only. They are throwaway research artifacts.
- Every prototype must be referenced from its investigation's FINDINGS.md (in the Prototype Index table): filename, purpose, status, what it demonstrated.
- Prototype-scoped technology choices are fine (e.g., "use NetworkX for this prototype"). These do not require an ADR.
- Prototypes are not production code. They will be discarded when the research question is resolved.

## Methodology

- **Steel-man then stress-test.** Build the strongest version of an idea before trying to break it.
- **Every claim needs a mechanism and conditions.** No "ATHENA is more efficient" — specify HOW and WHEN.
- **Honest limitations are non-optional.** Section 6 of VISION.md exists for a reason. New artifacts must carry the same adversarial rigor.
- **Distinguish proven from conjectured.** Flag which components require novel research vs. existing techniques.

## Competitive Context

ATHENA is positioned against: Sakana AI Scientist V2 (agentic tree search, retry-prune failure handling), Google AI Co-Scientist (tournament evolution, Elo ranking), AI2 CodeScientist (genetic search, syntactic reflection). Differentiation is STRUCTURAL, not incremental. Compare on architecture, not marketing.

## What Not To Do

- Do not add capabilities not present in VISION.md without flagging them as gaps requiring adversarial review
- Do not use grant-proposal rhetoric ("groundbreaking", "revolutionary")
- Technology choices beyond ADR 001 require research and a new ADR. Prototype-scoped choices are fine.
- Do not soften limitations or downgrade severity ratings from VISION.md
- Do not write production code. Prototypes are research artifacts.
- Do not work on a research question without reading its FINDINGS.md first.
