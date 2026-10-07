use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};

use crate::models::notes::{
    self, CreateLabelRequest, CreateListItemRequest, CreateNoteRequest, UpdateLabelRequest,
    UpdateListItemRequest, UpdateNoteRequest,
};

fn now() -> String {
    Utc::now().to_rfc3339()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/notes")
        .add("/", get(list).post(create))
        .add("/labels", get(list_labels).post(create_label))
        .add("/labels/{id}", put(update_label).delete(delete_label))
        .add("/{id}", get(show).put(update).delete(remove))
        .add("/{id}/trash", put(trash))
        .add("/{id}/items", post(add_item))
        .add(
            "/{id}/items/{item_id}",
            put(update_item).delete(delete_item),
        )
        .add(
            "/{id}/labels/{label_id}",
            post(add_label).delete(remove_label),
        )
}

async fn get_note_labels(db: &DatabaseConnection, note_id: &str) -> Result<Vec<serde_json::Value>> {
    let nls = notes::note_label::Entity::find()
        .filter(notes::note_label::Column::NoteId.eq(note_id))
        .all(db)
        .await?;
    let mut res = Vec::new();
    for nl in nls {
        if let Some(l) = notes::label::Entity::find_by_id(&nl.label_id)
            .one(db)
            .await?
        {
            res.push(serde_json::json!({
                "id": l.id,
                "name": l.name,
                "owner_id": l.owner_id,
            }));
        }
    }
    Ok(res)
}

async fn get_list_items(db: &DatabaseConnection, note_id: &str) -> Result<Vec<serde_json::Value>> {
    let items = notes::list_item::Entity::find()
        .filter(notes::list_item::Column::NoteId.eq(note_id))
        .order_by_asc(notes::list_item::Column::Position)
        .all(db)
        .await?;
    Ok(items
        .into_iter()
        .map(|i| {
            serde_json::json!({
                "id": i.id,
                "note_id": i.note_id,
                "parent_id": i.parent_id,
                "position": i.position,
                "text": i.text,
                "checked": i.checked,
            })
        })
        .collect())
}

#[debug_handler]
async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<serde_json::Value>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let q = params.get("q").and_then(|v| v.as_str()).unwrap_or("");
    let color = params.get("color").and_then(|v| v.as_str());
    let pinned = params.get("pinned").and_then(|v| v.as_bool());
    let archived = params.get("archived").and_then(|v| v.as_bool());
    let trashed = params
        .get("trashed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let label = params
        .get("label")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let mut query = notes::note::Entity::find().filter(notes::note::Column::OwnerId.eq(uid));
    if !trashed {
        query = query.filter(notes::note::Column::TrashedAt.is_null());
    } else {
        query = query.filter(notes::note::Column::TrashedAt.is_not_null());
    }
    if !q.is_empty() {
        query = query.filter(
            Condition::any()
                .add(notes::note::Column::Title.contains(q))
                .add(notes::note::Column::BodyText.contains(q)),
        );
    }
    if let Some(c) = color {
        if !c.is_empty() {
            query = query.filter(notes::note::Column::Color.eq(c));
        }
    }
    if let Some(p) = pinned {
        query = query.filter(notes::note::Column::Pinned.eq(if p { 1i32 } else { 0i32 }));
    }
    if let Some(a) = archived {
        query = query.filter(notes::note::Column::Archived.eq(if a { 1i32 } else { 0i32 }));
    }
    if let Some(lbl) = label {
        if let Some(l) = notes::label::Entity::find_by_id(&lbl)
            .filter(notes::label::Column::OwnerId.eq(uid))
            .one(&db)
            .await?
        {
            let note_ids: Vec<String> = notes::note_label::Entity::find()
                .filter(notes::note_label::Column::LabelId.eq(l.id))
                .all(&db)
                .await?
                .into_iter()
                .map(|nl| nl.note_id)
                .collect();
            if !note_ids.is_empty() {
                query = query.filter(notes::note::Column::Id.is_in(note_ids));
            } else {
                return format::json(
                    serde_json::json!({"data": [], "total": 0, "page": 1, "perPage": 50}),
                );
            }
        } else {
            return format::json(
                serde_json::json!({"data": [], "total": 0, "page": 1, "perPage": 50}),
            );
        }
    }
    let page = params.get("page").and_then(|v| v.as_u64()).unwrap_or(1);
    let per_page = params.get("perPage").and_then(|v| v.as_u64()).unwrap_or(50);
    let total = query.clone().count(&db).await?;
    let rows = query
        .order_by_desc(notes::note::Column::Pinned)
        .order_by_desc(notes::note::Column::UpdatedAt)
        .paginate(&db, per_page)
        .fetch_page(page.saturating_sub(1))
        .await?;
    let mut items = Vec::new();
    for n in rows {
        let labels = get_note_labels(&db, &n.id).await?;
        let items_list = get_list_items(&db, &n.id).await?;
        items.push(serde_json::json!({
            "id": n.id, "owner_id": n.owner_id, "kind": n.kind, "title": n.title,
            "body_text": n.body_text, "color": n.color, "pinned": n.pinned, "archived": n.archived,
            "trashed_at": n.trashed_at, "created_at": n.created_at, "updated_at": n.updated_at, "version": n.version,
            "labels": labels, "items": items_list
        }));
    }
    format::json(
        serde_json::json!({"data": items, "total": total, "page": page, "perPage": per_page}),
    )
}

