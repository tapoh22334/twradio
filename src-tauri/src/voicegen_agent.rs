use serde::{Serialize, Deserialize};
use crate::scheduler;
use crate::voicegen_client;
use crate::voicegen_filter;
use wana_kana::to_hiragana::*;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub tweet_id: String,
    pub text: String,
    pub name: String,
}

impl From<scheduler::Record> for Playbook {
    fn from(record: scheduler::Record) -> Self {
        Playbook {
            tweet_id: record.tweet_id,
            text: record.text,
            name: record.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speech {
    pub tweet_id: String,
    pub text: Vec<u8>,
    pub name: Vec<u8>,
}

pub fn start(app_handle: tauri::AppHandle,
             mut playbook_rx: tokio::sync::mpsc::Receiver<Playbook>,
             speech_tx: tokio::sync::mpsc::Sender<Speech>
             )
{

    tokio::spawn(async move {

        let mut name_cache: HashMap::<String, Vec<u8>> = HashMap::new();
        loop {
            match playbook_rx.recv().await {
                Some(msg) => {

                    // Modify username for speech
                    let hira_name = to_hiragana(msg.name.as_str());
                    let speech_name = match name_cache.get(&hira_name) {
                        Some(hit) => hit,
                        None => {
                            let v = voicegen_client::request_voice(&hira_name).await.unwrap();

                            // To shorten TTS processing time, cache the user name speech
                            // TBD: Warning: cache out method is not implemented.
                            // It would consume more memory if the non follower is comes here.
                            name_cache.insert(hira_name.clone(), v);
                            name_cache.get(&hira_name).unwrap()
                        },
                    };

                    // Modify tweet message for speech
                    let hira_text = voicegen_filter::replace_retweet(msg.text.as_str());
                    let hira_text = voicegen_filter::replace_url(hira_text.as_str());
                    let hira_text = to_hiragana(hira_text.as_str());
                    let speech_text = voicegen_client::request_voice(&hira_text).await.unwrap();

                    println!("{:?}", msg.text);
                    println!("{:?}", hira_text);

                    let speech = Speech {tweet_id: msg.tweet_id, text: speech_text, name: speech_name.clone()};

                    speech_tx.send(speech).await.unwrap();
                },

                None => { return (); }
            }
        }
    });

}
