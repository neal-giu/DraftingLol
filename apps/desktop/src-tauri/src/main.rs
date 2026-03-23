mod adapters;
mod application;
mod commands;
mod domain;
mod services;
mod storage;
mod tests;

use commands::{
    health::healthcheck,
    recommendations::{recommend_draft_candidates, review_completed_draft},
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            healthcheck,
            recommend_draft_candidates,
            review_completed_draft
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Draft Team App desktop shell");
}
