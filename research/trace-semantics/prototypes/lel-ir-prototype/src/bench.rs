//! CausalOverlay construction benchmark for Thread #31 resolution.
//!
//! Measures wall-clock time for LEL log construction and simulated
//! overlay construction at 4 scales (10^3 to 10^6 events).
//! Zero external dependencies — uses std::time::Instant only.

use std::collections::HashMap;
use std::time::Instant;

use lel_ir_prototype::common::{
    ExperimentRef, Layer, ObservationMode, ProvenanceAnchor, SourceLocation, TemporalCoord, Value,
};
use lel_ir_prototype::event_kinds::EventKind;
use lel_ir_prototype::lel::{
    reset_event_id_counter, ExperimentSpec, LayeredEventLogBuilder, TraceEventBuilder,
};

fn main() {
    println!("LEL IR Prototype — CausalOverlay Construction Benchmark");
    println!("========================================================");
    println!("Thread #31: Overlay construction cost at Stage 1/2 boundary");
    println!();

    for &scale in &[1_000, 10_000, 100_000, 1_000_000] {
        run_benchmark(scale);
    }

    println!("Conclusion: If all overlay construction times are under ~500ms at 10^6,");
    println!("the O(n) pass is empirically confirmed as tractable for megabyte-scale traces.");
}

fn run_benchmark(n: usize) {
    // Reset global event ID counter for each run
    reset_event_id_counter();

    // Simple deterministic LCG (no extra dependencies)
    let mut rng_state: u64 = 0x5DEE_CE66_D1A4_F681;
    let mut next = || -> u64 {
        rng_state = rng_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        rng_state >> 33
    };

    // ── Phase 1: Log construction ──────────────────────────────
    let t_log_start = Instant::now();

    let exp_ref = ExperimentRef {
        experiment_id: format!("bench-{n}"),
        cycle_id: 1,
        hypothesis_id: "H-bench".to_string(),
    };
    let spec = ExperimentSpec {
        preconditions: vec![],
        postconditions: vec![],
        predictions: vec![],
        interventions: vec![],
        controlled_variables: vec![],
        dag_refs: vec![],
        provenance: ProvenanceAnchor {
            source_file: "bench".to_string(),
            source_location: SourceLocation::ExternalInput,
            raw_hash: 0,
        },
    };

    let mut builder = LayeredEventLogBuilder::new(exp_ref, spec);

    for i in 0..n {
        // Layer distribution: 70% Implementation, 20% Methodology, 10% Theory
        let layer = match next() % 10 {
            0..=6 => Layer::Implementation,
            7..=8 => Layer::Methodology,
            _ => Layer::Theory,
        };

        // ~30% of events carry dag_node_ref (50 unique node names)
        let dag_ref = if next() % 100 < 30 {
            Some(format!("node_{}", next() % 50))
        } else {
            None
        };

        // ~10% of events have 1-3 causal_refs to earlier events
        let causal_refs = if i > 0 && next() % 100 < 10 {
            let count = (next() % 3 + 1) as usize;
            (0..count)
                .map(|_| lel_ir_prototype::common::EventId((next() % i as u64).saturating_add(1)))
                .collect()
        } else {
            vec![]
        };

        let mut eb = TraceEventBuilder::new()
            .layer(layer)
            .kind(EventKind::ParameterRecord {
                name: format!("param_{}", i % 20),
                specified_value: None,
                actual_value: Value::Known(1.0, "nm".to_string()),
                units: Some("nm".to_string()),
                observation_mode: ObservationMode::Observational,
            })
            .temporal(TemporalCoord {
                simulation_step: i as u64,
                wall_clock_ns: Some(i as u64 * 1_000),
                logical_sequence: i as u64,
            })
            .causal_refs(causal_refs);

        if let Some(d) = dag_ref {
            eb = eb.dag_node_ref(d);
        }

        builder = builder.add_event(eb.build());
    }

    let log = builder.build();
    let log_ms = t_log_start.elapsed().as_secs_f64() * 1_000.0;

    // ── Phase 2: Overlay construction (single O(n) pass) ───────
    let t_overlay_start = Instant::now();

    let mut entity_by_dag_node: HashMap<String, Vec<u64>> = HashMap::new();
    let mut causal_ancestors: HashMap<u64, Vec<u64>> = HashMap::new();
    let mut overlay_entity_count: u64 = 0;
    let mut edge_count: u64 = 0;

    for event in &log.events {
        if let Some(dag_ref) = &event.dag_node_ref {
            entity_by_dag_node
                .entry(dag_ref.clone())
                .or_default()
                .push(event.id.0);
            overlay_entity_count += 1;
        }
        if !event.causal_refs.is_empty() {
            let refs: Vec<u64> = event.causal_refs.iter().map(|e| e.0).collect();
            edge_count += refs.len() as u64;
            causal_ancestors.insert(event.id.0, refs);
        }
    }

    let overlay_ms = t_overlay_start.elapsed().as_secs_f64() * 1_000.0;

    // ── Memory estimate ────────────────────────────────────────
    // HashMap overhead: ~64 bytes per bucket + key + value payload
    let entity_mem_bytes: usize = entity_by_dag_node
        .iter()
        .map(|(k, v)| k.len() + v.len() * 8 + 64)
        .sum();
    let ancestor_mem_bytes: usize = causal_ancestors
        .values()
        .map(|v| 8 + v.len() * 8 + 64)
        .sum();
    let total_overlay_kb = (entity_mem_bytes + ancestor_mem_bytes) as f64 / 1_024.0;

    // ── Report ─────────────────────────────────────────────────
    println!("Scale: {:>10} events", n);
    println!(
        "  Log construction:     {:>10.2} ms",
        log_ms
    );
    println!(
        "  Overlay construction: {:>10.2} ms",
        overlay_ms
    );
    println!(
        "  Overlay entities:     {:>10}  (~{:.0}% of events)",
        overlay_entity_count,
        overlay_entity_count as f64 / n as f64 * 100.0
    );
    println!(
        "  Derivation edges:     {:>10}  (~{:.0}% of events)",
        edge_count,
        edge_count as f64 / n as f64 * 100.0
    );
    println!(
        "  DAG node groups:      {:>10}",
        entity_by_dag_node.len()
    );
    println!(
        "  Estimated overlay mem:{:>10.1} KB",
        total_overlay_kb
    );
    println!();
}
