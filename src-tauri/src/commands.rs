use std::sync::atomic::Ordering;

use router::command_flags::TARGET_COMMAND_LEVEL;
use shared::hyperparameters::*;
use shared::stats_enum::StatsEnum;
use shared::{my_result::MyResult, settings_enum::SettingsEnum};
use tauri::Emitter;
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::global::{APP_HANDLE, COMMAND_CV, NUM_VIAS, SES_STRING, TIME_ELAPSED, TOTAL_LENGTH, USE_BAYESIAN};
use crate::handle_file_open;



#[tauri::command]
pub fn step_in()->MyResult<(), String> {
    let old_command_level = TARGET_COMMAND_LEVEL.fetch_sub(1, Ordering::SeqCst);
    if old_command_level == 0{
        TARGET_COMMAND_LEVEL.store(0, Ordering::SeqCst);
        println!("new command level is below 0, resetting to 0");
    }
    COMMAND_CV.notify_all();
    let app_handle = {
        let global_app_handle = APP_HANDLE.lock().unwrap();
        global_app_handle.clone().unwrap()
    };
    
    if old_command_level == 1 || old_command_level == 0{
        
        app_handle.emit("string-event", ("disable".to_string(), "step-in".to_string())).unwrap();
    }
    app_handle.emit("string-event", ("start-pause".to_string(), "pause".to_string())).unwrap();
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
        app_handle.emit("string-event", ("disable".to_string(), "step-out".to_string())).unwrap();
        app_handle.emit("string-event", ("start-pause".to_string(), "start".to_string())).unwrap();
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
    app_handle.emit("string-event", ("disable".to_string(), "step-out".to_string())).unwrap();
    MyResult::Ok(())
}

#[tauri::command]
pub fn save_result()->MyResult<(), String> {
    let app_handle = {
        let global_app_handle = APP_HANDLE.lock().unwrap();
        global_app_handle.clone().unwrap()
    };
    let ses_content = SES_STRING.lock().unwrap().clone();
    let ses_content = match ses_content {
        Some(content) => content,
        None => {
            return MyResult::Err("No SES content to save".to_string());
        }
    };
    let file_path = app_handle
        .dialog()
        .file()
        .add_filter("specctra session file", &["ses"])
        .blocking_save_file();
    // If the user canceled the dialog, just return Ok
    let Some(path) = file_path else {
        return MyResult::Ok(());
    };
    let FilePath::Path(path) = path else {
        return MyResult::Err("Url is not supported".to_string());
    };
    // Attempt to write the result data to the selected file
    match std::fs::write(&path, ses_content){
        Ok(_) => MyResult::Ok(()),
        Err(e) => MyResult::Err(format!("Failed to write file: {}", e)),
    }
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
        app_handle.emit("string-event", ("start-pause".to_string(), "pause".to_string())).unwrap();
        app_handle.emit("string-event", ("enable".to_string(), "step-out".to_string())).unwrap();
        app_handle.emit("string-event", ("disable".to_string(), "step-in".to_string())).unwrap();
        println!("Pausing algorithm");
    }else{
        // current is paused, so we start
        TARGET_COMMAND_LEVEL.store(4, Ordering::Relaxed);
        println!("Starting algorithm");
        app_handle.emit("string-event", ("start-pause".to_string(), "start".to_string())).unwrap();
        app_handle.emit("string-event", ("disable".to_string(), "step-out".to_string())).unwrap();
        app_handle.emit("string-event", ("enable".to_string(), "step-in".to_string())).unwrap();
        COMMAND_CV.notify_all();
    }    
    MyResult::Ok(())
}

