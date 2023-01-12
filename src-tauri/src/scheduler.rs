use serde::{Serialize, Deserialize};
use crate::twitter_data;
use crate::display_bridge;

const QUEUE_LENGTH : usize = 32;

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub tweet_id: String,
    pub text: String,
    pub name: String,
    pub username: String,
}

pub fn into(tweet: &twitter_data::Tweet, users: &Vec<twitter_data::User>) -> Record {
    let user = users.iter().find(|user| user.id == tweet.author_id).unwrap();

    Record {
        tweet_id: tweet.id.clone(),
        text: tweet.text.clone(),
        name: user.name.clone(),
        username: user.username.clone(),
    }
}

pub fn start(mut tweet_rx: tokio::sync::mpsc::Receiver<Record>) -> (
        tokio::sync::mpsc::Receiver<display_bridge::ViewElements>,
        tokio::sync::mpsc::Receiver<Record>
)
{
    let (display_tx, mut display_rx)
        = tokio::sync::mpsc::channel::<display_bridge::ViewElements>(QUEUE_LENGTH);

    let (speech_tx, mut speech_rx) = tokio::sync::mpsc::channel(QUEUE_LENGTH);

    tokio::spawn(async move {
        loop {
            match tweet_rx.recv().await {
                Some(msg) => {
                    display_tx.send(msg.into()).await.unwrap();
                },

                None => { return (); }
            }
        }
    });

    (display_rx, speech_rx)
}
