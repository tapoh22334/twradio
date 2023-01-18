use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tweet {
    pub author_id: String,
    pub created_at: String,
    pub edit_history_tweet_ids: Vec<String>,
    pub id: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub username: String,
    pub profile_image_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Includes {
    pub users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub newest_id: Option<String>,
    pub next_token: Option<String>,
    pub oldest_id: Option<String>,
    pub result_count: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TweetsResponse {
    pub data: Option<Vec<Tweet>>,
    pub includes: Option<Includes>,
    pub meta: Meta,
}

