use rusqlite_migration::{M, Migrations};

// Nuevas migraciones SIEMPRE se agregan al final; nunca editar una ya aplicada.
const MIGRATION_ARRAY: &[M] = &[M::up(include_str!("../../migrations/0001_init.sql"))];

pub const MIGRATIONS: Migrations = Migrations::from_slice(MIGRATION_ARRAY);

#[cfg(test)]
mod tests {
    #[test]
    fn migrations_are_valid() {
        super::MIGRATIONS.validate().unwrap();
    }
}
