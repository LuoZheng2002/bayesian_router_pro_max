use std::{path::PathBuf, sync::{Arc, Mutex}, thread::JoinHandle};

use parser::{parse_end_to_end::{parse_start_to_dsn_struct, parse_struct_to_end}, write_ses::write_ses_to_string};
use router::pcb_problem_solve::solve_pcb_problem;
use tauri::AppHandle;

use crate::global::SES_STRING;



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
    let mut pcb_problem = match parse_struct_to_end(&dsn_struct) {
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
    let use_bayesian = {
        let use_bayesian = crate::global::USE_BAYESIAN.lock().unwrap();
        *use_bayesian
    };
    let result = solve_pcb_problem(&pcb_problem, use_bayesian);
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