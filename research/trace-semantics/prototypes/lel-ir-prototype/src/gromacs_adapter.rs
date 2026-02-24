use crate::adapter::{AdapterError, DslAdapter};
use crate::common::*;
use crate::event_kinds::EventKind;
use crate::lel::*;

pub struct GromacsAdapter;

const MDP_MARKER: &str = "--- MDP ---";
const LOG_MARKER: &str = "--- LOG ---";
const GROMACS_MIN_CONVERGENCE_WINDOW: usize = 4;
const GROMACS_REL_DELTA_THRESHOLD: f64 = 1.0e-4;

pub fn classify_mdp_parameter(
    key: &str,
    _value: &str,
) -> (Layer, BoundaryClassification, Option<&'static str>) {
    let normalized = key.trim().to_ascii_lowercase();

    match normalized.as_str() {
        "coulombtype" | "vdwtype" | "fourierspacing" | "pme_order" => (
            Layer::Theory,
            BoundaryClassification::PrimaryLayer,
            None,
        ),
        "rcoulomb" | "rvdw" => (
            Layer::Theory,
            BoundaryClassification::DualAnnotated {
                secondary_layer: Layer::Methodology,
                rationale:
                    "Cutoff radius affects both force field accuracy and computational performance"
                        .to_string(),
            },
            Some("nm"),
        ),
        "integrator" | "nsteps" | "tcoupl" | "pcoupl" | "gen_vel" => (
            Layer::Methodology,
            BoundaryClassification::PrimaryLayer,
            None,
        ),
        "ref_t" | "gen_temp" => (
            Layer::Methodology,
            BoundaryClassification::PrimaryLayer,
            Some("K"),
        ),
        "ref_p" => (
            Layer::Methodology,
            BoundaryClassification::PrimaryLayer,
            Some("bar"),
        ),
        "dt" | "tau_t" | "tau_p" => (
            Layer::Methodology,
            BoundaryClassification::DualAnnotated {
                secondary_layer: Layer::Implementation,
                rationale:
                    "Timestep affects both sampling methodology and numerical stability".to_string(),
            },
            Some("ps"),
        ),
        "constraints" => (
            Layer::Methodology,
            BoundaryClassification::DualAnnotated {
                secondary_layer: Layer::Implementation,
                rationale:
                    "Constraint algorithm choice affects both sampling validity and computational cost"
                        .to_string(),
            },
            None,
        ),
        "nstlog" | "nstxout" | "nstenergy" | "nstlist" => (
            Layer::Implementation,
            BoundaryClassification::PrimaryLayer,
            None,
        ),
        _ => (
            Layer::Implementation,
            BoundaryClassification::ContextDependent {
                default_layer: Layer::Implementation,
                context_note: "GROMACS parameter not in classification table".to_string(),
            },
            None,
        ),
    }
}

pub fn parse_mdp(content: &str) -> Result<Vec<TraceEvent>, AdapterError> {
    let mut events = Vec::new();
    let mut logical_sequence = 1_u64;

    for (idx, raw_line) in content.lines().enumerate() {
        let line_num = (idx + 1) as u32;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };

        let key = raw_key.trim();
        let value = raw_value.split(';').next().unwrap_or("").trim();
        let (layer, boundary, unit) = classify_mdp_parameter(key, value);

        let parsed_value = match value.parse::<f64>() {
            Ok(numeric) => Value::Known(numeric, unit.unwrap_or("").to_string()),
            Err(_) => Value::KnownCat(value.to_string()),
        };

        let event = TraceEventBuilder::new()
            .layer(layer)
            .boundary(boundary)
            .kind(EventKind::ParameterRecord {
                name: key.to_string(),
                specified_value: None,
                actual_value: parsed_value,
                units: unit.map(|u| u.to_string()),
                observation_mode: ObservationMode::Observational,
            })
            .temporal(TemporalCoord {
                simulation_step: 0,
                wall_clock_ns: None,
                logical_sequence,
            })
            .provenance(ProvenanceAnchor {
                source_file: "input.mdp".to_string(),
                source_location: SourceLocation::LineRange {
                    start: line_num,
                    end: line_num,
                },
                raw_hash: 0,
            })
            .dag_node_ref(key.to_string())
            .build();

        logical_sequence += 1;
        events.push(event);
    }

    Ok(events)
}

