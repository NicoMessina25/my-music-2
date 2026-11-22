use crate::db::Db;
use crate::error::AppResult;
use crate::model::song::{SongInput, SongView};
use crate::repo::song as repo;
use tauri::State;

#[tauri::command]
pub fn list_songs(db: State<'_, Db>, include_archived: Option<bool>) -> AppResult<Vec<SongView>> {
    repo::list(&*db.conn()?, include_archived.unwrap_or(false))
}

#[tauri::command]
pub fn create_song(db: State<'_, Db>, input: SongInput) -> AppResult<SongView> {
    repo::create(&mut *db.conn()?, &input)
}

#[tauri::command]
pub fn get_song(db: State<'_, Db>, id: i64) -> AppResult<SongView> {
    repo::get(&*db.conn()?, id)
}

#[tauri::command]
pub fn update_song(db: State<'_, Db>, id: i64, input: SongInput) -> AppResult<SongView> {
    repo::update(&mut *db.conn()?, id, &input)
}

#[tauri::command]
pub fn archive_song(db: State<'_, Db>, id: i64) -> AppResult<()> {
    repo::archive(&*db.conn()?, id)
}
