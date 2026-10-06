use sea_orm::entity::prelude::*;

/// An intended purchase: priced, prioritised and dated, but not yet paid for.
///
/// Kept separate from `assets` deliberately. A wanted laptop is not a depreciating
/// asset, and recording it as one would start a schedule for money that never left the
/// bank. `converted_asset_id` is the bridge once the purchase actually happens.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "asset_buy_list")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub asset_type: String,
    pub estimated_cost: f64,
    pub priority: String,
    pub target_date: Option<String>,
    pub url: Option<String>,
    pub shop: Option<String>,
    pub status: String,
    pub bank_id: Option<String>,
    pub converted_asset_id: Option<String>,
    pub converted_at: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Lifecycle of a buy-list item: `planned` → `purchased` (or `dropped`).
pub const PLANNED: &str = "planned";
pub const PURCHASED: &str = "purchased";
pub const DROPPED: &str = "dropped";

/// Ordering weight for `priority`, so "high" sorts above "low" without a
/// case-sensitive string comparison deciding it.
pub fn priority_rank(p: &str) -> i32 {
    match p.trim().to_ascii_lowercase().as_str() {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 1,
    }
}
