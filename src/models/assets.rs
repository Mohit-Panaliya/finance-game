use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use validator::Validate;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "assets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub user_id: i64,
    pub name: String,
    pub asset_type: String,
    pub category: String,
    pub purchase_price: f64,
    pub current_value: f64,
    pub purchase_date: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub documents: Option<String>,
    pub roi_percentage: Option<f64>,
    pub annual_income: f64,
    pub depreciation_rate: Option<f64>,
    pub is_liquid: bool,
    pub risk_level: String,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub game_building_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAssetRequest {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
    pub asset_type: String,
    pub category: String,
    pub purchase_price: Option<f64>,
    pub current_value: Option<f64>,
    pub purchase_date: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub documents: Option<String>,
    pub roi_percentage: Option<f64>,
    pub annual_income: Option<f64>,
    pub depreciation_rate: Option<f64>,
    pub is_liquid: Option<bool>,
    pub risk_level: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
    pub game_building_id: Option<String>,
}

impl CreateAssetRequest {
    pub fn into_active(self, user_id: i64) -> ActiveModel {
        ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            name: Set(self.name),
            asset_type: Set(self.asset_type),
            category: Set(self.category),
            purchase_price: Set(self.purchase_price.unwrap_or(0.0)),
            current_value: Set(self.current_value.unwrap_or(0.0)),
            purchase_date: Set(self.purchase_date),
            location: Set(self.location),
            description: Set(self.description),
            documents: Set(self.documents),
            roi_percentage: Set(self.roi_percentage),
            annual_income: Set(self.annual_income.unwrap_or(0.0)),
            depreciation_rate: Set(self.depreciation_rate),
            is_liquid: Set(self.is_liquid.unwrap_or(false)),
            risk_level: Set(self.risk_level.unwrap_or_else(|| "moderate".to_string())),
            sync_status: Set(self.sync_status.unwrap_or_else(|| "synced".to_string())),
            last_synced_at: Set(self.last_synced_at),
            game_building_id: Set(self.game_building_id),
            created_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateAssetRequest {
    pub name: Option<String>,
    pub asset_type: Option<String>,
    pub category: Option<String>,
    pub purchase_price: Option<f64>,
    pub current_value: Option<f64>,
    pub purchase_date: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub documents: Option<String>,
    pub roi_percentage: Option<f64>,
    pub annual_income: Option<f64>,
    pub depreciation_rate: Option<f64>,
    pub is_liquid: Option<bool>,
    pub risk_level: Option<String>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<String>,
    pub game_building_id: Option<String>,
}

impl UpdateAssetRequest {
    pub fn apply(self, current: &Model) -> ActiveModel {
        let mut am = current.clone().into_active_model();
        if let Some(v) = self.name {
            am.name = Set(v);
        }
        if let Some(v) = self.asset_type {
            am.asset_type = Set(v);
        }
        if let Some(v) = self.category {
            am.category = Set(v);
        }
        if let Some(v) = self.purchase_price {
            am.purchase_price = Set(v);
        }
        if let Some(v) = self.current_value {
            am.current_value = Set(v);
        }
        if let Some(v) = self.purchase_date {
            am.purchase_date = Set(v);
        }
        if let Some(v) = self.location {
            am.location = Set(Some(v));
        }
        if let Some(v) = self.description {
            am.description = Set(Some(v));
        }
        if let Some(v) = self.documents {
            am.documents = Set(Some(v));
        }
        if let Some(v) = self.roi_percentage {
            am.roi_percentage = Set(Some(v));
        }
        if let Some(v) = self.annual_income {
            am.annual_income = Set(v);
        }
        if let Some(v) = self.depreciation_rate {
            am.depreciation_rate = Set(Some(v));
        }
        if let Some(v) = self.is_liquid {
            am.is_liquid = Set(v);
        }
        if let Some(v) = self.risk_level {
            am.risk_level = Set(v);
        }
        if let Some(v) = self.sync_status {
            am.sync_status = Set(v);
        }
        if let Some(v) = self.last_synced_at {
            am.last_synced_at = Set(Some(v));
        }
        if let Some(v) = self.game_building_id {
            am.game_building_id = Set(Some(v));
        }
        am.updated_at = Set(Some(now()));
        am
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetResponse {
    pub id: String,
    pub user_id: i64,
    pub name: String,
    pub asset_type: String,
    pub category: String,
    pub purchase_price: f64,
    pub current_value: f64,
    pub purchase_date: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub documents: Option<String>,
    pub roi_percentage: Option<f64>,
    pub annual_income: f64,
    pub depreciation_rate: Option<f64>,
    pub is_liquid: bool,
    pub risk_level: String,
    pub sync_status: String,
    pub last_synced_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub game_building_id: Option<String>,
    pub gain_loss: f64,
    pub gain_pct: Option<f64>,
}

impl From<Model> for AssetResponse {
    fn from(m: Model) -> Self {
        let gain_loss = m.current_value - m.purchase_price;
        let gain_pct = if m.purchase_price > 0.0 {
            Some(gain_loss / m.purchase_price * 100.0)
        } else {
            None
        };
        Self {
            id: m.id,
            user_id: m.user_id,
            name: m.name,
            asset_type: m.asset_type,
            category: m.category,
            purchase_price: m.purchase_price,
            current_value: m.current_value,
            purchase_date: m.purchase_date,
            location: m.location,
            description: m.description,
            documents: m.documents,
            roi_percentage: m.roi_percentage,
            annual_income: m.annual_income,
            depreciation_rate: m.depreciation_rate,
            is_liquid: m.is_liquid,
            risk_level: m.risk_level,
            sync_status: m.sync_status,
            last_synced_at: m.last_synced_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
            game_building_id: m.game_building_id,
            gain_loss,
            gain_pct,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSummary {
    pub total_value: f64,
    pub total_invested: f64,
    pub total_gain_loss: f64,
    pub count: i64,
}
