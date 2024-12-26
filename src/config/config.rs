use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigList {
    pub id: String,
    pub name: String,
}
