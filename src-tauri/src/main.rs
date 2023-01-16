#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod audio_player;
mod scheduler;
mod display_bridge;
mod voicegen_agent;
mod voicegen_client;
mod voicegen_filter;
mod twitter_agent;
mod twitter_data;
mod twitter_client;
mod twitter_authorizator;

use tauri::Manager;

#[tauri::command]
fn setup_app(app_handle: tauri::AppHandle) {
    println!("twitter_agent::start");
    let tweet_rx = twitter_agent::start(app_handle.clone());

    println!("scheduler::start");
    let (display_rx, playbook_rx, audioctl_rx, audioctl_rdy_tx, speech_tx)
        = scheduler::start(app_handle.clone(), tweet_rx);

    println!("display_bridge::start");
    display_bridge::start(app_handle.clone(), display_rx);

    println!("voicegen_agent::start");
    voicegen_agent::start(app_handle.clone(), playbook_rx, speech_tx);

    println!("audio_player::start");
    audio_player::start(app_handle.clone(), audioctl_rx, audioctl_rdy_tx);
}


#[tokio::main]
async fn main() -> std::io::Result<()> {

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![setup_app])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