fn parse_step_from_line(line: &str) -> Option<u64> {
    line.split_whitespace()
        .find_map(|token| token.parse::<u64>().ok())
}

fn tokenize_energy_headers(header_line: &str) -> Vec<String> {
    fn split_columns(line: &str) -> Vec<String> {
        let mut columns = Vec::new();
        let mut current = String::new();
        let mut whitespace_run = 0_usize;

        for ch in line.chars() {
            if ch.is_whitespace() {
                whitespace_run += 1;
                if whitespace_run == 1 {
                    current.push(' ');
                } else if whitespace_run == 2 {
                    if current.ends_with(' ') {
                        current.pop();
                    }
                    if !current.trim().is_empty() {
                        columns.push(current.trim().to_string());
                        current.clear();
                    }
                }
            } else {
                whitespace_run = 0;
                current.push(ch);
            }
        }

        if !current.trim().is_empty() {
            columns.push(current.trim().to_string());
        }

        columns
    }

    let known = [
        ("Kinetic En.", "Kinetic_En."),
        ("Total Energy", "Total_Energy"),
        ("Pressure (bar)", "Pressure_(bar)"),
        ("Coulomb (SR)", "Coulomb_(SR)"),
        ("Coul. recip.", "Coul._recip."),
        ("LJ (SR)", "LJ_(SR)"),
        ("Proper Dih.", "Proper_Dih."),
        ("Improper Dih.", "Improper_Dih."),
        ("LJ (LR)", "LJ_(LR)"),
        ("Coulomb (LR)", "Coulomb_(LR)"),
        ("Disper. corr.", "Disper._corr."),
    ];

    let mut normalized = header_line.to_string();
    for (from, to) in known {
        normalized = normalized.replace(from, to);
    }

    let column_tokens = split_columns(&normalized);
    if !column_tokens.is_empty() {
        return column_tokens;
    }

    normalized.split_whitespace().map(|token| token.to_string()).collect()
}

