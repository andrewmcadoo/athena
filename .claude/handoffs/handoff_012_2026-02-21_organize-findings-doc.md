# Session Handoff: Organize Trace-Semantics FINDINGS.md

> Generated: 2026-02-21 | Handoff #12 | Previous: handoff_011_2026-02-21_parallel-open-items-investigation.md

---

## Continuation Directive

Reorganize `research/trace-semantics/FINDINGS.md` for better navigability and structure. **No content should be cut or added** — this is purely an organizational pass. The document has grown to 1879 lines across 14 steps of investigation and needs structural improvements to remain usable.

## Task Definition

**What:** Reorganize FINDINGS.md without changing content. The document has grown organically over 14+ investigation steps and 4 parallel WDK investigations. It's now 1879 lines with some structural inconsistencies.

**Why:** The document is the primary research artifact for trace-semantics. As it approaches 2000 lines, navigability becomes critical for future sessions that need to "read FINDINGS.md first" per CLAUDE.md protocol.

**Success criteria:**
- Same content, better organization
- No lines deleted or added (content-wise — section headers/dividers are fine)
- Consistent formatting across all log entries
- Clear section hierarchy
- Easy to navigate to specific investigations

**Constraints:** Per CLAUDE.md — "Append-only log. The Investigation Log is reverse chronological. New entries go at the top. Do not edit or delete previous entries." Reorganization must respect this constraint for the Investigation Log section. The Accumulated Findings section is a "living synthesis" and can be reorganized freely.

## Current FINDINGS.md Structure (1879 lines)

```
Lines 1-28:    Header (Research Question, Architecture References, Status, Key Definitions)
Lines 29-1509: Investigation Log (22 entries, reverse chronological)
Lines 1510-1841: Accumulated Findings
  1512-1683:   What We Know (75 items across 8 subsections)
  1684-1739:   What We Suspect (23 items across 4 subsections)
  1740-1841:   What We Don't Know (44 items, 5 resolved, 5 narrowed, across 5 subsections)
Lines 1842-1851: Prototype Index (table, 6 entries)
Lines 1853-1879: Next Steps (9 items, most completed/struck through)
```

### Investigation Log Entries (reverse chronological, lines 29-1509)

