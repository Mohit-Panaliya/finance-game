use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// `incomes.bank_id` and `expenses.bank_id` already exist (added by the init
/// migration), so income and expense rows can already point at the account they
/// moved through. What is missing is the other side of a loan: who owes whom,
/// how much, and whether any of it has been repaid. That lives in this table.
///
/// `direction` is the single source of truth for the sign of a debt:
///   - `lent`      -> the counterparty owes the user money ("they owe me")
///   - `borrowed`  -> the user owes the counterparty money ("I owe them")
/// `amount` is the original principal, `settled_amount` how much has already
/// been repaid, so the outstanding balance is always `amount - settled_amount`
/// rather than a stored number that could drift out of sync.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                r#"CREATE TABLE IF NOT EXISTS debts (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                direction TEXT NOT NULL DEFAULT 'lent',
                counterparty TEXT NOT NULL,
                amount REAL NOT NULL DEFAULT 0,
                settled_amount REAL NOT NULL DEFAULT 0,
                currency TEXT NOT NULL DEFAULT 'INR',
                kind TEXT NOT NULL DEFAULT 'loan',
                account_id TEXT,
                occurred_date TEXT NOT NULL,
                due_date TEXT,
                note TEXT,
                settled_date TEXT,
                sync_status TEXT NOT NULL DEFAULT 'synced',
                last_synced_at TEXT,
                created_at TEXT,
                updated_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_debts_user ON debts(user_id);
            CREATE INDEX IF NOT EXISTS idx_debts_direction ON debts(user_id, direction);"#,
            )
            .await?;

        Ok(())
    }

    /// Drops the table and its two indexes. The `incomes`/`expenses` bank
    /// linkage is untouched by this migration and therefore survives a rollback.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_debts_direction;
                 DROP INDEX IF EXISTS idx_debts_user;",
            )
            .await?;
        drop_table(m, "debts").await?;
        Ok(())
    }
}
