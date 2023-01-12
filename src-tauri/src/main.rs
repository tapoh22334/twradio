#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod scheduler;
mod display_bridge;
mod twitter_data;
mod twitter_client;
mod twitter_authorizator;

use tauri::Manager;

use webbrowser;
use twitter_v2::authorization::Oauth2Token;

async fn perform_oauth2_flow() -> Oauth2Token {
    let (shutdown_tx, token_rx) = twitter_authorizator::start_server();
    webbrowser::open(twitter_authorizator::entrypoint_url().as_str()).unwrap();

    let t = token_rx.await.ok().unwrap();
    shutdown_tx.send(()).ok().unwrap();
    t
}

async fn get_token_from_storage(app_handle: &tauri::AppHandle) -> Option<Oauth2Token> {
    println!("get token from storage");
    app_handle
        .emit_all("tauri://frontend/token-request", ())
        .unwrap();

    let (token_complete_tx, token_complete_rx) = tokio::sync::oneshot::channel::<Option<Oauth2Token>>();
    let token_complete_tx : std::sync::Mutex<Option<tokio::sync::oneshot::Sender<Option<Oauth2Token>>>> = std::sync::Mutex::new(Some(token_complete_tx));

    let id = app_handle.listen_global("tauri://backend/token-response", move |event| {
        let t : Option<Oauth2Token> = {
            if let Some(payload) = event.payload() {
                serde_json::from_str(payload).unwrap()
            } else {
                None
            }
        };

        let mut token_complete_tx = token_complete_tx.lock().unwrap();
        token_complete_tx.take().unwrap().send(t).unwrap();
    });

    let t: Option<Oauth2Token> = token_complete_rx.await.unwrap();
    // unlisten to the event using the `id` returned on the `listen_global` function
    // an `once_global` API is also exposed on the `App` struct
    app_handle.unlisten(id);

    t

}

fn save_token_into_storage(app_handle: &tauri::AppHandle, token: Oauth2Token) {
    app_handle
        .emit_all("tauri://frontend/token-register", token)
        .unwrap();
}

#[tauri::command]
fn setup_app(app_handle: tauri::AppHandle) {
    tokio::spawn(async move {
        let mut token: Oauth2Token = {
            if let Some(t) = get_token_from_storage(&app_handle).await {
                println!("token is already in storage");

                t
            } else {
                println!("Token not found. Request authorization");

                let t = perform_oauth2_flow().await;
                save_token_into_storage(&app_handle, t.clone());
                t
            }
        };

        if twitter_authorizator::refresh_token_if_expired(&mut token)
            .await
            .unwrap()
        {
            println!("Token refreshed");
        }

        let (tweet_tx, tweet_rx) = tokio::sync::mpsc::channel(32);
        let (display_rx, speech_rx) = scheduler::start(tweet_rx);
        display_bridge::start(app_handle.clone(), display_rx);

        let tweets = match twitter_client::request_tweet(&token).await {
            Ok(t) => t,
            Err(e) => {
                // TBD: Recovery if the token is expired
                panic!("TBD: not implemented error handling! {:?}", e);
            },
        };

        for tweet in tweets.data {
            let record: scheduler::Record = scheduler::into(&tweet, &tweets.includes.users);
            tweet_tx.send(record).await.unwrap();
        }

    });
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
