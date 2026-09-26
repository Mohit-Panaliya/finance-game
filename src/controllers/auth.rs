use axum::http::HeaderValue;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::models::{
    game::{villages, buildings, troops, achievements, user_achievements},
    users,
    users::{LoginParams, RegisterParams},
};
use crate::views::auth::{CurrentResponse, LoginResponse};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/auth")
        .add("/login", post(login))
        .add("/register", post(register))
        .add("/me", get(current))
        .add("/logout", post(logout))
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

async fn seed_new_user_state(
    ctx: &AppContext,
    user_id: i64,
) -> Result<()> {
    let db = ctx
        .shared_store
        .get_ref::<sea_orm_turso::TursoConnection>()
        .unwrap();
    let now = chrono::Utc::now().to_rfc3339();
    let village_id = uuid::Uuid::new_v4().to_string();

    villages::ActiveModel {
        id: Set(village_id.clone()),
        user_id: Set(user_id),
        name: Set("My Fortress".to_string()),
        level: Set(1),
        xp: Set(0),
        xp_to_next: Set(100),
        gold: Set(500.0),
        gems: Set(20.0),
        elixir: Set(0.0),
        trophy: Set(0),
        shield_until: Set(None),
        last_tick_at: Set(Some(now.clone())),
        created_at: Set(Some(now.clone())),
        updated_at: Set(Some(now.clone())),
    }
    .insert(&*db)
    .await
    .map_err(|e| Error::string(&e.to_string()))?;

    let starter_buildings = [
        ("bank", "Bank Branch", 10.0, 100.0),
        ("gold_mine", "Gold Mine", 5.0, 50.0),
        ("vault", "Treasury Vault", 0.0, 200.0),
    ];
    for (kind, name, rate, hp) in starter_buildings {
        buildings::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            village_id: Set(village_id.clone()),
            user_id: Set(user_id),
            building_type: Set(kind.to_string()),
            source_kind: Set("manual".to_string()),
            source_id: Set(None),
            name: Set(name.to_string()),
            level: Set(1),
            grid_x: Set(0),
            grid_y: Set(0),
            hp: Set(hp as i64),
            max_hp: Set(hp as i64),
            production_rate: Set(rate),
            stored_resource: Set(0.0),
            last_collected_at: Set(Some(now.clone())),
            upgrade_cost: Set(100.0),
            is_upgrading: Set(false),
            upgrade_finishes_at: Set(None),
            created_at: Set(Some(now.clone())),
            updated_at: Set(Some(now.clone())),
        }
        .insert(&*db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    }

    let starter_troops = [
        ("raider", "Raider", 10, 20, 5, 10.0, 5),
        ("archer", "Archer", 8, 12, 3, 8.0, 4),
    ];
    for (troop_type, name, attack, hp, count, train_cost, train_time) in starter_troops {
        troops::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            user_id: Set(user_id),
            troop_type: Set(troop_type.to_string()),
            name: Set(name.to_string()),
            level: Set(1),
            attack: Set(attack),
            hp: Set(hp),
            count: Set(count),
            train_cost: Set(train_cost),
            train_time_secs: Set(train_time),
            training_finishes_at: Set(None),
            expense_id: Set(None),
            created_at: Set(Some(now.clone())),
            updated_at: Set(Some(now.clone())),
        }
        .insert(&*db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    }

    let defs = crate::game_engine::achievements::DEFS;
    for def in defs {
        let exists = user_achievements::Entity::find()
            .filter(user_achievements::Column::UserId.eq(user_id))
            .filter(user_achievements::Column::AchievementId.eq(def.id.to_string()))
            .one(&*db)
            .await
            .map_err(|e| Error::string(&e.to_string()))?;
        if exists.is_none() {
            user_achievements::ActiveModel {
                id: Set(uuid::Uuid::new_v4().to_string()),
                user_id: Set(user_id),
                achievement_id: Set(def.id.to_string()),
                unlocked_at: Set(None),
                progress: Set(0),
                is_claimed: Set(false),
            }
            .insert(&*db)
            .await
            .map_err(|e| Error::string(&e.to_string()))?;
        }
    }

    let _ = achievements::Entity::find().all(&*db).await;
    Ok(())
}

fn set_auth_cookie(resp: &mut Response, token: &str, max_age: u64) {
    let cookie = format!(
        "token={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}"
    );
    if let Ok(val) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert("set-cookie", val);
    }
}

#[debug_handler]
async fn login(
    State(ctx): State<AppContext>,
    Json(params): Json<LoginParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_username(&ctx.db, &params.username).await else {
        return unauthorized("Invalid credentials!");
    };
    if !user.verify_password(&params.password) {
        return unauthorized("Invalid credentials!");
    }
    let jwt_config = ctx.config.get_jwt_config()?;
    let token = user.generate_jwt(&jwt_config.secret, jwt_config.expiration)?;

    let mut resp = format::json(LoginResponse::new(
        users::UserResponse::from(user),
        &token,
    ))?;
    set_auth_cookie(
        &mut resp,
        &token,
        jwt_config.expiration.saturating_mul(86_400),
    );
    Ok(resp)
}

#[debug_handler]
async fn register(
    State(ctx): State<AppContext>,
    Json(params): Json<RegisterParams>,
) -> Result<Response> {
    let user = users::Model::create_with_password(&ctx.db, &params).await?;
    seed_new_user_state(&ctx, user.id).await?;
    format::json(LoginResponse::new(users::UserResponse::from(user), ""))
}

#[debug_handler]
async fn current(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user_id = uid(&ctx, &auth).await?;
    let user = users::Entity::find_by_id(user_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::string("user not found"))?;
    format::json(CurrentResponse::new(users::UserResponse::from(user)))
}

#[debug_handler]
async fn logout() -> Result<Response> {
    let mut resp = format::json(serde_json::json!({ "ok": true }))?;
    set_auth_cookie(&mut resp, "", 0);
    Ok(resp)
}
