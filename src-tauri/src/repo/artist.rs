use rusqlite::{Connection, OptionalExtension, Row};
use crate::model::artist::{Artist, ArtistInput};
use crate::error::{unique_to_conflict, AppError, AppResult};

const COLS: &str = "id, name, archived_at";
const DUP: &str = "Ya existe un artista con ese nombre";

fn map(r: &Row) -> rusqlite::Result<Artist> {
    Ok(Artist { id: r.get(0)?, name: r.get(1)?, archived_at: r.get(2)? })
}

fn clean(input: &ArtistInput) -> AppResult<&str> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("El nombre es obligatorio".into()));
    }
    Ok(name)
}

pub fn list(conn: &Connection, include_archived: bool) -> AppResult<Vec<Artist>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLS} FROM artist WHERE (?1 OR archived_at IS NULL) ORDER BY name COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map([include_archived], map)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Artist> {
    conn.query_row(&format!("SELECT {COLS} FROM artist WHERE id = ?1"), [id], map)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Artista {id} inexistente")))
}

pub fn create(conn: &Connection, input: &ArtistInput) -> AppResult<Artist> {
    let name = clean(input)?;
    conn.execute("INSERT INTO artist (name) VALUES (?1)", [name])
        .map_err(|e| unique_to_conflict(e, DUP))?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, input: &ArtistInput) -> AppResult<Artist> {
    let name = clean(input)?;
    let n = conn
        .execute("UPDATE artist SET name = ?1 WHERE id = ?2", (name, id))
        .map_err(|e| unique_to_conflict(e, DUP))?;
    if n == 0 {
        return Err(AppError::NotFound(format!("Artista {id} inexistente")));
    }
    get(conn, id)
}

/// Equivalente al "eliminar solo si no es autor de canciones" del Análisis.
pub fn archive(conn: &Connection, id: i64) -> AppResult<()> {
    let active_songs: i64 = conn.query_row(
        "SELECT count(*) FROM song WHERE artist_id = ?1 AND archived_at IS NULL",
        [id], |r| r.get(0),
    )?;
    if active_songs > 0 {
        return Err(AppError::Conflict(format!(
            "El artista tiene {active_songs} canción(es) activa(s)"
        )));
    }
    conn.execute(
        "UPDATE artist SET archived_at = datetime('now') WHERE id = ?1 AND archived_at IS NULL",
        [id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn duplicate_name_is_conflict_case_insensitive() {
        let conn = open_in_memory().unwrap();
        create(&conn, &ArtistInput { name: "Soda Stereo".into() }).unwrap();
        let err = create(&conn, &ArtistInput { name: "  soda stereo ".into() }).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn empty_name_is_validation_error() {
        let conn = open_in_memory().unwrap();
        let err = create(&conn, &ArtistInput { name: "   ".into() }).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}