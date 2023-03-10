use crate::scheduler;
use serde::{Deserialize, Serialize};

use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewElements {
    pub tweet_id: String,
    pub author_id: String,
    pub created_at: String,
    pub text: String,
    pub name: String,
    pub username: String,
    pub profile_image_url: String,
    pub attachments: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayContrl {
    Add(String, ViewElements),
    Scroll(String, String),
    Delete(String, String),
}

impl From<scheduler::Record> for ViewElements {
    fn from(record: scheduler::Record) -> Self {
        ViewElements {
            tweet_id: record.tweet_id,
            author_id: record.author_id,
            created_at: record.created_at,
            text: record.text,
            name: record.name,
            username: record.username,
            profile_image_url: record.profile_image_url,
            attachments: record.attachments,
        }
    }
}

pub fn start(
    app_handle: tauri::AppHandle,
    mut display_rx: tokio::sync::mpsc::Receiver<DisplayContrl>,
) {
    tokio::spawn(async move {
        loop {
            match display_rx.recv().await {
                Some(msg) => match msg {
                    DisplayContrl::Add(name, ve) => {
                        let url = format!("tauri://frontend/display/{name}/add");
                        app_handle
                            .emit_all(url.as_str(), ve)
                            .unwrap();
                    }

                    DisplayContrl::Delete(name, twid) => {
                        let url = format!("tauri://frontend/display/{name}/delete");
                        app_handle
                            .emit_all(url.as_str(), twid)
                            .unwrap();
                    }

                    DisplayContrl::Scroll(name, twid) => {
                        let url = format!("tauri://frontend/display/{name}/scroll");
                        app_handle
                            .emit_all(url.as_str(), twid)
                            .unwrap();
                    }
                },

                None => {
                    return ();
                }
            }
        }
    });
}
