use std::{sync::{atomic::{AtomicBool, AtomicUsize}, Condvar, Mutex}, thread::JoinHandle};

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



pub static COMMAND_MUTEX: Mutex<()> = Mutex::new(());
pub static COMMAND_CV: Condvar = Condvar::new();

lazy_static!{
    pub static ref ALGORITHM_THREAD_HANDLE: Mutex<Option<AlgorithmThreadHandle>> = Mutex::new(None);
    pub static ref APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
}


