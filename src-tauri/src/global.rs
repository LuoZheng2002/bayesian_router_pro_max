use std::{sync::{atomic::{AtomicBool, AtomicUsize}, Condvar, Mutex}, thread::JoinHandle};

use lazy_static::lazy_static;

use crate::algorithm_thread::AlgorithmThreadHandle;

pub static PROGRAM_SHOULD_EXIT: AtomicBool = AtomicBool::new(false);
pub static CAN_SUBMIT_RENDER_MODEL: AtomicBool = AtomicBool::new(true);
pub static SUBMIT_RENDER_MODEL_MUTEX: Mutex<()> = Mutex::new(());
pub static SUBMIT_RENDER_MODEL_CV: Condvar = Condvar::new();

pub static USE_BAYESIAN: AtomicBool = AtomicBool::new(false);

pub static SUBMISSION_INTERVAL_MILLIS: AtomicUsize = AtomicUsize::new(20);
pub static SES_STRING: Mutex<Option<String>> = Mutex::new(None);

lazy_static!{
    pub static ref ALGORITHM_THREAD_HANDLE: Mutex<Option<AlgorithmThreadHandle>> = Mutex::new(None);
    pub static ref APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
}

use std::sync::{Condvar, Mutex, atomic::AtomicU8};

use lazy_static::lazy_static;

pub enum CommandFlag {
    AstarFrontierOrUpdatePosterior,
    AstarInOut,
    UpdatePosteriorResult,
    ProbaModelResult,
    Auto,
}

impl CommandFlag {
    pub fn get_level(&self) -> u8 {
        match self {
            CommandFlag::AstarFrontierOrUpdatePosterior => 0,
            CommandFlag::AstarInOut => 1,
            CommandFlag::UpdatePosteriorResult => 2,
            CommandFlag::ProbaModelResult => 3,
            CommandFlag::Auto => 4,
        }
    }
}

pub static COMMAND_MUTEX: Mutex<()> = Mutex::new(());
pub static COMMAND_CV: Condvar = Condvar::new();
pub static TARGET_COMMAND_LEVEL: AtomicU8 = AtomicU8::new(0);

