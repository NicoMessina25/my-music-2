use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::error::{AppError, AppResult, unique_to_conflict};
use crate::model::song::{SongInput, SongView};
use crate::repo::artist;

// ───────────── Helpers de validación ─────────────
const DUP: &str = "Esa canción ya existe para ese artista";

/// Recorta espacios; si queda vacío lo trata como NULL.
fn clean(value: &Option<String>) -> Option<String> {
    match value {
        Some(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    }
}

/// Convierte "2024" en Some(2024) solo si tiene exactamente `len` dígitos.
fn parse_num(s: &str, len: usize) -> Option<u32> {
    if s.len() == len && s.chars().all(|c| c.is_ascii_digit()) {
        s.parse().ok()
    } else {
        None
    }
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Acepta 'YYYY', 'YYYY-MM' o 'YYYY-MM-DD' (fecha real).
pub fn validate_release_date(s: &str) -> AppResult<()> {
    let parts: Vec<&str> = s.split('-').collect();

    let ok = match parts.as_slice() {
        [y] => parse_num(y, 4).is_some(),
        [y, m] => matches!((parse_num(y, 4), parse_num(m, 2)), (Some(_), Some(1..=12))),
        [y, m, d] => match (parse_num(y, 4), parse_num(m, 2), parse_num(d, 2)) {
            (Some(y), Some(m @ 1..=12), Some(d)) => d >= 1 && d <= days_in_month(y, m),
            _ => false,
        },
        _ => false,
    };

    if ok {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "Fecha inválida: '{s}'. Usá AAAA, AAAA-MM o AAAA-MM-DD"
        )))
    }
}

// ───────────── Consultas ─────────────

const SELECT_SONG: &str = "
    SELECT s.id, s.title, s.artist_id, a.name, s.album, s.release_date,
           s.duration_ms, s.notes, s.created_at, s.archived_at
    FROM song s
    JOIN artist a ON a.id = s.artist_id";

/// Convierte una fila del SELECT de arriba en un SongView.
fn map_row(row: &Row) -> rusqlite::Result<SongView> {
    Ok(SongView {
        id: row.get(0)?,
        title: row.get(1)?,
        artist_id: row.get(2)?,
        artist_name: row.get(3)?,
        album: row.get(4)?,
        release_date: row.get(5)?,
        duration_ms: row.get(6)?,
        notes: row.get(7)?,
        created_at: row.get(8)?,
        archived_at: row.get(9)?,
    })
}

fn parse_song(input: &SongInput) -> AppResult<SongInput> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(AppError::Validation("El título es obligatorio".into()));
    }
    let release_date = clean(&input.release_date);
    if let Some(d) = &release_date {
        validate_release_date(d)?;
    }
    if let Some(ms) = input.duration_ms
        && ms <= 0
    {
        return Err(AppError::Validation("La duración debe ser positiva".into()));
    }

    Ok(SongInput {
        title: title.into(),
        album: clean(&input.album),
        artist_name: input.artist_name.clone(),
        duration_ms: input.duration_ms,
        notes: clean(&input.notes),
        release_date,
    })
}

pub fn create(conn: &mut Connection, input: &SongInput) -> AppResult<SongView> {
    // 1) Parsear ANTES de abrir la transacción
    let input = parse_song(input)?;

    // 2) Transacción: artista + canción, todo o nada
    let tx = conn.transaction()?;
    let artist_id = artist::get_or_create(&tx, &input.artist_name)?;

    tx.execute(
        "INSERT INTO song (title, artist_id, album, release_date, duration_ms, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            input.title,
            artist_id,
            input.album,
            input.release_date,
            input.duration_ms,
            input.notes,
        ],
    )
    .map_err(|e| unique_to_conflict(e, DUP))?;

    let id = tx.last_insert_rowid();
    tx.commit()?;
    get(conn, id)
}

pub fn list(conn: &Connection, include_archived: bool) -> AppResult<Vec<SongView>> {
    let sql = format!(
        "{SELECT_SONG}
         WHERE (?1 OR s.archived_at IS NULL)
         ORDER BY a.name COLLATE NOCASE, s.title COLLATE NOCASE"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([include_archived], map_row)?;
    let songs = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(songs)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<SongView> {
    let sql = format!("{SELECT_SONG} WHERE s.id = ?1");
    conn.query_row(&sql, [id], map_row)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("No existe la canción {id}")))
}

pub fn update(conn: &mut Connection, id: i64, input: &SongInput) -> AppResult<SongView> {
    // 1) Validar ANTES de abrir la transacción
    let input = parse_song(input)?;

    // 2) Transacción: artista + canción, todo o nada
    let tx = conn.transaction()?;
    let artist_id = artist::get_or_create(&tx, &input.artist_name)?;

    let n = tx
        .execute(
            "UPDATE song
            SET title = ?1, 
                artist_id = ?2, 
                album = ?3, 
                release_date = ?4, 
                duration_ms = ?5, 
                notes = ?6
            WHERE id = ?7",
            params![
                input.title,
                artist_id,
                input.album,
                input.release_date,
                input.duration_ms,
                input.notes,
                id
            ],
        )
        .map_err(|e| unique_to_conflict(e, DUP))?;

    if n == 0 {
        return Err(AppError::NotFound(format!("Canción {id} inexistente")));
    }

    tx.commit()?;
    get(conn, id)
}