#[debug_handler]
async fn show(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let note = notes::note::Entity::find_by_id(&id)
        .filter(notes::note::Column::OwnerId.eq(uid))
        .one(&db)
        .await?;
    match note {
        Some(n) => {
            let labels = get_note_labels(&db, &n.id).await?;
            let items_list = get_list_items(&db, &n.id).await?;
            format::json(serde_json::json!({
                "id": n.id, "owner_id": n.owner_id, "kind": n.kind, "title": n.title,
                "body_text": n.body_text, "color": n.color, "pinned": n.pinned, "archived": n.archived,
                "trashed_at": n.trashed_at, "created_at": n.created_at, "updated_at": n.updated_at, "version": n.version,
                "labels": labels, "items": items_list
            }))
        }
        None => not_found(),
    }
}

#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateNoteRequest>,
) -> Result<Response> {
    validator::Validate::validate(&params).map_err(|e| Error::BadRequest(e.to_string()))?;
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let txn = db.begin().await?;
    let note_active = params.clone().into_active(uid);
    let note: notes::note::Model = note_active
        .insert(&txn)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    if note.kind == "list" {
        if let Some(items) = params.items.clone() {
            if items.len() > 1000 {
                txn.rollback().await?;
                return Err(Error::BadRequest("too many items".to_string()));
            }
            for (i, item) in items.iter().enumerate() {
                let li = notes::list_item::ActiveModel {
                    id: Set(uuid::Uuid::new_v4().to_string()),
                    note_id: Set(note.id.clone()),
                    parent_id: Set(item.parent_id.clone()),
                    position: Set(item.position.unwrap_or(i as i32)),
                    text: Set(item.text.clone()),
                    checked: Set(item.checked.unwrap_or(0)),
                    ..Default::default()
                };
                li.insert(&txn)
                    .await
                    .map_err(|e| Error::string(&e.to_string()))?;
            }
        }
    }
    if let Some(label_ids) = params.label_ids.clone() {
        if label_ids.len() > 50 {
            txn.rollback().await?;
            return Err(Error::BadRequest("too many labels".to_string()));
        }
        for lid in label_ids {
            if notes::label::Entity::find_by_id(&lid)
                .filter(notes::label::Column::OwnerId.eq(uid))
                .one(&txn)
                .await?
                .is_some()
            {
                let nl = notes::note_label::ActiveModel {
                    note_id: Set(note.id.clone()),
                    label_id: Set(lid),
                    ..Default::default()
                };
                let _ = nl.insert(&txn).await;
            }
        }
    }
    txn.commit().await?;
    show(auth, State(ctx), Path(note.id)).await
}

#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<UpdateNoteRequest>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let txn = db.begin().await?;
    let existing = notes::note::Entity::find_by_id(&id)
        .filter(notes::note::Column::OwnerId.eq(uid))
        .one(&txn)
        .await?;
    let Some(existing) = existing else {
        txn.rollback().await?;
        return not_found();
    };
    let mut active = params.clone().apply(&existing);
    active.id = Set(existing.id.clone());
    let _: notes::note::Model = active
        .update(&txn)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    if let Some(label_ids) = params.label_ids.clone() {
        if label_ids.len() > 50 {
            txn.rollback().await?;
            return Err(Error::BadRequest("too many labels".to_string()));
        }
        notes::note_label::Entity::delete_many()
            .filter(notes::note_label::Column::NoteId.eq(id.clone()))
            .exec(&txn)
            .await?;
        for lid in label_ids {
            if notes::label::Entity::find_by_id(&lid)
                .filter(notes::label::Column::OwnerId.eq(uid))
                .one(&txn)
                .await?
                .is_some()
            {
                let nl = notes::note_label::ActiveModel {
                    note_id: Set(id.clone()),
                    label_id: Set(lid),
                    ..Default::default()
                };
                let _ = nl.insert(&txn).await;
            }
        }
    }
    txn.commit().await?;
    show(auth, State(ctx.clone()), Path(id)).await
}

