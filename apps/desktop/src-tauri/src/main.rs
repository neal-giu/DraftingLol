mod adapters;
mod application;
mod commands;
mod domain;
mod services;
mod storage;
mod tests;

use commands::{
    config::{load_app_config, save_app_config},
    health::healthcheck,
    recommendations::{recommend_draft_candidates, review_completed_draft},
    workflows::{
        get_draft_diagnostics, get_live_draft_recommendations, load_history_reviews,
        run_sandbox_simulation,
    },
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            healthcheck,
            recommend_draft_candidates,
            review_completed_draft,
            load_app_config,
            save_app_config,
            get_live_draft_recommendations,
            get_draft_diagnostics,
            run_sandbox_simulation,
            load_history_reviews
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Draft Team App desktop shell");
}
