use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// A card repayment is a TRANSFER (bank -> card), not an expense: the purchase
/// it settles was already recorded when the card was charged. Storing
/// repayments as expense rows double-counted spending in lists, summaries and
/// analytics, so they live in their own table and never touch the expenses ledger.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "card_repayments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: i64,
    pub card_id: String,
    pub bank_id: Option<String>,
    pub amount: f64,
    pub paid_on: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
