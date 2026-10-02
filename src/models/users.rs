use async_trait::async_trait;
use loco_rs::{auth::jwt, hash, prelude::*};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(unique)]
    pub pid: String,
    #[sea_orm(unique)]
    pub email: String,
    pub password: String,
    #[sea_orm(unique)]
    pub api_key: String,
    pub name: String,
    pub reset_token: Option<String>,
    pub reset_sent_at: Option<String>,
    pub email_verification_token: Option<String>,
    pub email_verification_sent_at: Option<String>,
    pub email_verified_at: Option<String>,
    pub magic_link_token: Option<String>,
    pub magic_link_expiration: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginParams {
    #[serde(alias = "email")]
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterParams {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub pid: String,
    pub email: String,
    pub name: String,
    pub api_key: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Model> for UserResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            pid: m.pid,
            email: m.email,
            name: m.name,
            api_key: m.api_key,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

impl Model {
    pub async fn find_by_email<C: ConnectionTrait>(db: &C, email: &str) -> ModelResult<Self> {
        Entity::find()
            .filter(Column::Email.eq(email))
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    pub async fn find_by_username<C: ConnectionTrait>(db: &C, username: &str) -> ModelResult<Self> {
        Self::find_by_email(db, username).await
    }

    pub async fn find_by_pid<C: ConnectionTrait>(db: &C, pid: &str) -> ModelResult<Self> {
        Entity::find()
            .filter(Column::Pid.eq(pid))
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    pub async fn find_by_api_key<C: ConnectionTrait>(db: &C, api_key: &str) -> ModelResult<Self> {
        Entity::find()
            .filter(Column::ApiKey.eq(api_key))
            .one(db)
            .await?
            .ok_or(ModelError::EntityNotFound)
    }

    #[must_use]
    pub fn verify_password(&self, password: &str) -> bool {
        hash::verify_password(password, &self.password)
    }

    pub async fn create_with_password<C: ConnectionTrait>(
        db: &C,
        params: &RegisterParams,
    ) -> ModelResult<Self> {
        if Entity::find()
            .filter(Column::Email.eq(&params.email))
            .one(db)
            .await?
            .is_some()
        {
            return Err(ModelError::EntityAlreadyExists);
        }

        let password_hash =
            hash::hash_password(&params.password).map_err(|e| ModelError::Any(e.into()))?;

        ActiveModel {
            pid: Set(Uuid::new_v4().to_string()),
            email: Set(params.email.clone()),
            password: Set(password_hash),
            name: Set(params.name.clone()),
            api_key: Set(format!("lo-{}", Uuid::new_v4())),
            created_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(ModelError::from)
    }

    pub fn generate_jwt(&self, secret: &str, expiration: u64) -> ModelResult<String> {
        jwt::JWT::new(secret)
            .generate_token(expiration, self.pid.clone(), Map::new())
            .map_err(ModelError::from)
    }
}

#[async_trait]
impl Authenticable for Model {
    async fn find_by_api_key(db: &DatabaseConnection, api_key: &str) -> ModelResult<Self> {
        Self::find_by_api_key(db, api_key).await
    }

    async fn find_by_claims_key(db: &DatabaseConnection, claims_key: &str) -> ModelResult<Self> {
        Self::find_by_pid(db, claims_key).await
    }
}
