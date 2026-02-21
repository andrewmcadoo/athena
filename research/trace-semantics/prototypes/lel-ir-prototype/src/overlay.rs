use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::lel::LayeredEventLog;

/// Index-only causal overlay entity mapped 1:1 with `LayeredEventLog.events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayEntity {
    pub event_idx: usize,
    pub dag_node: Option<String>,
    pub causal_parents: Vec<usize>,
}

/// Graph traversal overlay for hybrid LEL+DGR queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalOverlay {
    pub entities: Vec<OverlayEntity>,
    pub entity_by_dag_node: HashMap<String, Vec<usize>>,
}

/// Candidate confounder detected via common-cause ancestry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfounderCandidate {
    pub dag_node: String,
    pub observable_ancestor_events: Vec<usize>,
    pub intervention_ancestor_events: Vec<usize>,
}

impl CausalOverlay {
    /// Construct the overlay in a single O(n) pass over the log events.
    pub fn from_log(log: &LayeredEventLog) -> Self {
        let n = log.events.len();
        let mut entities = Vec::with_capacity(n);
        let mut entity_by_dag_node: HashMap<String, Vec<usize>> = HashMap::new();

        for (event_idx, event) in log.events.iter().enumerate() {
            let causal_parents = event
                .causal_refs
                .iter()
                .filter_map(|event_id| log.indexes.by_id.get(event_id).copied())
                .collect();

            if let Some(dag_node) = &event.dag_node_ref {
                entity_by_dag_node
                    .entry(dag_node.clone())
                    .or_default()
                    .push(event_idx);
            }

            entities.push(OverlayEntity {
                event_idx,
                dag_node: event.dag_node_ref.clone(),
                causal_parents,
            });
        }

        Self {
            entities,
            entity_by_dag_node,
        }
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn entity(&self, idx: usize) -> Option<&OverlayEntity> {
        self.entities.get(idx)
    }

    /// Return all transitive ancestors reachable via `causal_parents`.
    /// The start node is not included.
    pub fn transitive_ancestors(&self, start_idx: usize) -> Vec<usize> {
        let Some(start_entity) = self.entities.get(start_idx) else {
            return Vec::new();
        };

        let mut visited = HashSet::new();
        let mut queue: VecDeque<usize> = VecDeque::new();
        let mut ancestors = Vec::new();

        for &parent_idx in &start_entity.causal_parents {
            queue.push_back(parent_idx);
        }

        while let Some(current_idx) = queue.pop_front() {
            if !visited.insert(current_idx) {
                continue;
            }

            ancestors.push(current_idx);
            for &parent_idx in &self.entities[current_idx].causal_parents {
                if !visited.contains(&parent_idx) {
                    queue.push_back(parent_idx);
                }
            }
        }

        ancestors
    }

    pub fn detect_confounders(
        &self,
        log: &LayeredEventLog,
        observable_var: &str,
        intervention_var: &str,
    ) -> Vec<ConfounderCandidate> {
        debug_assert_eq!(self.entities.len(), log.events.len());

        if !log.indexes.by_variable.contains_key(observable_var)
            || !log.indexes.by_variable.contains_key(intervention_var)
        {
            return Vec::new();
        }

        let observable_positions: Vec<usize> = log.indexes.by_variable[observable_var]
            .iter()
            .filter_map(|event_id| log.indexes.by_id.get(event_id).copied())
            .collect();
        let intervention_positions: Vec<usize> = log.indexes.by_variable[intervention_var]
            .iter()
            .filter_map(|event_id| log.indexes.by_id.get(event_id).copied())
            .collect();

        let observable_ancestors: HashSet<usize> = observable_positions
            .iter()
            .flat_map(|&idx| self.transitive_ancestors(idx))
            .collect();
        let intervention_ancestors: HashSet<usize> = intervention_positions
            .iter()
            .flat_map(|&idx| self.transitive_ancestors(idx))
            .collect();

        let mut shared_ancestors: Vec<usize> = observable_ancestors
            .intersection(&intervention_ancestors)
            .copied()
            .collect();
        shared_ancestors.sort_unstable();

        let controlled_parameters: HashSet<&str> = log
            .spec
            .controlled_variables
            .iter()
            .map(|controlled| controlled.parameter.as_str())
            .collect();

        let mut grouped: BTreeMap<String, ConfounderCandidate> = BTreeMap::new();

        for ancestor_idx in shared_ancestors {
            let Some(dag_node) = self.entities[ancestor_idx].dag_node.as_deref() else {
                continue;
            };
            if dag_node == intervention_var {
                continue;
            }
            if controlled_parameters.contains(dag_node) {
                continue;
            }

            let entry = grouped
                .entry(dag_node.to_string())
                .or_insert_with(|| ConfounderCandidate {
                    dag_node: dag_node.to_string(),
                    observable_ancestor_events: Vec::new(),
                    intervention_ancestor_events: Vec::new(),
                });
            entry.observable_ancestor_events.push(ancestor_idx);
            entry.intervention_ancestor_events.push(ancestor_idx);
        }

        grouped.into_values().collect()
    }
}
