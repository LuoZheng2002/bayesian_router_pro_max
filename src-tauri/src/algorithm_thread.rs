use std::{path::PathBuf, sync::{atomic::Ordering, Arc, Mutex}, thread::JoinHandle};

use parser::{parse_end_to_end::{parse_start_to_dsn_struct, parse_struct_to_end}, write_ses::write_ses_to_string};
use router::{display_injection::DisplayInjection, pcb_problem_solve::solve_pcb_problem};
use shared::pcb_render_model::PcbRenderModel;
use tauri::AppHandle;

use crate::{global::{SES_STRING, SUBMIT_RENDER_MODEL_CV, SUBMIT_RENDER_MODEL_MUTEX}, submit_pcb_render_model::{self, block_until_signal, can_submit_render_model, submit_render_model}};



pub struct AlgorithmThreadHandle{
    pub stop_requested: Arc<Mutex<bool>>,
    pub join_handle: Option<JoinHandle<()>>,
    pub file_path: PathBuf,
}




pub fn algorithm_thread(
    file_path: PathBuf,
    file_content: String, 
    stop_requested: Arc<Mutex<bool>>,
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
    };
    println!("Ready to solve PCB problem");
    block_until_signal();
    // std::thread::sleep(std::time::Duration::from_millis(100)); // Give time for the UI to update

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
            return;
        }
    };
    let ses_string = match write_ses_to_string(&dsn_struct, &result){
        Ok(ses) => {
            println!("SES file written successfully");
            ses
        },
        Err(e) => {
            println!("Failed to write SES file: {}", e);
            println!("Exiting algorithm thread due to write error");
            return;
        }
    };
    {
        let mut ses_string_lock = SES_STRING.lock().unwrap();
        *ses_string_lock = Some(ses_string);
    }
    println!("Auto routing work completed, exiting");
}