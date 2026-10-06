use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "depreciation_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub asset_id: String,
    pub user_id: i64,
    pub period_index: i64,
    pub period_start: String,
    pub period_end: String,
    pub opening_book_value: f64,
    pub depreciation_amount: f64,
    pub closing_book_value: f64,
    pub accumulated_depreciation: f64,
    pub method: String,
    pub is_current: i64,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
