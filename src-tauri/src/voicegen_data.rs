use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Style {
    pub name: String,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Speeker {
    pub name: String,
    pub speaker_uuid: String,
    pub styles: Vec<Style>,
    pub version: String,
}

pub type SpeekersResponse = Vec<Speeker>;

