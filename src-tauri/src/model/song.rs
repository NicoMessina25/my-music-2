use serde::{Deserialize, Serialize};

use crate::model::artist::ArtistRef;

/// Lo que manda el front para crear o editar una canción.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongInput {
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub release_date: Option<String>,
    pub duration_ms: Option<i64>,
    pub notes: Option<String>,
}

/// Lo que devolvemos al front: la canción + el nombre del artista.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongView {
    pub id: i64,
    pub title: String,
    pub artists: Vec<ArtistRef>,
    pub album: Option<String>,
    pub release_date: Option<String>,
    pub duration_ms: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub archived_at: Option<String>,
}
