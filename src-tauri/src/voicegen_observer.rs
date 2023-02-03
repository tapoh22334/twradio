use crate::scheduler;
use crate::voicegen_client;
use crate::voicegen_data;
use serde::{Deserialize, Serialize};

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
    pub fn vec_from(
        engine: &str,
        addr: std::net::SocketAddr,
        speaker: voicegen_data::Speaker,
    ) -> Vec<Speaker> {
        let mut v = Vec::<Speaker>::new();

        for style in speaker.styles {
            v.push(Speaker {
                addr,
                engine: engine.to_string(),
                name: speaker.name.clone(),
                style: style.name,
                speaker: style.id,
            })
        }

        v
    }
}

const OFFSET_TIME: u64 = 3000;

pub fn start(app_handle: tauri::AppHandle) {
    let addrs = vec![
        (
            "VOICEVOX",
            std::net::SocketAddr::from(([127, 0, 0, 1], 50021)),
        ),
        (
            "COEIROINK",
            std::net::SocketAddr::from(([127, 0, 0, 1], 50031)),
        ),
        (
            "LMROID",
            std::net::SocketAddr::from(([127, 0, 0, 1], 50073)),
        ),
        (
            "SHAREVOX",
            std::net::SocketAddr::from(([127, 0, 0, 1], 50025)),
        ),
        (
            "ITVOICE",
            std::net::SocketAddr::from(([127, 0, 0, 1], 49540)),
        ),
    ];

    // Wait while speaker detect
    let (wait_tx, wait_rx) = tokio::sync::oneshot::channel::<()>();
    let ctx = std::sync::Mutex::new(Some(wait_tx));
    let id = app_handle
        .clone()
        .listen_global("tauri://backend/ipc-init", move |event| {
            let mut rdy_tx = ctx.lock().unwrap();
            rdy_tx.take().unwrap().send(()).unwrap()
        });

    // ipc-init is called once by frontend
    tokio::spawn(async move {
        // Wait while speaker detect
        let _ = wait_rx.await.unwrap();
        app_handle.unlisten(id);

        // Main loop
        let mut latest_vec = Vec::<Speaker>::new();
        loop {
            let mut vec = Vec::<Speaker>::new();

            for (engine, addr) in &addrs {
                let resp = voicegen_client::request_speakers(*addr).await;
                match resp {
                    Ok(speakers) => {
                        for speaker in speakers {
                            let mut v: Vec<Speaker> = Speaker::vec_from(engine, *addr, speaker);
                            vec.append(&mut v);
                        }
                    }

                    Err(e) => {
                        println!("voicegen_observer: {:?}", e);
                    }
                }
            }

            if vec != latest_vec {
                latest_vec = vec.clone();
                app_handle
                    .emit_all("tauri://frontend/speakers-register", vec.clone())
                    .unwrap();

                app_handle
                    .emit_all("tauri://frontend/speakers-ready", ())
                    .unwrap();
            }

            if vec.len() == 0 {
                app_handle
                    .emit_all(
                        "tauri://frontend/no-voicegen-found",
                        "TTSエンジンを起動してください",
                    )
                    .unwrap();
            } else {
                app_handle
                    .emit_all("tauri://frontend/no-voicegen-found", "")
                    .unwrap();
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(OFFSET_TIME)).await;
        }
    });
}
