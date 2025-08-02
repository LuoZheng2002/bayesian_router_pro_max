use std::{collections::HashMap, sync::{atomic::{AtomicBool, AtomicUsize}, Condvar, Mutex}, thread::JoinHandle};

use lazy_static::lazy_static;
use tauri::AppHandle;

use crate::algorithm_thread::AlgorithmThreadHandle;

pub static PROGRAM_SHOULD_EXIT: AtomicBool = AtomicBool::new(false);
pub static CAN_SUBMIT_RENDER_MODEL: AtomicBool = AtomicBool::new(true);
pub static SUBMIT_RENDER_MODEL_MUTEX: Mutex<()> = Mutex::new(());
pub static SUBMIT_RENDER_MODEL_CV: Condvar = Condvar::new();

pub static USE_BAYESIAN: AtomicBool = AtomicBool::new(true);

pub static SUBMISSION_INTERVAL_MILLIS: AtomicUsize = AtomicUsize::new(300);
pub static SES_STRING: Mutex<Option<String>> = Mutex::new(None);

pub static TOTAL_LENGTH: Mutex<f64> = Mutex::new(0.0);
pub static NUM_VIAS: Mutex<usize> = Mutex::new(0);
pub static TIME_ELAPSED: Mutex<f64> = Mutex::new(0.0);

pub static CLEANUP_EMIT_CALLS: Mutex<Option<Box<dyn Fn(&AppHandle) + Send>>> = Mutex::new(None);



pub static COMMAND_MUTEX: Mutex<()> = Mutex::new(());
pub static COMMAND_CV: Condvar = Condvar::new();

lazy_static!{
    pub static ref ALGORITHM_THREAD_HANDLE: Mutex<Option<AlgorithmThreadHandle>> = Mutex::new(None);
    pub static ref APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
    pub static ref DEMO_FILE_NAME_TO_CONTENT: Mutex<HashMap<String, String>> = {
        let mut map = HashMap::new();
        map.insert("digistump.dsn".to_string(), include_str!("../../examples/digistump.dsn").to_string());
        map.insert("echo.dsn".to_string(), include_str!("../../examples/echo.dsn").to_string());
        map.insert("music.dsn".to_string(), include_str!("../../examples/music.dsn").to_string());
        map.insert("ping.dsn".to_string(), include_str!("../../examples/ping.dsn").to_string());
        map.insert("differential.dsn".to_string(), include_str!("../../examples/differential.dsn").to_string());
        Mutex::new(map)
    };    
}
