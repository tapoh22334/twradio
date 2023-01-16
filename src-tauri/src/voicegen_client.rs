use serde::{Serialize, Deserialize};
use reqwest::Url;

fn base_url() -> Url {
    Url::parse("http://localhost:50031/").unwrap()
}

#[derive(Serialize)]
struct Body {
   text: String,
   speaker_id: u32
}

pub async fn request_voice(text: &String) -> Result<Vec<u8>, reqwest::Error> {
    let client = reqwest::Client::new();

    let url = base_url().join("audio_query").unwrap();
    let audio_query = client.post(url)
                    .query(&[("text", text.as_str()), ("speaker", "0")])
                    .send()
                    .await?
                    .text()
                    .await?;

    let url = base_url().join("synthesis").unwrap();
    let data = client.post(url)
                    .query(&[("speaker", "0")])
                    .body(audio_query)
                    .send()
                    .await?
                    .bytes()
                    .await?;

    Ok(data.to_vec())
}

