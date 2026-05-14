use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SessionRatings {
    pub mental_energy: u8,
    pub physical_energy: u8,
    pub cognitive_load: u8,
    #[serde(default)]
    pub motivation: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub description: String,
    pub duration: Duration,
    pub start: String,
    pub tags: Vec<String>,
    pub notes: String,
    pub state: String,
    pub ratings: Option<SessionRatings>,
}
