use loco_rs::prelude::*;
use serde::Deserialize;

use crate::game_engine::sync::{apply_push, pull_changes, SyncOp};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/sync")
        .add("/push", post(push))
        .add("/pull", get(pull))
}


#[derive(Deserialize)]
struct PushRequest {
    ops: Vec<SyncOp>,
}

#[debug_handler]
async fn push(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(body): Json<PushRequest>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .ok_or_else(|| Error::string("database unavailable"))?;
    let results = apply_push(&db, super::uid(&ctx, &auth).await?, &body.ops)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(serde_json::json!({ "results": results, "ok": true }))
}

#[debug_handler]
async fn pull(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<serde_json::Value>,
) -> Result<Response> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .ok_or_else(|| Error::string("database unavailable"))?;
    let since = params.get("since").and_then(|v| v.as_str());
    let payload = pull_changes(&db, super::uid(&ctx, &auth).await?, since)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(payload)
}
