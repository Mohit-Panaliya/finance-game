# AGENTS.md — finance-game (Codespace working copy)

Loco (Rust) + Vue 3 + Ionic PWA. JWT auth, per-user data, Turso/libsql via
vendored crates, sqlx for the loco DB.

## Non-negotiables

- Do NOT commit, push, amend, or `git add` anything unless explicitly asked.
- NEVER `git add -f` anything under `docs/`, `AGENTS.md`, or `.opencode/`
  (skill installs, gitignored tooling).
- No DB triggers (Turso refuses them). NO CHECK constraints in migrations
  (turso_core 0.4.4 bails at DB-open) — enforce value limits in Rust.
- Money is f64, rounded to 2 decimals at every boundary (`round2`).
- Owner check (JWT user id) on every read/write path.
- Static routes register before `/{id}` param routes (first-match wins).
- Never build an update ActiveModel from bare `Model.into()` — all-`Unchanged`
  fields make SeaORM `UpdateOne` a silent noop. Set changed fields explicitly.
- `DATABASE_URL` must contain the `_loco` suffix; JWT_SECRET must be valid
  padded base64. `pkill -f` patterns must use the `[t]`-style self-match guard
  and run from script files, not inline ssh.

## Verification ladder (run after every batch, all must pass)

```
cargo fmt --all && cargo fmt --all -- --check
cargo check --quiet
cargo test --quiet --lib
cargo build --release
cd frontend && npx vue-tsc --noEmit && npm run build
bash scripts/smoke.sh http://127.0.0.1:8000   # against release server, ≥172/0
```

Extend `scripts/smoke.sh` with assertions for every new endpoint. Frontend work must
also honor theme tokens (no hardcoded colours/fonts), `--density-*` rhythm, and the
skill checklists in `.opencode/skills/` when doing UI work.
