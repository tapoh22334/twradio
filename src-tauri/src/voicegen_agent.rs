use serde::{Serialize, Deserialize};
use crate::scheduler;
use crate::voicegen_client;

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
        loop {
            match playbook_rx.recv().await {
                Some(msg) => {
                    let speech_text = voicegen_client::request_voice(&msg.text).await.unwrap();
                    let speech_name = voicegen_client::request_voice(&msg.name).await.unwrap();

                    let speech = Speech {tweet_id: msg.tweet_id, text: speech_text, name: speech_name};

                    speech_tx.send(speech).await.unwrap();
                },

                None => { return (); }
            }
        }
    });

}
