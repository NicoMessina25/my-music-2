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
        // song con artist inexistente debe fallar por FK
        let err = conn.execute("INSERT INTO song (title, artist_id) VALUES ('x', 999)", []);
        assert!(err.is_err());
    }
}
