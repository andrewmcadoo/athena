use crate::adapter::{AdapterError, DslAdapter};
use crate::common::*;
use crate::convergence;
use crate::event_kinds::EventKind;
use crate::lel::*;

pub struct VaspAdapter;

const INCAR_MARKER: &str = "--- INCAR ---";
const OSZICAR_MARKER: &str = "--- OSZICAR ---";
const OUTCAR_MARKER: &str = "--- OUTCAR ---";

pub fn classify_incar_parameter(
    key: &str,
    _value: &str,
) -> (Layer, BoundaryClassification, Option<&'static str>) {
    let normalized = key.trim().to_ascii_uppercase();

    match normalized.as_str() {
        "GGA" | "METAGGA" | "ISMEAR" => (
            Layer::Theory,
            BoundaryClassification::PrimaryLayer,
            None,
        ),
        "ENCUT" => (
            Layer::Theory,
            BoundaryClassification::DualAnnotated {
                secondary_layer: Layer::Implementation,
                rationale: "cutoff determines both physics accuracy and memory/compute cost"
                    .to_string(),
            },
            Some("eV"),
        ),
        "PREC" => (
            Layer::Theory,
            BoundaryClassification::DualAnnotated {
                secondary_layer: Layer::Implementation,
                rationale: "precision affects both physical accuracy and FFT grid resources"
                    .to_string(),
            },
            None,
        ),
        "SIGMA" => (
            Layer::Theory,
            BoundaryClassification::DualAnnotated {
                secondary_layer: Layer::Methodology,
                rationale:
                    "smearing width affects both electronic structure accuracy and BZ integration convergence"
                        .to_string(),
            },
            Some("eV"),
        ),
        "IBRION" | "NSW" | "ISIF" | "POTIM" => (
            Layer::Methodology,
            BoundaryClassification::PrimaryLayer,
            None,
        ),
        "EDIFF" => (
            Layer::Methodology,
            BoundaryClassification::PrimaryLayer,
            Some("eV"),
        ),
        "EDIFFG" => (
            Layer::Methodology,
            BoundaryClassification::PrimaryLayer,
            Some("eV/Ang"),
        ),
        "NCORE" | "KPAR" | "NPAR" | "NSIM" | "NELM" => (
            Layer::Implementation,
            BoundaryClassification::PrimaryLayer,
            None,
        ),
        "ALGO" => (
            Layer::Implementation,
            BoundaryClassification::DualAnnotated {
                secondary_layer: Layer::Methodology,
                rationale: "algorithm can affect which SCF minimum is found".to_string(),
            },
            None,
        ),
        "LREAL" => (
            Layer::Implementation,
            BoundaryClassification::DualAnnotated {
                secondary_layer: Layer::Theory,
                rationale: "real-space projection trades accuracy for speed".to_string(),
            },
            None,
        ),
        _ => (
            Layer::Implementation,
            BoundaryClassification::ContextDependent {
                default_layer: Layer::Implementation,
                context_note: "VASP parameter not in classification table".to_string(),
            },
            None,
        ),
    }
}

pub fn parse_incar(content: &str) -> Result<Vec<TraceEvent>, AdapterError> {
    let mut events = Vec::new();
    let mut logical_sequence = 1_u64;

    for (idx, raw_line) in content.lines().enumerate() {
        let line_num = (idx + 1) as u32;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('!') || line.starts_with('#') {
            continue;
        }

        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };

        let key = raw_key.trim().to_ascii_uppercase();
        let value = raw_value.split(['!', '#']).next().unwrap_or("").trim();
        let (layer, boundary, unit) = classify_incar_parameter(&key, value);

        let parsed_value = match value.parse::<f64>() {
            Ok(numeric) => Value::Known(numeric, unit.unwrap_or("").to_string()),
            Err(_) => Value::KnownCat(value.to_string()),
        };

        let event = TraceEventBuilder::new()
            .layer(layer)
            .boundary(boundary)
            .kind(EventKind::ParameterRecord {
                name: key.clone(),
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
                source_file: "INCAR".to_string(),
                source_location: SourceLocation::LineRange {
                    start: line_num,
                    end: line_num,
                },
                raw_hash: 0,
            })
            .dag_node_ref(key)
            .build();

        logical_sequence += 1;
        events.push(event);
    }

    Ok(events)
}

fn parse_first_f64_token(input: &str) -> Option<f64> {
    input.split_whitespace().next()?.parse::<f64>().ok()
}

fn parse_value_after_marker(line: &str, marker: &str) -> Option<f64> {
    parse_first_f64_token(line.split(marker).nth(1)?)
}

