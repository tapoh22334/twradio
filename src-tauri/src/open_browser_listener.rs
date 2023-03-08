use tauri::Manager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Payload {
  url: String,
}

pub fn start( app_handle: tauri::AppHandle) {
    app_handle
        .listen_global("tauri://backend/open_browser", move |event| {
            let json = event.payload().unwrap();
            println!("json: {:?}", json);

            let payload: Payload = serde_json::from_str(json).unwrap();
            println!("url: {:?}", payload.url);

            webbrowser::open(payload.url.as_str()).unwrap();
        });
}
