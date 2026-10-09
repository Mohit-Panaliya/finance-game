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

/// Loan-only detail for a debt. These are optional: a plain "I owe Ada 500" row
/// leaves every field at its zero/default and behaves exactly as before. The
/// limits are enforced in the request handlers, not as CHECK constraints here,
/// because the Turso engine used for storage refuses to open a database that
/// contains CHECK constraints.
///
/// `interest_rate` is an annual percentage, `emi_amount` the monthly instalment,
/// `tenure_months` how many instalments were agreed, and `lender` the
/// institution (as opposed to the person in `counterparty`).
const UP: &[&str] = &[
    "ALTER TABLE debts ADD COLUMN interest_rate REAL NOT NULL DEFAULT 0",
    "ALTER TABLE debts ADD COLUMN emi_amount REAL NOT NULL DEFAULT 0",
    "ALTER TABLE debts ADD COLUMN tenure_months INTEGER NOT NULL DEFAULT 0",
    "ALTER TABLE debts ADD COLUMN lender TEXT",
];

const DOWN: &[&str] = &[
    "ALTER TABLE debts DROP COLUMN lender",
    "ALTER TABLE debts DROP COLUMN tenure_months",
    "ALTER TABLE debts DROP COLUMN emi_amount",
    "ALTER TABLE debts DROP COLUMN interest_rate",
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