fn extract_vasp_version(line: &str) -> Option<String> {
    line.split_whitespace()
        .find(|token| token.contains("vasp.") || token.contains("VASP"))
        .map(|token| {
            token
                .trim_matches(|ch: char| matches!(ch, '(' | ')' | ',' | ':'))
                .to_string()
        })
}

pub fn parse_oszicar(content: &str, seq_offset: u64) -> Result<Vec<TraceEvent>, AdapterError> {
    let mut events = Vec::new();
    let mut logical_sequence = seq_offset + 1;
    let mut current_ionic_step = 0_u64;

    for (idx, raw_line) in content.lines().enumerate() {
        let line_num = (idx + 1) as u32;
        let line = raw_line.trim();

        if line.starts_with("DAV:") || line.starts_with("RMM:") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let iteration = tokens.get(1).and_then(|token| token.parse::<u64>().ok());
            let delta_e = tokens.get(3).and_then(|token| token.parse::<f64>().ok());

            if let (Some(iteration), Some(delta_e)) = (iteration, delta_e) {
                let event = TraceEventBuilder::new()
                    .layer(Layer::Methodology)
                    .kind(EventKind::ConvergencePoint {
                        iteration,
                        metric_name: "dE".to_string(),
                        metric_value: Value::Known(delta_e, "eV".to_string()),
                        converged: None,
                    })
                    .temporal(TemporalCoord {
                        simulation_step: current_ionic_step,
                        wall_clock_ns: None,
                        logical_sequence,
                    })
                    .provenance(ProvenanceAnchor {
                        source_file: "OSZICAR".to_string(),
                        source_location: SourceLocation::LineRange {
                            start: line_num,
                            end: line_num,
                        },
                        raw_hash: 0,
                    })
                    .build();

                logical_sequence += 1;
                events.push(event);
            }
            continue;
        }

        if line.contains("F=") {
            if let Some(step) = line
                .split_whitespace()
                .find_map(|token| token.parse::<u64>().ok())
            {
                current_ionic_step = step;
            }

            let total_energy = parse_value_after_marker(line, "F=");
            let e0_energy = parse_value_after_marker(line, "E0=");
            let delta_e = parse_value_after_marker(line, "d E =")
                .or_else(|| parse_value_after_marker(line, "dE ="));

            if let Some(total_energy) = total_energy {
                for prior in events.iter_mut().rev() {
                    if let EventKind::ConvergencePoint { converged, .. } = &mut prior.kind {
                        *converged = Some(true);
                        break;
                    }
                }

                let mut components = Vec::new();
                if let Some(e0_energy) = e0_energy {
                    components.push(("E0".to_string(), Value::Known(e0_energy, "eV".to_string())));
                }
                if let Some(delta_e) = delta_e {
                    components.push(("dE".to_string(), Value::Known(delta_e, "eV".to_string())));
                }

                let event = TraceEventBuilder::new()
                    .layer(Layer::Implementation)
                    .kind(EventKind::EnergyRecord {
                        total: Value::Known(total_energy, "eV".to_string()),
                        components,
                    })
                    .temporal(TemporalCoord {
                        simulation_step: current_ionic_step,
                        wall_clock_ns: None,
                        logical_sequence,
                    })
                    .provenance(ProvenanceAnchor {
                        source_file: "OSZICAR".to_string(),
                        source_location: SourceLocation::LineRange {
                            start: line_num,
                            end: line_num,
                        },
                        raw_hash: 0,
                    })
                    .build();

                logical_sequence += 1;
                events.push(event);
            }
        }
    }

    Ok(events)
}

