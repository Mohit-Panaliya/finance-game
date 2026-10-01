#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20260922_000001_init;
mod m20260930_000002_drop_game_tables;
mod m20261001_000003_debts;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260922_000001_init::Migration),
            Box::new(m20260930_000002_drop_game_tables::Migration),
            Box::new(m20261001_000003_debts::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
