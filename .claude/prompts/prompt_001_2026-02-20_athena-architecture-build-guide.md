TASK TYPE:
Technical documentation generation — concise architecture build guide (GUIDE.md)

INSTRUCTIONS:
1. Read the project artifacts: VISION.md, ARCHITECTURE.md, decisions/001-python-rust-core.md, all research/*/FINDINGS.md files, evaluation/hidden-confounder/README.md, and CLAUDE.md. Ignore AGENTS.md and .beads/ — these are tooling configuration, not project artifacts.
2. Organize the guide into three sections: What's Done, Where We Are, What's Next.
3. "What's Done" — list each completed artifact with a one-line summary of what it established.
4. "Where We Are" — state the current phase, list the five open research investigations with their statuses and priorities in a table, note the evaluation spec status.
5. "What's Next" — extract next steps from each research/*/FINDINGS.md Next Steps section. Since all investigations are NOT STARTED, treat all next steps as remaining work. Order by dependency chain then priority. Distinguish research work from engineering work from integration work.
6. Write to GUIDE.md at the project root.

DO:
- Use terse, scannable formatting (tables, bullet lists, status markers like DONE / NOT STARTED / BLOCKED)
- Derive all content from existing project artifacts — no invention
- Order remaining work by dependency chain, then priority
- Include cross-references to source documents (e.g., "ARCHITECTURE.md §4.5")
- Keep the entire document under 200 lines. If space is tight, collapse individual FINDINGS.md next steps into per-investigation summaries rather than listing each step.
- Use the dependency relationships from ARCHITECTURE.md Appendix:
  - trace-semantics blocks LFI effectiveness
  - adversarial-reward blocks adversarial experiment design
  - both block the litmus test (evaluation/hidden-confounder/README.md Hard Dependencies)
  - exploration-convergence and surprise-over-dags affect performance/calibration but do not block core functionality
  - structural-priors has Critical severity but #4 priority (deferred, not prevented, by warm-start)
- Note which items are research problems vs. engineering problems (per ARCHITECTURE.md §8.1)
- Assume the reader has full context on ATHENA — do not explain concepts, just reference them
- Exclude per-step scope estimates; they live in the source FINDINGS.md files

DON'T:
- Include pseudo-code, code snippets, or implementation examples
- Repeat explanations from VISION.md or ARCHITECTURE.md — summarize in one line and cite the source
- Use verbose prose, filler, or grant-proposal rhetoric ("groundbreaking", "revolutionary")
- Add technology choices beyond what ADR 001 records
- Include content that could be replaced by a pointer to the source document
- Add sections for things that don't exist yet (no "API Design" or "Deployment" sections)
- Include speculative phasing beyond what artifacts explicitly state

EXAMPLES:

Example 1 — Good "What's Next" table entry:
| 1 | Trace semantics IR design | Blocks: LFI, litmus test | NOT STARTED | Research | trace-semantics/FINDINGS.md |

Example 2 — Bad (too verbose):
"The Trace Semantics Engine is a critical component that requires novel research into intermediate representation design for translating raw DSL trace logs from structured simulation frameworks into semantic failure representations suitable for three-way causal fault classification..."

Example 3 — Good "What's Done" entry:
- **ARCHITECTURE.md** — 11-component architecture with dependency graph, evaluation strategy, 6 systemic risks. Source of truth for structural decisions.

Example 4 — Good "Where We Are" investigation table:
| # | Investigation | Status | Priority | Blocks |
|---|---------------|--------|----------|--------|
| 1 | Trace Semantics IR | NOT STARTED | P1 | LFI effectiveness, litmus test |

CONTEXT:
- Project: ATHENA — falsification-driven AI co-scientist targeting domains with asymmetric verification costs
- Current phase: Research (Active Investigation) per CLAUDE.md. No investigation log entries exist yet in any FINDINGS.md.
- Audience: Project owner only (personal reference). No external readers.
- Three non-negotiable constraints: DSL-only environments, warm-started causal priors, bounded adversarial design
- Completed artifacts: VISION.md (8 sections, stress-tested), ARCHITECTURE.md (11 components, 5 functional groups, Appendix with 5 research deps), ADR 001 (Rust for Causal Graph Manager, Trace Semantics Engine, Bayesian Surprise Evaluator; Python for remaining 8 components; PyO3/maturin interop), 5 research FINDINGS.md files (all NOT STARTED), evaluation hidden-confounder spec (NOT STARTED, blocked by trace-semantics + adversarial-reward)
- Priority order of research dependencies: (1) trace semantics, (2) adversarial reward, (3) exploration convergence, (4) structural priors, (5) surprise over DAGs
- Post-research phases are not formally planned. CLAUDE.md mentions prototypes as research artifacts; evaluation spec is blocked by research findings.
