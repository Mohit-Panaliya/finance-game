use std::borrow::Cow;

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{header, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
    Result,
};
use migration::Migrator;

use crate::{controllers, embedded, initializers::turso_db::TursoDb, tasks};

const INDEX_HTML: &str = "index.html";

fn content_type_for(path: &str) -> &'static str {
    let ext = path.rsplit_once('.').map(|(_, ext)| ext.to_ascii_lowercase());
    match ext.as_deref() {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") | Some("map") => "application/json",
        Some("webmanifest") => "application/manifest+json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("txt") => "text/plain; charset=utf-8",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}

fn asset_response(data: Cow<'static, [u8]>, content_type: &'static str, head: bool) -> Response {
    let body = if head {
        Body::empty()
    } else {
        match data {
            Cow::Borrowed(bytes) => Body::from(bytes),
            Cow::Owned(bytes) => Body::from(bytes),
        }
    };
    let mut response = Response::new(body);
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response
}

async fn serve_embedded_assets(method: Method, uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { INDEX_HTML } else { path };

    if path == "api" || path.starts_with("api/") {
        return (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"error":"not found"}"#,
        )
            .into_response();
    }

    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "method not allowed").into_response();
    }

    let head = method == Method::HEAD;

    if let Some(file) = embedded::Assets::get(path) {
        return asset_response(file.data, content_type_for(path), head);
    }

    if !path.contains('.') {
        if let Some(index) = embedded::Assets::get(INDEX_HTML) {
            return asset_response(index.data, "text/html; charset=utf-8", head);
        }
    }

    (StatusCode::NOT_FOUND, "Not Found").into_response()
}

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(TursoDb)])
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::auth::routes())
            .add_route(controllers::banks::routes())
            .add_route(controllers::assets::routes())
            .add_route(controllers::expenses::routes())
            .add_route(controllers::credit_cards::routes())
            .add_route(controllers::fixed_deposits::routes())
            .add_route(controllers::investments::routes())
            .add_route(controllers::incomes::routes())
            .add_route(controllers::game::routes())
            .add_route(controllers::sync::routes())
        // inject-routes-below (do not remove this comment)
    }

    async fn after_routes(router: axum::Router, _ctx: &AppContext) -> Result<axum::Router> {
        Ok(router.fallback(serve_embedded_assets))
    }

    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::seed_achievements::SeedAchievements);
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &loco_rs::bgworker::Queue) -> Result<()> {
        Ok(())
    }

    async fn truncate(_ctx: &AppContext) -> Result<()> {
        Ok(())
    }

    async fn seed(_ctx: &AppContext, _path: &std::path::Path) -> Result<()> {
        Ok(())
    }
}