pub fn archive(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE song
            SET archived_at = datetime('now')
            WHERE id = ?1 AND archived_at IS NULL",
        [id],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

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

    fn input_complete(
        title: &str,
        artist: &str,
        album: &str,
        release_date: &str,
        duration_ms: i64,
        notes: &str,
    ) -> SongInput {
        SongInput {
            title: title.into(),
            artist_name: artist.into(),
            album: Some(album.into()),
            release_date: Some(release_date.into()),
            duration_ms: Some(duration_ms),
            notes: Some(notes.into()),
        }
    }

    #[test]
    fn create_song_and_artist() {
        let mut conn = open_in_memory().unwrap();
        let s: SongView = create(&mut conn, &input("Song 2", "Blur")).unwrap();
        assert_eq!(s.title, "Song 2");
        assert_eq!(s.artist_name, "Blur");
    }

    #[test]
    fn reuses_artist_no_case_sensitive() {
        let mut conn = open_in_memory().unwrap();
        let a = create(&mut conn, &input("Song 2", "Blur")).unwrap();
        let b = create(&mut conn, &input("Parklife", "BLUR")).unwrap();
        assert_eq!(a.artist_id, b.artist_id);
    }

    #[test]
    fn duplicated_song_conflicts_and_cleans_up() {
        let mut conn = open_in_memory().unwrap();
        create(&mut conn, &input("Song 2", "Blur")).unwrap();
        let err = create(&mut conn, &input("song 2", "Blur")).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
        assert_eq!(list(&conn, true).unwrap().len(), 1);
    }

    #[test]
    fn validate_dates() {
        for ok in ["1997", "1997-02", "2024-02-29"] {
            assert!(validate_release_date(ok).is_ok(), "debería aceptar {ok}");
        }
        for bad in [
            "97",
            "1997-13",
            "1997-2",
            "2023-02-29",
            "1997-02-30",
            "abcd",
        ] {
            assert!(
                validate_release_date(bad).is_err(),
                "debería rechazar {bad}"
            );
        }
    }

    #[test]
    fn update_modifies_all() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 4", "Nicky")).unwrap();

        let s_input = input_complete("  Song 5", "Nicky", "album", "2026-01", 42343, "A");

        let s2 = update(&mut conn, s.id, &s_input).unwrap();

        assert_eq!(s2.title, "Song 5");
        assert_eq!(s2.album, Some("album".into()));
        assert_eq!(s2.release_date, Some("2026-01".into()));
        assert_eq!(s2.duration_ms, Some(42343));
        assert_eq!(s2.notes, Some("A".into()));
        assert_eq!(s2.artist_id, s.artist_id);
    }

    #[test]
    fn update_new_artist() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 4", "Nicky")).unwrap();

        let s2 = update(&mut conn, s.id, &input(&s.title, "Pedro")).unwrap();

        let a = artist::get(&conn, s2.artist_id).unwrap();

        assert_eq!(a.name, "Pedro");
        assert_ne!(s.artist_id, s2.artist_id);
    }

    #[test]
    fn update_existing_artist() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 6", "Pedro")).unwrap();
        let s2 = create(&mut conn, &input("Song 4", "Nicky")).unwrap();

        let s3 = update(&mut conn, s2.id, &input(&s2.title, "PEDRO")).unwrap();

        assert_eq!(s3.artist_id, s.artist_id);
    }

    #[test]
    fn update_inexisting_song() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 6", "Pedro")).unwrap();
        let len = artist::list(&conn, true).unwrap().len();

        let err = update(&mut conn, -1, &input(&s.title, "ALANA")).unwrap_err();

        assert!(matches!(err, AppError::NotFound(_)));
        assert_eq!(artist::list(&conn, true).unwrap().len(), len);
    }

    #[test]
    fn update_existing_song() {
        let mut conn = open_in_memory().unwrap();
        create(&mut conn, &input("Song 6", "Pedro")).unwrap();
        let s = create(&mut conn, &input("Song 4", "Nicky")).unwrap();

        let err = update(&mut conn, s.id, &input("Song 6", "PEDRO")).unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));

        let s2 = get(&conn, s.id).unwrap();
        assert_eq!(s2.title, "Song 4");
        assert_eq!(s2.artist_name, "Nicky");
    }

    #[test]
    fn update_modifies_album() {
        let mut conn = open_in_memory().unwrap();
        let s = create(
            &mut conn,
            &SongInput {
                album: Some("album1".into()),
                ..input("Song 6", "Pedro")
            },
        )
        .unwrap();

        let s2 = update(
            &mut conn,
            s.id,
            &SongInput {
                album: Some("album2".into()),
                ..input("Song 6", "Pedro")
            },
        )
        .unwrap();

        assert_eq!(s2.album, Some("album2".into()));
    }

    #[test]
    fn update_invalid_input() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 6", "Pedro")).unwrap();

        let err = update(
            &mut conn,
            s.id,
            &SongInput {
                release_date: Some("fsdf".into()),
                ..input("Song 6", "Pedro")
            },
        )
        .unwrap_err();

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn archive_once() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 6", "Pedro")).unwrap();

        archive(&conn, s.id).unwrap();

        let len_not_include = list(&conn, false).unwrap().len();
        let len_include = list(&conn, true).unwrap().len();

        assert_eq!(len_not_include, 0);
        assert_eq!(len_include, 1);
    }

    #[test]
    fn archive_twice() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 6", "Pedro")).unwrap();

        let archived_at = "2020-01-01";

        conn.execute(
            "
            UPDATE song 
            SET archived_at = ?2
            WHERE id = ?1
        ",
            params![s.id, archived_at],
        )
        .unwrap();

        archive(&conn, s.id).unwrap();

        let archived_at_after = get(&conn, s.id).unwrap().archived_at;

        assert_eq!(archived_at_after, Some(archived_at.into()));
    }

    #[test]
    fn archive_inexistent_song() {
        let conn = open_in_memory().unwrap();
        archive(&conn, -1).unwrap();
    }
}
