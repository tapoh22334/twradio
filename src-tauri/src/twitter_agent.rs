use serde::{Serialize, Deserialize};

use twitter_v2::authorization::Oauth2Token;

use crate::twitter_data;
use crate::scheduler;
use crate::twitter_client;
use crate::twitter_authorizator;

use tauri::Manager;

const QUEUE_LENGTH : usize = 256;

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

pub fn start(app_handle: tauri::AppHandle) -> tokio::sync::mpsc::Receiver<scheduler::Record>
{
    let (tweet_tx, tweet_rx) = tokio::sync::mpsc::channel(QUEUE_LENGTH);

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

    tweet_rx
}
