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
mod voicegen_data;
mod twitter_agent;
mod twitter_data;
mod twitter_client;
mod twitter_authorizator;

use tauri::Manager;

// async command function must return Result to avoid issue
// https://github.com/tauri-apps/tauri/discussions/4317
#[tauri::command]
async fn setup_app(state: tauri::State<'_, tokio::sync::Mutex<tokio::sync::mpsc::Sender<twitter_authorizator::AuthControl>>>) -> Result<(), ()> {
    let tx = state.lock().await;
    tx.send(twitter_authorizator::AuthControl::Authorize).await.unwrap();
    Ok(())
}

#[tauri::command]
async fn set_paused(paused: bool, state: tauri::State<'_, tokio::sync::Mutex<tokio::sync::mpsc::Sender<audio_player::AudioControl>>>) -> Result<(), ()> {
    let tx = state.lock().await;

    if paused {
        tx.send(audio_player::AudioControl::Pause).await.unwrap();
    } else {
        tx.send(audio_player::AudioControl::Resume).await.unwrap();
    }
    println!("tauri://backend/paused {:?}", paused);

    Ok(())
}

#[tauri::command]
async fn set_volume(volume: u32, state: tauri::State<'_, tokio::sync::Mutex<tokio::sync::mpsc::Sender<audio_player::AudioControl>>>) -> Result<(), ()> {
    let tx = state.lock().await;

    tx.send(audio_player::AudioControl::Volume(volume)).await.unwrap();
    println!("tauri://backend/set_volume {:?}", volume);

    Ok(())
}


const QUEUE_LENGTH : usize = 256;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let (authctl_tx, authctl_rx)
        = tokio::sync::mpsc::channel::<twitter_authorizator::AuthControl>(1);
    let authctl_tx_c = authctl_tx.clone();

    let (display_tx, display_rx)
        = tokio::sync::mpsc::channel::<display_bridge::ViewElements>(QUEUE_LENGTH);

    let (playbook_tx, playbook_rx) 
        = tokio::sync::mpsc::channel::<voicegen_agent::Playbook>(QUEUE_LENGTH);

    let (speech_tx, speech_rx) 
        = tokio::sync::mpsc::channel::<voicegen_agent::Speech>(QUEUE_LENGTH);

    let (audioctl_tx, audioctl_rx) 
        = tokio::sync::mpsc::channel::<audio_player::AudioControl>(QUEUE_LENGTH);
    let audioctl_tx_c = audioctl_tx.clone();

    let (audioctl_rdy_tx, audioctl_rdy_rx) 
        = tokio::sync::mpsc::channel::<audio_player::AudioControlRdy>(1);

    println!("twitter_authorizator::start");

    tauri::Builder::default()
        .setup(move |app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            let app_handle = app.app_handle();

            let token_rx = twitter_authorizator::start(app_handle.clone(), authctl_rx);

            println!("twitter_agent::start");
            let tweet_rx = twitter_agent::start(app_handle.clone(), authctl_tx_c, token_rx);

            println!("scheduler::start");
            scheduler::start( app_handle.clone(),
                tweet_rx,
                display_tx.clone(),
                playbook_tx.clone(),
                speech_rx,
                audioctl_tx.clone(),
                audioctl_rdy_rx);

            println!("display_bridge::start");
            display_bridge::start(app_handle.clone(), display_rx);

            println!("voicegen_agent::start");
            voicegen_agent::start(app_handle.clone(), playbook_rx, speech_tx);

            println!("audio_player::start");
            audio_player::start(app_handle.clone(), audioctl_rx, audioctl_rdy_tx);
            let tick_tx = audioctl_tx.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let _ = tick_tx.send(audio_player::AudioControl::Tick).await;
                }
            });

            Ok(())
        })
        .manage(tokio::sync::Mutex::new(authctl_tx))
        .manage(tokio::sync::Mutex::new(audioctl_tx_c))
        .invoke_handler(tauri::generate_handler![setup_app, set_paused, set_volume])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
