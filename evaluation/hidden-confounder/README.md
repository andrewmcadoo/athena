# Hidden Confounder Litmus Test

## Purpose

End-to-end evaluation environment for validating ATHENA's core architectural claim: that causal fault isolation is more sample-efficient than stochastic generation in the presence of systematic methodological traps.

This is a specification document, not an implementation. It defines what the evaluation environment must contain and how validation is measured.

## What the Environment Must Contain

### 1. DSL Environment

A structured simulation framework with API-separated theory and implementation layers. The environment must:

- Expose a parameter space large enough for both hypothesis generation and adversarial experiment design
- Produce structured trace logs sufficient for the Trace Semantics Engine to parse
- Enforce deterministic execution within its valid subspace (no irreducible stochasticity in the testable region)
- Provide domain constraint specifications (valid parameter ranges, available observables)

### 2. Synthetic Confounded Dataset

A dataset containing a complex, non-linear physical relationship with a deliberately engineered hidden spurious confounder. The confounder must:

- Enable 98% accuracy on in-distribution validation if exploited
- Fail entirely on a strictly withheld out-of-distribution (OOD) test set
- Be invisible to systems that optimize for in-distribution metrics
- Be discoverable only through interventional experiments that probe confounding structure
- Be sophisticated enough that simple statistical checks (correlation analysis, variance inflation) do not trivially detect it

### 3. Generation-First Baseline

A baseline system configured identically to a generation-first architecture (comparable to Sakana V2 or Google Co-Scientist with tournament selection). The baseline must:

- Operate within the same DSL environment as ATHENA
- Receive identical computational budget and cycle limits
- Use scalar reward optimization (validation accuracy, Elo-style ranking)
- Lack structured fault isolation — failures are registered as low-reward signals

### 4. Evaluation Harness

Infrastructure for running both systems and measuring outcomes. The harness must:

- Enforce the 50-cycle limit strictly
- Enforce identical computational budgets
- Record full execution traces for both systems (for post-hoc analysis)
- Measure: whether each system identifies the confounder, whether it bypasses it, whether the output DAG represents the true causal relationship
- Support blinded evaluation (evaluator does not know which system produced which output)

## Validation Criteria

ATHENA's thesis is validated if and only if, within the 50-cycle limit:

1. ATHENA identifies the spurious confounder
2. ATHENA bypasses it and outputs a causal DAG representing the true relationship
3. The generation-first baseline does not achieve the same

The thesis is falsified if:

- ATHENA exhausts its budget analyzing logs without discovering the true mechanism, OR
- The generation-first system successfully evolves past the confounder through volumetric search without causal analysis

The test is deliberately designed to be passable by either architecture in principle. It does not assume ATHENA wins.

## Hard Dependencies

| Dependency | Investigation | Why Required |
| :--- | :--- | :--- |
| Trace Semantics IR | `research/trace-semantics/` | The DSL environment's trace output must be parseable by the Trace Semantics Engine. The IR design determines what trace format the environment must produce. |
| Adversarial Reward Function | `research/adversarial-reward/` | The adversarial agent's ability to discover the confounder depends on its reward function correctly identifying confounder-probing experiments as high-information-gain. |

## Current Status

**NOT STARTED**

This specification cannot be implemented until the trace-semantics and adversarial-reward research investigations produce actionable findings. The environment design depends on knowing what the IR looks like (to produce compatible trace logs) and how the adversary evaluates experiments (to ensure the confounder is discoverable by design).

## References

- ARCHITECTURE.md Section 7.2 (End-to-End Evaluation)
- VISION.md Section 7 (The Litmus Test)
- Luo et al. SPR task design (VISION.md ref 11) — inspiration for the confounder environment
