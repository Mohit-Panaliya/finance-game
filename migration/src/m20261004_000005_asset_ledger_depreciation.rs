use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Gives an asset and an investment a *source account*, so buying one leaves the bank
/// it was paid from instead of appearing out of nowhere. `fixed_deposits` already had
/// `bank_id` but nothing ever moved the principal.
///
/// Also replaces the single `assets.depreciation_rate` float with enough detail to
/// actually depreciate: a method, a useful life, a salvage value and a start date, plus
/// a `depreciation_entries` schedule generated from them.
///
/// And adds `asset_buy_list` so intended purchases can be tracked as intent — priced,
/// prioritised and dated — without touching a real balance until they are actually bought.
///
/// One `execute_unprepared` per statement: the Turso driver splits batches itself.
/// Generic over the connection because `SchemaManager::get_connection` hands back a
/// `SchemaManagerConnection` enum, not a bare `DatabaseConnection`.
async fn exec_all(c: &impl ConnectionTrait, stmts: &[&str]) -> Result<(), DbErr> {
    for stmt in stmts {
        c.execute_unprepared(stmt).await?;
    }
    Ok(())
}

const UP: &[&str] = &[
    // ── which account the money came from ─────────────────────────────────────
    "ALTER TABLE assets ADD COLUMN bank_id TEXT",
    "ALTER TABLE investments ADD COLUMN bank_id TEXT",
    "CREATE INDEX IF NOT EXISTS idx_assets_bank ON assets (bank_id)",
    "CREATE INDEX IF NOT EXISTS idx_investments_bank ON investments (bank_id)",
    // ── depreciation detail ───────────────────────────────────────────────────
    // `straight_line` matches the old single-rate behaviour, so existing rows keep
    // depreciating the way they did.
    "ALTER TABLE assets ADD COLUMN depreciation_method TEXT NOT NULL DEFAULT 'straight_line'",
    "ALTER TABLE assets ADD COLUMN useful_life_months INTEGER",
    "ALTER TABLE assets ADD COLUMN salvage_value REAL NOT NULL DEFAULT 0",
    "ALTER TABLE assets ADD COLUMN depreciation_start_date TEXT",
    // ── the generated schedule ───────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS depreciation_entries (
        id TEXT PRIMARY KEY NOT NULL,
        asset_id TEXT NOT NULL,
        user_id BIGINT NOT NULL,
        period_index INTEGER NOT NULL,
        period_start TEXT NOT NULL,
        period_end TEXT NOT NULL,
        opening_book_value REAL NOT NULL,
        depreciation_amount REAL NOT NULL,
        closing_book_value REAL NOT NULL,
        accumulated_depreciation REAL NOT NULL,
        method TEXT NOT NULL,
        is_current INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS idx_depreciation_asset ON depreciation_entries (asset_id, period_index)",
    // ── intended purchases ───────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS asset_buy_list (
        id TEXT PRIMARY KEY NOT NULL,
        user_id BIGINT NOT NULL,
        title TEXT NOT NULL,
        description TEXT,
        asset_type TEXT NOT NULL DEFAULT 'Other',
        estimated_cost REAL NOT NULL DEFAULT 0,
        priority TEXT NOT NULL DEFAULT 'medium',
        target_date TEXT,
        url TEXT,
        shop TEXT,
        status TEXT NOT NULL DEFAULT 'planned',
        bank_id TEXT,
        converted_asset_id TEXT,
        converted_at TEXT,
        sync_status TEXT NOT NULL DEFAULT 'synced',
        last_synced_at TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT
    )",
    "CREATE INDEX IF NOT EXISTS idx_buy_list_user ON asset_buy_list (user_id, status)",
];

const DOWN: &[&str] = &[
    "DROP INDEX IF EXISTS idx_buy_list_user",
    "DROP TABLE IF EXISTS asset_buy_list",
    "DROP INDEX IF EXISTS idx_depreciation_asset",
    "DROP TABLE IF EXISTS depreciation_entries",
    "DROP INDEX IF EXISTS idx_investments_bank",
    "DROP INDEX IF EXISTS idx_assets_bank",
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
