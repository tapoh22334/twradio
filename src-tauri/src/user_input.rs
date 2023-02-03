use serde::{Serialize, Deserialize};
use crate::voicegen_observer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserInput {
    Jump(String),
    Paused(bool),
    Speaker(voicegen_observer::Speaker),
    SpeechRate(f64),
}