| Line | Entry | Date | Size |
|------|-------|------|------|
| 31 | WDK#26 — INCAR Classification Table | 2026-02-21 | 121 lines |
| 152 | WDK#25 — VASP Closed-Source Ceiling | 2026-02-21 | 98 lines |
| 250 | WDK#39 — prediction_id Harmonization | 2026-02-21 | 46 lines |
| 296 | WDK#35+#36 — ContractTerm Value Extensions | 2026-02-21 | 78 lines |
| 374 | Step 14 — UncertaintySummary (WDK#40) | 2026-02-22 | 125 lines |
| 499 | Step 13 — Convergence Trajectories (WDK#13) | 2026-02-22 | 106 lines |
| 605 | Step 12 — R17 Comparison Formalization | 2026-02-22 | 108 lines |
| 713 | Step 11 — Hidden Confounder Litmus | 2026-02-22 | 23 lines |
| 736 | Step 10 — VASP Adapter | 2026-02-22 | 26 lines |
| 762 | Step 9 — GROMACS Adapter | 2026-02-21 | 26 lines |
| 788 | Step 7 — R17+R18 Queries | 2026-02-21 | 29 lines |
| 817 | Step 6 — CausalOverlay + R14 | 2026-02-21 | 30 lines |
| 847 | Step 5c — Open Thread Resolution | 2026-02-21 | 32 lines |
| 879 | Step 5a — Candidate IR Schemas | 2026-02-20 | 44 lines |
| 923 | Step 3b — Requirements Coverage Matrix | 2026-02-20 | 38 lines |
| 961 | Step 2c — Comparative IR Synthesis | 2026-02-20 | 29 lines |
| 990 | Step 1d — Cross-Framework Synthesis | 2026-02-20 | 31 lines |
| 1021 | 21% RCA Baseline | 2026-02-20 | 137 lines |
| 1158 | LFI Audit → IR Requirements | 2026-02-20 | 156 lines |
| 1314 | Provenance/Workflow IR Survey | 2026-02-20 | 41 lines |
| 1355 | RCA/FV IR Survey | 2026-02-20 | 43 lines |
| 1398 | VASP Trace Survey | 2026-02-20 | 41 lines |
| 1439 | GROMACS Trace Characterization | 2026-02-20 | 37 lines |
| 1476 | OpenMM Trace Characterization | 2026-02-20 | 34 lines |

### Structural Issues to Consider

1. **Inconsistent entry header formatting**: Early entries use "Entry 001 —" or "Entry 1 —" prefix; later entries use step numbers; WDK entries use "WDK#XX —" prefix. Consider whether to normalize.

2. **Investigation Log dates are not strictly reverse-chronological**: Most Step entries are dated 2026-02-20 through 2026-02-22. The 4 WDK entries from this session are dated 2026-02-21 but are placed above the 2026-02-22 Step entries. This is correct per the append-only rule (newest first), but the dates appear non-monotonic because the WDK entries were written in the same session as the Step 12-14 entries (which used 2026-02-22 dates).

3. **Accumulated Findings section has grown to 75 "What We Know" items** with 8 subsections. The subsections (DSL Trace Architecture, IR Design Patterns, Provenance Models, IR Requirements, Cross-Framework Synthesis, Comparative IR Synthesis, Open Thread Resolution, Requirements Coverage, Baseline Characterization, Candidate IR Schemas) are chronologically organized by when they were added. A thematic reorganization might improve findability.

4. **What We Don't Know has many resolved/narrowed items** (~~struck through~~). 10 of 44 items are resolved or narrowed. These could be grouped separately or moved to a "Resolved" subsection.

5. **Next Steps section is mostly struck through** — 7 of 9 items are complete. Could be reorganized to separate completed from remaining.

6. **The 21% RCA Baseline and LFI Requirements entries are the longest** (137 and 156 lines respectively). They contain detailed tables and taxonomies that might benefit from internal sub-headers.

## Decisions Made This Session

1. **Parallel team approach for WDK investigation**: 4 sub-agents investigated independently, each producing a draft FINDINGS.md log entry. Lead agent reviewed and appended all entries.

2. **WDK#35 resolved**: `value: Option<Value>` is the right design for ContractTerm. Checking logic belongs in adapters/LFI, not the IR schema.

3. **WDK#36 resolved**: Two new Value variants needed — `KnownGrid` (inline spectral) + `DataRef` (volumetric references). Follows existing `StateSnapshot.data_ref` pattern.

4. **WDK#39 resolved**: Change `ComparisonResult.prediction_id` to `SpecElementId`. Zero adapter impact (derived event type).

5. **WDK#25 narrowed**: ~70-80% of VASP failures allow external fault isolation. Asymmetric distribution (bulk metals ~85-90%, strongly correlated ~50-65%).

6. **WDK#26 narrowed**: INCAR table covers ~20% of common params. 6 new ambiguous params identified. Strategy B (context-dependent flags) recommended.

## Files Map

| Path | Role | Status |
|------|------|--------|
| `research/trace-semantics/FINDINGS.md` | Master research log (1879 lines) | TARGET for reorganization |
| `research/trace-semantics/prototypes/lel-ir-prototype/src/*.rs` | Prototype source | Unchanged — reference only |
| `research/trace-semantics/dsl-evaluation/*.md` | DSL analysis docs | Unchanged — reference only |

## Loop State

N/A — single-session work. The parallel WDK investigation is complete. The reorganization is a new, independent task.

## Open Questions

1. **Should the Investigation Log entries be renumbered or given consistent prefixes?** Currently a mix of "Entry 001", "Step X", "WDK#XX" prefixes. The append-only rule says don't edit previous entries — but normalizing headers might be considered "formatting" not "content."

2. **Should the Accumulated Findings be reorganized thematically vs chronologically?** Currently organized by when subsections were added. Thematic (e.g., grouping all VASP-related items) might be more useful but harder to maintain.

3. **Should resolved WDK items be moved to a separate section?** 10 items in What We Don't Know are struck through. Moving them to "Resolved" would clean up the section but changes the numbering that other entries reference.

4. **How strictly to interpret "no content changes"?** Adding a table of contents, internal cross-reference anchors, or divider lines between sections could be considered "adding content" or "reorganization."

## Session Artifacts

- Commit: `89b0d0d` — feat(trace-semantics): resolve/narrow WDK#25,#26,#35,#36,#39
- Beads: athena-0rx, athena-zgz, athena-nyj, athena-hmz (all closed)
- Previous handoff: `.claude/handoffs/handoff_011_2026-02-21_parallel-open-items-investigation.md`

## Documentation Updated

No documentation updates needed — CLAUDE.md and AGENTS.md are current. The FINDINGS.md changes were committed.
