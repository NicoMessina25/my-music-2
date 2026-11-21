use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::error::{AppError, AppResult, unique_to_conflict};
use crate::model::song::{SongInput, SongView};
use crate::repo::artist;

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
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
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

pub fn create(conn: &mut Connection, input: &SongInput) -> AppResult<i64> {
    // 1) Validar ANTES de abrir la transacción
    let title = input.title.trim();
    if title.is_empty() {
        return Err(AppError::Validation("El título es obligatorio".into()));
    }
    let release_date = clean(&input.release_date);
    if let Some(d) = &release_date {
        validate_release_date(d)?;
    }
    if let Some(ms) = input.duration_ms {
        if ms <= 0 {
            return Err(AppError::Validation("La duración debe ser positiva".into()));
        }
    }

    // 2) Transacción: artista + canción, todo o nada
    let tx = conn.transaction()?;
    let artist_id = artist::get_or_create(&tx, &input.artist_name)?;

    tx.execute(
        "INSERT INTO song (title, artist_id, album, release_date, duration_ms, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            title,
            artist_id,
            clean(&input.album),
            release_date,
            input.duration_ms,
            clean(&input.notes),
        ],
    )
    .map_err(|e| unique_to_conflict(e, "Esa canción ya existe para ese artista"))?;

    let id = tx.last_insert_rowid();
    tx.commit()?;
    Ok(id)
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

    #[test]
    fn crea_cancion_y_artista() {
        let mut conn = open_in_memory().unwrap();
        let id = create(&mut conn, &input("Song 2", "Blur")).unwrap();
        let s = get(&conn, id).unwrap();
        assert_eq!(s.title, "Song 2");
        assert_eq!(s.artist_name, "Blur");
    }

    #[test]
    fn reutiliza_artista_sin_importar_mayusculas() {
        let mut conn = open_in_memory().unwrap();
        let a = create(&mut conn, &input("Song 2", "Blur")).unwrap();
        let b = create(&mut conn, &input("Parklife", "BLUR")).unwrap();
        assert_eq!(
            get(&conn, a).unwrap().artist_id,
            get(&conn, b).unwrap().artist_id
        );
    }

    #[test]
    fn duplicada_da_conflicto_y_no_deja_basura() {
        let mut conn = open_in_memory().unwrap();
        create(&mut conn, &input("Song 2", "Blur")).unwrap();
        let err = create(&mut conn, &input("song 2", "Blur")).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
        assert_eq!(list(&conn, true).unwrap().len(), 1);
    }

    #[test]
    fn valida_fechas() {
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
}