#[tauri::command]
pub fn get_settings(setting: &str) -> SettingsEnum{
    println!("get_settings called for setting: {}", setting);
    match setting{
        "use_bayesian_inference" =>{
            let use_bayesian = USE_BAYESIAN.load(Ordering::Relaxed);
            println!("use_bayesian_inference: {}", use_bayesian);
            SettingsEnum::Bool(use_bayesian)
        },
        "astar_max_expansions" => {
            let astar_max_expansions = ASTAR_MAX_EXPANSIONS.load(Ordering::Relaxed);
            println!("astar_max_expansions: {}", astar_max_expansions);
            SettingsEnum::Usize(astar_max_expansions)
        },
        "astar_stride" => {
            let astar_stride = {
                ASTAR_STRIDE.lock().unwrap().clone()
            };
            let astar_stride = astar_stride.to_num::<f64>();
            println!("astar_stride: {}", astar_stride);
            SettingsEnum::Float(astar_stride)
        },
        "trace_score_halved" => {
            let half_probability_raw_score = HALF_PROBABILITY_RAW_SCORE.load(Ordering::Relaxed);
            println!("trace_score_halved: {}", half_probability_raw_score);
            SettingsEnum::Float(half_probability_raw_score)
        },
        "opportunity_cost_halved" => {
            let half_probability_opportunity_cost = HALF_PROBABILITY_OPPORTUNITY_COST.load(Ordering::Relaxed);
            println!("opportunity_cost_halved: {}", half_probability_opportunity_cost);
            SettingsEnum::Float(half_probability_opportunity_cost)
        },
        "max_trace_generation_attempts" => {
            let max_generation_attempts = MAX_GENERATION_ATTEMPTS.load(Ordering::Relaxed);
            println!("max_trace_generation_attempts: {}", max_generation_attempts);
            SettingsEnum::Usize(max_generation_attempts)
        },
        "first_iteration_prior_probability" => {
            let first_iteration_probability = FIRST_ITERATION_PROBABILITY.load(Ordering::Relaxed);
            println!("first_iteration_prior_probability: {}", first_iteration_probability);
            SettingsEnum::Float(first_iteration_probability)
        },
        "second_iteration_prior_probability" => {
            let second_iteration_probability = SECOND_ITERATION_PROBABILITY.load(Ordering::Relaxed);
            println!("second_iteration_prior_probability: {}", second_iteration_probability);
            SettingsEnum::Float(second_iteration_probability)
        },
        "second_iteration_num_traces" => {
            let second_iteration_num_traces = SECOND_ITERATION_NUM_TRACES.load(Ordering::Relaxed);
            println!("second_iteration_num_traces: {}", second_iteration_num_traces);
            SettingsEnum::Usize(second_iteration_num_traces)
        },
        "via_cost" => {
            let via_cost = VIA_COST.load(Ordering::Relaxed);
            println!("via_cost: {}", via_cost);
            SettingsEnum::Float(via_cost)
        },
        "num_top_ranked_to_try" => {
            let num_top_ranked_to_try = NUM_TOP_RANKED_TO_TRY.load(Ordering::Relaxed);
            println!("num_top_ranked_to_try: {}", num_top_ranked_to_try);
            SettingsEnum::Usize(num_top_ranked_to_try)
        },
        "sample_iterations" => {
            let sample_iterations = SAMPLE_ITERATIONS.load(Ordering::Relaxed);
            println!("sample_iterations: {}", sample_iterations);
            SettingsEnum::Usize(sample_iterations)
        },
        "update_probability_skip_stride" => {
            let update_proba_skip_stride = UPDATE_PROBA_SKIP_STRIDE.load(Ordering::Relaxed);
            println!("update_probability_skip_stride: {}", update_proba_skip_stride);
            SettingsEnum::Usize(update_proba_skip_stride)
        },
        _=>{
            panic!("Unknown setting: {}", setting);
        }
    }
}

