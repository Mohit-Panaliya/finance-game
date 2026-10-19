use async_trait::async_trait;
use loco_rs::{app::AppContext, prelude::*};

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

        // Loco (auth) and the finance engine are separate databases. If both
        // point at the same file they hold independent WALs and clobber each
        // other: finance rows survive a restart while newly created users
        // vanish.
        if turso_path == path {
            return Err(loco_rs::Error::string(
                "DATABASE_URL must contain the `_loco` suffix so auth and finance use separate database files; both resolved to the same path",
            ));
        }

        std::env::set_var("LIMBO_DISABLE_FILE_LOCK", "1");

        // Schema via sea-orm (sqlx), runtime connection via the backend
        // factory: Turso on servers, stock SQLite on Android.
        let schema_db = crate::db_backend::open_finance(&turso_path, true).await?;
        schema_db.close().await.ok();

        #[cfg(target_os = "android")]
        {
            let conn = crate::db_backend::open_finance(&turso_path, false).await?;
            ctx.shared_store.insert(conn);
        }
        #[cfg(not(target_os = "android"))]
        {
            let conn = crate::db_backend::FinanceDb::connect(&turso_path)
                .await
                .map_err(|e| loco_rs::Error::string(&format!("Turso init failed: {e}")))?;
            ctx.shared_store.insert(conn);
        }
        tracing::info!("Finance database initialized from {turso_path}");
        Ok(())
    }
}
