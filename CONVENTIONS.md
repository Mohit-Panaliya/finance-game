# Conventions — Finance Game (Loco + sea-orm-turso + Vue Ionic PWA)

CRITICAL adapter rules (from sea-orm-turso) — violating these breaks runtime:

1. ALL integer entity fields MUST be `i64` (never i32/i64 auto-inc as other types). Adapter returns BigInt for every SQLite integer.
2. Boolean fields MUST be `bool` with column names matching heuristic: `is_*`, `has_*`, `active`, `enabled` etc.
3. ALL required fields MUST be explicitly `Set()` in ActiveModel — Turso does NOT apply SQL DEFAULT for NotSet.
4. Timestamps: use `Option<String>` (RFC3339), NEVER DateTime<Utc> — adapter returns Text as String only.
5. Date fields (date-only): `Option<String>` as "YYYY-MM-DD" — safest with this adapter.
6. Primary keys: `i64` auto_increment for server tables. For offline-first game tables use `String` UUID (text, set client-side or with uuid::Uuid::new_v4().to_string()).
7. Enums: store as `String` (not DeriveActiveEnum with db_type=Enum — SQLite treats as text anyway; String is safest).
8. JSON arrays: `Vec<String>` via sea-orm with-json (serializes to TEXT).
9. `DbErr` does not implement Serialize — always `.to_string()` before JSON responses.
10. No RETURNING support — insert then re-fetch if needed.

Loco pattern:
- DB access in handlers: `let db = ctx.shared_store.get_ref::<sea_orm_turso::TursoConnection>().unwrap();`
- Then `Entity::find()...one(&*db).await?` / `active.insert(&*db).await?`
- Routes: axum 0.8 style `/{id}`, prefix `/api/...`
- Handlers: `#[debug_handler]`, return `Result<Response>`, use `format::json(...)`, `not_found()`, `bad_request(...)`, `unauthorized(...)`
- Auth: `auth::auth::JWT` extractor → `auth.claims.pid` (user id as string)
- JWT from config `ctx.config.get_jwt_config()?`

Project layout:
```
Cargo.toml          — workspace + app
config/*.yaml       — development/production/test
migration/          — sea-orm-migration crate
src/
  app.rs            — impl Hooks (routes, initializers)
  lib.rs            — modules
  bin/main.rs       — cli::main
  initializers/turso_db.rs — inserts TursoConnection into shared_store
  models/           — entities + request/response DTOs
  controllers/      — HTTP handlers
  game_engine/      — game logic (XP, buildings, battles, achievements)
  dtos/             — shared DTOs
frontend/           — Vue 3 + Ionic + Vite PWA (served from frontend/dist)
```

Finance entities: users, banks, assets, expenses, credit_cards, fixed_deposits, investments, incomes
Game entities: villages, buildings, troops, resources, achievements, user_achievements, battles, sync_log

API surface (all under /api):
- auth: login, register, me, logout
- finance: banks, assets, expenses, credit-cards, fds, investments, incomes (CRUD + summary)
- game: village, buildings (upgrade/collect), troops (train), battle (raid), achievements, leaderboard, stats
- sync: POST /api/sync/push, GET /api/sync/pull (delta by timestamp)

Frontend rules:
- NO native <input type="date">, <select>, <dialog> — replace with game components:
  - GameDatePicker → 3D wheel/roller picker (day/month/year) with gold frame
  - GameSelect → gem/chest styled dropdown sheet with animated open
  - GameModal → full-screen chest/panel modal with backdrop blur, scale+fade in
- Everything animated: CSS keyframes, transforms, particle sparks, floating numbers
- Clash-of-Clans aesthetic: isometric-ish village, thick outlines, saturated colors, bouncy easing

## Current State (ship)

As-built rules for the shipped build — match these in any new code:

- **User id in handlers**: `let uid = super::uid(&ctx, &auth).await?;` — shared async helper in `controllers/mod.rs` that maps the JWT UUID `pid` to `users.id` (i64). Never read `auth.claims.pid` as an id directly.
- **DB access**: `let db = ctx.shared_store.get_ref::<TursoConnection>().unwrap();` then pass `&*db` (the guard is not `ConnectionTrait`; deref to the inner connection).
- **Optional model fields**: wrap `Some(v)` in `Set()`; required fields always `Set(...)` (Turso applies no SQL DEFAULT for `NotSet`).
- **Timestamps / dates**: RFC3339 `String` (`Option<String>` when nullable); date-only fields `"YYYY-MM-DD"`. No `DateTime<Utc>`.
- **Frontend inputs**: no native date/select controls — use `GameDatePicker` and `GameSelect` (see Frontend rules above).
- **Orientation**: landscape-first PWA; portrait is blocked by a rotate-device overlay.
