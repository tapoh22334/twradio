#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod audio_player;
mod display_bridge;
mod scheduler;
mod open_browser_listener;
mod twitter_agent;
mod twitter_authorizator;
mod twitter_client;
mod twitter_data;
mod user_input;
mod voicegen_agent;
mod voicegen_client;
mod voicegen_data;
mod voicegen_filter;
mod voicegen_observer;

use tauri::Manager;

// async command function must return Result to avoid issue
// https://github.com/tauri-apps/tauri/discussions/4317
#[tauri::command]
async fn setup_app(
    state: tauri::State<
        '_,
        tokio::sync::Mutex<tokio::sync::mpsc::Sender<twitter_authorizator::AuthControl>>,
    >,
) -> Result<(), ()> {
    let tx = state.lock().await;
    tx.send(twitter_authorizator::AuthControl::Authorize)
        .await
        .unwrap();
    Ok(())
}


#[tauri::command]
async fn set_timeline(
    timeline: twitter_agent::Timeline,
    state: tauri::State<
        '_,
        tokio::sync::Mutex<tokio::sync::mpsc::Sender<twitter_agent::Timeline>>,
    >,
) -> Result<(), ()> {
    let tx = state.lock().await;

    println!("tauri://backend/timeline {:?}", timeline);
    tx.send(timeline).await.unwrap();

    Ok(())
}

#[tauri::command]
async fn set_timeline_view(
    timeline: twitter_agent::Timeline,
    state: tauri::State<'_, tokio::sync::Mutex<tokio::sync::mpsc::Sender<user_input::UserInput>>>,
) -> Result<(), ()> {
    let tx = state.lock().await;

    println!("tauri://backend/set_timeline_view {:?}", timeline);
    tx.send(user_input::UserInput::TimelineView(timeline))
        .await
        .unwrap();

    Ok(())
}

#[tauri::command]
async fn set_paused(
    paused: bool,
    state: tauri::State<
        '_,
        tokio::sync::Mutex<tokio::sync::mpsc::Sender<audio_player::AudioControl>>,
    >,
    userin: tauri::State<'_, tokio::sync::Mutex<tokio::sync::mpsc::Sender<user_input::UserInput>>>,
) -> Result<(), ()> {
    {
        let tx = state.lock().await;
        let ctl = if paused {
            audio_player::AudioControl::Pause
        } else {
            audio_player::AudioControl::Resume
        };
        tx.send(ctl).await.unwrap();
    }

    {
        let tx = userin.lock().await;
        let ctl = user_input::UserInput::Paused(if paused { true } else { false });
        tx.send(ctl).await.unwrap();
    }

    println!("tauri://backend/paused {:?}", paused);

    Ok(())
}

#[tauri::command]
async fn set_volume(
    volume: u32,
    state: tauri::State<
        '_,
        tokio::sync::Mutex<tokio::sync::mpsc::Sender<audio_player::AudioControl>>,
    >,
) -> Result<(), ()> {
    let tx = state.lock().await;

    tx.send(audio_player::AudioControl::Volume(volume))
        .await
        .unwrap();
    println!("tauri://backend/set_volume {:?}", volume);

    Ok(())
}

#[tauri::command]
async fn set_speaker(
    speaker: voicegen_observer::Speaker,
    state: tauri::State<'_, tokio::sync::Mutex<tokio::sync::mpsc::Sender<user_input::UserInput>>>,
) -> Result<(), ()> {
    let tx = state.lock().await;

    println!("tauri://backend/set_speaker {:?}", speaker);
    tx.send(user_input::UserInput::Speaker(speaker))
        .await
        .unwrap();

    Ok(())
}

#[tauri::command]
async fn set_speech_rate(
    speech_rate: f64,
    state: tauri::State<'_, tokio::sync::Mutex<tokio::sync::mpsc::Sender<user_input::UserInput>>>,
) -> Result<(), ()> {
    let tx = state.lock().await;

    println!("tauri://backend/set_speech_rate {:?}", speech_rate);
    tx.send(user_input::UserInput::SpeechRate(speech_rate))
        .await
        .unwrap();

    Ok(())
}

#[tauri::command]
async fn jump(
    twid: &str,
    state: tauri::State<'_, tokio::sync::Mutex<tokio::sync::mpsc::Sender<user_input::UserInput>>>,
) -> Result<(), ()> {
    let tx = state.lock().await;

    tx.send(user_input::UserInput::Jump(twid.to_string()))
        .await
        .unwrap();
    println!("tauri://backend/jump {:?}", twid);

    Ok(())
}

const QUEUE_LENGTH: usize = 256;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let (authctl_tx, authctl_rx) =
        tokio::sync::mpsc::channel::<twitter_authorizator::AuthControl>(1);

    let (display_tx, display_rx) =
        tokio::sync::mpsc::channel::<display_bridge::DisplayContrl>(QUEUE_LENGTH);

    let (playbook_tx, playbook_rx) = tokio::sync::mpsc::channel::<voicegen_agent::Playbook>(1);

    let (speech_tx, speech_rx) = tokio::sync::mpsc::channel::<Option<voicegen_agent::Speech>>(1);

    let (timeline_tx, timeline_rx) = tokio::sync::mpsc::channel::<twitter_agent::Timeline>(1);

    let (audioctl_tx, audioctl_rx) =
        tokio::sync::mpsc::channel::<audio_player::AudioControl>(QUEUE_LENGTH);

    let (audioctl_rdy_tx, audioctl_rdy_rx) =
        tokio::sync::mpsc::channel::<audio_player::AudioControlRdy>(1);

    let (user_tx, user_rx) = tokio::sync::mpsc::channel::<user_input::UserInput>(QUEUE_LENGTH);

    println!("twitter_authorizator::start");

    let authctl_tx_c = authctl_tx.clone();
    let audioctl_tx_c = audioctl_tx.clone();

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
            let tweet_rx = twitter_agent::start(app_handle.clone(),
                                                authctl_tx.clone(),
                                                token_rx,
                                                timeline_rx);

            println!("voicegen_observer::start");
            voicegen_observer::start(app_handle.clone());

            println!("scheduler::start");
            scheduler::start(
                app_handle.clone(),
                display_tx.clone(),
                playbook_tx.clone(),
                audioctl_tx.clone(),
                tweet_rx,
                speech_rx,
                audioctl_rdy_rx,
                user_rx,
            );

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

            // open_browser_listener
            open_browser_listener::start(app_handle);

            Ok(())
        })
        .manage(tokio::sync::Mutex::new(authctl_tx_c))
        .manage(tokio::sync::Mutex::new(audioctl_tx_c))
        .manage(tokio::sync::Mutex::new(user_tx))
        .manage(tokio::sync::Mutex::new(timeline_tx))
        .invoke_handler(tauri::generate_handler![
            setup_app,
            set_paused,
            set_timeline,
            set_timeline_view,
            set_volume,
            set_speaker,
            set_speech_rate,
            jump
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
