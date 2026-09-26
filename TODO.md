# TODO.md

## finance-game data-persistence bug (2026-09-26)
[x] Fix: loco/sqlx + libsql(turso) no longer share one sqlite file/WAL
[x] Fix: loco pool idle_timeout no longer closes connections (WAL unlink stopped)
[x] Fix: turso whole-file fcntl lock disabled (LIMBO_DISABLE_FILE_LOCK) so external sqlite readers work
[x] Verify: cargo build --release clean, restart persistence OK, scripts/smoke.sh ALL checks PASSED

## ERP (Rust-in-memory-ERP) — status 2026-09-26
[x] 41/41 HTTP E2E flows (backend/scripts/e2e_flows.py)
[x] DB reopen panic fixed: schema_migrations tracking + 033 stock_bins DROP/CREATE rebuild
[x] $N UPDATE silent-no-op bug class fixed (18 sites: accounts, permissions, job_cards, naming)
[x] Benchmark (backend/scripts/benchmark.py): boot 620ms, idle 55MB/peak 194MB, flow 31ms, ~300 RPS reads
[x] Frontend: tsc 0, build ok, vitest 106/106; ui-ux-pro-max design system applied
[x] Pushed: Rust-in-memory-ERP@3ff9833
[ ] Docker deployment E2E
[ ] GL posting on invoice submit (currently 0 rows)
[ ] DB pool tuning (max_connections 5 limits read concurrency)
