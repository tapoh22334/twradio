use crate::voicegen_observer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserInput {
    Jump(String),
    Paused(bool),
    Speaker(voicegen_observer::Speaker),
    SpeechRate(f64),
}
