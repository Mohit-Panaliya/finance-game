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

const UP: &[&str] = &[
    // notes table
    "CREATE TABLE notes (
        id TEXT PRIMARY KEY NOT NULL,
        owner_id BIGINT NOT NULL,
        kind TEXT NOT NULL DEFAULT 'text',
        title TEXT NOT NULL DEFAULT '',
        body_text TEXT,
        color TEXT NOT NULL DEFAULT 'DEFAULT',
        pinned INTEGER NOT NULL DEFAULT 0,
        archived INTEGER NOT NULL DEFAULT 0,
        trashed_at TEXT,
        created_at TEXT,
        updated_at TEXT,
        version INTEGER NOT NULL DEFAULT 1
    )",
    "CREATE INDEX idx_notes_owner ON notes (owner_id)",
    "CREATE INDEX idx_notes_owner_trashed ON notes (owner_id, trashed_at)",
    // list_items
    "CREATE TABLE list_items (
        id TEXT PRIMARY KEY NOT NULL,
        note_id TEXT NOT NULL,
        parent_id TEXT,
        position INTEGER NOT NULL DEFAULT 0,
        text TEXT NOT NULL DEFAULT '',
        checked INTEGER NOT NULL DEFAULT 0
    )",
    "CREATE INDEX idx_list_items_note ON list_items (note_id, parent_id, position)",
    // labels
    "CREATE TABLE labels (
        id TEXT PRIMARY KEY NOT NULL,
        owner_id BIGINT NOT NULL,
        name TEXT NOT NULL,
        name_ci TEXT NOT NULL
    )",
    "CREATE UNIQUE INDEX idx_labels_owner_name ON labels (owner_id, name_ci)",
    "CREATE INDEX idx_labels_owner ON labels (owner_id)",
    // note_labels
    "CREATE TABLE note_labels (
        note_id TEXT NOT NULL,
        label_id TEXT NOT NULL,
        PRIMARY KEY (note_id, label_id)
    )",
    "CREATE INDEX idx_note_labels_label ON note_labels (label_id)",
];

const DOWN: &[&str] = &[
    "DROP INDEX IF EXISTS idx_note_labels_label",
    "DROP TABLE IF EXISTS note_labels",
    "DROP INDEX IF EXISTS idx_labels_owner",
    "DROP INDEX IF EXISTS idx_labels_owner_name",
    "DROP TABLE IF EXISTS labels",
    "DROP INDEX IF EXISTS idx_list_items_note",
    "DROP TABLE IF EXISTS list_items",
    "DROP INDEX IF EXISTS idx_notes_owner_trashed",
    "DROP INDEX IF EXISTS idx_notes_owner",
    "DROP TABLE IF EXISTS notes",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_all(m.get_connection(), UP).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_all(m.get_connection(), DOWN).await
    }
}
