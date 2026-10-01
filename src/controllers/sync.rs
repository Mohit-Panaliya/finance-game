use loco_rs::prelude::*;
use serde::Deserialize;

use crate::sync_engine::{apply_push, pull_changes, SyncOp};

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
    // `since` may arrive as an RFC3339 string or as a numeric epoch-ms value
    let since = params
        .get("since")
        .and_then(|v| v.as_str().map(str::to_string).or_else(|| v.as_i64().map(|n| n.to_string())));
    let payload = pull_changes(&db, super::uid(&ctx, &auth).await?, since.as_deref())
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(payload)
}
