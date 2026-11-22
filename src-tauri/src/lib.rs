mod commands;
mod db;
mod error;
mod model;
mod repo;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init()) // si la plantilla lo trae, dejalo
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;

            let db_name = if cfg!(debug_assertions) {
                "mymusic-dev.db"
            } else {
                "mymusic.db"
            };
            let conn = db::open(&dir.join(db_name))?;

            app.manage(db::Db(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::artist::list_artists,
            commands::artist::update_artist,
            commands::song::list_songs,
            commands::song::create_song,
            commands::song::update_song,
            commands::song::get_song,
            commands::song::archive_song
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
