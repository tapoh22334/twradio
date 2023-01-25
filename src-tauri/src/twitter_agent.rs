use twitter_v2::authorization::Oauth2Token;

use crate::scheduler;
use crate::twitter_client;
use crate::twitter_authorizator;

const QUEUE_LENGTH : usize = 24;
const REQUEST_PERIOD: u64 = 15000; // milliseconds


pub fn start(app_handle: tauri::AppHandle, 
             authctl_tx: tokio::sync::mpsc::Sender<twitter_authorizator::AuthControl>,
             mut token_rx: tokio::sync::mpsc::Receiver<Oauth2Token>)
    -> tokio::sync::mpsc::Receiver<scheduler::Record>
{
    let (tweet_tx, tweet_rx) = tokio::sync::mpsc::channel(QUEUE_LENGTH);

    tokio::spawn(async move {
        let mut token = token_rx.recv().await.unwrap();

        let user_id = match twitter_client::request_user_id(&token).await {
            Ok(t) => t,
            Err(e) => {
                // TBD: Recovery if the token is expired
                panic!("TBD: not implemented error handling! {:?}", e);
            },
        };

        let mut start_time_str: String;
        let mut start_time: Option<&str> = None;
        let mut latest_tweet_id: String = "".to_string();

        loop {
            let tweets = match twitter_client::request_tweet_new(&token, user_id.as_str(), start_time).await {
                Ok(t) => t,
                Err(e) => {
                    match e {
                        twitter_client::RequestError::Unauthorized => {
                            println!("{:?}", e);
                            authctl_tx.send(twitter_authorizator::AuthControl::Authorize).await.unwrap();
                            token = token_rx.recv().await.unwrap();
                            continue;
                        },

                        twitter_client::RequestError::Unknown(msg) => {
                            panic!("{:?}", msg);
                        },

                    }
                },
            };

            let result_count = tweets.meta.result_count;

            if result_count == 0 {
                println!("twitter_agent: no data returned");

            } else {
                println!("{:?}", tweets);

                let latest_tweet = tweets.data.as_ref().unwrap().get(0).unwrap().clone();
                start_time_str = latest_tweet.created_at.clone();
                start_time = Some(start_time_str.as_str());

                let users = tweets.includes.unwrap().users;
                let mut rev_data = tweets.data.unwrap();
                rev_data.reverse();
                for tweet in rev_data {
                    if latest_tweet_id == tweet.id {
                        println!("twitter_agent: duplicated tweet");
                    } else {
                        latest_tweet_id = tweet.id.clone();
                        let record: scheduler::Record = scheduler::into(&tweet, &users);
                        tweet_tx.send(record).await.unwrap();
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(REQUEST_PERIOD)).await;
        }
    });

    tweet_rx
}
