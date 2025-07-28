use std::sync::atomic::Ordering;

use router::command_flags::TARGET_COMMAND_LEVEL;
use shared::hyperparameters::*;
use shared::{my_result::MyResult, settings_enum::SettingsEnum};
use tauri::Emitter;

use crate::global::{APP_HANDLE, COMMAND_CV, USE_BAYESIAN};



#[tauri::command]
pub fn step_in()->MyResult<(), String> {
    let old_command_level = TARGET_COMMAND_LEVEL.fetch_sub(1, Ordering::SeqCst);
    if old_command_level == 0{
        TARGET_COMMAND_LEVEL.store(0, Ordering::SeqCst);
        println!("new command level is below 0, resetting to 0");
    }
    COMMAND_CV.notify_all();

    
    if old_command_level == 1 || old_command_level == 0{
        let app_handle = {
            let global_app_handle = APP_HANDLE.lock().unwrap();
            global_app_handle.clone().unwrap()
        };
        app_handle.emit("string-event", ("disable", "step-in")).unwrap();
    }    
    MyResult::Ok(())
}

#[tauri::command]
pub fn step_out()->MyResult<(), String> {
    let old_command_level = TARGET_COMMAND_LEVEL.fetch_add(1, Ordering::SeqCst);
    if old_command_level == 4{
        println!("new command level exceeds 4, resetting to 4");
        TARGET_COMMAND_LEVEL.store(4, Ordering::SeqCst);
    }
    COMMAND_CV.notify_all();
    if old_command_level == 3 || old_command_level == 4{
        let app_handle = {
            let global_app_handle = APP_HANDLE.lock().unwrap();
            global_app_handle.clone().unwrap()
        };
        app_handle.emit("string-event", ("disable", "step-out")).unwrap();
    }    
    MyResult::Ok(())
}

#[tauri::command]
pub fn step_over()->MyResult<(), String> {
    COMMAND_CV.notify_all();
    MyResult::Ok(())
}

#[tauri::command]
pub fn view_stats()->MyResult<(), String> {
    let app_handle = {
        let global_app_handle = APP_HANDLE.lock().unwrap();
        global_app_handle.clone().unwrap()
    };
    app_handle.emit("string-event", ("disable", "step-out")).unwrap();
    MyResult::Ok(())
}

#[tauri::command]
pub fn save_result()->MyResult<(), String> {
    MyResult::Ok(())
}

#[tauri::command]
pub fn start_pause() -> MyResult<(), String>{
    let app_handle = {
        let global_app_handle = APP_HANDLE.lock().unwrap();
        global_app_handle.clone().unwrap()
    };
    let current_command_level = TARGET_COMMAND_LEVEL.load(Ordering::Relaxed);
    if current_command_level == 4{
        // current is starting, so we pause
        TARGET_COMMAND_LEVEL.store(0, Ordering::Relaxed);
        app_handle.emit("string-event", ("start-pause", "pause")).unwrap();
        println!("Pausing algorithm");
    }else{
        // current is paused, so we start
        TARGET_COMMAND_LEVEL.store(4, Ordering::Relaxed);
        println!("Starting algorithm");
        app_handle.emit("string-event", ("start-pause", "start")).unwrap();
        COMMAND_CV.notify_all();
    }    
    MyResult::Ok(())
}