pub fn parse_outcar(content: &str, seq_offset: u64) -> Result<Vec<TraceEvent>, AdapterError> {
    let mut events: Vec<TraceEvent> = Vec::new();
    let mut logical_sequence = seq_offset + 1;
    let mut resource_event_idx: Option<usize> = None;
    let mut pending_parallelization: Option<String> = None;
    let mut saw_terminal_status = false;

    for (idx, raw_line) in content.lines().enumerate() {
        let line_num = (idx + 1) as u32;
        let line = raw_line.trim();

        if line.contains("running on") && line.contains("total cores") {
            if let Some(core_count) = line
                .split_whitespace()
                .find_map(|token| token.parse::<u64>().ok())
            {
                let parallelization_value = format!("{} cores", core_count);
                if let Some(resource_idx) = resource_event_idx {
                    if let EventKind::ResourceStatus { parallelization, .. } =
                        &mut events[resource_idx].kind
                    {
                        *parallelization = Some(parallelization_value.clone());
                    }
                } else {
                    pending_parallelization = Some(parallelization_value);
                }
            }
            continue;
        }

        if resource_event_idx.is_none() && (line.contains("vasp.") || line.contains("VASP")) {
            let version = extract_vasp_version(line).unwrap_or_else(|| "VASP".to_string());
            let event = TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::ResourceStatus {
                    platform_type: version.clone(),
                    device_ids: vec![version],
                    memory_allocated: None,
                    memory_peak: None,
                    parallelization: pending_parallelization.clone(),
                    warnings: vec![],
                })
                .temporal(TemporalCoord {
                    simulation_step: 0,
                    wall_clock_ns: None,
                    logical_sequence,
                })
                .provenance(ProvenanceAnchor {
                    source_file: "OUTCAR".to_string(),
                    source_location: SourceLocation::LineRange {
                        start: line_num,
                        end: line_num,
                    },
                    raw_hash: 0,
                })
                .build();

            logical_sequence += 1;
            events.push(event);
            resource_event_idx = Some(events.len() - 1);
            continue;
        }

        if line.contains("free  energy   TOTEN") {
            if let Some(total_energy) = parse_value_after_marker(line, "=") {
                let event = TraceEventBuilder::new()
                    .layer(Layer::Implementation)
                    .kind(EventKind::EnergyRecord {
                        total: Value::Known(total_energy, "eV".to_string()),
                        components: Vec::new(),
                    })
                    .temporal(TemporalCoord {
                        simulation_step: 0,
                        wall_clock_ns: None,
                        logical_sequence,
                    })
                    .provenance(ProvenanceAnchor {
                        source_file: "OUTCAR".to_string(),
                        source_location: SourceLocation::LineRange {
                            start: line_num,
                            end: line_num,
                        },
                        raw_hash: 0,
                    })
                    .build();

                logical_sequence += 1;
                events.push(event);
            }
            continue;
        }

        if line.contains("POSITION") && line.contains("TOTAL-FORCE") {
            let event = TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::StateSnapshot {
                    snapshot_type: SnapshotType::Forces,
                    data_ref: "OUTCAR:forces".to_string(),
                })
                .temporal(TemporalCoord {
                    simulation_step: 0,
                    wall_clock_ns: None,
                    logical_sequence,
                })
                .provenance(ProvenanceAnchor {
                    source_file: "OUTCAR".to_string(),
                    source_location: SourceLocation::LineRange {
                        start: line_num,
                        end: line_num,
                    },
                    raw_hash: 0,
                })
                .build();

            logical_sequence += 1;
            events.push(event);
            continue;
        }

        if line.contains("General timing and accounting") {
            let event = TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::ExecutionStatus {
                    status: ExecutionOutcome::Success,
                    framework_error_id: None,
                })
                .temporal(TemporalCoord {
                    simulation_step: 0,
                    wall_clock_ns: None,
                    logical_sequence,
                })
                .provenance(ProvenanceAnchor {
                    source_file: "OUTCAR".to_string(),
                    source_location: SourceLocation::LineRange {
                        start: line_num,
                        end: line_num,
                    },
                    raw_hash: 0,
                })
                .build();

            saw_terminal_status = true;
            logical_sequence += 1;
            events.push(event);
            continue;
        }

        if line.contains("EDDDAV") || line.contains("VERY BAD NEWS") {
            let event = TraceEventBuilder::new()
                .layer(Layer::Implementation)
                .kind(EventKind::ExecutionStatus {
                    status: ExecutionOutcome::CrashDivergent,
                    framework_error_id: None,
                })
                .temporal(TemporalCoord {
                    simulation_step: 0,
                    wall_clock_ns: None,
                    logical_sequence,
                })
                .provenance(ProvenanceAnchor {
                    source_file: "OUTCAR".to_string(),
                    source_location: SourceLocation::LineRange {
                        start: line_num,
                        end: line_num,
                    },
                    raw_hash: 0,
                })
                .build();

            saw_terminal_status = true;
            logical_sequence += 1;
            events.push(event);
        }
    }

    if !saw_terminal_status {
        let timeout_line = content.lines().count().max(1) as u32;
        let event = TraceEventBuilder::new()
            .layer(Layer::Implementation)
            .kind(EventKind::ExecutionStatus {
                status: ExecutionOutcome::Timeout,
                framework_error_id: None,
            })
            .temporal(TemporalCoord {
                simulation_step: 0,
                wall_clock_ns: None,
                logical_sequence,
            })
            .provenance(ProvenanceAnchor {
                source_file: "OUTCAR".to_string(),
                source_location: SourceLocation::LineRange {
                    start: timeout_line,
                    end: timeout_line,
                },
                raw_hash: 0,
            })
            .confidence(ConfidenceMeta {
                completeness: Completeness::PartiallyInferred {
                    inference_method: "no completion marker in OUTCAR".to_string(),
                },
                field_coverage: 0.5,
                notes: vec![],
            })
            .build();
        events.push(event);
    }

    Ok(events)
}