#[debug_handler]
async fn trash(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let existing = notes::note::Entity::find_by_id(&id)
        .filter(notes::note::Column::OwnerId.eq(uid))
        .one(&db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };
    let mut active: notes::note::ActiveModel = existing.clone().into();
    active.trashed_at = Set(Some(now()));
    active.updated_at = Set(Some(now()));
    active.version = Set(existing.version + 1);
    let _ = active.update(&db).await?;
    show(auth, State(ctx), Path(id)).await
}

#[debug_handler]
async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let txn = db.begin().await?;
    let existing = notes::note::Entity::find_by_id(&id)
        .filter(notes::note::Column::OwnerId.eq(uid))
        .one(&txn)
        .await?;
    if existing.is_none() {
        txn.rollback().await?;
        return not_found();
    }
    notes::note_label::Entity::delete_many()
        .filter(notes::note_label::Column::NoteId.eq(id.clone()))
        .exec(&txn)
        .await?;
    notes::list_item::Entity::delete_many()
        .filter(notes::list_item::Column::NoteId.eq(id.clone()))
        .exec(&txn)
        .await?;
    notes::note::Entity::delete_by_id(&id).exec(&txn).await?;
    txn.commit().await?;
    format::empty()
}

#[debug_handler]
async fn add_item(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<CreateListItemRequest>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let txn = db.begin().await?;
    let note = notes::note::Entity::find_by_id(&id)
        .filter(notes::note::Column::OwnerId.eq(uid))
        .one(&txn)
        .await?;
    if note.is_none() {
        txn.rollback().await?;
        return not_found();
    }
    let count = notes::list_item::Entity::find()
        .filter(notes::list_item::Column::NoteId.eq(id.clone()))
        .count(&txn)
        .await?;
    if count >= 1000 {
        txn.rollback().await?;
        return Err(Error::BadRequest("too many items".to_string()));
    }
    if let Some(ref pid) = params.parent_id {
        let parent = notes::list_item::Entity::find_by_id(pid).one(&txn).await?;
        match parent {
            Some(p) => {
                if p.parent_id.is_some() {
                    txn.rollback().await?;
                    return Err(Error::BadRequest("depth exceeds 2".to_string()));
                }
            }
            None => {
                txn.rollback().await?;
                return Err(Error::BadRequest("parent not found".to_string()));
            }
        }
    }
    let li = notes::list_item::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        note_id: Set(id.clone()),
        parent_id: Set(params.parent_id),
        position: Set(params.position.unwrap_or(count as i32)),
        text: Set(params.text),
        checked: Set(params.checked.unwrap_or(0)),
        ..Default::default()
    };
    li.insert(&txn)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    txn.commit().await?;
    show(auth, State(ctx), Path(id)).await
}

#[debug_handler]
async fn update_item(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((id, item_id)): Path<(String, String)>,
    Json(params): Json<UpdateListItemRequest>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let txn = db.begin().await?;
    let note = notes::note::Entity::find_by_id(&id)
        .filter(notes::note::Column::OwnerId.eq(uid))
        .one(&txn)
        .await?;
    if note.is_none() {
        txn.rollback().await?;
        return not_found();
    }
    let existing = notes::list_item::Entity::find_by_id(&item_id)
        .filter(notes::list_item::Column::NoteId.eq(id.clone()))
        .one(&txn)
        .await?;
    let Some(mut existing) = existing else {
        txn.rollback().await?;
        return not_found();
    };
    if let Some(ref npid) = params.parent_id {
        // `Some("")` clears the parent (back to top level); JSON null
        // deserializes as `None` which means "don't change".
        let cur_parent = existing
            .parent_id
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");
        if npid.is_empty() {
            existing.parent_id = None;
        } else if npid != cur_parent {
            let parent = notes::list_item::Entity::find_by_id(npid).one(&txn).await?;
            match parent {
                Some(p) => {
                    if p.parent_id.is_some() {
                        txn.rollback().await?;
                        return Err(Error::BadRequest("depth exceeds 2".to_string()));
                    }
                }
                None => {
                    txn.rollback().await?;
                    return Err(Error::BadRequest("parent not found".to_string()));
                }
            }
        }
        if !npid.is_empty() {
            existing.parent_id = Some(npid.clone());
        }
    }
    // NOTE: `existing.into()` yields all-`Unchanged` fields, which SeaORM's
    // `UpdateOne` silently treats as a noop (re-selects old values, Ok).
    // Build the ActiveModel with explicit `Set(..)` like the note `update`
    // handler does, so provided fields actually persist.
    let mut am = notes::list_item::ActiveModel {
        id: Set(item_id.clone()),
        ..Default::default()
    };
    if let Some(v) = params.text {
        existing.text = v.clone();
        am.text = Set(v);
    }
    if let Some(v) = params.checked {
        existing.checked = v;
        am.checked = Set(v);
    }
    if let Some(v) = params.position {
        existing.position = v;
        am.position = Set(v);
    }
    if params.parent_id.is_some() {
        am.parent_id = Set(existing.parent_id.clone());
    }
    am.update(&txn)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    txn.commit().await?;
    show(auth, State(ctx), Path(id)).await
}