#[tauri::command]
pub fn set_settings(setting: &str, value: SettingsEnum) -> MyResult<(),String>{
    println!("set_settings called for setting: {}, value: {:?}", setting, value);
    match setting {
        "use_bayesian_inference" => {
            if let SettingsEnum::Bool(val) = value {
                USE_BAYESIAN.store(val, Ordering::SeqCst);
                println!("use_bayesian_inference set to: {}", USE_BAYESIAN.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "astar_max_expansions" => {
            if let SettingsEnum::Usize(val) = value {
                ASTAR_MAX_EXPANSIONS.store(val, Ordering::SeqCst);
                println!("astar_max_expansions set to: {}", ASTAR_MAX_EXPANSIONS.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "astar_stride" => {
            if let SettingsEnum::Float(val) = value {
                let new_astar_stride = astar_stride_from_raw(val);
                let mut astar_stride = ASTAR_STRIDE.lock().unwrap();
                *astar_stride = new_astar_stride;
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "trace_score_halved" => {
            if let SettingsEnum::Float(val) = value {
                HALF_PROBABILITY_RAW_SCORE.store(val, Ordering::SeqCst);
                println!("trace_score_halved set to: {}", HALF_PROBABILITY_RAW_SCORE.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "opportunity_cost_halved" => {
            if let SettingsEnum::Float(val) = value {
                HALF_PROBABILITY_OPPORTUNITY_COST.store(val, Ordering::SeqCst);
                println!("opportunity_cost_halved set to: {}", HALF_PROBABILITY_OPPORTUNITY_COST.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "max_trace_generation_attempts" => {
            if let SettingsEnum::Usize(val) = value {
                MAX_GENERATION_ATTEMPTS.store(val, Ordering::SeqCst);
                println!("max_trace_generation_attempts set to: {}", MAX_GENERATION_ATTEMPTS.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "first_iteration_prior_probability" => {
            if let SettingsEnum::Float(val) = value {
                FIRST_ITERATION_PROBABILITY.store(val, Ordering::SeqCst);
                println!("first_iteration_prior_probability set to: {}", FIRST_ITERATION_PROBABILITY.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "second_iteration_prior_probability" => {
            if let SettingsEnum::Float(val) = value {
                SECOND_ITERATION_PROBABILITY.store(val, Ordering::SeqCst);
                println!("second_iteration_prior_probability set to: {}", SECOND_ITERATION_PROBABILITY.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "second_iteration_num_traces" => {
            if let SettingsEnum::Usize(val) = value {
                SECOND_ITERATION_NUM_TRACES.store(val, Ordering::SeqCst);
                println!("second_iteration_num_traces set to: {}", SECOND_ITERATION_NUM_TRACES.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "via_cost" => {
            if let SettingsEnum::Float(val) = value {
                VIA_COST.store(val, Ordering::SeqCst);
                println!("via_cost set to: {}", VIA_COST.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "num_top_ranked_to_try" => {
            if let SettingsEnum::Usize(val) = value {
                NUM_TOP_RANKED_TO_TRY.store(val, Ordering::SeqCst);
                println!("num_top_ranked_to_try set to: {}", NUM_TOP_RANKED_TO_TRY.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "sample_iterations" => {
            if let SettingsEnum::Usize(val) = value {
                SAMPLE_ITERATIONS.store(val, Ordering::SeqCst);
                println!("sample_iterations set to: {}", SAMPLE_ITERATIONS.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        "update_probability_skip_stride" => {
            if let SettingsEnum::Usize(val) = value {
                UPDATE_PROBA_SKIP_STRIDE.store(val, Ordering::SeqCst);
                println!("update_probability_skip_stride set to: {}", UPDATE_PROBA_SKIP_STRIDE.load(Ordering::SeqCst));
                MyResult::Ok(())
            } else {
                MyResult::Err("Invalid value type".into())
            }
        },
        _=>{
            MyResult::Err(format!("Unknown setting: {}", setting))
        }
    }
}

#[tauri::command]
pub fn get_stats(stat: &str) -> StatsEnum{
    println!("get_stats called for stat: {}", stat);
    match stat{
        "total_length" => {
            let total_length = TOTAL_LENGTH.lock().unwrap().clone();
            StatsEnum::Float(total_length)
        },
        "num_vias" => {
            let num_vias = NUM_VIAS.lock().unwrap().clone();
            StatsEnum::Usize(num_vias)
        },
        "time_elapsed" => {
            let time_elapsed = TIME_ELAPSED.lock().unwrap().clone();
            StatsEnum::Float(time_elapsed)
        },
        "num_bayesian_path_finding_calls" => {
            let num_bayesian_path_finding_calls = NUM_BAYESIAN_PATH_FINDING_CALLS.load(Ordering::Relaxed);
            StatsEnum::Usize(num_bayesian_path_finding_calls)
        },
        "num_naive_path_finding_calls" => {
            let num_naive_path_finding_calls = NUM_NAIVE_PATH_FINDING_CALLS.load(Ordering::Relaxed);
            StatsEnum::Usize(num_naive_path_finding_calls)
        },
        _ => panic!("Unknown stat: {}", stat),
    }
}

#[tauri::command]
pub fn open_file()->MyResult<(), String>{
    handle_file_open::open_file()
}