use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::Ordering, Arc, Mutex},
    thread,
    time::Duration,
};

use shared::{
    color_float3::ColorFloat3,
    hyperparameters::{NUM_TOP_RANKED_TO_TRY, UPDATE_PROBA_SKIP_STRIDE},
    pcb_problem::{ConnectionID, NetName, PcbProblem, PcbSolution},
    pcb_render_model::{PcbRenderModel, RenderableBatch, ShapeRenderable, UpdatePcbRenderModel},
    prim_shape::PrimShape, trace_path::TracePath,
};

use crate::{
    backtrack_node::BacktrackNode, block_or_sleep, command_flags::{CommandFlag, TARGET_COMMAND_LEVEL}, display_injection::{self, DisplayInjection}, naive_backtrack_algo::naive_backtrack
};


pub struct TraceCache{
    pub traces: HashMap<ConnectionID, Vec<TracePath>>,
}


pub fn bayesian_backtrack(
    pcb_problem: &PcbProblem,
    trace_cache: &mut TraceCache,
    display_injection: &mut DisplayInjection,
) -> Result<PcbSolution, String> {
    if display_injection.stop_requested.load(Ordering::Relaxed) {
        println!("Stop requested, not running Bayesian backtrack");
        return Err("Stop requested".to_string());
    }
    let connections = pcb_problem.nets.iter()
        .flat_map(|(_, net_info)| net_info.connections.keys().cloned())
        .collect::<Vec<_>>();
    let mut node_stack: Vec<BacktrackNode> = Vec::new();

    fn last_updated_node_index(node_stack: &Vec<BacktrackNode>) -> usize {
        for (index, node) in node_stack.iter().enumerate().rev() {
            if node.prob_up_to_date {
                return index; // Return the index of the last updated node
            }
        }
        // because the first node is always up to date, it is impossible to reach here
        panic!("No updated node found in the stack");
    }

    // fn print_current_stack(node_stack: &Vec<BacktrackNode>) {
    //     println!("Current stack: num_items: {}", node_stack.len());
    //     for (index, node) in node_stack.iter().enumerate() {
    //         println!(
    //             "\tNode {}: up_to_date: {}, num fixed traces: {}, num remaining trace candidates: {}, ",
    //             index,
    //             node.prob_up_to_date,
    //             node.fixed_traces.len(),
    //             node.remaining_trace_candidates.len()
    //         );
    //         // for fixed_trace in node.fixed_traces.values() {
    //         //     println!(
    //         //         "\t\tFixed trace: net_name: {}, connection_id: {}",
    //         //         fixed_trace.net_name.0, fixed_trace.connection_id.0
    //         //     );
    //         // }
    //     }
    // }

    fn node_to_pcb_render_model(problem: &PcbProblem, node: &BacktrackNode) -> PcbRenderModel {
        let mut trace_shape_renderables: Vec<RenderableBatch> = Vec::new();
        let mut pad_shape_renderables: Vec<ShapeRenderable> = Vec::new();
        let mut other_shape_renderables: Vec<ShapeRenderable> = Vec::new();
        let mut net_name_to_color: HashMap<NetName, ColorFloat3> = HashMap::new();
        for (_, net_info) in problem.nets.iter() {
            net_name_to_color.insert(net_info.net_name.clone(), net_info.color);
            for pad in net_info.pads.values(){
                let pad_renderables = pad.to_renderables(net_info.color.to_float4(1.0));
                let pad_clearance_renderables = pad.to_clearance_renderables(net_info.color.to_float4(0.5));
                pad_shape_renderables.extend(pad_renderables);
                pad_shape_renderables.extend(pad_clearance_renderables);
            }
        }
        for fixed_trace in node.fixed_traces.values() {
            let renderable_batches = fixed_trace
                .trace_path
                .to_renderables(net_name_to_color[&fixed_trace.net_name].to_float4(1.0));
            trace_shape_renderables.extend(renderable_batches);
        }
        for line in &problem.obstacle_border_outlines {
            other_shape_renderables.push(ShapeRenderable {
                shape: PrimShape::Line(line.clone()),
                color: [1.0, 0.0, 1.0, 1.0], // magenta color for borders
            });
        }
        PcbRenderModel {
            width: problem.width,
            height: problem.height,
            center: problem.center,
            trace_shape_renderables,
            pad_shape_renderables,
            other_shape_renderables,
        }
    }

    fn display_when_necessary(
        node: &BacktrackNode,
        pcb_problem: &PcbProblem,
        command_flag: CommandFlag,
        display_injection: &mut DisplayInjection,
        fall_through: bool, 
    ) {
        if display_injection.stop_requested.load(Ordering::Relaxed) {
            println!("Stop requested, not displaying node");
            return;
        }
        // println!("Displaying node in bayesian backtrack");
        let target_command_level = TARGET_COMMAND_LEVEL.load(Ordering::Relaxed);
        let task_command_level = command_flag.get_level();
        if target_command_level <= task_command_level {
            let render_model = node_to_pcb_render_model(pcb_problem, node);
            while !(display_injection.can_submit_render_model)() {
                // wait until we can submit the render model
            }            
            (display_injection.submit_render_model)(render_model);
            if !fall_through{
                (display_injection.block_until_signal)();        
            }
        } else {
            if (display_injection.can_submit_render_model)() {
                // If we can submit the render model, do it
                let render_model = node_to_pcb_render_model(pcb_problem, node);
                (display_injection.submit_render_model)(render_model);
            }
        }
    }

    let first_node =
        BacktrackNode::from_fixed_traces(pcb_problem, &HashMap::new(), Vec::new(), trace_cache, display_injection)?;
    // assume the first node has trace candidates
    node_stack.push(first_node);

    let mut heuristics: Option<Vec<ConnectionID>> = None;

    while node_stack.len() > 0 {
        if display_injection.stop_requested.load(Ordering::Relaxed) {
            println!("Stop requested, exiting Bayesian backtrack");
            return Err("Stop requested".to_string());
        }
        // print_current_stack(&node_stack);
        display_when_necessary(
            node_stack.last().unwrap(),
            pcb_problem,
            CommandFlag::ProbaModelResult,
            display_injection,
            false,
        );
        let top_node = node_stack.last_mut().unwrap();
        if top_node.is_solution(pcb_problem) {
            println!("Found a solution!");
            // println!("Number of samples taken: {}", shared::hyperparameters::SAMPLE_CNT.load(Ordering::SeqCst));
            // If the top node is a solution, we can return it
            let fixed_traces = top_node.fixed_traces.clone();
            let solution = PcbSolution {
                determined_traces: fixed_traces,
                scale_down_factor: pcb_problem.scale_down_factor,
            };
            // println!("Successfully found a solution with sample count {}", shared::hyperparameters::SAMPLE_CNT.load(Ordering::SeqCst));
            

            display_when_necessary(top_node, pcb_problem, CommandFlag::Auto, display_injection, true);
            // heuristics = Some(top_node.fix_sequence.clone());
            // break; // break the loop to return the solution
            return Ok(solution);
        }
        let display_and_block_closure = |node: &BacktrackNode| {
            display_when_necessary(node, pcb_problem, CommandFlag::ProbaModelResult, display_injection, false);
        };
        let new_node =
            top_node.try_fix_top_k_ranked_trace(display_and_block_closure, NUM_TOP_RANKED_TO_TRY.load(Ordering::Relaxed));
        if new_node.is_some(){
            println!(
                "Successfully fixed the top ranked trace, pushing new node onto the stack"
            );
            // assert!(new_node.prob_up_to_date, "New node must be up to date");
            let mut new_node = new_node.unwrap();
            if node_stack.len() % UPDATE_PROBA_SKIP_STRIDE.load(Ordering::Relaxed) == 0 {
                let result = new_node.try_update_proba_model(pcb_problem, trace_cache, display_injection);
                if let Err(err) = result {
                    println!("Failed to update the probabilistic model: {}", err);
                    panic!("Failed to update the probabilistic model");
                }
            }
            node_stack.push(new_node);
            continue; // Continue to the next iteration
        }else{
            let mut connections_set: HashSet<ConnectionID> = connections.iter().cloned().collect();
            let mut temp_heuristics: Vec<ConnectionID> = Vec::new();
            assert!(top_node.fixed_traces.len() == top_node.fix_sequence.len(), "Fixed traces and fix sequence must have the same length");
            for connection_id in top_node.fix_sequence.iter(){
                temp_heuristics.push(*connection_id);
                connections_set.remove(connection_id);
            }
            for connection_id in connections_set.iter() {
                temp_heuristics.push(*connection_id);
            }
            assert!(temp_heuristics.len() == connections.len(), "Heuristics must contain all connections");
            println!("Failed to find a solution in Bayesian backtrack, bringing the heuristics to naive backtrack");
            heuristics = Some(temp_heuristics);
            break;
        }       
    }
    // println!("Number of samples taken by Bayesian backtrack: {}", SAMPLE_CNT.load(Ordering::SeqCst));
    // SAMPLE_CNT.store(0, Ordering::SeqCst);
    assert!(heuristics.is_some(), "Heuristics must be set before calling naive backtrack");
    let result = naive_backtrack(pcb_problem, trace_cache, heuristics, display_injection);
    // println!("Number of samples taken by Naive backtrack: {}", SAMPLE_CNT.load(Ordering::SeqCst));
    // SAMPLE_CNT.store(0, Ordering::SeqCst);
    result
}