#[debug_handler]
async fn delete_item(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((id, item_id)): Path<(String, String)>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let txn = db.begin().await?;
    let note = notes::note::Entity::find_by_id(&id)
        .filter(notes::note::Column::OwnerId.eq(uid))
        .one(&txn)
        .await?;
    if note.is_none() {
        txn.rollback().await?;
        return not_found();
    }
    notes::list_item::Entity::delete_many()
        .filter(notes::list_item::Column::ParentId.eq(item_id.clone()))
        .exec(&txn)
        .await?;
    notes::list_item::Entity::delete_by_id(&item_id)
        .exec(&txn)
        .await?;
    txn.commit().await?;
    show(auth, State(ctx), Path(id)).await
}

#[debug_handler]
async fn add_label(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((id, label_id)): Path<(String, String)>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let note = notes::note::Entity::find_by_id(&id)
        .filter(notes::note::Column::OwnerId.eq(uid))
        .one(&db)
        .await?;
    if note.is_none() {
        return not_found();
    }
    if notes::label::Entity::find_by_id(&label_id)
        .filter(notes::label::Column::OwnerId.eq(uid))
        .one(&db)
        .await?
        .is_some()
    {
        let nl = notes::note_label::ActiveModel {
            note_id: Set(id.clone()),
            label_id: Set(label_id),
            ..Default::default()
        };
        let _ = nl.insert(&db).await;
    }
    show(auth, State(ctx), Path(id)).await
}

#[debug_handler]
async fn remove_label(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path((id, label_id)): Path<(String, String)>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let note = notes::note::Entity::find_by_id(&id)
        .filter(notes::note::Column::OwnerId.eq(uid))
        .one(&db)
        .await?;
    if note.is_none() {
        return not_found();
    }
    notes::note_label::Entity::delete_many()
        .filter(notes::note_label::Column::NoteId.eq(id.clone()))
        .filter(notes::note_label::Column::LabelId.eq(label_id))
        .exec(&db)
        .await?;
    show(auth, State(ctx), Path(id)).await
}

#[debug_handler]
async fn list_labels(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let labels = notes::label::Entity::find()
        .filter(notes::label::Column::OwnerId.eq(uid))
        .all(&db)
        .await?;
    let res: Vec<_> = labels
        .into_iter()
        .map(|l| serde_json::json!({"id": l.id, "name": l.name, "owner_id": l.owner_id}))
        .collect();
    format::json(res)
}

#[debug_handler]
async fn create_label(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateLabelRequest>,
) -> Result<Response> {
    validator::Validate::validate(&params).map_err(|e| Error::BadRequest(e.to_string()))?;
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let labels_count = notes::label::Entity::find()
        .filter(notes::label::Column::OwnerId.eq(uid))
        .count(&db)
        .await?;
    if labels_count >= 50 {
        return Err(Error::BadRequest("too many labels".to_string()));
    }
    let name = params.name.trim();
    let name_ci = name.to_lowercase();
    let label = notes::label::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        owner_id: Set(uid),
        name: Set(name.to_string()),
        name_ci: Set(name_ci),
        ..Default::default()
    };
    let created = label
        .insert(&db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(
        serde_json::json!({"id": created.id, "name": created.name, "owner_id": created.owner_id}),
    )
}

#[debug_handler]
async fn update_label(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<UpdateLabelRequest>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let existing = notes::label::Entity::find_by_id(&id)
        .filter(notes::label::Column::OwnerId.eq(uid))
        .one(&db)
        .await?;
    let Some(existing) = existing else {
        return not_found();
    };
    let name = params.name.trim();
    let name_ci = name.to_lowercase();
    let mut am: notes::label::ActiveModel = existing.into();
    am.name = Set(name.to_string());
    am.name_ci = Set(name_ci);
    let updated = am
        .update(&db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    format::json(
        serde_json::json!({"id": updated.id, "name": updated.name, "owner_id": updated.owner_id}),
    )
}

#[debug_handler]
async fn delete_label(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let db = ctx.db.clone();
    let uid = super::uid(&ctx, &auth).await?;
    let txn = db.begin().await?;
    let existing = notes::label::Entity::find_by_id(&id)
        .filter(notes::label::Column::OwnerId.eq(uid))
        .one(&txn)
        .await?;
    if existing.is_none() {
        txn.rollback().await?;
        return not_found();
    }
    notes::note_label::Entity::delete_many()
        .filter(notes::note_label::Column::LabelId.eq(id.clone()))
        .exec(&txn)
        .await?;
    notes::label::Entity::delete_by_id(&id).exec(&txn).await?;
    txn.commit().await?;
    format::empty()
}
