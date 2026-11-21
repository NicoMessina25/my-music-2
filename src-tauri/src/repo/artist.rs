use rusqlite::{Connection, OptionalExtension, Row};
use crate::model::artist::{Artist, ArtistInput, ArtistView};
use crate::error::{unique_to_conflict, AppError, AppResult};

const COLS: &str = "id, name, archived_at";
const DUP: &str = "Ya existe un artista con ese nombre";

fn map(r: &Row) -> rusqlite::Result<Artist> {
    Ok(Artist { id: r.get(0)?, name: r.get(1)?, archived_at: r.get(2)? })
}

fn map_view(r: &Row) -> rusqlite::Result<ArtistView> {
    Ok(ArtistView { id: r.get(0)?, name: r.get(1)?, archived_at: r.get(2)?, songs_count: r.get(3)? })
}

fn clean(input: &ArtistInput) -> AppResult<&str> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("El nombre es obligatorio".into()));
    }
    Ok(name)
}

pub fn list(conn: &Connection, include_archived: bool) -> AppResult<Vec<ArtistView>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, a.archived_at, COUNT(s.id) AS songs_count
        FROM artist a
        LEFT JOIN song s ON s.artist_id = a.id AND s.archived_at IS NULL
        WHERE s.id IS NOT NULL OR ?1
        GROUP BY a.id, a.name, a.archived_at
        ORDER BY a.name COLLATE NOCASE"
    )?;
    let rows = stmt.query_map([include_archived], map_view)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Artist> {
    conn.query_row(&format!("SELECT {COLS} FROM artist WHERE id = ?1"), [id], map)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Artista {id} inexistente")))
}

pub fn get_or_create(conn: &Connection, name: &str) -> AppResult<i64> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("El artista es obligatorio".into()));
    }
    if let Some(id) = conn
        .query_row("SELECT id FROM artist WHERE lower(name) = lower(?1)", [name], |r| r.get(0))
        .optional()?
    {
        conn.execute("UPDATE artist SET archived_at = NULL WHERE id = ?1 AND archived_at IS NOT NULL", [id])?;
        return Ok(id);
    }
    conn.execute("INSERT INTO artist (name) VALUES (?1)", [name])?;
    Ok(conn.last_insert_rowid())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::open_in_memory, model::song::SongInput, repo::song};

    fn input(title: &str, artist: &str) -> SongInput {
        SongInput {
            title: title.into(),
            artist_name: artist.into(),
            album: None,
            release_date: None,
            duration_ms: None,
            notes: None,
        }
    }

    #[test]
    fn reuses_existing_artist_case_insensitive() {
        let conn = open_in_memory().unwrap();
        let a = get_or_create(&conn, "Soda Stereo").unwrap();
        let b = get_or_create(&conn, "  soda stereo ").unwrap();
        assert_eq!(a, b);
        assert_eq!(list(&conn, true).unwrap().len(), 1);
    }

    #[test]
    fn empty_name_is_validation_error() {
        let conn = open_in_memory().unwrap();
        let err = get_or_create(&conn, "   ").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn rename_to_existing_name_is_conflict() {
        let conn = open_in_memory().unwrap();
        get_or_create(&conn, "Soda Stereo").unwrap();
        let virus = get_or_create(&conn, "Virus").unwrap();
        let err = update(&conn, virus, &ArtistInput { name: "soda stereo".into() }).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn rename_same_artist_changing_case_is_ok() {
        let conn = open_in_memory().unwrap();
        let id = get_or_create(&conn, "soda stereo").unwrap();
        let a = update(&conn, id, &ArtistInput { name: "Soda Stereo".into() }).unwrap();
        assert_eq!(a.name, "Soda Stereo");
    }

    #[test]
    fn update_missing_is_not_found() {
        let conn = open_in_memory().unwrap();
        let err = update(&conn, 999, &ArtistInput { name: "X".into() }).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn list_artists_with_archived_songs() {
        let mut conn = open_in_memory().unwrap();

        song::create(&mut conn, &input("song1", "Kygo")).unwrap();
        let id = song::create(&mut conn, &input("song2", "Kygo")).unwrap();

        conn.execute("UPDATE song SET archived_at = '2026-11-06' WHERE id = ?1", [id]).unwrap();

        let artists = list(&conn, false).unwrap();
        assert_eq!(artists.len(), 1);
        assert_eq!(artists[0].songs_count, 1);
    }

    #[test]
    fn list_artists_with_all_archived_songs() {
        let mut conn = open_in_memory().unwrap();

        let id = song::create(&mut conn, &input("song2", "Kygo")).unwrap();

        conn.execute("UPDATE song SET archived_at = '2026-11-06' WHERE id = ?1", [id]).unwrap();

        let artists = list(&conn, false).unwrap();
        assert_eq!(artists.len(), 0);
    }
}