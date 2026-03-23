use tauri::State;

use crate::services::AppState;

#[tauri::command]
pub fn healthcheck(state: State<'_, AppState>) -> String {
    format!(
        "ok|db={}|catalog={}|engine={}|live={}",
        state.database_path().display(),
        state.catalog().source_database().display(),
        state.recommendation().engine_name(),
        state.live().status_label()
    )
}
