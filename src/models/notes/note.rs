use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "notes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub owner_id: i64,
    pub kind: String,
    pub title: String,
    pub body_text: Option<String>,
    pub color: String,
    pub pinned: i32,
    pub archived: i32,
    pub trashed_at: Option<String>,
    pub remind_at: Option<String>,
    pub repeat: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: i32,
}

/// Repeat cadences accepted for reminders. Anything else is a 400.
pub fn valid_repeat(v: &str) -> bool {
    matches!(v, "none" | "daily" | "weekly" | "monthly")
}

/// A reminder is overdue when set, in the past, and the note is not trashed.
/// RFC3339 UTC strings compare lexicographically.
pub fn is_overdue(remind_at: &Option<String>, trashed_at: &Option<String>) -> bool {
    match (remind_at, trashed_at) {
        (Some(r), None) => r.as_str() < now().as_str(),
        _ => false,
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate)]
pub struct CreateNoteRequest {
    #[validate(length(max = 1000, message = "title too long"))]
    pub title: Option<String>,
    pub body_text: Option<String>,
    pub kind: Option<String>,
    pub color: Option<String>,
    pub pinned: Option<i32>,
    pub archived: Option<i32>,
    pub remind_at: Option<String>,
    pub repeat: Option<String>,
    pub label_ids: Option<Vec<String>>,
    pub items: Option<Vec<CreateListItemRequest>>,
}

impl CreateNoteRequest {
    pub fn into_active(self, user_id: i64) -> ActiveModel {
        ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            owner_id: Set(user_id),
            kind: Set(self.kind.unwrap_or_else(|| "text".to_string())),
            title: Set(self.title.unwrap_or_default()),
            body_text: Set(self.body_text),
            color: Set(self.color.unwrap_or_else(|| "DEFAULT".to_string())),
            pinned: Set(self.pinned.unwrap_or(0)),
            archived: Set(self.archived.unwrap_or(0)),
            trashed_at: Set(None),
            remind_at: Set(self.remind_at.filter(|r| !r.is_empty())),
            repeat: Set(self.repeat.unwrap_or_else(|| "none".to_string())),
            created_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            version: Set(1),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateNoteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trashed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remind_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_ids: Option<Vec<String>>,
}

impl UpdateNoteRequest {
    pub fn apply(self, current: &Model) -> ActiveModel {
        let mut am: ActiveModel = current.clone().into();
        if let Some(v) = self.title {
            am.title = Set(v);
        }
        if let Some(v) = self.body_text {
            am.body_text = Set(Some(v));
        }
        if let Some(v) = self.kind {
            am.kind = Set(v);
        }
        if let Some(v) = self.color {
            am.color = Set(v);
        }
        if let Some(v) = self.pinned {
            am.pinned = Set(v);
        }
        if let Some(v) = self.archived {
            am.archived = Set(v);
        }
        if let Some(v) = self.remind_at {
            if v.is_empty() {
                am.remind_at = Set(None);
            } else {
                am.remind_at = Set(Some(v));
            }
        }
        if let Some(v) = self.repeat {
            am.repeat = Set(v);
        }
        if let Some(v) = self.trashed_at {
            if v.is_empty() {
                am.trashed_at = Set(None);
            } else {
                am.trashed_at = Set(Some(v));
            }
        }
        am.updated_at = Set(Some(now()));
        am.version = Set(current.version + 1);
        am
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateListItemRequest {
    pub parent_id: Option<String>,
    pub text: String,
    pub checked: Option<i32>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateListItemRequest {
    pub parent_id: Option<String>,
    pub text: Option<String>,
    pub checked: Option<i32>,
    pub position: Option<i32>,
}
