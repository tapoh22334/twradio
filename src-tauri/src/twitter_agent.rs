use twitter_v2::authorization::Oauth2Token;

use crate::scheduler;
use crate::twitter_authorizator;
use crate::twitter_client;

use tauri::Manager;

//const QUEUE_LENGTH : usize = 24;
const QUEUE_LENGTH: usize = 1;
const REQUEST_PERIOD: u64 = 10000; // milliseconds

fn emit_clear_error(
        app_handle: &tauri::AppHandle,
    ) {

    app_handle
        .emit_all("tauri://frontend/authorization-failed", "")
        .unwrap();

    app_handle
        .emit_all("tauri://frontend/other-error", "")
        .unwrap();
}

fn emit_error_authorization_failed(
        app_handle: &tauri::AppHandle,
    ) {

    app_handle
        .emit_all(
            "tauri://frontend/authorization-failed",
            "ログアウトし再度Twitterにログインしてください",
            )
        .unwrap();
}

fn emit_error_other(
        app_handle: &tauri::AppHandle,
    ) {

    app_handle
        .emit_all(
            "tauri://frontend/other-error",
            "ネットワークに異常があります",
            )
        .unwrap();
}

pub fn start(
    app_handle: tauri::AppHandle,
    authctl_tx: tokio::sync::mpsc::Sender<twitter_authorizator::AuthControl>,
    mut token_rx: tokio::sync::mpsc::Receiver<Oauth2Token>,
) -> tokio::sync::mpsc::Receiver<scheduler::Record> {
    let (tweet_tx, tweet_rx) = tokio::sync::mpsc::channel(QUEUE_LENGTH);

    tokio::spawn(async move {
        let mut token = token_rx.recv().await.unwrap();

        let user_id = match twitter_client::request_user_id(&token).await {
            Ok(t) => {
                t
            }
            Err(e) => { 
                println!("twitter_agent: user id error {:?}", e);
                "".to_string()
            }
        };

        let mut since_id: Option<&str> = None;
        let mut since_id_string;

        loop {
            let mut tweets =
                match twitter_client::request_tweet_new(&token, user_id.as_str(), since_id).await {
                    //let tweets = match twitter_client::request_user_timeline(&token, user_id.as_str(), start_time).await {
                    Ok(t) => {
                        emit_clear_error(&app_handle);
                        t
                    }

                    Err(e) => match e {
                        twitter_client::RequestError::Unauthorized => {
                            println!("twitter_agent: unauthorized {:?}", e);

                            authctl_tx
                                .send(twitter_authorizator::AuthControl::Authorize)
                                .await
                                .unwrap();
                            token = token_rx.recv().await.unwrap();

                            emit_error_authorization_failed(&app_handle);

                            tokio::time::sleep(tokio::time::Duration::from_millis(REQUEST_PERIOD))
                                .await;
                            continue;
                        }

                        twitter_client::RequestError::Unknown(msg) => {
                            println!("twitter_agent: unknown {:?}", msg);
 
                            emit_error_other(&app_handle);

                            tokio::time::sleep(tokio::time::Duration::from_millis(REQUEST_PERIOD))
                                .await;
                            continue;
                        }
                    },
                };

            if tweets["meta"]["result_count"] == 0 {
                println!("twitter_agent: no data returned");

            } else {
                println!("{:?}", tweets);

                let users = tweets["includes"]["users"].clone();
                let media = tweets["includes"]["media"].clone();

                let mut rev_data = tweets["data"].as_array_mut().unwrap();
                rev_data.reverse();
                for tweet in rev_data {
                    since_id_string = tweet["id"].as_str().unwrap().to_string();
                    since_id = Some(since_id_string.as_str());

                    let empty_vec = Vec::new();
                    let record: scheduler::Record = scheduler::into(&tweet, &users.as_array().unwrap(), &media.as_array().unwrap_or(&empty_vec));
                    tweet_tx.send(record).await.unwrap();
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(REQUEST_PERIOD)).await;
        }
    });

    tweet_rx
}
