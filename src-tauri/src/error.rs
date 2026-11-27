use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    NotFound(String),
    #[error("error interno: lock de la base inválido")]
    Lock,
}

#[derive(Serialize)]
struct ApiAppError {
    pub kind: &'static str,
    pub message: String,
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let kind = match self {
            AppError::Db(_) => "db",
            AppError::Validation(_) => "validation",
            AppError::Conflict(_) => "conflict",
            AppError::NotFound(_) => "notFound",
            AppError::Lock => "lock",
        };

        ApiAppError {
            kind,
            message: self.to_string(),
        }
        .serialize(s)
    }
}

pub type AppResult<T> = Result<T, AppError>;

/// Traduce una violación de UNIQUE de SQLite a un Conflict con mensaje propio.
pub fn unique_to_conflict(e: rusqlite::Error, msg: &str) -> AppError {
    match &e {
        rusqlite::Error::SqliteFailure(err, _)
            if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
        {
            AppError::Conflict(msg.into())
        }
        _ => AppError::Db(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use serde_json::json;

    // ───────────── serialización hacia el front ─────────────

    #[test]
    fn serializes_validation() {
        let value = serde_json::to_value(AppError::Validation("Título vacío".into())).unwrap();
        assert_eq!(
            value,
            json!({ "kind": "validation", "message": "Título vacío" })
        );
    }

    #[test]
    fn serializes_conflict() {
        let value = serde_json::to_value(AppError::Conflict("Ya existe".into())).unwrap();
        assert_eq!(value, json!({ "kind": "conflict", "message": "Ya existe" }));
    }

    #[test]
    fn serializes_not_found() {
        let value = serde_json::to_value(AppError::NotFound("No existe".into())).unwrap();
        assert_eq!(value, json!({ "kind": "notFound", "message": "No existe" }));
    }

    #[test]
    fn serializes_lock_with_its_fixed_message() {
        let value = serde_json::to_value(AppError::Lock).unwrap();
        assert_eq!(
            value,
            json!({ "kind": "lock", "message": "error interno: lock de la base inválido" })
        );
    }

    #[test]
    fn serializes_db_with_rusqlite_message() {
        let err = AppError::Db(rusqlite::Error::QueryReturnedNoRows);
        // El mensaje lo arma rusqlite: se compara contra su Display en vez de
        // un texto literal, para no depender de la redacción de la librería.
        let expected = rusqlite::Error::QueryReturnedNoRows.to_string();

        let value = serde_json::to_value(err).unwrap();

        assert_eq!(value, json!({ "kind": "db", "message": expected }));
    }

    // ───────────── unique_to_conflict ─────────────

    fn conn_with_unique_table() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE t (x TEXT NOT NULL UNIQUE)", [])
            .unwrap();
        conn.execute("INSERT INTO t (x) VALUES ('a')", []).unwrap();
        conn
    }

    #[test]
    fn unique_violation_becomes_conflict_with_given_message() {
        let conn = conn_with_unique_table();
        let err = conn
            .execute("INSERT INTO t (x) VALUES ('a')", [])
            .unwrap_err();

        let AppError::Conflict(msg) = unique_to_conflict(err, "Duplicado") else {
            panic!("se esperaba AppError::Conflict");
        };
        assert_eq!(msg, "Duplicado");
    }

    #[test]
    fn other_constraint_violation_stays_db_error() {
        let conn = conn_with_unique_table();
        let err = conn
            .execute("INSERT INTO t (x) VALUES (NULL)", [])
            .unwrap_err();

        assert!(matches!(
            unique_to_conflict(err, "Duplicado"),
            AppError::Db(_)
        ));
    }
}
