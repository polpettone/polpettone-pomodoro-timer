use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Session {
    pub description: String,
    pub duration: Duration,
    pub start: String,
    pub state: String,
}
