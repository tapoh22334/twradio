#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]


mod twitter_client;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    twitter_client::start_server().await;
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
