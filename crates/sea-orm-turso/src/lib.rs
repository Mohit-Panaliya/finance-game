//! # sea-orm-turso
//!
//! A universal [SeaORM](https://docs.rs/sea-orm) adapter for [Turso Database](https://turso.tech).
//!
//! This crate provides a [`TursoConnection`] that implements SeaORM's [`ConnectionTrait`],
//! allowing you to use Turso as the database backend for any SeaORM-based application.
//!
//! ## Features
//!
//! - **MVCC**: Full multi-version concurrency control via Turso engine
//! - **In-process**: No network overhead, runs directly in your application
//! - **Async**: Native async/await support via Tokio
//! - **WAL persistence**: In-memory speed with file-based durability
//! - **Cloud sync**: Optional sync with Turso Cloud (via `turso::sync`)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use sea_orm_turso::TursoConnection;
//! use sea_orm::{Database, EntityTrait, QueryFilter};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to a local Turso database
//! let db = TursoConnection::connect("app.db").await?;
//!
//! // Use with SeaORM entities
//! // let users = user::Entity::find().all(&db).await?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Integration with Loco
//!
//! ```rust,ignore
//! use sea_orm_turso::TursoConnection;
//!
//! // In your app.rs after_context hook:
//! // let turso = TursoConnection::connect("app.db").await.unwrap();
//! // ctx.shared_store.insert(turso);
//! ```

mod connection;
mod error;
mod row;

pub use connection::TursoConnection;
pub use error::TursoError;

// Re-export turso types for convenience
pub use turso::{Builder, Database, Value as TursoValue};

/// Create a new Turso connection backed by a local file
///
/// This is the simplest way to create a Turso connection. The database
/// file will be created if it doesn't exist.
///
/// # Example
///
/// ```rust,no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use sea_orm_turso;
///
/// let db = sea_orm_turso::connect("my_database.db").await?;
/// # Ok(())
/// # }
/// ```
pub async fn connect(path: &str) -> Result<TursoConnection, TursoError> {
    TursoConnection::connect(path).await
}

/// Create a new in-memory Turso connection (no persistence)
///
/// Useful for testing or ephemeral data.
///
/// # Example
///
/// ```rust,no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use sea_orm_turso;
///
/// let db = sea_orm_turso::connect_memory().await?;
/// # Ok(())
/// # }
/// ```
pub async fn connect_memory() -> Result<TursoConnection, TursoError> {
    TursoConnection::connect_memory().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, DbBackend, Statement};

    #[tokio::test]
    async fn test_connect_memory() {
        let db = connect_memory().await.unwrap();

        // Test basic execute
        let result = db
            .execute_raw(Statement::from_string(
                DbBackend::Sqlite,
                "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, name TEXT)".to_string(),
            ))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connect_file() {
        let db = connect("test_turso.db").await.unwrap();

        // Test basic execute
        let result = db
            .execute_raw(Statement::from_string(
                DbBackend::Sqlite,
                "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, name TEXT)".to_string(),
            ))
            .await;

        assert!(result.is_ok());

        // Cleanup
        let _ = std::fs::remove_file("test_turso.db");
    }
}