fn parse_energy_row(header_line: &str, value_line: &str) -> Option<Vec<(String, f64)>> {
    let value_tokens: Vec<&str> = value_line.split_whitespace().collect();
    if value_tokens.is_empty() {
        return None;
    }

    let values: Vec<f64> = value_tokens
        .iter()
        .map(|token| token.parse::<f64>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;

    let mut headers = tokenize_energy_headers(header_line);
    if headers.len() != values.len() {
        let raw_tokens: Vec<&str> = header_line.split_whitespace().collect();
        if raw_tokens.len() < values.len() {
            return None;
        }

        // Greedy right-to-left grouping fallback for multi-word headers.
        let mut grouped_rev = Vec::with_capacity(values.len());
        let mut end = raw_tokens.len();
        for remaining in (1..=values.len()).rev() {
            if remaining == 1 {
                grouped_rev.push(raw_tokens[0..end].join(" "));
                break;
            }

            let min_start = remaining - 1;
            let mut start = end - 1;

            if start > min_start {
                let current = raw_tokens[start];
                if matches!(current, "Energy" | "En." | "(bar)" | "(SR)" | "recip.") {
                    start -= 1;
                }
            }

            grouped_rev.push(raw_tokens[start..end].join(" "));
            end = start;
        }
        grouped_rev.reverse();
        headers = grouped_rev;
    }

    if headers.len() != values.len() {
        return None;
    }

    Some(headers.into_iter().zip(values).collect())
}

pub fn parse_log(content: &str, seq_offset: u64) -> Result<Vec<TraceEvent>, AdapterError> {
    let lines: Vec<&str> = content.lines().collect();
    let mut events = Vec::new();
    let mut logical_sequence = seq_offset + 1;
    let mut current_step = 0_u64;

    let mut version_line: Option<u32> = None;
    let mut hardware_line: Option<u32> = None;
    let mut version_string: Option<String> = None;
    let mut platform_type: Option<String> = None;

    for (idx, line) in lines.iter().enumerate() {
        if version_string.is_none() && line.contains("GROMACS") {
            version_string = Some(line.trim().to_string());
            version_line = Some((idx + 1) as u32);
        }
        if platform_type.is_none() && line.contains("GPU") {
            platform_type = Some("GPU".to_string());
            hardware_line = Some((idx + 1) as u32);
        } else if platform_type.is_none() && line.contains("CPU") {
            platform_type = Some("CPU".to_string());
            hardware_line = Some((idx + 1) as u32);
        }

        if version_string.is_some() && platform_type.is_some() {
            break;
        }
    }

    if version_string.is_some() || platform_type.is_some() {
        let start = version_line
            .into_iter()
            .chain(hardware_line)
            .min()
            .unwrap_or(1);
        let end = version_line
            .into_iter()
            .chain(hardware_line)
            .max()
            .unwrap_or(start);

        let resource = TraceEventBuilder::new()
            .layer(Layer::Implementation)
            .kind(EventKind::ResourceStatus {
                platform_type: platform_type.unwrap_or_else(|| "CPU".to_string()),
                device_ids: vec![version_string.unwrap_or_else(|| "GROMACS".to_string())],
                memory_allocated: None,
                memory_peak: None,
                parallelization: None,
                warnings: vec![],
            })
            .temporal(TemporalCoord {
                simulation_step: 0,
                wall_clock_ns: None,
                logical_sequence,
            })
            .provenance(ProvenanceAnchor {
                source_file: "simulation.log".to_string(),
                source_location: SourceLocation::LineRange { start, end },
                raw_hash: 0,
            })
            .build();
        logical_sequence += 1;
        events.push(resource);
    }

    let mut idx = 0_usize;
    let mut completion_line: Option<u32> = None;
    let mut completion_status: Option<ExecutionOutcome> = None;

    while idx < lines.len() {
        let line = lines[idx];

        if line.trim_start().starts_with("Step") {
            if let Some(step) = parse_step_from_line(line) {
                current_step = step;
            } else if line.contains("Time") && idx + 1 < lines.len() {
                if let Some(step) = parse_step_from_line(lines[idx + 1]) {
                    current_step = step;
                }
            }
        }

        if line.contains("Energies (kJ/mol)") {
            let mut row_idx = idx + 1;
            let mut pairs = Vec::<(String, f64)>::new();
            let mut block_end_line = (idx + 1) as u32;

            while row_idx + 1 < lines.len() {
                let header_line = lines[row_idx];
                let value_line = lines[row_idx + 1];

                if header_line.trim().is_empty() {
                    row_idx += 1;
                    continue;
                }
                if header_line.contains("Energies (kJ/mol)")
                    || header_line.contains("Step")
                    || header_line.contains("Finished mdrun")
                    || header_line.contains("Fatal error")
                {
                    break;
                }

                if let Some(row_pairs) = parse_energy_row(header_line, value_line) {
                    block_end_line = (row_idx + 2) as u32;
                    pairs.extend(row_pairs);
                    row_idx += 2;
                } else {
                    break;
                }
            }

            if !pairs.is_empty() {
                let mut total_energy = None;
                let mut components = Vec::new();
                let mut numerical_findings = Vec::new();

                for (name, value) in &pairs {
                    if name == "Total Energy" || name == "Total_Energy" {
                        total_energy = Some(*value);
                    } else {
                        components.push((name.clone(), Value::Known(*value, "kJ/mol".to_string())));
                    }

                    if value.is_nan() {
                        numerical_findings.push((
                            NumericalEventType::NaNDetected,
                            name.clone(),
                            "NaN detected in energy component".to_string(),
                        ));
                    } else if value.is_infinite() {
                        numerical_findings.push((
                            NumericalEventType::InfDetected,
                            name.clone(),
                            "Inf detected in energy component".to_string(),
                        ));
                    }
                }

                let Some(total) = total_energy else {
                    let warning_event = TraceEventBuilder::new()
                        .layer(Layer::Implementation)
                        .kind(EventKind::NumericalStatus {
                            event_type: NumericalEventType::ConvergenceFailure,
                            affected_quantity: "Total Energy".to_string(),
                            severity: Severity::Warning,
                            detail: Value::KnownCat(
                                "Energy block parsed but Total Energy header not found".to_string(),
                            ),
                        })
                        .temporal(TemporalCoord {
                            simulation_step: current_step,
                            wall_clock_ns: None,
                            logical_sequence,
                        })
                        .provenance(ProvenanceAnchor {
                            source_file: "simulation.log".to_string(),
                            source_location: SourceLocation::LineRange {
                                start: (idx + 1) as u32,
                                end: (idx + 1) as u32,
                            },
                            raw_hash: 0,
                        })
                        .build();
                    logical_sequence += 1;
                    events.push(warning_event);
                    idx = row_idx;
                    continue;
                };

                let energy_event = TraceEventBuilder::new()
                    .layer(Layer::Implementation)
                    .kind(EventKind::EnergyRecord {
                        total: Value::Known(total, "kJ/mol".to_string()),
                        components,
                    })
                    .temporal(TemporalCoord {
                        simulation_step: current_step,
                        wall_clock_ns: None,
                        logical_sequence,
                    })
                    .provenance(ProvenanceAnchor {
                        source_file: "simulation.log".to_string(),
                        source_location: SourceLocation::LineRange {
                            start: (idx + 1) as u32,
                            end: block_end_line,
                        },
                        raw_hash: 0,
                    })
                    .build();
                logical_sequence += 1;

                events.push(energy_event);
                for (event_type, affected_quantity, detail) in numerical_findings {
                    let numerical_event = TraceEventBuilder::new()
                        .layer(Layer::Implementation)
                        .kind(EventKind::NumericalStatus {
                            event_type,
                            affected_quantity,
                            severity: Severity::Warning,
                            detail: Value::KnownCat(detail),
                        })
                        .temporal(TemporalCoord {
                            simulation_step: current_step,
                            wall_clock_ns: None,
                            logical_sequence,
                        })
                        .provenance(ProvenanceAnchor {
                            source_file: "simulation.log".to_string(),
                            source_location: SourceLocation::LineRange {
                                start: (idx + 1) as u32,
                                end: block_end_line,
                            },
                            raw_hash: 0,
                        })
                        .build();
                    logical_sequence += 1;
                    events.push(numerical_event);
                }
            }

            idx = row_idx;
            continue;
        }

        if line.contains("Finished mdrun") {
            completion_line = Some((idx + 1) as u32);
            completion_status = Some(ExecutionOutcome::Success);
        } else if line.contains("Fatal error") {
            completion_line = Some((idx + 1) as u32);
            completion_status = Some(ExecutionOutcome::CrashDivergent);
        }

        idx += 1;
    }

    let completion_line = completion_line.unwrap_or(lines.len().max(1) as u32);
    let completion_kind = EventKind::ExecutionStatus {
        status: completion_status.clone().unwrap_or(ExecutionOutcome::Timeout),
        framework_error_id: None,
    };

    let mut completion_builder = TraceEventBuilder::new()
        .layer(Layer::Implementation)
        .kind(completion_kind)
        .temporal(TemporalCoord {
            simulation_step: current_step,
            wall_clock_ns: None,
            logical_sequence,
        })
        .provenance(ProvenanceAnchor {
            source_file: "simulation.log".to_string(),
            source_location: SourceLocation::LineRange {
                start: completion_line,
                end: completion_line,
            },
            raw_hash: 0,
        });

    if completion_status.is_none() {
        completion_builder = completion_builder.confidence(ConfidenceMeta {
            completeness: Completeness::PartiallyInferred {
                inference_method: "no completion marker in log".to_string(),
            },
            field_coverage: 0.5,
            notes: vec![],
        });
    }

    events.push(completion_builder.build());
    Ok(events)
}

fn gromacs_energy_total(event: &TraceEvent) -> Option<f64> {
    match &event.kind {
        EventKind::EnergyRecord {
            total: Value::Known(total, _),
            ..
        } => Some(*total),
        _ => None,
    }
}

fn derive_gromacs_convergence_summary(events: &[TraceEvent]) -> Option<TraceEvent> {
    let energy_events: Vec<(&TraceEvent, f64)> = events
        .iter()
        .filter_map(|event| gromacs_energy_total(event).map(|total| (event, total)))
        .collect();

    if energy_events.len() < GROMACS_MIN_CONVERGENCE_WINDOW {
        return None;
    }

    let window = &energy_events[energy_events.len() - GROMACS_MIN_CONVERGENCE_WINDOW..];
    let deltas: Vec<f64> = window.windows(2).map(|pair| pair[1].1 - pair[0].1).collect();
    if deltas.is_empty() {
        return None;
    }

    let energy_scale =
        (window.iter().map(|(_, value)| value.abs()).sum::<f64>() / window.len() as f64).max(1.0);
    let rel_abs_deltas: Vec<f64> = deltas
        .iter()
        .map(|delta| delta.abs() / energy_scale)
        .collect();
    let max_rel_delta = rel_abs_deltas
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let mean_rel_delta = rel_abs_deltas.iter().sum::<f64>() / rel_abs_deltas.len() as f64;
    let sign_changes = deltas
        .windows(2)
        .filter(|pair| pair[0] * pair[1] < 0.0)
        .count();

    let (metric_name, metric_value, converged, note) =
        if sign_changes >= 2 && mean_rel_delta > GROMACS_REL_DELTA_THRESHOLD {
            (
                "derived_oscillation_rel_delta_mean",
                mean_rel_delta,
                Some(false),
                "energy deltas alternate sign across the derivation window",
            )
        } else if max_rel_delta <= GROMACS_REL_DELTA_THRESHOLD {
            (
                "derived_convergence_rel_delta_max",
                max_rel_delta,
                Some(true),
                "max relative energy delta is below convergence threshold",
            )
        } else {
            (
                "derived_stall_rel_delta_mean",
                mean_rel_delta,
                Some(false),
                "energy deltas remain above threshold without oscillation",
            )
        };

    let mut causal_refs = window
        .iter()
        .map(|(event, _)| event.id)
        .collect::<Vec<_>>();
    if let Some(exec_event) = events
        .iter()
        .rev()
        .find(|event| matches!(event.kind, EventKind::ExecutionStatus { .. }))
    {
        causal_refs.push(exec_event.id);
    }
    if let Some(numerical_event) = events
        .iter()
        .rev()
        .find(|event| matches!(event.kind, EventKind::NumericalStatus { .. }))
    {
        causal_refs.push(numerical_event.id);
    }

    let from_elements = causal_refs.iter().map(|id| ElementId(id.0)).collect();
    let simulation_step = window.last().map(|(event, _)| event.temporal.simulation_step)?;
    let logical_sequence = events
        .last()
        .map(|event| event.temporal.logical_sequence + 1)
        .unwrap_or(1);

    Some(
        TraceEventBuilder::new()
            .layer(Layer::Methodology)
            .kind(EventKind::ConvergencePoint {
                iteration: simulation_step,
                metric_name: metric_name.to_string(),
                metric_value: Value::Known(metric_value, "relative".to_string()),
                converged,
            })
            .temporal(TemporalCoord {
                simulation_step,
                wall_clock_ns: None,
                logical_sequence,
            })
            .causal_refs(causal_refs)
            .provenance(ProvenanceAnchor {
                source_file: "simulation.log".to_string(),
                source_location: SourceLocation::ExternalInput,
                raw_hash: 0,
            })
            .confidence(ConfidenceMeta {
                completeness: Completeness::Derived { from_elements },
                field_coverage: 1.0,
                notes: vec![note.to_string()],
            })
            .build(),
    )
}

impl DslAdapter for GromacsAdapter {
    fn parse_trace(&self, raw: &str) -> Result<LayeredEventLog, AdapterError> {
        let mdp_marker_pos = raw.find(MDP_MARKER);
        let log_marker_pos = raw.find(LOG_MARKER);

        let mut mdp_content: Option<&str> = None;
        let mut log_content: Option<&str> = None;

        if let Some(mdp_pos) = mdp_marker_pos {
            let mdp_start = mdp_pos + MDP_MARKER.len();
            let mdp_end = log_marker_pos
                .filter(|log_pos| *log_pos > mdp_pos)
                .unwrap_or(raw.len());
            mdp_content = Some(raw[mdp_start..mdp_end].trim());
        }

        if let Some(log_pos) = log_marker_pos {
            let log_start = log_pos + LOG_MARKER.len();
            log_content = Some(raw[log_start..].trim());
        }

        if mdp_marker_pos.is_none() && log_marker_pos.is_none() {
            if raw.lines().any(|line| line.contains('=')) {
                mdp_content = Some(raw);
            } else {
                log_content = Some(raw);
            }
        }

        let mut mdp_events = if let Some(content) = mdp_content {
            parse_mdp(content)?
        } else {
            Vec::new()
        };
        let mdp_event_ids: Vec<EventId> = mdp_events.iter().map(|event| event.id).collect();

        let mut log_events = if let Some(content) = log_content {
            parse_log(content, mdp_events.len() as u64)?
        } else {
            Vec::new()
        };

        let mut last_energy_event_id: Option<EventId> = None;
        for event in &mut log_events {
            match &event.kind {
                EventKind::EnergyRecord { .. } => {
                    event.causal_refs = mdp_event_ids.clone();
                    last_energy_event_id = Some(event.id);
                }
                EventKind::NumericalStatus { .. } => {
                    if let Some(energy_id) = last_energy_event_id {
                        event.causal_refs = vec![energy_id];
                    }
                }
                EventKind::ExecutionStatus { .. } => {
                    if let Some(energy_id) = last_energy_event_id {
                        event.causal_refs = vec![energy_id];
                    }
                }
                _ => {}
            }
        }

        if let Some(summary_event) = derive_gromacs_convergence_summary(&log_events) {
            log_events.push(summary_event);
        }

        let mut ref_t_value: Option<Value> = None;
        let mut ref_p_value: Option<Value> = None;
        for event in &mdp_events {
            if let EventKind::ParameterRecord {
                name, actual_value, ..
            } = &event.kind
            {
                if name == "ref_t" {
                    ref_t_value = Some(actual_value.clone());
                } else if name == "ref_p" {
                    ref_p_value = Some(actual_value.clone());
                }
            }
        }

        let mut controlled_variables = Vec::new();
        if let Some(value) = ref_t_value {
            controlled_variables.push(ControlledVariable {
                id: SpecElementId(1),
                parameter: "temperature".to_string(),
                held_value: value,
            });
        }
        if let Some(value) = ref_p_value {
            controlled_variables.push(ControlledVariable {
                id: SpecElementId(2),
                parameter: "pressure".to_string(),
                held_value: value,
            });
        }

        let experiment_ref = ExperimentRef {
            experiment_id: "gromacs-trace".to_string(),
            cycle_id: 0,
            hypothesis_id: "H0-gromacs-adapter".to_string(),
        };

        let spec = ExperimentSpec {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            predictions: Vec::new(),
            interventions: Vec::new(),
            controlled_variables,
            dag_refs: Vec::new(),
            provenance: ProvenanceAnchor {
                source_file: "input.mdp".to_string(),
                source_location: SourceLocation::ExternalInput,
                raw_hash: 0,
            },
        };

        let mut builder = LayeredEventLogBuilder::new(experiment_ref, spec);
        for event in mdp_events.drain(..) {
            builder = builder.add_event(event);
        }
        for event in log_events {
            builder = builder.add_event(event);
        }

        Ok(builder.build())
    }
}
