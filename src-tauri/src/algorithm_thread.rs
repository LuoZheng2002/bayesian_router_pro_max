use std::{path::PathBuf, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread::JoinHandle, time::Instant};

use parser::{parse_end_to_end::{parse_start_to_dsn_struct, parse_struct_to_end}, write_ses::write_ses_to_string};
use router::{display_injection::DisplayInjection, pcb_problem_solve::solve_pcb_problem};
use shared::{hyperparameters::{NUM_BAYESIAN_PATH_FINDING_CALLS, NUM_NAIVE_PATH_FINDING_CALLS}, pcb_render_model::PcbRenderModel};
use tauri::{AppHandle, Emitter};

use crate::{global::{SES_STRING, SUBMIT_RENDER_MODEL_CV, SUBMIT_RENDER_MODEL_MUTEX, TIME_ELAPSED}, submit_pcb_render_model::{self, block_until_signal, can_submit_render_model, submit_render_model}};



pub struct AlgorithmThreadHandle{
    pub stop_requested: Arc<AtomicBool>,
    pub join_handle: Option<JoinHandle<()>>,
    pub file_path: PathBuf,
}



pub fn cleanup(){
    let app_handle = {
        let app_handle = crate::global::APP_HANDLE.lock().unwrap();
        app_handle.clone().unwrap()
    };
    app_handle.emit("string-event", ("enable".to_string(), "save-result".to_string())).unwrap();
    app_handle.emit("string-event", ("enable".to_string(), "view-stats".to_string())).unwrap();
    app_handle.emit("string-event", ("start-pause".to_string(), "pause".to_string())).unwrap();
    app_handle.emit("string-event", ("disable".to_string(), "step-in".to_string())).unwrap();
    app_handle.emit("string-event", ("disable".to_string(), "step-out".to_string())).unwrap();
    app_handle.emit("string-event", ("disable".to_string(), "step-over".to_string())).unwrap();
}

pub fn algorithm_thread(
    file_path: PathBuf,
    file_content: String, 
    stop_requested: Arc<AtomicBool>,
) {
    println!("Algorithm thread started with file: {}", file_path.to_string_lossy());
    
    let dsn_struct = match parse_start_to_dsn_struct(file_content.clone()) {
        Ok(structure) => structure,
        Err(e) => {
            println!("Failed to parse DSN file: {}", e);
            println!("Exiting algorithm thread due to parse error");
            return;
        }
    };
    let pcb_problem = match parse_struct_to_end(&dsn_struct) {
        Ok(problem) => problem,
        Err(e) => {
            println!("Failed to parse DSN file: {}", e);
            println!("Exiting algorithm thread due to parse error");
            return;
        }
    };
    {
        let mut ses_string = SES_STRING.lock().unwrap();
        *ses_string = None; // Clear previous SES string
    }
    // pcb_problem.num_layers = 1; // Set to 1 for single layer PCB
    
    let can_submit_render_model_closure = ||{
        can_submit_render_model()
    };
    let submit_pcb_render_model_closure = |pcb_render_model: PcbRenderModel| {
        submit_render_model(pcb_render_model);
    };
    let block_until_signal_closure = || {
        block_until_signal();
    };
    let mut display_injection = DisplayInjection{
        can_submit_render_model: Box::new(can_submit_render_model_closure),
        block_until_signal: Box::new(block_until_signal_closure),
        submit_render_model: Box::new(submit_pcb_render_model_closure),
        stop_requested: stop_requested.clone(),
    };
    NUM_BAYESIAN_PATH_FINDING_CALLS.store(0, Ordering::Relaxed);
    NUM_NAIVE_PATH_FINDING_CALLS.store(0, Ordering::Relaxed);

    println!("Ready to solve PCB problem");
    let start = Instant::now();
    let use_bayesian = crate::global::USE_BAYESIAN.load(Ordering::Relaxed); 
    let result = solve_pcb_problem(&pcb_problem, use_bayesian, &mut display_injection);
    let result = match result {
        Ok(result) => {
            println!("PCB problem solved successfully");
            result
        }
        Err(e) => {
            println!("Failed to solve PCB problem: {}", e);
            println!("Exiting algorithm thread due to solve error");
            cleanup();
            return;
        }
    };
    let duration = start.elapsed();
    println!("PCB problem solved in: {:.2?}", duration);
    {
        let mut time_elapsed = TIME_ELAPSED.lock().unwrap();
        *time_elapsed = duration.as_secs_f64();
    }

    let mut total_length: f64 = 0.0;
    for (_, fixed_trace) in &result.determined_traces{
        let trace_path = &fixed_trace.trace_path;
        total_length += trace_path.calculate_total_length();
    }
    {
        let mut total_length_lock = crate::global::TOTAL_LENGTH.lock().unwrap();
        *total_length_lock = total_length;
    }
    let mut num_vias: usize = 0;
    for (_, fixed_trace) in &result.determined_traces{
        let trace_path = &fixed_trace.trace_path;
        num_vias += trace_path.get_num_vias();
    }
    {
        let mut num_vias_lock = crate::global::NUM_VIAS.lock().unwrap();
        *num_vias_lock = num_vias;
    }
   

    let ses_string = match write_ses_to_string(&dsn_struct, &result){
        Ok(ses) => {
            println!("SES file written successfully");
            ses
        },
        Err(e) => {
            println!("Failed to write SES file: {}", e);
            println!("Exiting algorithm thread due to write error");
            cleanup();
            return;
        }
    };
    
    {
        let mut ses_string_lock = SES_STRING.lock().unwrap();
        *ses_string_lock = Some(ses_string);
    }
    println!("Auto routing work completed, exiting");
    cleanup();
}