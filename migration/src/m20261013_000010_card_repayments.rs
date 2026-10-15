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

// A card repayment is a TRANSFER (bank -> card), not an expense: the purchase it
// settles was already recorded when the card was charged. The old repay flow wrote
// an unlinked expenses row per repayment; those rows never moved balances (the
// ledger hooks only fire on linked rows) but polluted expense lists, summaries and
// analytics — double-counted spending. This creates the proper table and removes
// the legacy phantom rows. No CHECK constraints (turso_core rejects them).
const UP: &[&str] = &[
    "CREATE TABLE card_repayments (
        id TEXT PRIMARY KEY NOT NULL,
        user_id BIGINT NOT NULL,
        card_id TEXT NOT NULL,
        bank_id TEXT,
        amount REAL NOT NULL,
        paid_on TEXT NOT NULL,
        created_at TEXT NOT NULL
    )",
    "CREATE INDEX idx_card_repayments_user ON card_repayments (user_id)",
    "CREATE INDEX idx_card_repayments_card ON card_repayments (card_id)",
    "CREATE INDEX idx_card_repayments_bank ON card_repayments (bank_id)",
    "DELETE FROM expenses WHERE tags LIKE '%card-repayment:%'",
];

const DOWN: &[&str] = &["DROP TABLE card_repayments"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_all(m.get_connection(), UP).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_all(m.get_connection(), DOWN).await
    }
}
