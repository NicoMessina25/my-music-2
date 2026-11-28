mod migrations;

use crate::error::{AppError, AppResult};
use std::sync::MutexGuard;

use rusqlite::Connection;
use std::{path::Path, sync::Mutex, time::Duration};

pub use migrations::MIGRATIONS;

/// Estado compartido de Tauri: una única conexión protegida por Mutex.
pub struct Db(pub Mutex<Connection>);

pub type DbResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Abre (o crea) la base en disco, la configura y la lleva a la última migración.
pub fn open(path: &Path) -> DbResult<Connection> {
    let mut conn = Connection::open(path)?;
    configure(&conn)?;
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |_| Ok(()))?;
    MIGRATIONS.to_latest(&mut conn)?;
    Ok(conn)
}

/// Base en memoria para tests: mismo esquema, sin WAL.
#[cfg(test)]
pub fn open_in_memory() -> DbResult<Connection> {
    let mut conn = Connection::open_in_memory()?;
    configure(&conn)?;
    MIGRATIONS.to_latest(&mut conn)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(())
}

impl Db {
    pub fn conn(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.0.lock().map_err(|_| AppError::Lock)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_is_created_and_fks_enforced() {
        let conn = super::open_in_memory().unwrap();

        // song_artist con canción y artista inexistentes: la tabla existe
        // (esquema creado) y el INSERT tiene que fallar justo por la FK.
        let err = conn
            .execute(
                "INSERT INTO song_artist (song_id, artist_id, position) VALUES (999, 999, 0)",
                [],
            )
            .unwrap_err();

        let rusqlite::Error::SqliteFailure(failure, _) = err else {
            panic!("se esperaba un error de SQLite, llegó: {err:?}");
        };
        assert_eq!(
            failure.extended_code,
            rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY
        );
    }
}
