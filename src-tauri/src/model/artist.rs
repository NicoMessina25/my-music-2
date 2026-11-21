use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistInput {
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistView {
    pub id: i64,
    pub name: String,
    pub archived_at: Option<String>,
    pub songs_count: i64
}