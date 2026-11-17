use tauri::State;
use crate::db::Db;
use crate::model::artist::{Artist, ArtistInput};
use crate::error::AppResult;
use crate::repo::artist as repo;

#[tauri::command]
pub fn list_artists(db: State<'_, Db>, include_archived: Option<bool>) -> AppResult<Vec<Artist>> {
    repo::list(&*db.conn()?, include_archived.unwrap_or(false))
}

#[tauri::command]
pub fn create_artist(db: State<'_, Db>, input: ArtistInput) -> AppResult<Artist> {
    repo::create(&*db.conn()?, &input)
}

#[tauri::command]
pub fn update_artist(db: State<'_, Db>, id: i64, input: ArtistInput) -> AppResult<Artist> {
    repo::update(&*db.conn()?, id, &input)
}

#[tauri::command]
pub fn archive_artist(db: State<'_, Db>, id: i64) -> AppResult<()> {
    repo::archive(&*db.conn()?, id)
}