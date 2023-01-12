use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tweet {
    pub author_id: String,
    pub edit_history_tweet_ids: Vec<String>,
    pub id: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Includes {
    pub users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub newest_id: String,
    pub next_token: String,
    pub oldest_id: String,
    pub result_count: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TweetsResponse {
    pub data: Vec<Tweet>,
    pub includes: Includes,
    pub meta: Meta,
}

