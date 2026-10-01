use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Tables that held game state and are removed by this migration, children
/// before parents so no dangling references are left behind.
const GAME_TABLES: [&str; 6] = [
    "user_achievements",
    "achievements",
    "battles",
    "troops",
    "buildings",
    "villages",
];

const GAME_INDEXES: [&str; 4] = [
    "idx_buildings_village",
    "idx_troops_user",
    "idx_battles_user",
    "idx_user_achievements_user",
];

/// Dead FK columns left over from the game layer: always NULL, never indexed.
const DEAD_COLUMNS: [(&str, &str); 7] = [
    ("banks", "game_building_id"),
    ("assets", "game_building_id"),
    ("credit_cards", "game_building_id"),
    ("fixed_deposits", "game_building_id"),
    ("investments", "game_building_id"),
    ("incomes", "game_building_id"),
    ("expenses", "game_troop_id"),
];

/// True when the error only means "the object was not there", which is the one
/// failure mode we are willing to swallow while cleaning up game leftovers.
fn is_missing_object(err: &DbErr) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("no such table")
        || msg.contains("no such index")
        || msg.contains("no such column")
        || msg.contains("does not exist")
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // ---- indexes first: they are the only dependents of the tables ----
        for idx in GAME_INDEXES {
            m.get_connection()
                .execute_unprepared(&format!("DROP INDEX IF EXISTS {idx};"))
                .await?;
        }

        // ---- game tables, children before parents ----
        for t in GAME_TABLES {
            drop_table(m, t).await?;
        }

        // ---- dead game FK columns on the finance tables ----
        // The init migration is already applied in production and must not be
        // edited, so the columns are dropped here. Every one of them is always
        // NULL and is not indexed, so losing the drop is harmless; failing a
        // production boot because a column was already gone is not. Errors
        // that mean "object missing" are therefore ignored, while every other
        // error class (locked database, corrupt schema, driver failure) is
        // propagated so real problems still surface.
        for (table, column) in DEAD_COLUMNS {
            let stmt = format!("ALTER TABLE {table} DROP COLUMN {column};");
            if let Err(e) = m.get_connection().execute_unprepared(&stmt).await {
                if !is_missing_object(&e) {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Recreates the 6 dropped game tables and their 4 indexes so a rollback
    /// restores the previous schema. The dead `game_building_id` /
    /// `game_troop_id` columns on the finance tables are intentionally NOT
    /// re-added: they are unused and nullable, and no code reads them any more.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                r#"CREATE TABLE IF NOT EXISTS villages (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                name TEXT NOT NULL DEFAULT 'My Fortress',
                level BIGINT NOT NULL DEFAULT 1,
                xp BIGINT NOT NULL DEFAULT 0,
                xp_to_next BIGINT NOT NULL DEFAULT 100,
                gold REAL NOT NULL DEFAULT 500,
                gems REAL NOT NULL DEFAULT 20,
                elixir REAL NOT NULL DEFAULT 0,
                trophy BIGINT NOT NULL DEFAULT 0,
                shield_until TEXT,
                last_tick_at TEXT,
                created_at TEXT,
                updated_at TEXT
            )"#,
            )
            .await?;

        m.get_connection()
            .execute_unprepared(
                r#"CREATE TABLE IF NOT EXISTS buildings (
                id TEXT PRIMARY KEY,
                village_id TEXT NOT NULL,
                user_id BIGINT NOT NULL,
                building_type TEXT NOT NULL,
                source_kind TEXT NOT NULL,
                source_id TEXT,
                name TEXT NOT NULL,
                level BIGINT NOT NULL DEFAULT 1,
                grid_x BIGINT NOT NULL DEFAULT 0,
                grid_y BIGINT NOT NULL DEFAULT 0,
                hp BIGINT NOT NULL DEFAULT 100,
                max_hp BIGINT NOT NULL DEFAULT 100,
                production_rate REAL NOT NULL DEFAULT 0,
                stored_resource REAL NOT NULL DEFAULT 0,
                last_collected_at TEXT,
                upgrade_cost REAL NOT NULL DEFAULT 100,
                is_upgrading INTEGER NOT NULL DEFAULT 0,
                upgrade_finishes_at TEXT,
                created_at TEXT,
                updated_at TEXT
            )"#,
            )
            .await?;

        m.get_connection()
            .execute_unprepared(
                r#"CREATE TABLE IF NOT EXISTS troops (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                troop_type TEXT NOT NULL,
                name TEXT NOT NULL,
                level BIGINT NOT NULL DEFAULT 1,
                attack BIGINT NOT NULL DEFAULT 10,
                hp BIGINT NOT NULL DEFAULT 20,
                count BIGINT NOT NULL DEFAULT 0,
                train_cost REAL NOT NULL DEFAULT 10,
                train_time_secs BIGINT NOT NULL DEFAULT 5,
                training_finishes_at TEXT,
                expense_id TEXT,
                created_at TEXT,
                updated_at TEXT
            )"#,
            )
            .await?;

        m.get_connection()
            .execute_unprepared(
                r#"CREATE TABLE IF NOT EXISTS achievements (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                icon TEXT NOT NULL DEFAULT 'star',
                xp_reward BIGINT NOT NULL DEFAULT 50,
                gem_reward REAL NOT NULL DEFAULT 5,
                requirement_kind TEXT NOT NULL,
                requirement_value BIGINT NOT NULL,
                tier TEXT NOT NULL DEFAULT 'bronze',
                sort_order BIGINT NOT NULL DEFAULT 0
            )"#,
            )
            .await?;

        m.get_connection()
            .execute_unprepared(
                r#"CREATE TABLE IF NOT EXISTS user_achievements (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                achievement_id TEXT NOT NULL,
                unlocked_at TEXT,
                progress BIGINT NOT NULL DEFAULT 0,
                is_claimed INTEGER NOT NULL DEFAULT 0
            )"#,
            )
            .await?;

        m.get_connection()
            .execute_unprepared(
                r#"CREATE TABLE IF NOT EXISTS battles (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                battle_type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'ongoing',
                stars BIGINT NOT NULL DEFAULT 0,
                destruction_percent REAL NOT NULL DEFAULT 0,
                gold_reward REAL NOT NULL DEFAULT 0,
                gem_reward REAL NOT NULL DEFAULT 0,
                xp_reward BIGINT NOT NULL DEFAULT 0,
                trophy_change BIGINT NOT NULL DEFAULT 0,
                enemy_troops TEXT,
                enemy_buildings TEXT,
                created_at TEXT,
                updated_at TEXT
            )"#,
            )
            .await?;

        m.get_connection()
            .execute_unprepared(
                r#"CREATE INDEX IF NOT EXISTS idx_buildings_village ON buildings(village_id);
               CREATE INDEX IF NOT EXISTS idx_troops_user ON troops(user_id);
               CREATE INDEX IF NOT EXISTS idx_battles_user ON battles(user_id);
               CREATE INDEX IF NOT EXISTS idx_user_achievements_user ON user_achievements(user_id);"#,
            )
            .await?;

        Ok(())
    }
}
