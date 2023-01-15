use reqwest::Url;

fn base_url() -> Url {
    Url::parse("http://localhost:50031/").unwrap()
}

pub async fn request_voice(text: &String) -> Result<Vec<u8>, reqwest::Error> {
    //let client = reqwest::Client::new();

    //let url = base_url().join("users/me").unwrap();
    //let me = client.get(url)
    //                .send()
    //                .await?
    //                .text()
    //                .await?;

    //let me: serde_json::Value = serde_json::from_str(me.as_str()).unwrap();
    //let user_id = me.get("data").unwrap()
    //                .get("id").unwrap()
    //                .as_str().unwrap();

    let bytes = include_bytes!("../../sample/user.wav").to_vec();
    Ok(bytes)
}

