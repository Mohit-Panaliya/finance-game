use async_trait::async_trait;
use loco_rs::{app::AppContext, prelude::*};

pub struct TursoDb;

#[async_trait]
impl Initializer for TursoDb {
    fn name(&self) -> String {
        "turso_db".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        #[cfg(target_os = "android")]
        {
            return Ok(());
        }

        #[cfg(not(target_os = "android"))]
        {
            let db_url = &ctx.config.database.uri;
            let path = db_url
                .replace("sqlite://", "")
                .split('?')
                .next()
                .unwrap_or(db_url)
                .to_string();
            let turso_path = path.replace("_loco.sqlite", ".sqlite");

            if turso_path == path {
                return Err(loco_rs::Error::string(
                    "DATABASE_URL must contain the `_loco` suffix so auth and finance use separate database files; both resolved to the same path",
                ));
            }

            std::env::set_var("LIMBO_DISABLE_FILE_LOCK", "1");

            crate::db_backend::open_finance(&turso_path, true).await?;

            let conn = crate::db_backend::open_finance(&turso_path, false).await?;
            ctx.shared_store.insert(conn);

            tracing::info!("Finance database initialized from {turso_path}");
            return Ok(());
        }
    }
}
