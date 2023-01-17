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


pub async fn request_tweet(token: &Oauth2Token, user_id: &str) -> Result<twitter_data::TweetsResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let auth_val = format!("Bearer {}", token.access_token().secret());

    let url = base_url().join(format!("users/{user_id}/timelines/reverse_chronological").as_str()).unwrap();
    let timeline= client.get(url)
                    .header(reqwest::header::AUTHORIZATION, auth_val.clone())
                    .query(&[("expansions", "author_id"), ("tweet.fields", "created_at"), ("max_results", "1")])
                    .send()
                    .await?;

    println!("{:?}", timeline.status());
    let timeline = timeline.text().await.unwrap();
    println!("{:?}", timeline);
    let timeline = serde_json::from_str::<twitter_data::TweetsResponse>(timeline.as_str()).unwrap();

    Ok(timeline)
}

pub async fn request_tweet_new(token: &Oauth2Token, user_id: &str, start_time: Option<&str>) -> Result<twitter_data::TweetsResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let auth_val = format!("Bearer {}", token.access_token().secret());

    let mut query = [("expansions", "author_id"), ("tweet.fields", "created_at"), ("max_results", "25")].to_vec();
    match start_time {
        Some(s) => { query.push(("start_time", s)); },
        None => {}
    };

    let url = base_url().join(format!("users/{user_id}/timelines/reverse_chronological").as_str()).unwrap();
    let timeline= client.get(url)
                    .header(reqwest::header::AUTHORIZATION, auth_val.clone())
                    .query(&query)
                    .send()
                    .await?;

    println!("{:?}", timeline.status());
    let timeline = timeline.text().await.unwrap();
    println!("{:?}", timeline);
    let timeline = serde_json::from_str::<twitter_data::TweetsResponse>(timeline.as_str()).unwrap();

    Ok(timeline)
}

pub async fn request_tweet_next(token: &Oauth2Token, user_id: &str, next_token: &str) -> Result<twitter_data::TweetsResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let auth_val = format!("Bearer {}", token.access_token().secret());

    let url = base_url().join(format!("users/{user_id}/timelines/reverse_chronological").as_str()).unwrap();

    let timeline = client.get(url)
                    .header(reqwest::header::AUTHORIZATION, auth_val.clone())
                    .query(&[("pagination_token", next_token), ("expansions", "author_id"), ("tweet.fields", "created_at"), ("max_results", "1")])
                    .send()
                    .await?
                    .json()
                    .await?;

    Ok(timeline)
}
