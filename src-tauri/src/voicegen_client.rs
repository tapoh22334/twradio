use serde::{Serialize, Deserialize};
use reqwest::Url;

use crate::voicegen_data;

fn base_url() -> Url {
    Url::parse("http://localhost:50031/").unwrap()
}

#[derive(Serialize)]
struct Body {
   text: String,
   speaker_id: u32
}

#[derive(Debug)]
pub enum RequestError {
    Unknown(String),
}


pub async fn request_voice(addr: std::net::SocketAddr, speaker: u64, text: &String) -> Result<Vec<u8>, RequestError> {
    let client = reqwest::Client::new();

    // Generate query
    let url: String = format!("http://{}/audio_query", addr);
    let audio_query = client.post(url)
                    .query(&[("text", text.as_str()), ("speaker", speaker.to_string().as_str())])

                    .send()
                    .await
                    .map_err(|e| { 
                        RequestError::Unknown(e.to_string())
                    })?

                    .text()
                    .await
                    .map_err(|e| { 
                        RequestError::Unknown(e.to_string())
                    })?;

    // Generate wav
    let url = base_url().join("synthesis").unwrap();
    let data = client.post(url)
                    .query(&[("speaker", "0")])
                    .body(audio_query)

                    .send()
                    .await
                    .map_err(|e| { 
                        RequestError::Unknown(e.to_string())
                    })?

                    .bytes()
                    .await
                    .map_err(|e| { 
                        RequestError::Unknown(e.to_string())
                    })?;

    Ok(data.to_vec())
}

pub async fn request_speakers(addr: std::net::SocketAddr) -> Result<voicegen_data::SpeekersResponse, RequestError>
{
    let client = reqwest::Client::new();
    let url: String = format!("http://{}/speakers", addr);

    // Request speakers
    let speakers_response = client.get(url)
                    .send()
                    .await
                    .map_err(|e| { 
                        RequestError::Unknown(e.to_string())
                    })?;

    //println!("{:?}", speakers_response.status());
    let speakers_response = speakers_response.text().await.unwrap();
    //println!("{:?}", speakers_response);

    let speakers_response = 
            serde_json::from_str::<voicegen_data::SpeekersResponse>(speakers_response.as_str())
                    .map_err(|e| { 
                        RequestError::Unknown(e.to_string())
                    })?;

    Ok(speakers_response)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn ts_request_speakers() {
        let speakers_response: voicegen_data::SpeekersResponse =
            request_speakers(std::net::SocketAddr::from(([127, 0, 0, 1], 50031)))
            .await
            .unwrap();

        println!("{:?}", speakers_response);
    }
}
