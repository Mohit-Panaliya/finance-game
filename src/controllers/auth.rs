use axum::http::HeaderValue;
use loco_rs::prelude::*;
use sea_orm::EntityTrait;

use crate::models::users;
use crate::models::users::{LoginParams, RegisterParams};
use crate::views::auth::{CurrentResponse, LoginResponse};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/auth")
        .add("/login", post(login))
        .add("/register", post(register))
        .add("/me", get(current))
        .add("/logout", post(logout))
}

fn set_auth_cookie(resp: &mut Response, token: &str, max_age: u64) {
    let cookie = format!("token={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}");
    if let Ok(val) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert("set-cookie", val);
    }
}

#[debug_handler]
async fn login(State(ctx): State<AppContext>, Json(params): Json<LoginParams>) -> Result<Response> {
    let Ok(user) = users::Model::find_by_username(&ctx.db, &params.username).await else {
        return unauthorized("Invalid credentials!");
    };
    if !user.verify_password(&params.password) {
        return unauthorized("Invalid credentials!");
    }
    let jwt_config = ctx.config.get_jwt_config()?;
    let token = user.generate_jwt(&jwt_config.secret, jwt_config.expiration)?;

    let mut resp = format::json(LoginResponse::new(users::UserResponse::from(user), &token))?;
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
    // Sign the new user straight in: registering used to return an empty token,
    // which left the client unauthenticated and bounced it back to the login page.
    let jwt_config = ctx.config.get_jwt_config()?;
    let token = user.generate_jwt(&jwt_config.secret, jwt_config.expiration)?;
    let mut resp = format::json(LoginResponse::new(users::UserResponse::from(user), &token))?;
    set_auth_cookie(
        &mut resp,
        &token,
        jwt_config.expiration.saturating_mul(86_400),
    );
    Ok(resp)
}

#[debug_handler]
async fn current(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user_id = super::uid(&ctx, &auth).await?;
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