#[tauri::command]
pub fn get_settings(setting: &str) -> SettingsEnum{
    println!("get_settings called for setting: {}", setting);
    match setting{
        "use_bayesian_inference" =>{
            SettingsEnum::Bool(USE_BAYESIAN.load(Ordering::Relaxed))
        },
        "astar_max_expansions" => {
            SettingsEnum::Usize(ASTAR_MAX_EXPANSIONS.load(Ordering::Relaxed))
        },
        "astar_stride" => {
            let astar_stride = {
                ASTAR_STRIDE.lock().unwrap().clone()
            };
            SettingsEnum::Float(astar_stride.to_num::<f64>())
        },
        "trace_score_halved" => {
            SettingsEnum::Float(HALF_PROBABILITY_RAW_SCORE.load(Ordering::Relaxed))
        },
        "opportunity_cost_halved" => {
            SettingsEnum::Float(HALF_PROBABILITY_OPPORTUNITY_COST.load(Ordering::Relaxed))
        },
        "max_trace_generation_attempts" => {
            SettingsEnum::Usize(MAX_GENERATION_ATTEMPTS.load(Ordering::Relaxed))
        },
        "first_iteration_prior_probability" => {
            SettingsEnum::Float(FIRST_ITERATION_PROBABILITY.load(Ordering::Relaxed))
        },
        "second_iteration_prior_probability" => {
            SettingsEnum::Float(SECOND_ITERATION_PROBABILITY.load(Ordering::Relaxed))
        },
        "second_iteration_num_traces" => {
            SettingsEnum::Usize(SECOND_ITERATION_NUM_TRACES.load(Ordering::Relaxed))
        },
        "via_cost" => {
            SettingsEnum::Float(VIA_COST.load(Ordering::Relaxed))
        },
        "num_top_ranked_to_try" => {
            SettingsEnum::Usize(NUM_TOP_RANKED_TO_TRY.load(Ordering::Relaxed))
        },
        "sample_iterations" => {
            SettingsEnum::Usize(SAMPLE_ITERATIONS.load(Ordering::Relaxed))
        },
        "update_probability_skip_stride" => {
            SettingsEnum::Usize(UPDATE_PROBA_SKIP_STRIDE.load(Ordering::Relaxed))
        },
        _=>{
            panic!("Unknown setting: {}", setting);
        }
    }
}

#[tauri::command]
pub fn set_settings(setting: &str, value: SettingsEnum) -> Result<(),String>{
    match setting {
        "use_bayesian_inference" => {
            if let SettingsEnum::Bool(val) = value {
                USE_BAYESIAN.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "astar_max_expansions" => {
            if let SettingsEnum::Usize(val) = value {
                ASTAR_MAX_EXPANSIONS.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "astar_stride" => {
            if let SettingsEnum::Float(val) = value {
                let new_astar_stride = astar_stride_from_raw(val);
                let mut astar_stride = ASTAR_STRIDE.lock().unwrap();
                *astar_stride = new_astar_stride;
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "trace_score_halved" => {
            if let SettingsEnum::Float(val) = value {
                HALF_PROBABILITY_RAW_SCORE.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "opportunity_cost_halved" => {
            if let SettingsEnum::Float(val) = value {
                HALF_PROBABILITY_OPPORTUNITY_COST.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "max_trace_generation_attempts" => {
            if let SettingsEnum::Usize(val) = value {
                MAX_GENERATION_ATTEMPTS.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "first_iteration_prior_probability" => {
            if let SettingsEnum::Float(val) = value {
                FIRST_ITERATION_PROBABILITY.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "second_iteration_prior_probability" => {
            if let SettingsEnum::Float(val) = value {
                SECOND_ITERATION_PROBABILITY.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "second_iteration_num_traces" => {
            if let SettingsEnum::Usize(val) = value {
                SECOND_ITERATION_NUM_TRACES.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "via_cost" => {
            if let SettingsEnum::Float(val) = value {
                VIA_COST.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "num_top_ranked_to_try" => {
            if let SettingsEnum::Usize(val) = value {
                NUM_TOP_RANKED_TO_TRY.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "sample_iterations" => {
            if let SettingsEnum::Usize(val) = value {
                SAMPLE_ITERATIONS.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        "update_probability_skip_stride" => {
            if let SettingsEnum::Usize(val) = value {
                UPDATE_PROBA_SKIP_STRIDE.store(val, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Invalid value type".into())
            }
        },
        _=>{
            Err(format!("Unknown setting: {}", setting))
        }
    }
}