impl DslAdapter for VaspAdapter {
    fn parse_trace(&self, raw: &str) -> Result<LayeredEventLog, AdapterError> {
        let mut marker_positions = Vec::new();
        if let Some(position) = raw.find(INCAR_MARKER) {
            marker_positions.push((position, INCAR_MARKER));
        }
        if let Some(position) = raw.find(OSZICAR_MARKER) {
            marker_positions.push((position, OSZICAR_MARKER));
        }
        if let Some(position) = raw.find(OUTCAR_MARKER) {
            marker_positions.push((position, OUTCAR_MARKER));
        }
        marker_positions.sort_by_key(|(position, _)| *position);

        let mut incar_content: Option<&str> = None;
        let mut oszicar_content: Option<&str> = None;
        let mut outcar_content: Option<&str> = None;

        if marker_positions.is_empty() {
            if raw.lines().any(|line| line.contains('=')) {
                incar_content = Some(raw);
            } else {
                outcar_content = Some(raw);
            }
        } else {
            for (idx, (position, marker)) in marker_positions.iter().enumerate() {
                let start = *position + marker.len();
                let end = marker_positions
                    .get(idx + 1)
                    .map(|(next_position, _)| *next_position)
                    .unwrap_or(raw.len());
                let section = raw[start..end].trim();

                match *marker {
                    INCAR_MARKER => incar_content = Some(section),
                    OSZICAR_MARKER => oszicar_content = Some(section),
                    OUTCAR_MARKER => outcar_content = Some(section),
                    _ => {}
                }
            }
        }

        let mut incar_events = if let Some(content) = incar_content {
            parse_incar(content)?
        } else {
            Vec::new()
        };
        let incar_event_ids: Vec<EventId> = incar_events.iter().map(|event| event.id).collect();

        let mut oszicar_events = if let Some(content) = oszicar_content {
            parse_oszicar(content, incar_events.len() as u64)?
        } else {
            Vec::new()
        };

        let mut outcar_events = if let Some(content) = outcar_content {
            parse_outcar(content, (incar_events.len() + oszicar_events.len()) as u64)?
        } else {
            Vec::new()
        };

        let mut last_convergence_id: Option<EventId> = None;
        let mut last_energy_event_id: Option<EventId> = None;

        for event in &mut oszicar_events {
            match &event.kind {
                EventKind::ConvergencePoint { .. } => {
                    event.causal_refs = incar_event_ids.clone();
                    last_convergence_id = Some(event.id);
                }
                EventKind::EnergyRecord { .. } => {
                    if let Some(convergence_id) = last_convergence_id {
                        event.causal_refs = vec![convergence_id];
                    }
                    last_energy_event_id = Some(event.id);
                }
                _ => {}
            }
        }

        for event in &mut outcar_events {
            match &event.kind {
                EventKind::EnergyRecord { .. } => {
                    event.causal_refs = incar_event_ids.clone();
                    last_energy_event_id = Some(event.id);
                }
                EventKind::StateSnapshot { .. } => {
                    event.causal_refs = incar_event_ids.clone();
                }
                EventKind::ExecutionStatus { .. } => {
                    if let Some(energy_event_id) = last_energy_event_id {
                        event.causal_refs = vec![energy_event_id];
                    }
                }
                _ => {}
            }
        }

        let mut derived_events =
            convergence::derive_vasp_scf_convergence_summary(&oszicar_events, "OSZICAR");

        let experiment_ref = ExperimentRef {
            experiment_id: "vasp-trace".to_string(),
            cycle_id: 0,
            hypothesis_id: "H0-vasp-adapter".to_string(),
        };

        let spec = ExperimentSpec {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            predictions: Vec::new(),
            interventions: Vec::new(),
            controlled_variables: Vec::new(),
            dag_refs: Vec::new(),
            provenance: ProvenanceAnchor {
                source_file: "INCAR".to_string(),
                source_location: SourceLocation::ExternalInput,
                raw_hash: 0,
            },
        };

        let mut builder = LayeredEventLogBuilder::new(experiment_ref, spec);
        for event in incar_events.drain(..) {
            builder = builder.add_event(event);
        }
        for event in oszicar_events.drain(..) {
            builder = builder.add_event(event);
        }
        for event in outcar_events.drain(..) {
            builder = builder.add_event(event);
        }
        for event in derived_events.drain(..) {
            builder = builder.add_event(event);
        }

        Ok(builder.build())
    }
}
