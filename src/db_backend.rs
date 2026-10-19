//! Finance-database connection factory.
//!
//! Server builds open the vendored Turso/libsql engine. Android builds open
//! the same SQLite file format via stock SQLite (sea-orm's
//! `DatabaseConnection`), because the libsql C core does not cross-compile
//! cleanly and buys nothing on-device — replication between devices is the
//! app-level sync protocol (`/api/sync`, `sync_log` changelog), not libsql
//! embedded replicas. Both engines write identical files, so databases move
//! freely between targets (backup export/import already relies on this).

#[cfg(target_os = "android")]
pub type FinanceDb = sea_orm::DatabaseConnection;

#[cfg(not(target_os = "android"))]
pub type FinanceDb = sea_orm_turso::TursoConnection;

/// Open (creating if needed) and migrate the finance database at `path`
/// (a plain filesystem path, no URL scheme).
pub async fn open_finance(path: &str, migrate: bool) -> Result<FinanceDb, loco_rs::Error> {
    let mut opts = sea_orm::ConnectOptions::new(format!("sqlite://{path}?mode=rwc"));
    opts.max_connections(1).min_connections(1);
    let db = sea_orm::Database::connect(opts)
        .await
        .map_err(|e| loco_rs::Error::string(&format!("Finance db connect failed: {e}")))?;
    if migrate {
        crate::migration::Migrator::up(&db, None)
            .await
            .map_err(|e| loco_rs::Error::string(&format!("Finance migrate failed: {e}")))?;
    }
    Ok(db)
}
