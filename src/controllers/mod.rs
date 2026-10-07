pub mod analysis;
pub mod asset_ledger;
pub mod assets;
pub mod auth;
pub mod banks;
pub mod credit_cards;
pub mod debts;
pub mod expenses;
pub mod fixed_deposits;
pub mod incomes;
pub mod investments;
pub mod notes;
pub mod sync;

use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::models::users;

pub async fn uid(ctx: &AppContext, auth_jwt: &loco_rs::prelude::auth::JWT) -> Result<i64> {
    if let Ok(id) = auth_jwt.claims.pid.parse::<i64>() {
        return Ok(id);
    }
    let user = users::Entity::find()
        .filter(users::Column::Pid.eq(auth_jwt.claims.pid.clone()))
        .one(&ctx.db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?
        .ok_or_else(|| Error::NotFound)?;
    Ok(user.id)
}
