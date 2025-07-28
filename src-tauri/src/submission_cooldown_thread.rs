use std::sync::atomic::Ordering;

use crate::global::{CAN_SUBMIT_RENDER_MODEL, PROGRAM_SHOULD_EXIT, SUBMISSION_INTERVAL_MILLIS, SUBMIT_RENDER_MODEL_CV, SUBMIT_RENDER_MODEL_MUTEX};




pub fn submission_cooldown_thread(){
    while !PROGRAM_SHOULD_EXIT.load(Ordering::Relaxed){
        {
            let guard = SUBMIT_RENDER_MODEL_MUTEX.lock().unwrap();
            SUBMIT_RENDER_MODEL_CV.wait(guard).unwrap();
        }
        let interval = SUBMISSION_INTERVAL_MILLIS.load(Ordering::Relaxed);
        std::thread::sleep(std::time::Duration::from_millis(interval as u64));
        CAN_SUBMIT_RENDER_MODEL.store(true, Ordering::Relaxed);
    }
}