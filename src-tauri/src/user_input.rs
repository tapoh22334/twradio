use crate::voicegen_observer;
use crate::twitter_agent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserInput {
    Jump(String),
    Paused(bool),
    Speaker(voicegen_observer::Speaker),
    SpeechRate(f64),
    TimelineView(twitter_agent::Timeline),
}
