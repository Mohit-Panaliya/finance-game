# Finance Forge

**Gamified personal-finance PWA — Clash of Clans style.**  
Your village = your net worth. Banks/assets = buildings. Expenses = troops. Incomes = gold mines. Battles = raids.

[Live](https://game.brightive.shop) · [API](https://game.brightive.shop/_health) · [GitHub](https://github.com/Mohit-Panaliya/finance-game)

---

## Stack

| Layer | Tech |
|-------|------|
| Backend | Rust **Loco 1.1** (axum) + **SeaORM 2.0** |
| Database | **libsql** (Turso local) via `sea-orm-turso` crate — single SQLite file, WAL mode |
| Frontend | **Vue 3 + Ionic 8 + Vite PWA** (Pinia, service worker, IndexedDB offline queue) |
| Deploy | Single release binary (`rust-embed` serves `frontend/dist`) + **Caddy** (auto HTTPS) |
| Infra | Oracle Cloud Free Tier (1/8 OCPU, 1 GB RAM, Ubuntu 24.04) |

---

## Quickstart

### Development
```bash
# Backend
cargo build --release
./target/release/finance-game-cli start -e development -p 5155

# Frontend (hot reload)
cd frontend && npm install && npm run dev   # Vite :5173, proxies /api → :8000
```

### Production Build
```bash
cd frontend && npm run build   # → frontend/dist
cargo build --release          # embeds dist, outputs target/release/finance-game-cli
```

### One-line Deploy (on fresh Ubuntu)
```bash
# Build locally, scp binary + config, run as systemd on :8080, Caddy on :443
```

---

## Features

### Core Game Loop
- **Village** — net worth dashboard (gold = liquid cash, gems = investments)
- **Buildings** — Banks, Assets, Fixed Deposits, Investments, Credit Cards
  - Collect income (interest/dividends)
  - Upgrade (increase limits/APY)
- **Troops** — Expense categories (Food, Transport, Housing, etc.)
  - Train = log expense
- **Battles** — Raid enemy villages (simulated), earn gold/xp
- **Achievements** — 15 tiers (First Bank, Saver, Investor, Raider…)
- **Leaderboard** — Global net worth ranking

### Finance CRUD (7 entities)
`banks` · `assets` · `credit-cards` · `expenses` · `fixed-deposits` · `investments` · `incomes`  
Each: list, create, read, update, delete, summary

### Analysis Screen (Clash-style)
- Net worth breakdown (banks, assets, FDs, investments, gold)
- Yearly income by type (salary, business, investment, other)
- Monthly expenses by category
- ROI % by investment type
- Cash flow waterfall
- Top-5 expenses / income sources
- Investment performance, FD maturity timeline, credit utilization
- Savings rate %

### Offline-First PWA
- Service worker caches shell + assets
- IndexedDB queues mutations (create/update/delete) when offline
- Background sync on reconnect (`/api/sync/push` + `/pull`)
- Landscape-first (portrait shows rotate overlay)
- Installable: Chrome "Install app" → standalone APK-like experience

---

## API Map (all under `/api`)

| Group | Routes |
|-------|--------|
| **Auth** | `POST /register` `POST /login` `GET /me` `POST /logout` |
| **Finance ×7** | `GET /` `POST /` `GET /:id` `PUT /:id` `DELETE /:id` `GET /summary` |
| **Game** | `GET /village` `POST /buildings/collect` `POST /buildings/upgrade` `GET /troops` `POST /troops/train` `POST /battle` `GET /battles` `GET /achievements` `POST /achievements/:id/claim` `GET /leaderboard` `GET /stats` `GET /analysis` |
| **Sync** | `POST /push` `GET /pull?since=` |
| **Health** | `GET /_health` |

All protected routes use **JWT Bearer** (or cookie). User resolved via `uid()` (JWT `pid` → `users.id`).

---

## Data Model (16 tables)

| Table | Purpose |
|-------|---------|
| `users` | Loco auth (email, password hash, pid UUID) |
| `banks` | Savings/checking accounts |
| `assets` | Real estate, vehicles, gold, crypto |
| `credit_cards` | Limits, balances, due dates |
| `expenses` | Categorized spending |
| `fixed_deposits` | Term deposits with maturity |
| `investments` | Stocks, MFs, bonds with qty/price |
| `incomes` | Salary, business, dividends (recurring support) |
| `villages` | Per-user game state (gold, gems, xp, level) |
| `buildings` | Village buildings (type, level, production) |
| `troops` | Trained expense-troops |
| `battles` | Battle logs (rewards, enemy snapshot) |
| `user_achievements` | Progress per achievement |
| `achievements` | Static definitions (15) |
| `sync_log` | Offline mutation log (timestamp, entity, op, payload) |
| `seaql_migrations` | Migration history |

---

## Deployment (Current)

| Item | Value |
|------|-------|
| **Domain** | `game.brightive.shop` (A → 129.154.251.25) |
| **TLS** | Caddy (Let's Encrypt, auto-renew) |
| **App** | systemd `finance-game` on `:8080` |
| **Proxy** | Caddy `reverse_proxy localhost:8080` |
| **DB** | `/home/ubuntu/finance_game_production.sqlite` (auto-migrate) |
| **JWT** | Rotated on deploy (base64, 32 bytes) |
| **Logs** | `journalctl -u finance-game -f` |

```bash
# On server
sudo systemctl status finance-game caddy
curl https://game.brightive.shop/_health
```

---

## Android / iOS

**PWA — no store needed.**  
Open https://game.brightive.shop in Chrome/Safari → menu → **"Install app"** / **"Add to Home Screen"**.  
Runs full-screen, landscape, offline.

---

## Resource Footprint (OCI Free Tier)

| Metric | Value |
|--------|-------|
| RAM | ~65 MB (app 48 MB + Caddy 12 MB) |
| CPU | <1% idle, <5% under load |
| Disk | ~100 MB (binary 50 MB + DB + config) |
| Network | ~1 KB/request |

---

## Project Structure

```
finance-game/
├── src/
│   ├── controllers/        # 9 controllers (auth, banks, assets, cards, expenses, fds, investments, incomes, game)
│   ├── models/             # SeaORM entities + active models
│   ├── game_engine/        # Combat, building, troop logic
│   ├── initializers/       # Turso DB init + achievments seed
│   ├── views/              # API response serializers
│   ├── embedded.rs         # rust-embed (frontend/dist)
│   └── app.rs              # Router + fallback SPA handler
├── crates/sea-orm-turso/   # Local libsql adapter (RefGuard<Connection>)
├── migration/              # SeaORM migrations (16 tables)
├── config/*.yaml           # dev / prod / test
├── frontend/
│   ├── src/
│   │   ├── pages/          # Village, Battle, Records, Army, Achievements, Analysis
│   │   ├── components/game/  # GameDatePicker, GameSelect, SyncChip, panels
│   │   ├── stores/         # Pinia (auth, village, offline queue)
│   │   └── theme/game.css  # Clash palette, stone panels, landscape lock
│   └── vite.config.ts      # PWA manifest (landscape, fullscreen)
├── scripts/smoke.sh        # 33-check E2E test
├── deploy.sh               # Build + systemd deploy
├── DEPLOY.md               # Production runbook
├── CONVENTIONS.md          # Code patterns
└── TODO.md                 # Task tracker
```

---

## Conventions (Key)

- `ctx.shared_store.get_ref::<TursoConnection>()` → `&*db` (NOT `ConnectionTrait`)
- Optional fields: `Set(Some(v))`, required: `Set(v)`
- Timestamps: RFC3339 strings; dates: `YYYY-MM-DD`
- Integers: `i64`; enums: `String`
- No native `<select>`/`<input type=date>` → `GameSelect` / `GameDatePicker`
- Landscape lock: `screen.orientation.lock('landscape')` + CSS rotate overlay

---

## License

MIT — build your own village.
