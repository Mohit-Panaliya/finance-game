use async_trait::async_trait;
use loco_rs::{app::AppContext, prelude::*};
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::MigratorTrait;
use sea_orm_turso::TursoConnection;

pub struct TursoDb;

#[async_trait]
impl Initializer for TursoDb {
    fn name(&self) -> String {
        "turso_db".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let db_url = &ctx.config.database.uri;
        let path = db_url
            .replace("sqlite://", "")
            .split('?')
            .next()
            .unwrap_or(db_url)
            .to_string();
        let turso_path = path.replace("_loco.sqlite", ".sqlite");

        std::env::set_var("LIMBO_DISABLE_FILE_LOCK", "1");

        let mut opts = ConnectOptions::new(format!("sqlite://{turso_path}?mode=rwc"));
        opts.max_connections(1).min_connections(1);
        let schema_db = Database::connect(opts)
            .await
            .map_err(|e| loco_rs::Error::string(&format!("Turso schema connect failed: {e}")))?;
        migration::Migrator::up(&schema_db, None)
            .await
            .map_err(|e| loco_rs::Error::string(&format!("Turso migrate failed: {e}")))?;
        schema_db.close().await.ok();

        let conn = TursoConnection::connect(&turso_path)
            .await
            .map_err(|e| loco_rs::Error::string(&format!("Turso init failed: {e}")))?;
        ctx.shared_store.insert(conn);
        tracing::info!("Turso database initialized from {turso_path}");
        Ok(())
    }
}
