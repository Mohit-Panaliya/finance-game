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

// What was actually paid at conversion time (NULL until purchased).
// Limits enforced in Rust; no CHECK constraints (turso_core rejects them).
const UP: &[&str] = &["ALTER TABLE asset_buy_list ADD COLUMN actual_cost REAL"];

const DOWN: &[&str] = &["ALTER TABLE asset_buy_list DROP COLUMN actual_cost"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_all(m.get_connection(), UP).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_all(m.get_connection(), DOWN).await
    }
}
