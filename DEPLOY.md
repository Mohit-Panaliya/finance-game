# Production deployment — finance-game

Single release binary `finance-game-cli`: Loco 1.1 API + frontend embedded via
`rust-embed` (`src/embedded.rs`) and served by the router fallback
`serve_embedded_assets` (`src/app.rs`). No Node runtime needed at serve time.

## deploy.sh

```bash
./deploy.sh              # build frontend + release, start detached (default PORT=5155)
./deploy.sh build        # build only: frontend/npm run build + cargo build --release
./deploy.sh start        # same as default
./deploy.sh stop         # SIGTERM (then SIGKILL after 10s) using finance-game.pid
./deploy.sh status       # show pidfile / process state
./deploy.sh restart      # stop, rebuild, start
PORT=8080 ./deploy.sh    # custom port
```

- Detached via `nohup ... start -e production -b $BINDING -p $PORT &`
- PID written to `./finance-game.pid`; printed on start
- Log: `/var/log/finance-game.log` when writable, else `./finance-game.log`
- Start polls `GET /_health` for up to 15s and reports OK / failure
- Refuses to start twice (live pidfile check)

## config/production.yaml

Boots standalone — no env var is strictly required:

| Key | Value | Env override |
|---|---|---|
| `auth.jwt.secret` | generated base64 default (44 chars / 32 bytes) | `JWT_SECRET` |
| `logger` | `level: info`, `format: compact`, `pretty_backtrace: false` | `LOG_LEVEL` |
| `server.binding` | `0.0.0.0` | `BINDING` |
| `server.port` | `5155` | `PORT` |
| `server.host` | `http://localhost` | `HOST` |
| `database.uri` | `sqlite://finance_game_production.sqlite?mode=rwc` | `DATABASE_URL` |
| `workers` | `mode: BackgroundAsync`, `enable: false` | — |

Middleware notes (matches `config/development.yaml`):

- `server.middlewares.fallback.enable: false` — required, otherwise Loco's own
  404 page middleware would replace the embedded-asset router fallback.
- `server.middlewares.static` enabled with `must_exist: false`,
  `precompressed: false`, `folder.uri: "/"`. Static is a disk fast-path only;
  `Hooks::after_routes` runs after middleware application and re-registers
  `router.fallback(serve_embedded_assets)`, so SPA deep links are always
  served from the binary even when `frontend/dist` is absent.
- `set_cors` intentionally omitted (production is same-origin).
- `workers.enable`, `seeder`, `tuning` are accepted-but-ignored keys in the
  Loco 1.1 schema (serde ignores unknown fields); kept for parity with dev.

First boot auto-migrates `finance_game_production.sqlite` (relative to cwd —
always start the server from the project/deploy directory).

## Verified (2026-09-26)

```
$ cargo build --release            # 0 errors (20 pre-existing dead-code warnings)
$ timeout 10 ./target/release/finance-game-cli start -e production -p 5199
environment: production
   database: automigrate
     logger: info
listening on http://0.0.0.0:5199

$ curl http://127.0.0.1:5199/_health   -> {"ok":true}
$ curl http://127.0.0.1:5199/          -> index.html (embedded)
$ curl http://127.0.0.1:5199/village   -> index.html (SPA deep link)
$ curl http://127.0.0.1:5199/api/nope  -> 404
```

Process was killed by `timeout` (graceful "shutting down..."), port 5199 closed
afterwards; no processes left running.

## Smoke test — `scripts/smoke.sh` (2026-09-26)

Read-only end-to-end check against an **already running** server (never
starts/stops or reconfigures it):

```
BASE_URL=http://127.0.0.1:5155 ./scripts/smoke.sh             # 33 checks
EXPECT_BANK='SmokeBank-smoke-<ts>' ./scripts/smoke.sh --persist  # + 4f
```

Covers: `/_health`, register/login/me (JWT Bearer), bank CRUD
(create `.id` / `.total>=1` / PUT `current_balance==99`), income + expense
create, game (village `.village.id`, building collect, battle `.rewards`,
analysis `.net_worth.total` number, achievements `>=10`), sync push `.ok==true`
+ pull, frontend `/` + `/analysis` HTML containing `Finance Forge`,
`/api/nope` -> 404.

`--persist` only prints a reminder (script does not restart the server).
Persistence proof = run twice against a restarted server, exporting
`EXPECT_BANK=<first run's bank name>` on run 2; check `4f` re-logs in as the
first-run user (email is recovered from the bank name prefix) and asserts
`GET /api/banks` still lists that bank. Bank name is unique per run:
`SmokeBank-<email prefix>` where email = `smoke-$(date +%s)@t.com`.

Exit code: 0 if every check PASS, 1 otherwise (summary line prints counts).

Verified 2026-09-26 against dev server :5155 — run 1: 33/33 PASS,
run 2 (`--persist` + `EXPECT_BANK`): 36/36 PASS.

## Live production run — 2026-09-26 (port 5156)

Dev server held 5155, so production was deployed on 5156:

```
$ PORT=5156 ./deploy.sh          # build + detached start, health poll OK
$ ss -ltnp | grep 5156           # 0.0.0.0:5156  (server.binding = 0.0.0.0)
$ curl http://127.0.0.1:5156/_health           -> {"ok":true}  200
$ curl http://172.19.72.91:5156/_health        -> {"ok":true}  200 (external)
$ BASE_URL=http://127.0.0.1:5156 ./scripts/smoke.sh   -> ALL 33 PASSED
```

- URL: `http://172.19.72.91:5156` (BINDING `0.0.0.0` in `config/production.yaml`)
- pid: pidfile `finance-game.pid`; log: `finance-game.log` in app dir
  (`/var/log/finance-game.log` only when that dir is writable)
- **Corrupt production DB recovery**: `finance_game_production.sqlite` had been
  left malformed (`PRAGMA integrity_check` -> "database disk image is malformed";
  register 500, login 401). Fix: `./deploy.sh stop`, move the file (and any
  `-wal`/`-shm`) aside to `corrupt-backup-<ts>.…`, `./deploy.sh` again ->
  automigrate creates a fresh schema. Backups kept in the app dir.
- A concurrent `./deploy.sh restart` (or `pkill`) from another workstream stops
  the instance — re-check `./deploy.sh status` before trusting a smoke run.
