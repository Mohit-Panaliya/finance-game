use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game_engine::{
    achievements::{self as game_achievements, UserAggregates},
    battle as game_battle,
    economy as game_economy,
    troops as game_troops,
    xp as game_xp,
};
use crate::models::{
    credit_cards,
    assets, banks, expenses, fixed_deposits, incomes, investments, users,
    game::{villages, buildings, troops as game_troops_model, user_achievements, battles},
};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/game")
        .add("/village", get(village_state))
        .add("/buildings/collect", post(collect_building))
        .add("/buildings/upgrade", post(upgrade_building))
        .add("/troops", get(list_troops))
        .add("/troops/train", post(train_troops))
        .add("/battle", post(battle_start))
        .add("/battles", get(list_battles))
        .add("/achievements", get(list_achievements))
        .add("/achievements/{id}/claim", post(claim_achievement))
        .add("/leaderboard", get(leaderboard))
        .add("/stats", get(stats))
        .add("/analysis", get(analysis))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStats {
    pub net_worth: f64,
    pub yearly_income: f64,
    pub monthly_expense: f64,
    pub total_gain: f64,
    pub invested: f64,
    pub roi: f64,
}

async fn uid(ctx: &AppContext, auth: &auth::JWT) -> Result<i64> {
    if let Ok(id) = auth.claims.pid.parse::<i64>() {
        return Ok(id);
    }
    let user = users::Entity::find()
        .filter(users::Column::Pid.eq(auth.claims.pid.clone()))
        .one(&ctx.db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?
        .ok_or_else(|| Error::string("user not found"))?;
    Ok(user.id)
}

async fn load_village(db: &sea_orm_turso::TursoConnection, user_id: i64) -> Result<villages::Model> {
    if let Some(v) = villages::Entity::find()
        .filter(villages::Column::UserId.eq(user_id))
        .one(db)
        .await?
    {
        return Ok(v);
    }
    let now = Utc::now().to_rfc3339();
    let starter = villages::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        user_id: Set(user_id),
        name: Set(format!("Village {user_id}")),
        level: Set(1),
        xp: Set(0),
        xp_to_next: Set(100),
        gold: Set(500.0),
        gems: Set(50.0),
        elixir: Set(0.0),
        trophy: Set(0),
        shield_until: Set(None),
        last_tick_at: Set(Some(now.clone())),
        created_at: Set(Some(now.clone())),
        updated_at: Set(Some(now)),
    };
    starter
        .insert(db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    villages::Entity::find()
        .filter(villages::Column::UserId.eq(user_id))
        .one(db)
        .await?
        .ok_or_else(|| Error::NotFound)
}

async fn compute_aggregates(db: &sea_orm_turso::TursoConnection, user_id: i64) -> Result<UserAggregates> {
    let banks = banks::Entity::find()
        .filter(banks::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let assets = assets::Entity::find()
        .filter(assets::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let fds = fixed_deposits::Entity::find()
        .filter(fixed_deposits::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let investments = investments::Entity::find()
        .filter(investments::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let expenses = expenses::Entity::find()
        .filter(expenses::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let battles_list = battles::Entity::find()
        .filter(battles::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let village = load_village(db, user_id).await?;

    let bank_balance: f64 = banks.iter().map(|b| b.current_balance).sum();
    let asset_value: f64 = assets.iter().map(|a| a.current_value).sum();
    let fd_value: f64 = fds.iter().map(|f| f.current_value).sum();
    let inv_value: f64 = investments.iter().map(|i| i.current_value).sum();
    let net_worth = bank_balance + asset_value + fd_value + inv_value + village.gold;

    let buildings = buildings::Entity::find()
        .filter(buildings::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let troops = game_troops_model::Entity::find()
        .filter(game_troops_model::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    Ok(UserAggregates {
        bank_count: banks.len() as i64,
        asset_count: assets.len() as i64,
        fd_count: fds.len() as i64,
        investment_count: investments.len() as i64,
        net_worth,
        total_income: 0.0,
        total_expense: 0.0,
        buildings_count: buildings.len() as i64,
        troops_count: troops.len() as i64,
        battle_wins: battles_list.iter().filter(|b| b.stars > 0 && b.status == "won").count() as i64,
        highest_trophy: village.trophy,
        buildings_level_sum: buildings.iter().map(|b| b.level).sum(),
        expense_count: expenses.len() as i64,
        level: village.level,
        trophy: village.trophy,
        expense_streak: 0,
    })
}

async fn unlock_new_achievements(
    db: &sea_orm_turso::TursoConnection,
    user_id: i64,
) -> Result<Vec<String>> {
    let agg = compute_aggregates(db, user_id).await?;
    let existing = user_achievements::Entity::find()
        .filter(user_achievements::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let mut by_id: std::collections::HashMap<String, user_achievements::Model> = existing
        .iter()
        .map(|ua| (ua.achievement_id.clone(), ua.clone()))
        .collect();
    let unlocked_ids: std::collections::HashSet<_> = by_id
        .values()
        .filter(|ua| ua.unlocked_at.is_some())
        .map(|ua| ua.achievement_id.clone())
        .collect();
    let mut newly_unlocked = Vec::new();
    for def in game_achievements::DEFS {
        let def_id = def.id.to_string();
        if unlocked_ids.contains(&def_id) {
            continue;
        }
        if let Some(row) = by_id.get(&def_id) {
            if game_achievements::is_unlocked(def, &agg) {
                let mut ua: user_achievements::ActiveModel = row.clone().into();
                ua.unlocked_at = Set(Some(Utc::now().to_rfc3339()));
                ua.progress = Set(def.requirement_value);
                ua.update(db).await.map_err(|e| Error::string(&e.to_string()))?;
                newly_unlocked.push(def.code.to_string());
            } else {
                let progress = game_achievements::progress_for(def, &agg);
                if progress > 0 && row.progress != progress {
                    let mut ua: user_achievements::ActiveModel = row.clone().into();
                    ua.progress = Set(progress);
                    ua.update(db).await.map_err(|e| Error::string(&e.to_string()))?;
                }
            }
            continue;
        }
        if game_achievements::is_unlocked(def, &agg) {
            let ua = user_achievements::ActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                user_id: Set(user_id),
                achievement_id: Set(def_id),
                unlocked_at: Set(Some(Utc::now().to_rfc3339())),
                progress: Set(def.requirement_value),
                is_claimed: Set(false),
            };
            ua.insert(db).await.map_err(|e| Error::string(&e.to_string()))?;
            newly_unlocked.push(def.code.to_string());
        } else {
            let progress = game_achievements::progress_for(def, &agg);
            if progress > 0 {
                let ua = user_achievements::ActiveModel {
                    id: Set(Uuid::new_v4().to_string()),
                    user_id: Set(user_id),
                    achievement_id: Set(def_id),
                    unlocked_at: Set(None),
                    progress: Set(progress),
                    is_claimed: Set(false),
                };
                ua.insert(db).await.map_err(|e| Error::string(&e.to_string()))?;
            }
        }
    }
    Ok(newly_unlocked)
}

fn get_field_i64(v: &villages::ActiveModel, level: bool) -> i64 {
    match (level, &v.level, &v.xp) {
        (true, sea_orm::ActiveValue::Set(v), _) | (true, sea_orm::ActiveValue::Unchanged(v), _) => *v,
        (false, _, sea_orm::ActiveValue::Set(v)) | (false, _, sea_orm::ActiveValue::Unchanged(v)) => *v,
        _ => 0,
    }
}

fn get_field_i642(v: &villages::ActiveModel) -> i64 {
    match &v.xp_to_next {
        sea_orm::ActiveValue::Set(v) | sea_orm::ActiveValue::Unchanged(v) => *v,
        _ => 100,
    }
}

fn set_village_xp(v: &mut villages::ActiveModel, level: i64, x: i64, to_next: i64) {
    v.level = Set(level);
    v.xp = Set(x);
    v.xp_to_next = Set(to_next);
}

fn add_gold(v: &mut villages::ActiveModel, amount: f64) {
    let cur = match &v.gold {
        sea_orm::ActiveValue::Set(val) | sea_orm::ActiveValue::Unchanged(val) => *val,
        _ => 0.0,
    };
    v.gold = Set(cur + amount);
}

fn add_gems(v: &mut villages::ActiveModel, amount: f64) {
    let cur = match &v.gems {
        sea_orm::ActiveValue::Set(val) | sea_orm::ActiveValue::Unchanged(val) => *val,
        _ => 0.0,
    };
    v.gems = Set(cur + amount);
}

async fn game_stats(db: &sea_orm_turso::TursoConnection, user_id: i64) -> Result<GameStats> {
    let banks = banks::Entity::find()
        .filter(banks::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let assets = assets::Entity::find()
        .filter(assets::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let fds = fixed_deposits::Entity::find()
        .filter(fixed_deposits::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let investments = investments::Entity::find()
        .filter(investments::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let expenses = expenses::Entity::find()
        .filter(expenses::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let incomes = incomes::Entity::find()
        .filter(incomes::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    let village = load_village(db, user_id).await?;

    let bank_balance: f64 = banks.iter().map(|b| b.current_balance).sum();
    let asset_value: f64 = assets.iter().map(|a| a.current_value).sum();
    let fd_value: f64 = fds.iter().map(|f| f.current_value).sum();
    let inv_value: f64 = investments.iter().map(|i| i.current_value).sum();
    let invested = bank_balance + asset_value + fd_value + inv_value;
    let total_income: f64 = incomes.iter().map(|i| i.amount).sum();
    let total_expense: f64 = expenses.iter().map(|e| e.amount).sum();
    let net_worth = invested + village.gold;
    let total_gain = total_income - total_expense;
    let roi = if invested > 0.0 { (total_gain / invested) * 100.0 } else { 0.0 };
    let monthly_expense = total_expense / 12.0;

    Ok(GameStats {
        net_worth,
        yearly_income: total_income,
        monthly_expense,
        total_gain,
        invested,
        roi,
    })
}

#[debug_handler]
async fn village_state(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;
    let village = load_village(&*db, user_id).await?;
    let buildings = buildings::Entity::find()
        .filter(buildings::Column::UserId.eq(user_id))
        .all(&*db)
        .await?;
    let troops = game_troops_model::Entity::find()
        .filter(game_troops_model::Column::UserId.eq(user_id))
        .all(&*db)
        .await?;
    let agg = compute_aggregates(&*db, user_id).await?;
    let mut village_am = village.clone().into_active_model();
    let _ = unlock_new_achievements(&*db, user_id).await?;
    let achievements_list = user_achievements::Entity::find()
        .filter(user_achievements::Column::UserId.eq(user_id))
        .all(&*db)
        .await?;
    let unlocked = achievements_list.iter().filter(|ua| ua.unlocked_at.is_some()).count() as i64;
    let claimed = achievements_list.iter().filter(|ua| ua.is_claimed).count() as i64;

    format::json(serde_json::json!({
        "village": villages::VillageResponse::from(village),
        "buildings": buildings,
        "troops": troops,
        "stats": agg,
        "achievements_unlocked": unlocked,
        "achievements_claimed": claimed,
    }))
}

#[debug_handler]
async fn collect_building(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<serde_json::Value>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;
    let building_id = params.get("building_id").and_then(|v| v.as_str()).ok_or_else(|| Error::string("building_id required"))?;
    let village = load_village(&*db, user_id).await?;
    let building = buildings::Entity::find_by_id(building_id)
        .filter(buildings::Column::UserId.eq(user_id))
        .one(&*db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    // Extract needed values before moving building
    let building_production_rate = building.production_rate;
    let building_last_collected = building.last_collected_at.clone();
    
    let mut b_am = building.into_active_model();
    let now = Utc::now().to_rfc3339();
    let (start, now_ts) = match (
        building_last_collected.as_deref(),
        chrono::DateTime::parse_from_rfc3339(&now),
    ) {
        (Some(s), Ok(n)) => match chrono::DateTime::parse_from_rfc3339(s) {
            Ok(s) => (s, n),
            Err(_) => return bad_request("invalid date"),
        },
        _ => return bad_request("never collected"),
    };
    let hours = (now_ts - start).num_seconds() as f64 / 3600.0;
    let production = if hours > 0.0 { building_production_rate * hours } else { 0.0 };
    let current = match &b_am.stored_resource {
        sea_orm::ActiveValue::Set(val) | sea_orm::ActiveValue::Unchanged(val) => *val,
        _ => 0.0,
    };
    let max_hp = match &b_am.max_hp {
        sea_orm::ActiveValue::Set(val) | sea_orm::ActiveValue::Unchanged(val) => *val,
        _ => 100,
    };
    let new_stored = (current + production).min(max_hp as f64);
    b_am.stored_resource = Set(new_stored);
    b_am.last_collected_at = Set(Some(now.clone()));
    b_am.updated_at = Set(Some(now.clone()));
    let _b_updated = b_am.update(&*db).await.map_err(|e| Error::string(&e.to_string()))?;

    let mut v_am = village.into_active_model();
    let gold_production = new_stored - current;
    if gold_production > 0.0 {
        add_gold(&mut v_am, gold_production);
    }
    v_am.updated_at = Set(Some(now.clone()));
    let v_am_final = v_am.update(&*db).await.map_err(|e| Error::string(&e.to_string()))?;

    let agg = compute_aggregates(&*db, user_id).await?;
    let _ = unlock_new_achievements(&*db, user_id).await?;

    // v_am_final is now a Model, gold is f64 directly
    format::json(serde_json::json!({
        "collected": gold_production,
        "stored": new_stored,
        "village_gold": v_am_final.gold,
    }))
}

#[debug_handler]
async fn upgrade_building(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<serde_json::Value>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;
    let building_id = params.get("building_id").and_then(|v| v.as_str()).ok_or_else(|| Error::string("building_id required"))?;
    let village = load_village(&*db, user_id).await?;
    let building = buildings::Entity::find_by_id(building_id)
        .filter(buildings::Column::UserId.eq(user_id))
        .one(&*db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    // building.level is i64, not ActiveValue
    let level = building.level;
    let building_type = building.building_type.clone();
    
    let cost = game_economy::upgrade_cost(&building_type, level);
    if village.gold < cost {
        return bad_request("not enough gold");
    }
    let mut v_am = village.into_active_model();
    add_gold(&mut v_am, -cost);
    let duration = game_economy::upgrade_duration_secs(&building_type, level);
    let finishes_at = Utc::now() + chrono::Duration::seconds(duration);
    let mut b_am = building.into_active_model();
    b_am.is_upgrading = Set(true);
    b_am.upgrade_finishes_at = Set(Some(finishes_at.to_rfc3339()));
    b_am.updated_at = Set(Some(Utc::now().to_rfc3339()));
    let _b_updated = b_am.update(&*db).await.map_err(|e| Error::string(&e.to_string()))?;
    v_am.updated_at = Set(Some(Utc::now().to_rfc3339()));
    let v_am_final = v_am.update(&*db).await.map_err(|e| Error::string(&e.to_string()))?;

    format::json(serde_json::json!({
        "upgrading": true,
        "finishes_at": finishes_at.to_rfc3339(),
        "gold": v_am_final.gold,
    }))
}

#[debug_handler]
async fn list_troops(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;
    let troops = game_troops_model::Entity::find()
        .filter(game_troops_model::Column::UserId.eq(user_id))
        .all(&*db)
        .await?;
    format::json(troops)
}

#[debug_handler]
async fn train_troops(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<serde_json::Value>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;
    let troop_type = params.get("troop_type").and_then(|v| v.as_str()).ok_or_else(|| Error::string("troop_type required"))?;
    let count = params.get("count").and_then(|v| v.as_i64()).unwrap_or(1);
    let village = load_village(&*db, user_id).await?;
    let cost = game_troops::train_cost(troop_type) * count as f64;
    if village.gold < cost {
        return bad_request("not enough gold");
    }
    let train_time = game_troops::train_time(troop_type);
    let finishes_at = game_troops::training_complete_at(&Utc::now(), train_time, count);
    let mut v_am = village.into_active_model();
    add_gold(&mut v_am, -cost);
    v_am.updated_at = Set(Some(Utc::now().to_rfc3339()));
    let v_am_final = v_am.update(&*db).await.map_err(|e| Error::string(&e.to_string()))?;

    let new_troop = game_troops_model::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        user_id: Set(user_id),
        troop_type: Set(troop_type.to_string()),
        name: Set(troop_type.to_string()),
        level: Set(1),
        attack: Set(game_troops::base_attack(troop_type)),
        hp: Set(game_troops::base_hp(troop_type)),
        count: Set(count),
        train_cost: Set(cost),
        train_time_secs: Set(train_time * count),
        training_finishes_at: Set(Some(finishes_at.clone())),
        expense_id: Set(None),
        created_at: Set(Some(Utc::now().to_rfc3339())),
        updated_at: Set(Some(Utc::now().to_rfc3339())),
    };
    new_troop.insert(&*db).await.map_err(|e| Error::string(&e.to_string()))?;

    format::json(serde_json::json!({
        "training": true,
        "finishes_at": finishes_at,
        "gold": v_am_final.gold,
    }))
}

#[debug_handler]
async fn battle_start(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<serde_json::Value>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;
    let battle_type = params.get("battle_type").and_then(|v| v.as_str()).unwrap_or("raid");
    let village = load_village(&*db, user_id).await?;
    let troops = game_troops_model::Entity::find()
        .filter(game_troops_model::Column::UserId.eq(user_id))
        .all(&*db)
        .await?;
    let buildings = buildings::Entity::find()
        .filter(buildings::Column::UserId.eq(user_id))
        .all(&*db)
        .await?;

    let battle_troops: Vec<game_battle::BattleTroop> = troops.iter().map(|t| game_battle::BattleTroop {
        id: t.id.clone(),
        troop_type: t.troop_type.clone(),
        attack: t.attack,
        hp: t.hp,
        count: t.count,
        level: t.level,
    }).collect();

    let battle_buildings: Vec<game_battle::BattleBuilding> = buildings.iter().map(|b| game_battle::BattleBuilding {
        id: b.id.clone(),
        building_type: b.building_type.clone(),
        name: b.name.clone(),
        hp: b.hp,
        max_hp: b.max_hp,
        level: b.level,
    }).collect();

    let outcome = game_battle::simulate(battle_type, &battle_troops, &battle_buildings);

    let mut v_am = village.into_active_model();
    add_gold(&mut v_am, outcome.gold_reward);
    add_gems(&mut v_am, outcome.gem_reward);
    let v_am_final = v_am.update(&*db).await.map_err(|e| Error::string(&e.to_string()))?;

    let agg = compute_aggregates(&*db, user_id).await?;
    let _ = unlock_new_achievements(&*db, user_id).await?;

    let battle_record = battles::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        user_id: Set(user_id),
        battle_type: Set(battle_type.to_string()),
        status: Set(if outcome.won { "won" } else { "lost" }.to_string()),
        stars: Set(outcome.stars),
        destruction_percent: Set(outcome.destruction_percent),
        gold_reward: Set(outcome.gold_reward),
        gem_reward: Set(outcome.gem_reward),
        xp_reward: Set(outcome.xp_reward),
        trophy_change: Set(outcome.trophy_change),
        enemy_troops: Set(Some(serde_json::to_string(&outcome.enemy_troops).unwrap_or_default())),
        enemy_buildings: Set(Some(serde_json::to_string(&outcome.enemy_buildings).unwrap_or_default())),
        created_at: Set(Some(Utc::now().to_rfc3339())),
        updated_at: Set(Some(Utc::now().to_rfc3339())),
    };
    battle_record.insert(&*db).await.map_err(|e| Error::string(&e.to_string()))?;

    format::json(serde_json::json!({
        "won": outcome.won,
        "stars": outcome.stars,
        "destruction": outcome.destruction_percent,
        "rewards": {
            "gold": outcome.gold_reward,
            "gems": outcome.gem_reward,
            "xp": outcome.xp_reward,
        },
        "village": villages::VillageResponse::from(v_am_final),
    }))
}

#[debug_handler]
async fn list_battles(auth: auth::JWT, State(ctx): State<AppContext>, Query(params): Query<serde_json::Value>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;
    let page = params.get("page").and_then(|v| v.as_u64()).unwrap_or(1);
    let per_page = params.get("perPage").and_then(|v| v.as_u64()).unwrap_or(50);
    let query = battles::Entity::find()
        .filter(battles::Column::UserId.eq(user_id))
        .order_by_desc(battles::Column::CreatedAt);
    let total = query.clone().count(&*db).await?;
    let items = query
        .paginate(&*db, per_page)
        .fetch_page(page.saturating_sub(1))
        .await?;
    format::json(serde_json::json!({
        "data": items,
        "total": total,
        "page": page,
        "perPage": per_page,
    }))
}

#[debug_handler]
async fn list_achievements(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;
    let user_ach = user_achievements::Entity::find()
        .filter(user_achievements::Column::UserId.eq(user_id))
        .all(&*db)
        .await?;
    let unlocked: std::collections::HashMap<_, _> = user_ach.iter()
        .map(|ua| (ua.achievement_id.clone(), ua))
        .collect();
    let items: Vec<serde_json::Value> = game_achievements::DEFS.iter().map(|def| {
        let ua = unlocked.get(&def.id.to_string());
        serde_json::json!({
            "id": def.id,
            "code": def.code,
            "title": def.title,
            "description": def.description,
            "icon": def.icon,
            "xp_reward": def.xp_reward,
            "gem_reward": def.gem_reward,
            "tier": def.tier,
            "unlocked": ua.map(|u| u.unlocked_at.is_some()).unwrap_or(false),
            "progress": ua.map(|u| u.progress).unwrap_or(0),
            "is_claimed": ua.map(|u| u.is_claimed).unwrap_or(false),
        })
    }).collect();
    format::json(items)
}

#[debug_handler]
async fn claim_achievement(auth: auth::JWT, State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;
    let ua = user_achievements::Entity::find()
        .filter(user_achievements::Column::UserId.eq(user_id))
        .filter(user_achievements::Column::AchievementId.eq(id.clone()))
        .one(&*db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    if ua.unlocked_at.is_none() {
        return bad_request("not unlocked");
    }
    if ua.is_claimed {
        return bad_request("already claimed");
    }
    let mut ua_am = ua.into_active_model();
    ua_am.is_claimed = Set(true);
    ua_am.update(&*db).await.map_err(|e| Error::string(&e.to_string()))?;
    let def = game_achievements::DEFS.iter().find(|d| d.id.to_string() == id).ok_or_else(|| Error::NotFound)?;
    let mut village = load_village(&*db, user_id).await?.into_active_model();
    add_gold(&mut village, def.xp_reward as f64);
    add_gems(&mut village, def.gem_reward);
    village.updated_at = Set(Some(Utc::now().to_rfc3339()));
    village.update(&*db).await.map_err(|e| Error::string(&e.to_string()))?;
    format::json(serde_json::json!({ "claimed": true, "rewards": { "gold": def.xp_reward, "gems": def.gem_reward } }))
}

#[debug_handler]
async fn leaderboard(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let all = villages::Entity::find().all(&*db).await?;
    let mut sorted: Vec<_> = all.into_iter().map(|v| {
        let trophy = v.trophy;
        (trophy, v)
    }).collect();
    sorted.sort_by(|a, b| b.0.cmp(&a.0));
    let top = sorted.into_iter().take(50).enumerate().map(|(i, (t, v))| serde_json::json!({
        "rank": i + 1,
        "village_id": v.id,
        "name": v.name,
        "level": v.level,
        "trophy": t,
    })).collect::<Vec<_>>();
    format::json(top)
}

#[debug_handler]
async fn stats(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;
    let s = game_stats(&*db, user_id).await?;
    format::json(s)
}

// ============ ANALYSIS ENDPOINT ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    /// comma-separated, e.g. "housing,food" (axum query can't deserialize seqs)
    pub categories: Option<String>,
    pub entity_types: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetWorthBreakdown {
    pub banks: f64,
    pub assets: f64,
    pub fixed_deposits: f64,
    pub investments: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyIncomeByType {
    pub salary: f64,
    pub business: f64,
    pub investment: f64,
    pub other: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyExpenseByCategory {
    pub month: String,
    pub category: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROICalculation {
    pub total_invested: f64,
    pub total_current_value: f64,
    pub total_gain_loss: f64,
    pub roi_percentage: f64,
    pub by_type: Vec<InvestmentTypeROI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentTypeROI {
    pub investment_type: String,
    pub invested: f64,
    pub current_value: f64,
    pub gain_loss: f64,
    pub roi_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowPoint {
    pub month: String,
    pub income: f64,
    pub expense: f64,
    pub net: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopItem {
    pub label: String,
    pub amount: f64,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FDMaturity {
    pub id: String,
    pub name: String,
    pub principal: f64,
    pub current_value: f64,
    pub maturity_date: String,
    pub days_to_maturity: i64,
    pub interest_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditCardUtilization {
    pub card_name: String,
    pub limit: f64,
    pub balance: f64,
    pub available: f64,
    pub utilization_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsRate {
    pub total_income: f64,
    pub total_expense: f64,
    pub savings: f64,
    pub savings_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub net_worth: NetWorthBreakdown,
    pub yearly_income_by_type: YearlyIncomeByType,
    pub monthly_expenses: Vec<MonthlyExpenseByCategory>,
    pub roi: ROICalculation,
    pub cash_flow: Vec<CashFlowPoint>,
    pub top_5_expenses: Vec<TopItem>,
    pub top_5_income_sources: Vec<TopItem>,
    pub investment_performance: Vec<InvestmentTypeROI>,
    pub fd_maturity_timeline: Vec<FDMaturity>,
    pub credit_card_utilization: Vec<CreditCardUtilization>,
    pub savings_rate: SavingsRate,
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn month_key(date: &str) -> String {
    if date.len() >= 7 {
        date[..7].to_string()
    } else {
        "".to_string()
    }
}

async fn fetch_incomes(
    db: &sea_orm_turso::TursoConnection,
    user_id: i64,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Vec<incomes::Model>> {
    let mut q = incomes::Entity::find().filter(incomes::Column::UserId.eq(user_id));
    if let Some(s) = start {
        q = q.filter(incomes::Column::IncomeDate.gte(s));
    }
    if let Some(e) = end {
        q = q.filter(incomes::Column::IncomeDate.lte(e));
    }
    q.all(db).await.map_err(|e| Error::string(&e.to_string()))
}

async fn fetch_expenses(
    db: &sea_orm_turso::TursoConnection,
    user_id: i64,
    start: Option<&str>,
    end: Option<&str>,
    categories: Option<&Vec<String>>,
) -> Result<Vec<expenses::Model>> {
    let mut q = expenses::Entity::find().filter(expenses::Column::UserId.eq(user_id));
    if let Some(s) = start {
        q = q.filter(expenses::Column::ExpenseDate.gte(s));
    }
    if let Some(e) = end {
        q = q.filter(expenses::Column::ExpenseDate.lte(e));
    }
    if let Some(cats) = categories {
        if !cats.is_empty() {
            q = q.filter(expenses::Column::Category.is_in(cats.clone()));
        }
    }
    q.all(db).await.map_err(|e| Error::string(&e.to_string()))
}

#[debug_handler]
async fn analysis(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<AnalysisRequest>,
) -> Result<Response> {
    let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();
    let user_id = uid(&ctx, &auth).await?;

    let start = params.start_date.as_deref();
    let end = params.end_date.as_deref();
    let cats: Option<Vec<String>> = match params.categories.as_deref() {
        Some(raw) => {
            let v: Vec<String> = raw
                .split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(str::to_string)
                .collect();
            if v.is_empty() { None } else { Some(v) }
        }
        None => None,
    };

    // Fetch all data in parallel
    let (banks_list, assets_list, fds_list, investments_list, incomes_list, expenses_list, credit_cards_list) = tokio::join!(
        banks::Entity::find().filter(banks::Column::UserId.eq(user_id)).all(&*db),
        assets::Entity::find().filter(assets::Column::UserId.eq(user_id)).all(&*db),
        fixed_deposits::Entity::find().filter(fixed_deposits::Column::UserId.eq(user_id)).all(&*db),
        investments::Entity::find().filter(investments::Column::UserId.eq(user_id)).all(&*db),
        fetch_incomes(&*db, user_id, start, end),
        fetch_expenses(&*db, user_id, start, end, cats.as_ref()),
        credit_cards::Entity::find().filter(credit_cards::Column::UserId.eq(user_id)).all(&*db),
    );

    let banks = banks_list?;
    let assets = assets_list?;
    let fds = fds_list?;
    let investments = investments_list?;
    let incomes = incomes_list?;
    let expenses = expenses_list?;
    let credit_cards = credit_cards_list?;

    // Net worth breakdown
    let village = load_village(&*db, user_id).await?;
    let bank_balance: f64 = banks.iter().map(|b| b.current_balance).sum();
    let asset_value: f64 = assets.iter().map(|a| a.current_value).sum();
    let fd_value: f64 = fds.iter().map(|f| f.current_value).sum();
    let inv_value: f64 = investments.iter().map(|i| i.current_value).sum();
    let net_worth = NetWorthBreakdown {
        banks: bank_balance,
        assets: asset_value,
        fixed_deposits: fd_value,
        investments: inv_value,
        total: bank_balance + asset_value + fd_value + inv_value + village.gold,
    };

    // Yearly income by type
    let mut salary = 0.0;
    let mut business = 0.0;
    let mut investment_income = 0.0;
    let mut other_income = 0.0;
    for inc in &incomes {
        let annualized = if inc.is_recurring {
            match inc.recurrence.as_deref() {
                Some("weekly") => inc.amount * inc.frequency_multiplier as f64 * 52.0,
                Some("biweekly") | Some("fortnightly") => inc.amount * inc.frequency_multiplier as f64 * 26.0,
                Some("monthly") => inc.amount * inc.frequency_multiplier as f64 * 12.0,
                Some("quarterly") => inc.amount * inc.frequency_multiplier as f64 * 4.0,
                Some("yearly") | Some("annually") => inc.amount * inc.frequency_multiplier as f64,
                _ => inc.amount * inc.frequency_multiplier.max(1) as f64,
            }
        } else {
            inc.amount * inc.frequency_multiplier.max(1) as f64
        };
        match inc.income_type.to_lowercase().as_str() {
            "salary" => salary += annualized,
            "business" => business += annualized,
            "investment" => investment_income += annualized,
            _ => other_income += annualized,
        }
    }
    let yearly_income_by_type = YearlyIncomeByType {
        salary,
        business,
        investment: investment_income,
        other: other_income,
        total: salary + business + investment_income + other_income,
    };

    // Monthly expense by category
    let mut monthly_expense_map: std::collections::HashMap<String, std::collections::HashMap<String, f64>> = std::collections::HashMap::new();
    for exp in &expenses {
        let month = month_key(&exp.expense_date);
        if month.is_empty() { continue; }
        let entry = monthly_expense_map.entry(month).or_default();
        *entry.entry(exp.category.clone()).or_default() += exp.amount;
    }
    let mut monthly_expenses: Vec<MonthlyExpenseByCategory> = Vec::new();
    for (month, cats) in monthly_expense_map {
        for (category, amount) in cats {
            monthly_expenses.push(MonthlyExpenseByCategory { month: month.clone(), category, amount });
        }
    }
    monthly_expenses.sort_by(|a, b| a.month.cmp(&b.month).then(a.category.cmp(&b.category)));

    // ROI calculations
    let total_invested: f64 = investments.iter().map(|i| i.invested_amount).sum();
    let total_current_value: f64 = investments.iter().map(|i| i.current_value).sum();
    let total_gain_loss = total_current_value - total_invested;
    let roi_percentage = if total_invested > 0.0 { (total_gain_loss / total_invested) * 100.0 } else { 0.0 };

    let mut by_type_map: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();
    for inv in &investments {
        let entry = by_type_map.entry(inv.investment_type.clone()).or_default();
        entry.0 += inv.invested_amount;
        entry.1 += inv.current_value;
    }
    let by_type: Vec<InvestmentTypeROI> = by_type_map
        .into_iter()
        .map(|(investment_type, (invested, current_value))| {
            let gain_loss = current_value - invested;
            let roi_pct = if invested > 0.0 { (gain_loss / invested) * 100.0 } else { 0.0 };
            InvestmentTypeROI { investment_type, invested, current_value, gain_loss, roi_pct }
        })
        .collect();

    let roi = ROICalculation {
        total_invested,
        total_current_value,
        total_gain_loss,
        roi_percentage,
        by_type,
    };

    // Cash flow monthly trend
    let mut income_by_month: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut expense_by_month: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for inc in &incomes {
        let month = month_key(&inc.income_date);
        if !month.is_empty() {
            let net = inc.amount - inc.tax_withheld;
            *income_by_month.entry(month).or_default() += net;
        }
    }
    for exp in &expenses {
        let month = month_key(&exp.expense_date);
        if !month.is_empty() {
            *expense_by_month.entry(month).or_default() += exp.amount;
        }
    }
    let mut all_months: std::collections::HashSet<String> = std::collections::HashSet::new();
    all_months.extend(income_by_month.keys().cloned());
    all_months.extend(expense_by_month.keys().cloned());
    let mut cash_flow: Vec<CashFlowPoint> = all_months
        .into_iter()
        .map(|month| {
            let income = income_by_month.get(&month).copied().unwrap_or(0.0);
            let expense = expense_by_month.get(&month).copied().unwrap_or(0.0);
            CashFlowPoint { month, income, expense, net: income - expense }
        })
        .collect();
    cash_flow.sort_by(|a, b| a.month.cmp(&b.month));

    // Top 5 expenses
    let mut expense_by_title: std::collections::HashMap<String, (f64, String)> = std::collections::HashMap::new();
    for exp in &expenses {
        let e = expense_by_title.entry(exp.title.clone()).or_default(); e.0 += exp.amount;
        expense_by_title.get_mut(&exp.title).unwrap().1 = exp.category.clone();
    }
    let mut top_expenses: Vec<_> = expense_by_title
        .into_iter()
        .map(|(label, (amount, category))| TopItem { label, amount, category: Some(category) })
        .collect();
    top_expenses.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());
    let top_5_expenses = top_expenses.into_iter().take(5).collect();

    // Top 5 income sources
    let mut income_by_source: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for inc in &incomes {
        *income_by_source.entry(inc.source.clone()).or_default() += inc.amount - inc.tax_withheld;
    }
    let mut top_income: Vec<_> = income_by_source
        .into_iter()
        .map(|(label, amount)| TopItem { label, amount, category: None })
        .collect();
    top_income.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());
    let top_5_income_sources = top_income.into_iter().take(5).collect();

    // Investment performance (gain/loss by type)
    let investment_performance = roi.by_type.clone();

    // FD maturity timeline
    let fd_maturity_timeline: Vec<FDMaturity> = fds
        .into_iter()
        .map(|f| {
            let today = chrono::Utc::now().date_naive();
            let days_to_maturity = chrono::NaiveDate::parse_from_str(&f.maturity_date, "%Y-%m-%d")
                .map(|d| (d - today).num_days())
                .unwrap_or(0);
            FDMaturity {
                id: f.id,
                name: f.name,
                principal: f.principal_amount,
                current_value: f.current_value,
                maturity_date: f.maturity_date,
                days_to_maturity,
                interest_rate: f.interest_rate,
            }
        })
        .collect();

    // Credit card utilization
    let credit_card_utilization: Vec<CreditCardUtilization> = credit_cards
        .into_iter()
        .map(|c| {
            let utilization_pct = if c.credit_limit > 0.0 {
                (c.current_balance / c.credit_limit) * 100.0
            } else { 0.0 };
            CreditCardUtilization {
                card_name: c.name,
                limit: c.credit_limit,
                balance: c.current_balance,
                available: c.available_credit,
                utilization_pct,
            }
        })
        .collect();

    // Savings rate
    let total_income: f64 = incomes.iter().map(|i| i.amount - i.tax_withheld).sum();
    let total_expense: f64 = expenses.iter().map(|e| e.amount).sum();
    let savings = total_income - total_expense;
    let savings_rate_pct = if total_income > 0.0 { (savings / total_income) * 100.0 } else { 0.0 };
    let savings_rate = SavingsRate {
        total_income,
        total_expense,
        savings,
        savings_rate_pct,
    };

    let response = AnalysisResponse {
        net_worth,
        yearly_income_by_type,
        monthly_expenses,
        roi,
        cash_flow,
        top_5_expenses,
        top_5_income_sources,
        investment_performance,
        fd_maturity_timeline,
        credit_card_utilization,
        savings_rate,
    };

    format::json(response)
}
