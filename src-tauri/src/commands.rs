use shared::my_result::MyResult;

use crate::global::SUBMIT_RENDER_MODEL_CV;






#[tauri::command]
pub fn signal() -> MyResult<(), String> {
    SUBMIT_RENDER_MODEL_CV.notify_all();
    MyResult::Ok(())
}
