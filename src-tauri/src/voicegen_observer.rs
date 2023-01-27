use serde::{Serialize, Deserialize};
use crate::scheduler;
use crate::voicegen_client;
use crate::voicegen_data;

use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Speaker {
    pub addr: std::net::SocketAddr,
    pub engine: String,
    pub name: String,
    pub style: String,
    pub speaker: u64,
}

impl Speaker {
    pub fn vec_from(addr: std::net::SocketAddr, speaker: voicegen_data::Speaker) -> Vec<Speaker> {
        let mut v = Vec::<Speaker>::new();

        for style in speaker.styles {
            v.push (
                Speaker {
                    addr,
                    engine: "COEIROINK".to_string(),
                    name: speaker.name.clone(),
                    style: style.name,
                    speaker: style.id,
                }
            )
        }

        v
    }
}

const OFFSET_TIME: u64 = 3000;

pub fn start(app_handle: tauri::AppHandle)
{
    let coeiroink_addr = std::net::SocketAddr::from(([127, 0, 0, 1], 50031));

    // ipc-init is called once by frontend
    let _ = app_handle.clone().listen_global("tauri://backend/ipc-init", move |event| {
        let handle = app_handle.clone();

        tokio::spawn(async move {
            let mut latest_vec = Vec::<Speaker>::new();
            loop {
                let resp = voicegen_client::request_speakers(coeiroink_addr).await;
                let mut vec = Vec::<Speaker>::new();
                match resp {
                    Ok(speakers) => {
                        for speaker in speakers {
                            let mut v: Vec<Speaker> = Speaker::vec_from(coeiroink_addr, speaker);
                            vec.append(&mut v);
                        }

                        if vec != latest_vec {
                            latest_vec = vec.clone();
                            handle
                                .emit_all("tauri://frontend/speakers-register", vec)
                                .unwrap();

                            handle
                                .emit_all("tauri://frontend/speakers-ready", ())
                                .unwrap();
                        }
                    },

                    Err(e) => {
                        println!("voicegen_observer: {:?}", e);
                    },
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(OFFSET_TIME)).await;
            }
        });
    });

}
