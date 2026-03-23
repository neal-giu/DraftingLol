mod adapters;
mod application;
mod commands;
mod domain;
mod services;
mod storage;
mod tests;

use commands::health::healthcheck;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![healthcheck])
        .run(tauri::generate_context!())
        .expect("failed to run Draft Team App desktop shell");
}
