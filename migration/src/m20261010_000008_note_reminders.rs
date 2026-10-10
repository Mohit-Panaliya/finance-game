use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn exec_all(c: &impl ConnectionTrait, stmts: &[&str]) -> Result<(), DbErr> {
    for stmt in stmts {
        c.execute_unprepared(stmt).await?;
    }
    Ok(())
}

// Reminder support for Keep parity. Value limits are enforced in Rust, not in
// SQL: turso_core rejects any CHECK constraint at DB-open time.
// - remind_at: RFC3339 timestamp or NULL (no reminder). Empty string clears.
// - repeat: one of none|daily|weekly|monthly (anything else is rejected in Rust).
const UP: &[&str] = &[
    "ALTER TABLE notes ADD COLUMN remind_at TEXT",
    "ALTER TABLE notes ADD COLUMN repeat TEXT NOT NULL DEFAULT 'none'",
    "CREATE INDEX idx_notes_owner_remind ON notes (owner_id, remind_at)",
];

const DOWN: &[&str] = &["DROP INDEX idx_notes_owner_remind"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_all(m.get_connection(), UP).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_all(m.get_connection(), DOWN).await
    }
}
