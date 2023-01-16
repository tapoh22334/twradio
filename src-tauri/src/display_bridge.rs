use serde::{Serialize, Deserialize};
use crate::scheduler;

use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewElements {
    pub tweet_id: String,
    pub created_at: String,
    pub text: String,
    pub name: String,
    pub username: String,
}

impl From<scheduler::Record> for ViewElements {
    fn from(record: scheduler::Record) -> Self {
        ViewElements {
            tweet_id: record.tweet_id,
            created_at: record.created_at,
            text: record.text,
            name: record.name,
            username: record.username
        }
    }
}

pub fn start(app_handle: tauri::AppHandle, mut display_rx: tokio::sync::mpsc::Receiver<ViewElements>)
{
    tokio::spawn(async move {
        loop {
            match display_rx.recv().await {
                Some(msg) => {
                    app_handle
                        .emit_all("tauri://frontend/display", msg)
                        .unwrap();
                },

                None => { return (); }
            }
        }
    });
}
