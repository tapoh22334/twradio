use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserInput {
    Jump(String),
    Paused(bool),
}

