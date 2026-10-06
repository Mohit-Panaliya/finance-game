//! Depreciation scheduling for assets.
//!
//! Persists what [`crate::depreciation`] computes. Regeneration is deliberate, not
//! automatic: an asset's schedule is a statement about the past, so it is only rebuilt
//! when the inputs that define it (cost, salvage, life, method, start date) change.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::depreciation::{self, Method, Spec};
use crate::models::{assets, depreciation_entry};

/// One persisted row of a schedule.
pub async fn entries(
    db: &impl ConnectionTrait,
    user_id: i64,
    asset_id: &str,
) -> Result<Vec<depreciation::Period>, sea_orm::DbErr> {
    let rows = depreciation_entry::Entity::find()
        .filter(depreciation_entry::Column::UserId.eq(user_id))
        .filter(depreciation_entry::Column::AssetId.eq(asset_id))
        .order_by_asc(depreciation_entry::Column::PeriodIndex)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| depreciation::Period {
            index: r.period_index,
            start: r.period_start,
            end: r.period_end,
            opening: r.opening_book_value,
            charge: r.depreciation_amount,
            closing: r.closing_book_value,
            accumulated: r.accumulated_depreciation,
        })
        .collect())
}

/// Build the schedule for an asset, replacing whatever was stored.
///
/// Returns `Err` with a human-readable reason when the inputs cannot produce a legal
/// schedule (no useful life, salvage above cost, and so on) so the caller can surface it.
pub async fn rebuild(
    db: &impl ConnectionTrait,
    user_id: i64,
    asset: &assets::Model,
    today: &str,
) -> Result<Vec<depreciation::Period>, String> {
    let method = Method::parse(&asset.depreciation_method).unwrap_or(Method::StraightLine);
    let life = asset.useful_life_months.unwrap_or(0);
    if life < 1 {
        return Err(depreciation::ScheduleError::NoUsefulLife.to_string());
    }
    let start = asset
        .depreciation_start_date
        .clone()
        .unwrap_or_else(|| asset.purchase_date.clone());

    let periods = depreciation::generate(Spec {
        cost: asset.purchase_price,
        salvage: asset.salvage_value,
        life_months: life,
        method,
        total_units: None,
        units_used: None,
        start_date: &start,
    })
    .map_err(|e| e.to_string())?;

    depreciation_entry::Entity::delete_many()
        .filter(depreciation_entry::Column::UserId.eq(user_id))
        .filter(depreciation_entry::Column::AssetId.eq(asset.id.clone()))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    for p in &periods {
        depreciation_entry::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            asset_id: Set(asset.id.clone()),
            user_id: Set(user_id),
            period_index: Set(p.index),
            period_start: Set(p.start.clone()),
            period_end: Set(p.end.clone()),
            opening_book_value: Set(p.opening),
            depreciation_amount: Set(p.charge),
            closing_book_value: Set(p.closing),
            accumulated_depreciation: Set(p.accumulated),
            method: Set(method.as_str().to_string()),
            // `is_current` marks the period containing today, so the UI can highlight
            // it without recomputing date arithmetic on every render.
            is_current: Set(i64::from(
                today >= p.start.as_str() && today < p.end.as_str(),
            )),
            created_at: Set(chrono::Utc::now().to_rfc3339()),
        }
        .insert(db)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(periods)
}

/// Rebuild only if the defining inputs differ from what produced the stored schedule.
///
/// Returns the current schedule either way, so callers can treat it as read-or-write.
pub async fn ensure_schedule(
    db: &impl ConnectionTrait,
    user_id: i64,
    asset: &assets::Model,
    today: &str,
) -> Result<Vec<depreciation::Period>, String> {
    let existing = entries(db, user_id, &asset.id)
        .await
        .map_err(|e| e.to_string())?;
    let up_to_date = !existing.is_empty()
        && existing.len() as i64 == asset.useful_life_months.unwrap_or(0)
        && existing
            .last()
            .map(|p| (p.closing - asset.salvage_value).abs() < 0.01)
            .unwrap_or(false);
    if up_to_date {
        return Ok(existing);
    }
    rebuild(db, user_id, asset, today).await
}

/// Everything a UI needs to show depreciation for one asset, in one call.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DepreciationView {
    pub method: String,
    pub cost: f64,
    pub salvage_value: f64,
    pub depreciable: f64,
    pub useful_life_months: Option<i64>,
    pub schedule_start: Option<String>,
    pub book_value: f64,
    pub accumulated_depreciation: f64,
    pub depreciation_to_date: f64,
    pub fully_depreciated_on: Option<String>,
    pub periods: Vec<depreciation::Period>,
    /// Set when a schedule cannot be produced, so the UI can explain why.
    pub problem: Option<String>,
}

pub async fn view(
    db: &impl ConnectionTrait,
    user_id: i64,
    asset: &assets::Model,
    today: &str,
) -> Result<DepreciationView, sea_orm::DbErr> {
    let method = Method::parse(&asset.depreciation_method).unwrap_or(Method::StraightLine);
    let periods = match ensure_schedule(db, user_id, asset, today).await {
        Ok(p) => p,
        Err(reason) => {
            return Ok(DepreciationView {
                method: method.as_str().to_string(),
                cost: asset.purchase_price,
                salvage_value: asset.salvage_value,
                depreciable: 0.0,
                useful_life_months: asset.useful_life_months,
                schedule_start: asset.depreciation_start_date.clone(),
                book_value: asset.purchase_price,
                accumulated_depreciation: 0.0,
                depreciation_to_date: 0.0,
                fully_depreciated_on: None,
                periods: Vec::new(),
                problem: Some(reason),
            })
        }
    };

    Ok(DepreciationView {
        method: method.as_str().to_string(),
        cost: asset.purchase_price,
        salvage_value: asset.salvage_value,
        depreciable: ((asset.purchase_price - asset.salvage_value) * 100.0).round() / 100.0,
        useful_life_months: asset.useful_life_months,
        schedule_start: asset
            .depreciation_start_date
            .clone()
            .or_else(|| periods.first().map(|p| p.start.clone())),
        book_value: depreciation::book_value(&periods, asset.purchase_price, today),
        accumulated_depreciation: asset.purchase_price
            - depreciation::book_value(&periods, asset.purchase_price, today),
        depreciation_to_date: depreciation::accumulated(&periods, today),
        fully_depreciated_on: depreciation::fully_depreciated_on(&periods),
        periods,
        problem: None,
    })
}
