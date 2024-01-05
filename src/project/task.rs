use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize)]
#[serde(default)]
pub struct Task {
    // TODO: implement
}

impl Task {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {}
    }
}