# Finance Forge

Gamified personal-finance PWA — Clash-of-Clans style. Your **village is your net
worth**: banks/assets/fixed deposits are buildings you upgrade and collect from,
expenses are troops you train, incomes fund the war chest, and battles raid enemy
villages for gold. Offline-first, landscape-first, installable.

## Stack

- **Backend**: Rust [Loco 1.1](https://loco.rs) (axum) + SeaORM 2.0
- **DB**: one SQLite file via `crates/sea-orm-turso` (workspace crate, local libsql
  adapter) — no external database service, WAL mode
- **Frontend**: Vue 3 + Ionic 8 + Vite PWA (`vite-plugin-pwa`, Pinia)
- **Ship form**: a **single release binary** — `frontend/dist` embedded with
  `rust-embed` (`src/embedded.rs`) behind an SPA fallback (`src/app.rs`); no Node
  runtime needed at serve time

## Quickstart

Backend (dev, verified — release binary):

```bash
cargo build --release
./target/release/finance-game-cli start -e development -p 5155   # http://127.0.0.1:5155
```

Database auto-migrates on boot (`finance_game_development.sqlite`).

Frontend (hot reload):

```bash
cd frontend && npm install && npm run dev    # Vite :5173, proxies /api → 127.0.0.1:8000
```

Run the backend with `-p 8000` to match the proxy (or adjust `vite.config.ts`).

Full build (frontend must be built before it can be embedded):

```bash
cd frontend && npm run build   # → frontend/dist  (typecheck: npm run typecheck)
cargo build --release          # → target/release/finance-game-cli (embeds dist)
```

Production: `./deploy.sh` (build + detached start + `/_health` poll) — full
instructions in **[DEPLOY.md](DEPLOY.md)**.

## API map (all under `/api`)

| Group | Routes |
|---|---|
| **auth** | `POST /register`, `POST /login`, `GET /me`, `POST /logout` (JWT Bearer or cookie) |
| **finance ×7** | `banks`, `assets`, `credit-cards`, `expenses`, `fixed-deposits`, `investments`, `incomes` — each: `GET /` (list: `search`, `page`, `perPage`), `POST /`, `GET /{id}`, `PUT /{id}`, `DELETE /{id}`, `GET /summary` |
| **game** | `GET /village`, `POST /buildings/collect`, `POST /buildings/upgrade`, `GET /troops`, `POST /troops/train`, `POST /battle`, `GET /battles`, `GET /achievements`, `POST /achievements/{id}/claim`, `GET /leaderboard`, `GET /stats`, `GET /analysis` |
| **sync** | `POST /push`, `GET /pull?since=<RFC3339>` (delta by timestamp) |

Plus `GET /_health`. All authenticated routes resolve the user via
`super::uid(&ctx, &auth)` (JWT `pid` → `users.id`).

## Data model

One SQLite file per environment (`finance_game_{development,production}.sqlite`,
WAL), **16 tables**:

- **Finance**: `users`, `banks`, `assets`, `expenses`, `credit_cards`,
  `fixed_deposits`, `investments`, `incomes`
- **Game**: `villages`, `buildings`, `troops`, `achievements`,
  `user_achievements`, `battles`, `sync_log`
- **Bookkeeping**: `seaql_migrations` (migration state)

Schema lives in `migration/src/m20260922_000001_init.rs`; first boot auto-migrates.
Registration auto-provisions a village plus seeded buildings/troops.

## PWA

- **Landscape-first**: manifest `orientation: landscape` + `display: fullscreen`,
  portrait shows a rotate-device overlay, best-effort `screen.orientation.lock`
- **Offline**: mutations queue in IndexedDB (`frontend/src/services/offlineQueue.ts`),
  `syncStore` flushes via `/api/sync/push` and merges `/api/sync/pull`; service
  worker (vite-plugin-pwa) runtime-caches app assets
- **Installable**: manifest + SVG/PNG icons, theme `#1a0f00`

## Testing

`scripts/smoke.sh` — end-to-end smoke checks (health, auth, finance CRUD, game,
sync) against a running server, when present.

## Architecture

```
src/
  app.rs               — Hooks: route registration + embedded-asset SPA fallback
  controllers/         — HTTP handlers: auth, 7 finance resources, game, sync
  models/              — SeaORM entities + request/response DTOs
  game_engine/         — xp, economy, troops, battle, achievements, sync
  initializers/        — turso_db (TursoConnection → ctx.shared_store)
  embedded.rs          — rust-embed of frontend/dist
crates/sea-orm-turso/  — local libsql SeaORM driver (workspace crate)
migration/             — schema migrations
frontend/src/
  pages/               — Vue pages (Village, Army, Battle, Records, Analysis, …)
  components/game/     — GameModal, GameDatePicker, GameSelect, XpBar, ResourceBar, …
  stores/              — Pinia: auth, finance, game, sync
config/                — development / production / test YAML
deploy.sh              — production build + run (see DEPLOY.md)
```

## Docs

- [DEPLOY.md](DEPLOY.md) — production deployment (deploy.sh, production.yaml)
- [CONVENTIONS.md](CONVENTIONS.md) — adapter rules and code conventions
- `../../TODO.md` — build status / task board
