use crate::twitter_data;
use twitter_v2::authorization::Oauth2Token;

use reqwest::Url;

fn base_url() -> Url {
    Url::parse("https://api.twitter.com/2/").unwrap()
}

pub async fn request_user_id(token: &Oauth2Token) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let auth_val = format!("Bearer {}", token.access_token().secret());

    let url = base_url().join("users/me").unwrap();
    let me = client.get(url)
                    .header(reqwest::header::AUTHORIZATION, auth_val.clone())
                    .send()
                    .await?
                    .text()
                    .await?;

    let me: serde_json::Value = serde_json::from_str(me.as_str()).unwrap();
    let user_id = me.get("data").unwrap()
                    .get("id").unwrap()
                    .as_str().unwrap();

    Ok(user_id.to_string())
}


pub async fn request_tweet(token: &Oauth2Token) -> Result<twitter_data::TweetsResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let auth_val = format!("Bearer {}", token.access_token().secret());

    let user_id = request_user_id(token).await?;
    println!("user_id : {:?}", user_id);

    let url = base_url().join(format!("users/{user_id}/timelines/reverse_chronological").as_str()).unwrap();
    let timeline: twitter_data::TweetsResponse = client.get(url)
                    .header(reqwest::header::AUTHORIZATION, auth_val.clone())
                    .query(&[("expansions", "author_id")])
                    .send()
                    .await?
                    .json()
                    .await?;

    Ok(timeline)
}
