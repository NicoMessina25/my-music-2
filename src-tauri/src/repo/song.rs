use std::collections::HashSet;

use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::error::{AppError, AppResult};
use crate::model::artist::ArtistRef;
use crate::model::song::{SongInput, SongView};
use crate::repo::artist;

const DUP: &str = "Esa canción ya existe para esos artistas";

// ───────────── Helpers de validación ─────────────

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
    SELECT 
        s.id, 
        s.title, 
        json_group_array(json_object(
            'id', a.id, 'name', a.name
        ) ORDER BY sa.position) AS artists,
        s.album,
        s.release_date,
        s.duration_ms,
        s.notes,
        s.created_at,
        s.archived_at
    FROM song s
    JOIN song_artist sa ON sa.song_id = s.id 
    JOIN artist a ON a.id = sa.artist_id
";

const SELECT_SONG_GROUP_BY: &str = "
    GROUP BY s.id";

/// Convierte una fila del SELECT de arriba en un SongView.
fn map_row(row: &Row) -> rusqlite::Result<SongView> {
    let artists_json: String = row.get(2)?;
    let artists = serde_json::from_str::<Vec<ArtistRef>>(&artists_json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, Type::Text, Box::new(e)))?;

    Ok(SongView {
        id: row.get(0)?,
        title: row.get(1)?,
        artists,
        album: row.get(3)?,
        release_date: row.get(4)?,
        duration_ms: row.get(5)?,
        notes: row.get(6)?,
        created_at: row.get(7)?,
        archived_at: row.get(8)?,
    })
}

fn clean_artists(names: &[String]) -> AppResult<Vec<String>> {
    let mut seen = HashSet::new();
    let artists: Vec<String> = names
        .iter()
        .map(|n| n.trim())
        .filter(|n| !n.is_empty())
        .filter(|n| seen.insert(n.to_lowercase()))
        .map(String::from)
        .collect();

    if artists.is_empty() {
        return Err(AppError::Validation("Al menos un artista".into()));
    }
    Ok(artists)
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
    let artists = clean_artists(&input.artists)?;

    Ok(SongInput {
        title: title.into(),
        artists,
        album: clean(&input.album),
        release_date,
        duration_ms: input.duration_ms,
        notes: clean(&input.notes),
    })
}

/// get_or_create de cada nombre, en orden. Corta en el primer error.
fn resolve_artists(conn: &Connection, names: &[String]) -> AppResult<Vec<i64>> {
    names
        .iter()
        .map(|name| artist::get_or_create(conn, name))
        .collect()
}

/// Conflict si existe otra canción (archivada o no) con el mismo título
/// y el mismo conjunto de artistas, sin importar el orden.
fn ensure_not_duplicate(
    conn: &Connection,
    exclude_id: Option<i64>,
    title: &str,
    artist_ids: &[i64],
) -> AppResult<()> {
    let wanted: HashSet<i64> = artist_ids.iter().copied().collect();

    let sql = format!(
        "{SELECT_SONG}
         WHERE lower(s.title) = lower(?1) AND s.id IS NOT ?2
         {SELECT_SONG_GROUP_BY}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let candidates = stmt
        .query_map(params![title, exclude_id], map_row)?
        .collect::<Result<Vec<_>, _>>()?;

    let duplicated = candidates.iter().any(|song| {
        let ids: HashSet<i64> = song.artists.iter().map(|a| a.id).collect();
        ids == wanted
    });

    if duplicated {
        return Err(AppError::Conflict(DUP.into()));
    }
    Ok(())
}

/// Inserta las filas de song_artist; la posición es el índice en la lista.
fn insert_song_artists(conn: &Connection, song_id: i64, artist_ids: &[i64]) -> AppResult<()> {
    let mut stmt =
        conn.prepare("INSERT INTO song_artist (song_id, artist_id, position) VALUES (?1, ?2, ?3)")?;
    for (position, artist_id) in artist_ids.iter().enumerate() {
        stmt.execute(params![song_id, artist_id, position as i64])?;
    }
    Ok(())
}

pub fn create(conn: &mut Connection, input: &SongInput) -> AppResult<SongView> {
    let input = parse_song(input)?;

    let tx = conn.transaction()?;
    let artist_ids = resolve_artists(&tx, &input.artists)?;
    ensure_not_duplicate(&tx, None, &input.title, &artist_ids)?;

    tx.execute(
        "INSERT INTO song (title, album, release_date, duration_ms, notes)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            input.title,
            input.album,
            input.release_date,
            input.duration_ms,
            input.notes,
        ],
    )?;
    let id = tx.last_insert_rowid();
    insert_song_artists(&tx, id, &artist_ids)?;

    tx.commit()?;
    get(conn, id)
}

