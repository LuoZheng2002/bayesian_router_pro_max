use std::sync::{atomic::Ordering, Arc, Mutex};

use shared::{pcb_problem::{ConnectionID, PcbProblem, PcbSolution}, pcb_render_model::PcbRenderModel};

use crate::{bayesian_backtrack_algo::{bayesian_backtrack, TraceCache}, display_injection::{self, DisplayInjection}, naive_backtrack_algo::naive_backtrack};



/// this either calls naive backtrack or bayesian backtrack
pub fn solve_pcb_problem(
    pcb_problem: &PcbProblem,
    bayesian: bool,
    display_injection: &mut DisplayInjection,
) -> Result<PcbSolution, String> {
    let connections: Vec<ConnectionID> = pcb_problem.nets.iter().flat_map(|(_, net_info)| net_info.connections.keys().cloned()).collect::<Vec<_>>();
    let mut trace_cache = TraceCache{
        traces: connections.iter().map(|&connection_id| (connection_id, Vec::new())).collect(),
    };

    let result = if bayesian {
        // Call the Bayesian backtrack function
        bayesian_backtrack(pcb_problem, &mut trace_cache, display_injection)
    } else {
        // Call the naive backtrack function
        naive_backtrack(pcb_problem, &mut trace_cache, None, display_injection)
    };
    match result{
        Ok(solution) => {
            println!("PCB problem solved successfully");
            // println!("Sample Count: {}", SAMPLE_CNT.load(Ordering::SeqCst));
            if solution.determined_traces.len() < connections.len() {
                let err_msg = format!(
                    "Not all connections were solved. Expected: {}, Found: {}",
                    connections.len(),
                    solution.determined_traces.len()
                );
                println!("{}", err_msg);
                return Err(err_msg);
            }
            Ok(solution)
        }
        Err(e) => {
            println!("Failed to solve PCB problem: {}", e);
            // println!("Sample Count: {}", SAMPLE_CNT.load(Ordering::SeqCst));
            Err(e)
        }
    }
}
