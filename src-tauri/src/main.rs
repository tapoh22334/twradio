#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod twitter_client;

use webbrowser;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    tokio::spawn(twitter_client::start_server());
    webbrowser::open(twitter_client::entrypoint_url().as_str()).unwrap();

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