pub fn list(conn: &Connection, include_archived: bool) -> AppResult<Vec<SongView>> {
    let sql = format!(
        "{SELECT_SONG}
         WHERE (?1 OR s.archived_at IS NULL)
         {SELECT_SONG_GROUP_BY}
         ORDER BY MIN(CASE WHEN sa.position = 0 THEN a.name END) COLLATE NOCASE, s.title COLLATE NOCASE"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([include_archived], map_row)?;
    let songs = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(songs)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<SongView> {
    let sql = format!("{SELECT_SONG} WHERE s.id = ?1 {SELECT_SONG_GROUP_BY}");
    conn.query_row(&sql, [id], map_row)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("No existe la canción {id}")))
}

pub fn update(conn: &mut Connection, id: i64, input: &SongInput) -> AppResult<SongView> {
    let input = parse_song(input)?;

    let tx = conn.transaction()?;
    let artist_ids = resolve_artists(&tx, &input.artists)?;

    let n = tx.execute(
        "UPDATE song
         SET title = ?1, album = ?2, release_date = ?3, duration_ms = ?4, notes = ?5
         WHERE id = ?6",
        params![
            input.title,
            input.album,
            input.release_date,
            input.duration_ms,
            input.notes,
            id
        ],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(format!("Canción {id} inexistente")));
    }

    ensure_not_duplicate(&tx, Some(id), &input.title, &artist_ids)?;

    tx.execute("DELETE FROM song_artist WHERE song_id = ?1", [id])?;
    insert_song_artists(&tx, id, &artist_ids)?;

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

    fn input(title: &str, artists: &[&str]) -> SongInput {
        SongInput {
            title: title.into(),
            artists: artists.iter().map(|a| a.to_string()).collect(),
            album: None,
            release_date: None,
            duration_ms: None,
            notes: None,
        }
    }

    fn names(song: &SongView) -> Vec<&str> {
        song.artists.iter().map(|a| a.name.as_str()).collect()
    }

    fn titles(songs: &[SongView]) -> Vec<&str> {
        songs.iter().map(|s| s.title.as_str()).collect()
    }

    /// Estado real de song_artist: (nombre, posición), ordenado por posición.
    fn artist_rows(conn: &Connection, song_id: i64) -> Vec<(String, i64)> {
        let mut stmt = conn
            .prepare(
                "SELECT a.name, sa.position
                 FROM song_artist sa
                 JOIN artist a ON a.id = sa.artist_id
                 WHERE sa.song_id = ?1
                 ORDER BY sa.position",
            )
            .unwrap();
        stmt.query_map([song_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
    }

    // ───────────── create ─────────────

    #[test]
    fn create_song_and_artist() {
        let mut conn = open_in_memory().unwrap();

        let s = create(&mut conn, &input("Song 2", &["Blur"])).unwrap();

        assert_eq!(s.title, "Song 2");
        assert_eq!(names(&s), vec!["Blur"]);
    }

    #[test]
    fn create_keeps_artist_order() {
        let mut conn = open_in_memory().unwrap();

        let s = create(
            &mut conn,
            &input("Feel Good Inc.", &["Gorillaz", "De La Soul"]),
        )
        .unwrap();

        assert_eq!(names(&s), vec!["Gorillaz", "De La Soul"]);
        assert_eq!(
            artist_rows(&conn, s.id),
            vec![("Gorillaz".to_string(), 0), ("De La Soul".to_string(), 1)]
        );
    }

    #[test]
    fn reuses_artist_case_insensitive() {
        let mut conn = open_in_memory().unwrap();

        let a = create(&mut conn, &input("Song 2", &["Blur"])).unwrap();
        let b = create(&mut conn, &input("Parklife", &["BLUR"])).unwrap();

        assert_eq!(a.artists[0].id, b.artists[0].id);
    }

    #[test]
    fn duplicated_artists_in_input_keep_first() {
        let mut conn = open_in_memory().unwrap();

        let s = create(
            &mut conn,
            &input("Song 2", &["Blur", "blur", "  BLUR ", "Gorillaz"]),
        )
        .unwrap();

        assert_eq!(names(&s), vec!["Blur", "Gorillaz"]);
        assert_eq!(artist_rows(&conn, s.id).len(), 2);
    }

    #[test]
    fn blank_artists_are_ignored() {
        let mut conn = open_in_memory().unwrap();

        let s = create(&mut conn, &input("Song 2", &["   ", "Blur", ""])).unwrap();

        assert_eq!(names(&s), vec!["Blur"]);
    }

    #[test]
    fn create_without_artists_is_validation_error() {
        let mut conn = open_in_memory().unwrap();

        let empty = create(&mut conn, &input("Song 2", &[])).unwrap_err();
        let blanks = create(&mut conn, &input("Song 2", &["  ", ""])).unwrap_err();

        assert!(matches!(empty, AppError::Validation(_)));
        assert!(matches!(blanks, AppError::Validation(_)));
        assert_eq!(list(&conn, true).unwrap().len(), 0);
    }

    #[test]
    fn duplicated_song_conflicts_and_cleans_up() {
        let mut conn = open_in_memory().unwrap();
        create(&mut conn, &input("Song 2", &["Blur"])).unwrap();

        let err = create(&mut conn, &input("song 2", &["Blur"])).unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));
        assert_eq!(list(&conn, true).unwrap().len(), 1);
    }

    #[test]
    fn same_artists_in_other_order_conflicts() {
        let mut conn = open_in_memory().unwrap();
        create(
            &mut conn,
            &input("Feel Good Inc.", &["Gorillaz", "De La Soul"]),
        )
        .unwrap();

        let err = create(
            &mut conn,
            &input("feel good inc.", &["De La Soul", "Gorillaz"]),
        )
        .unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn same_title_with_different_artists_is_ok() {
        let mut conn = open_in_memory().unwrap();
        create(&mut conn, &input("Song 2", &["Blur"])).unwrap();

        create(&mut conn, &input("Song 2", &["Blur", "Gorillaz"])).unwrap();

        assert_eq!(list(&conn, true).unwrap().len(), 2);
    }

    #[test]
    fn duplicate_of_archived_song_conflicts() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 2", &["Blur"])).unwrap();
        archive(&conn, s.id).unwrap();

        let err = create(&mut conn, &input("Song 2", &["Blur"])).unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));
    }

    // ───────────── get / list ─────────────

    #[test]
    fn get_missing_is_not_found() {
        let conn = open_in_memory().unwrap();

        let err = get(&conn, 999).unwrap_err();

        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn list_orders_by_main_artist_then_title() {
        let mut conn = open_in_memory().unwrap();
        // "b" tiene a Alfa como secundario: debe ordenarse por Zeta, no por Alfa.
        create(&mut conn, &input("b", &["Zeta", "Alfa"])).unwrap();
        create(&mut conn, &input("c", &["Alfa"])).unwrap();
        create(&mut conn, &input("a", &["Alfa"])).unwrap();

        let songs = list(&conn, false).unwrap();

        assert_eq!(titles(&songs), vec!["a", "c", "b"]);
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

    // ───────────── update ─────────────

    #[test]
    fn update_modifies_all() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 4", &["Nicky"])).unwrap();

        let s2 = update(
            &mut conn,
            s.id,
            &SongInput {
                album: Some("album".into()),
                release_date: Some("2026-01".into()),
                duration_ms: Some(42343),
                notes: Some("A".into()),
                ..input("  Song 5", &["Nicky"])
            },
        )
        .unwrap();

        assert_eq!(s2.title, "Song 5");
        assert_eq!(s2.album, Some("album".into()));
        assert_eq!(s2.release_date, Some("2026-01".into()));
        assert_eq!(s2.duration_ms, Some(42343));
        assert_eq!(s2.notes, Some("A".into()));
        assert_eq!(s2.artists[0].id, s.artists[0].id);
    }

    #[test]
    fn update_without_changes_does_not_conflict_with_itself() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 2", &["Blur"])).unwrap();

        let s2 = update(&mut conn, s.id, &input("Song 2", &["Blur"])).unwrap();

        assert_eq!(s2.title, "Song 2");
        assert_eq!(names(&s2), vec!["Blur"]);
    }

    #[test]
    fn update_new_artist() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 4", &["Nicky"])).unwrap();

        let s2 = update(&mut conn, s.id, &input("Song 4", &["Pedro"])).unwrap();

        assert_eq!(names(&s2), vec!["Pedro"]);
        assert_ne!(s2.artists[0].id, s.artists[0].id);
    }

    #[test]
    fn update_existing_artist() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 6", &["Pedro"])).unwrap();
        let s2 = create(&mut conn, &input("Song 4", &["Nicky"])).unwrap();

        let s3 = update(&mut conn, s2.id, &input("Song 4", &["PEDRO"])).unwrap();

        assert_eq!(s3.artists[0].id, s.artists[0].id);
    }

    #[test]
    fn update_replaces_artist_set() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 2", &["Blur", "Gorillaz"])).unwrap();

        update(&mut conn, s.id, &input("Song 2", &["Damon Albarn"])).unwrap();

        assert_eq!(
            artist_rows(&conn, s.id),
            vec![("Damon Albarn".to_string(), 0)]
        );
    }

    #[test]
    fn update_reorders_artists() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 2", &["Blur", "Gorillaz"])).unwrap();

        let s2 = update(&mut conn, s.id, &input("Song 2", &["Gorillaz", "Blur"])).unwrap();

        assert_eq!(names(&s2), vec!["Gorillaz", "Blur"]);
    }

    #[test]
    fn update_inexisting_song() {
        let mut conn = open_in_memory().unwrap();
        create(&mut conn, &input("Song 6", &["Pedro"])).unwrap();
        let len = artist::list(&conn, true).unwrap().len();

        let err = update(&mut conn, -1, &input("Song 6", &["ALANA"])).unwrap_err();

        assert!(matches!(err, AppError::NotFound(_)));
        assert_eq!(artist::list(&conn, true).unwrap().len(), len);
    }

    #[test]
    fn update_to_existing_song_conflicts_and_keeps_original() {
        let mut conn = open_in_memory().unwrap();
        create(&mut conn, &input("Song 6", &["Pedro"])).unwrap();
        let s = create(&mut conn, &input("Song 4", &["Nicky"])).unwrap();

        let err = update(&mut conn, s.id, &input("Song 6", &["PEDRO"])).unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));
        let s2 = get(&conn, s.id).unwrap();
        assert_eq!(s2.title, "Song 4");
        assert_eq!(names(&s2), vec!["Nicky"]);
    }

    #[test]
    fn update_modifies_album() {
        let mut conn = open_in_memory().unwrap();
        let s = create(
            &mut conn,
            &SongInput {
                album: Some("album1".into()),
                ..input("Song 6", &["Pedro"])
            },
        )
        .unwrap();

        let s2 = update(
            &mut conn,
            s.id,
            &SongInput {
                album: Some("album2".into()),
                ..input("Song 6", &["Pedro"])
            },
        )
        .unwrap();

        assert_eq!(s2.album, Some("album2".into()));
    }

    #[test]
    fn update_invalid_input() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 6", &["Pedro"])).unwrap();

        let err = update(
            &mut conn,
            s.id,
            &SongInput {
                release_date: Some("fsdf".into()),
                ..input("Song 6", &["Pedro"])
            },
        )
        .unwrap_err();

        assert!(matches!(err, AppError::Validation(_)));
    }

    // ───────────── archive ─────────────

    #[test]
    fn archive_once() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 6", &["Pedro"])).unwrap();

        archive(&conn, s.id).unwrap();

        assert_eq!(list(&conn, false).unwrap().len(), 0);
        assert_eq!(list(&conn, true).unwrap().len(), 1);
    }

    #[test]
    fn archive_twice() {
        let mut conn = open_in_memory().unwrap();
        let s = create(&mut conn, &input("Song 6", &["Pedro"])).unwrap();
        let archived_at = "2020-01-01";
        conn.execute(
            "UPDATE song SET archived_at = ?2 WHERE id = ?1",
            params![s.id, archived_at],
        )
        .unwrap();

        archive(&conn, s.id).unwrap();

        assert_eq!(
            get(&conn, s.id).unwrap().archived_at,
            Some(archived_at.into())
        );
    }

    #[test]
    fn archive_inexistent_song() {
        let conn = open_in_memory().unwrap();
        archive(&conn, -1).unwrap();
    }
}
