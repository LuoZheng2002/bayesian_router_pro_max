use std::{
    collections::{BinaryHeap, HashMap},
    rc::Rc,
    sync::{atomic::Ordering, Arc, Mutex},
};

use ordered_float::NotNan;
use shared::{
    binary_heap_item::BinaryHeapItem,
    pcb_problem::{ConnectionID, FixedTrace, PcbProblem},
    pcb_render_model::PcbRenderModel,
};

use crate::{bayesian_backtrack_algo::TraceCache, display_injection::{self, DisplayInjection}, proba_model::{ProbaModel, ProbaTrace, Traces}};

#[derive(Debug, Clone)]
pub struct BacktrackNode {
    pub remaining_trace_candidates: BinaryHeap<BinaryHeapItem<NotNan<f64>, Rc<ProbaTrace>>>, // The remaining trace candidates to be processed, sorted by their scores)>
    pub fixed_traces: HashMap<ConnectionID, FixedTrace>,
    pub fix_sequence: Vec<ConnectionID>, // The sequence of connections that have been fixed in this node
    pub prob_up_to_date: bool, // Whether the probabilistic model is up to date
}

impl BacktrackNode {
    fn from_proba_model(proba_model: &ProbaModel) -> Self {
        let mut fixed_traces: HashMap<ConnectionID, FixedTrace> = HashMap::new();
        let mut remaining_trace_candidates: BinaryHeap<
            BinaryHeapItem<NotNan<f64>, Rc<ProbaTrace>>,
        > = BinaryHeap::new();
        for (connection_id, traces) in proba_model.connection_to_traces.iter() {
            match traces {
                Traces::Fixed(fixed_trace) => {
                    fixed_traces.insert(*connection_id, fixed_trace.clone());
                }
                Traces::Probabilistic(trace_map) => {
                    for (_proba_trace_id, proba_trace) in trace_map.iter() {
                        let posterior = proba_trace.get_posterior_with_fallback();
                        let not_nan_proba =
                            NotNan::new(posterior).expect("Probability must be non-NaN");
                        remaining_trace_candidates.push(BinaryHeapItem {
                            key: not_nan_proba,
                            value: proba_trace.clone(),
                        });
                    }
                }
            }
        }
        BacktrackNode {
            remaining_trace_candidates,
            fixed_traces,
            fix_sequence: proba_model.fix_sequence.clone(), // Use the fixed sequence from the probabilistic model
            prob_up_to_date: true, // Initially, the probabilistic model is up to date
        }
    }
    fn fix_trace(&mut self, connection_id: ConnectionID, fixed_trace: FixedTrace) {
        // Add the fixed trace to the fixed traces
        self.fixed_traces.insert(connection_id, fixed_trace);
        self.fix_sequence.push(connection_id);
        // Remove all trace candidates for this connection from the remaining candidates
        let mut remaining_trace_candidates_copy = self.remaining_trace_candidates.clone();
        let mut new_remaining_trace_candidates: BinaryHeap<
            BinaryHeapItem<NotNan<f64>, Rc<ProbaTrace>>,
        > = BinaryHeap::new();
        for candidate in remaining_trace_candidates_copy.drain() {
            if candidate.value.connection_id != connection_id {
                new_remaining_trace_candidates.push(candidate);
            }
        }
        self.remaining_trace_candidates = new_remaining_trace_candidates;
        // Mark the probabilistic model as no longer up to date
        self.prob_up_to_date = false;
    }
    /// If an attemp fails, return none; it will pop the priority queue in both scenarios
    /// assume there are still candidates in the priority queue
    pub fn try_fix_top_k_ranked_trace(
        &mut self,
        mut display_and_block: impl FnMut(&BacktrackNode),
        k: usize,
    ) -> Option<Self> {
        // for self, peek from the priority queue
        // if succeed, remove all traces from the same connection, and generate a new node with the same priority queue and a fixed trace
        // if fail, return error
        let mut result_candidate: Option<BinaryHeapItem<NotNan<f64>, Rc<ProbaTrace>>> = None;
        for i in 0..k {
            let top_ranked_candidate = self.remaining_trace_candidates.pop();
            let top_ranked_candidate = match top_ranked_candidate {
                Some(candidate) => candidate,
                None => {
                    println!("In try fix top k ranekd trace: No more trace candidates to fix");
                    return None; // No more candidates to fix
                }
            };
            let top_ranked_trace_path = &top_ranked_candidate.value.trace_path;
            let top_ranked_trace_net = &top_ranked_candidate.value.net_name;
            // check if the trace collides with any fixed trace
            let filtered_fixed_traces: Vec<_> = self
                .fixed_traces
                .values()
                .filter(|fixed_trace| fixed_trace.net_name != *top_ranked_trace_net)
                .collect();
            let mut collision_found = false;
            for fixed_trace in filtered_fixed_traces {
                if top_ranked_trace_path.collides_with(&fixed_trace.trace_path) {
                    // If it collides, we cannot fix this trace
                    println!("In try fix k top ranked trace:");
                    println!(
                        "Trial {}: Top ranked trace {} collides with a fixed trace {}, cannot fix it",
                        i, top_ranked_candidate.value.net_name.0, fixed_trace.net_name.0
                    );
                    let mut display_node = self.clone();
                    let connection_id = top_ranked_candidate.value.connection_id;
                    // Create a new fixed trace
                    let fixed_trace = FixedTrace {
                        net_name: top_ranked_candidate.value.net_name.clone(),
                        connection_id,
                        trace_path: top_ranked_trace_path.clone(),
                    };
                    display_node.fix_trace(connection_id, fixed_trace);
                    display_and_block(&display_node);
                    collision_found = true;
                    break;
                }
            }
            if !collision_found {
                result_candidate = Some(top_ranked_candidate);
                break;
            }
        }
        if let Some(result_candidate) = result_candidate {
            // If it does not collide, we can fix this trace
            let connection_id = result_candidate.value.connection_id;
            // Create a new fixed trace
            let fixed_trace = FixedTrace {
                net_name: result_candidate.value.net_name.clone(),
                connection_id,
                trace_path: result_candidate.value.trace_path.clone(),
            };
            // delete all trace candidates for this connection in the new node
            let mut new_node = self.clone();
            new_node.fix_trace(connection_id, fixed_trace);
            Some(new_node) // Return the new node with the fixed trace
        } else {
            None // No more candidates to fix
        }
    }
    pub fn from_fixed_traces(
        problem: &PcbProblem,
        fixed_traces: &HashMap<ConnectionID, FixedTrace>,
        fix_sequence: Vec<ConnectionID>,
        trace_cache: &mut TraceCache,
        display_injection: &mut DisplayInjection
    ) -> Result<Self, String> {
        if display_injection.stop_requested.load(Ordering::Relaxed) {
            println!("Stop requested, not creating BacktrackNode from fixed traces");
            return Err("Stop requested".to_string());
        }
        let proba_model = ProbaModel::create_and_solve(problem, fixed_traces, fix_sequence, trace_cache, display_injection)?;
        let result = BacktrackNode::from_proba_model(&proba_model);
        Ok(result)
    }
    /// if self is already up to date, return none
    pub fn try_update_proba_model(
        &mut self,
        problem: &PcbProblem,
        trace_cache: &mut TraceCache,
        display_injection: &mut DisplayInjection,
    ) -> Result<(), String> {
        if display_injection.stop_requested.load(Ordering::Relaxed) {
            println!("Stop requested, not updating ProbaModel");
            return Err("Stop requested".to_string());
        }
        if self.prob_up_to_date {
            return Err("Probabilistic model is already up to date".to_string()); // If the probabilistic model is already up to date, do nothing
        }
        let fixed_traces = &self.fixed_traces;
        let fix_sequence = self.fix_sequence.clone();
        let new_node = BacktrackNode::from_fixed_traces(problem, fixed_traces, fix_sequence, trace_cache, display_injection)?;
        *self = new_node; // Update self with the new node
        Ok(())
    }
    pub fn is_solution(&self, problem: &PcbProblem) -> bool {
        // Check if all connections in the problem have fixed traces in this node
        for net_info in problem.nets.values() {
            for connection_id in net_info.connections.keys() {
                if !self.fixed_traces.contains_key(connection_id) {
                    return false; // If any connection does not have a fixed trace, it's not a solution
                }
            }
        }
        true // All connections have fixed traces, so this is a solution
    }
    pub fn try_fix_any_trace(&mut self) -> Option<Self> {
        // Try to fix any trace from the remaining candidates
        while self.remaining_trace_candidates.len() > 0 {
            let top_ranked_candidate = self.remaining_trace_candidates.pop();
            let top_ranked_candidate = match top_ranked_candidate {
                Some(candidate) => candidate,
                None => {
                    println!("In try fix any trace: No more trace candidates to fix");
                    return None; // No more candidates to fix
                }
            };
            let top_ranked_trace_path = &top_ranked_candidate.value.trace_path;
            let top_ranked_trace_net = &top_ranked_candidate.value.net_name;

            let filtered_fixed_traces: Vec<_> = self
                .fixed_traces
                .values()
                .filter(|fixed_trace| fixed_trace.net_name != *top_ranked_trace_net)
                .collect();
            // Check if the trace collides with any fixed trace
            let mut collision_found = false;
            for fixed_trace in filtered_fixed_traces {
                if top_ranked_trace_path.collides_with(&fixed_trace.trace_path) {
                    // If it collides, we cannot fix this trace
                    println!("In try fix any trace:");
                    println!(
                        "Top ranked trace {} collides with a fixed trace {}, cannot fix it",
                        top_ranked_candidate.value.net_name.0, fixed_trace.net_name.0
                    );
                    collision_found = true;
                    break; // No need to check further, we found a collision
                }
            }
            if !collision_found {
                // If it does not collide, we can fix this trace
                let connection_id = top_ranked_candidate.value.connection_id;
                // Create a new fixed trace
                let fixed_trace = FixedTrace {
                    net_name: top_ranked_candidate.value.net_name.clone(),
                    connection_id,
                    trace_path: top_ranked_trace_path.clone(),
                };
                let mut new_node = self.clone();
                new_node.fix_trace(connection_id, fixed_trace);
                return Some(new_node); // Return the new node with the fixed trace
            }
        }
        None
    }
}
