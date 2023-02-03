use crate::scheduler;
use crate::voicegen_client;
use crate::voicegen_filter;
use serde::{Deserialize, Serialize};
use wana_kana::to_hiragana::*;

use std::collections::HashMap;

use tauri::Manager;

const REQUEST_PERIOD: u64 = 3000; // milliseconds
                                  //
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub tweet_id: String,
    pub text: String,
    pub name: String,
    pub addr: std::net::SocketAddr,
    pub speaker: u64,
    pub speech_rate: f64,
}

pub fn into(
    record: scheduler::Record,
    addr: std::net::SocketAddr,
    speaker: u64,
    speech_rate: f64,
) -> Playbook {
    Playbook {
        tweet_id: record.tweet_id,
        text: record.text,
        name: record.name,
        addr,
        speaker,
        speech_rate,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speech {
    pub tweet_id: String,
    pub text: Vec<u8>,
    pub name: Vec<u8>,
}

pub fn start(
    app_handle: tauri::AppHandle,
    mut playbook_rx: tokio::sync::mpsc::Receiver<Playbook>,
    speech_tx: tokio::sync::mpsc::Sender<Option<Speech>>,
) {
    // Wait while speaker detect
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let ctx = std::sync::Mutex::new(Some(tx));
    let id = app_handle
        .clone()
        .listen_global("tauri://backend/speakers-ready", move |event| {
            let mut rdy_tx = ctx.lock().unwrap();
            rdy_tx.take().unwrap().send(()).unwrap()
        });

    tokio::spawn(async move {
        // Wait while speaker detect
        let _ = rx.await.unwrap();
        app_handle.unlisten(id);

        let mut name_cache: HashMap<String, Vec<u8>> = HashMap::new();
        loop {
            match playbook_rx.recv().await {
                Some(msg) => {
                    let mut speech_name = None;
                    let mut speech_text = None;

                    // Modify username for speech
                    //
                    let hira_name = to_hiragana(msg.name.as_str());
                    let resp = voicegen_client::request_voice(
                        msg.addr,
                        msg.speaker,
                        msg.speech_rate,
                        &hira_name,
                    )
                    .await;

                    speech_name = match resp {
                        Ok(s) => {
                            app_handle
                                .emit_all("tauri://frontend/tts-failed", "")
                                .unwrap();
                            Some(s)
                        }
                        Err(e) => match e {
                            voicegen_client::RequestError::Unknown(emsg) => {
                                app_handle
                                    .emit_all(
                                        "tauri://frontend/tts-failed",
                                        "音声の取得に失敗しました",
                                    )
                                    .unwrap();

                                println!("voicegen_client: failed to process tts {:?}", emsg);
                                speech_tx.send(None).await.unwrap();
                                continue;
                            }
                        },
                    };

                    // Modify tweet message for speech
                    let hira_text = voicegen_filter::replace_retweet(msg.text.as_str());
                    let hira_text = voicegen_filter::replace_url(hira_text.as_str());
                    let hira_text = to_hiragana(hira_text.as_str());

                    let resp = voicegen_client::request_voice(
                        msg.addr,
                        msg.speaker,
                        msg.speech_rate,
                        &hira_text,
                    )
                    .await;
                    speech_text = match resp {
                        Ok(s) => {
                            app_handle
                                .emit_all("tauri://frontend/tts-failed", "")
                                .unwrap();
                            Some(s)
                        }
                        Err(e) => match e {
                            voicegen_client::RequestError::Unknown(emsg) => {
                                app_handle
                                    .emit_all(
                                        "tauri://frontend/tts-failed",
                                        "音声の取得に失敗しました",
                                    )
                                    .unwrap();

                                println!("voicegen_client: failed to process tts {:?}", emsg);
                                speech_tx.send(None).await.unwrap();
                                continue;
                            }
                        },
                    };

                    println!("{:?}", hira_text);

                    let speech = Speech {
                        tweet_id: msg.tweet_id,
                        text: speech_text.unwrap(),
                        name: speech_name.clone().unwrap(),
                    };

                    speech_tx.send(Some(speech)).await.unwrap();
                }

                None => {
                    println!("voicegen_agent: exit");
                    return ();
                }
            }
        }
    });
}
