use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // ---- users (Loco-compatible) ----
        create_table(
            m,
            "users",
            &[
                ("id", ColType::PkAuto),
                ("pid", ColType::Uuid),
                ("email", ColType::StringUniq),
                ("password", ColType::String),
                ("api_key", ColType::StringUniq),
                ("name", ColType::String),
                ("reset_token", ColType::StringNull),
                ("reset_sent_at", ColType::TimestampWithTimeZoneNull),
                ("email_verification_token", ColType::StringNull),
                (
                    "email_verification_sent_at",
                    ColType::TimestampWithTimeZoneNull,
                ),
                ("email_verified_at", ColType::TimestampWithTimeZoneNull),
                ("magic_link_token", ColType::StringNull),
                ("magic_link_expiration", ColType::TimestampWithTimeZoneNull),
            ],
            &[],
        )
        .await?;

        // ---- banks ----
        m.get_connection().execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS banks (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                name TEXT NOT NULL,
                bank_type TEXT NOT NULL,
                account_number TEXT NOT NULL,
                ifsc_code TEXT,
                branch TEXT,
                current_balance REAL NOT NULL DEFAULT 0,
                currency TEXT NOT NULL DEFAULT 'INR',
                is_active INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'synced',
                last_synced_at TEXT,
                created_at TEXT,
                updated_at TEXT,
                game_building_id TEXT
            )"#,
        )
        .await?;

        // ---- assets ----
        m.get_connection().execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS assets (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                name TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                category TEXT NOT NULL,
                purchase_price REAL NOT NULL DEFAULT 0,
                current_value REAL NOT NULL DEFAULT 0,
                purchase_date TEXT NOT NULL,
                location TEXT,
                description TEXT,
                documents TEXT,
                roi_percentage REAL,
                annual_income REAL NOT NULL DEFAULT 0,
                depreciation_rate REAL,
                is_liquid INTEGER NOT NULL DEFAULT 0,
                risk_level TEXT NOT NULL DEFAULT 'moderate',
                sync_status TEXT NOT NULL DEFAULT 'synced',
                last_synced_at TEXT,
                created_at TEXT,
                updated_at TEXT,
                game_building_id TEXT
            )"#,
        )
        .await?;

        // ---- expenses ----
        m.get_connection().execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS expenses (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                amount REAL NOT NULL,
                currency TEXT NOT NULL DEFAULT 'INR',
                expense_type TEXT NOT NULL,
                category TEXT NOT NULL,
                expense_date TEXT NOT NULL,
                is_recurring INTEGER NOT NULL DEFAULT 0,
                recurrence TEXT,
                recurrence_end_date TEXT,
                payment_method TEXT NOT NULL DEFAULT 'cash',
                bank_id TEXT,
                credit_card_id TEXT,
                tags TEXT,
                receipt_url TEXT,
                location TEXT,
                is_fixed INTEGER NOT NULL DEFAULT 0,
                priority TEXT NOT NULL DEFAULT 'medium',
                sync_status TEXT NOT NULL DEFAULT 'synced',
                last_synced_at TEXT,
                created_at TEXT,
                updated_at TEXT,
                game_troop_id TEXT
            )"#,
        )
        .await?;

        // ---- credit_cards ----
        m.get_connection().execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS credit_cards (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                name TEXT NOT NULL,
                bank_name TEXT NOT NULL,
                card_type TEXT NOT NULL,
                last_four_digits TEXT NOT NULL,
                credit_limit REAL NOT NULL DEFAULT 0,
                current_balance REAL NOT NULL DEFAULT 0,
                available_credit REAL NOT NULL DEFAULT 0,
                interest_rate REAL NOT NULL DEFAULT 0,
                billing_cycle_day BIGINT NOT NULL DEFAULT 1,
                due_date_day BIGINT NOT NULL DEFAULT 5,
                annual_fee REAL NOT NULL DEFAULT 0,
                reward_program TEXT,
                reward_points BIGINT NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'synced',
                last_synced_at TEXT,
                created_at TEXT,
                updated_at TEXT,
                game_building_id TEXT
            )"#,
        )
        .await?;

        // ---- fixed_deposits ----
        m.get_connection().execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS fixed_deposits (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                bank_id TEXT,
                name TEXT NOT NULL,
                fd_type TEXT NOT NULL DEFAULT 'regular',
                principal_amount REAL NOT NULL,
                interest_rate REAL NOT NULL,
                tenure_months BIGINT NOT NULL,
                start_date TEXT NOT NULL,
                maturity_date TEXT NOT NULL,
                compounding_frequency TEXT NOT NULL DEFAULT 'quarterly',
                current_value REAL NOT NULL DEFAULT 0,
                interest_earned REAL NOT NULL DEFAULT 0,
                tax_deducted REAL NOT NULL DEFAULT 0,
                is_auto_renew INTEGER NOT NULL DEFAULT 0,
                renewal_instructions TEXT,
                nominee TEXT,
                certificate_number TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                sync_status TEXT NOT NULL DEFAULT 'synced',
                last_synced_at TEXT,
                created_at TEXT,
                updated_at TEXT,
                game_building_id TEXT
            )"#,
        )
        .await?;

        // ---- investments ----
        m.get_connection().execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS investments (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                name TEXT NOT NULL,
                investment_type TEXT NOT NULL,
                instrument TEXT NOT NULL,
                symbol TEXT,
                invested_amount REAL NOT NULL,
                current_value REAL NOT NULL DEFAULT 0,
                units REAL,
                unit_price REAL,
                purchase_date TEXT NOT NULL,
                purchase_price REAL,
                broker_platform TEXT,
                account_ref TEXT,
                expected_return REAL,
                actual_return REAL,
                annualized_return REAL,
                dividend_yield REAL,
                risk_level TEXT NOT NULL DEFAULT 'moderate',
                is_liquid INTEGER NOT NULL DEFAULT 1,
                lock_in_until TEXT,
                tax_saving INTEGER NOT NULL DEFAULT 0,
                tags TEXT,
                notes TEXT,
                sync_status TEXT NOT NULL DEFAULT 'synced',
                last_synced_at TEXT,
                created_at TEXT,
                updated_at TEXT,
                game_building_id TEXT
            )"#,
        )
        .await?;

        // ---- incomes ----
        m.get_connection().execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS incomes (
                id TEXT PRIMARY KEY,
                user_id BIGINT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                amount REAL NOT NULL,
                currency TEXT NOT NULL DEFAULT 'INR',
                income_type TEXT NOT NULL,
                source TEXT NOT NULL,
                income_date TEXT NOT NULL,
                is_recurring INTEGER NOT NULL DEFAULT 0,
                recurrence TEXT,
                frequency_multiplier BIGINT NOT NULL DEFAULT 1,
                is_gross INTEGER NOT NULL DEFAULT 1,
                tax_withheld REAL NOT NULL DEFAULT 0,
                bank_id TEXT,
                tags TEXT,
                notes TEXT,
                sync_status TEXT NOT NULL DEFAULT 'synced',
                last_synced_at TEXT,
                created_at TEXT,
                updated_at TEXT,
                game_building_id TEXT
            )"#,
        )
        .await?;

        // ---- villages (game) ----
        m.get_connection().execute_unprepared(
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

        // ---- buildings (banks/assets/fds/investments as buildings) ----
        m.get_connection().execute_unprepared(
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

        // ---- troops (expenses = raid troops; savings = defenders) ----
        m.get_connection().execute_unprepared(
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

        // ---- achievements ----
        m.get_connection().execute_unprepared(
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

        m.get_connection().execute_unprepared(
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

        // ---- battles (income vs expense raids) ----
        m.get_connection().execute_unprepared(
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

        // ---- sync_log (offline delta sync) ----
        m.get_connection().execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS sync_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id BIGINT NOT NULL,
                entity TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                op TEXT NOT NULL,
                payload TEXT,
                client_ts TEXT,
                server_ts TEXT,
                status TEXT NOT NULL DEFAULT 'applied'
            )"#,
        )
        .await?;

        m.get_connection().execute_unprepared(
            r#"CREATE INDEX IF NOT EXISTS idx_banks_user ON banks(user_id);
               CREATE INDEX IF NOT EXISTS idx_assets_user ON assets(user_id);
               CREATE INDEX IF NOT EXISTS idx_expenses_user ON expenses(user_id);
               CREATE INDEX IF NOT EXISTS idx_cards_user ON credit_cards(user_id);
               CREATE INDEX IF NOT EXISTS idx_fds_user ON fixed_deposits(user_id);
               CREATE INDEX IF NOT EXISTS idx_investments_user ON investments(user_id);
               CREATE INDEX IF NOT EXISTS idx_incomes_user ON incomes(user_id);
               CREATE INDEX IF NOT EXISTS idx_buildings_village ON buildings(village_id);
               CREATE INDEX IF NOT EXISTS idx_troops_user ON troops(user_id);
               CREATE INDEX IF NOT EXISTS idx_battles_user ON battles(user_id);
               CREATE INDEX IF NOT EXISTS idx_sync_log_user ON sync_log(user_id, server_ts);
               CREATE INDEX IF NOT EXISTS idx_user_achievements_user ON user_achievements(user_id);"#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for t in [
            "sync_log",
            "user_achievements",
            "achievements",
            "battles",
            "troops",
            "buildings",
            "villages",
            "incomes",
            "investments",
            "fixed_deposits",
            "credit_cards",
            "expenses",
            "assets",
            "banks",
            "users",
        ] {
            drop_table(m, t).await?;
        }
        Ok(())
    }
}
