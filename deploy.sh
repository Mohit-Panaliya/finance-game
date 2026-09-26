#!/usr/bin/env bash
# finance-game production deploy helper (single release binary).
#
# Usage:
#   ./deploy.sh            # build frontend + release, start server detached
#   ./deploy.sh start      # same as above
#   ./deploy.sh build      # build only (no start)
#   ./deploy.sh stop       # stop the server started by this script
#   ./deploy.sh status     # show pidfile / process state
#   ./deploy.sh restart    # stop, rebuild, start
#
# Environment:
#   PORT          listen port                 (default 5155)
#   BINDING       listen address              (default 0.0.0.0)
#   HOST          public host for links       (default http://localhost)
#   JWT_SECRET    JWT secret (base64, 32+ b)  (default baked into config)
#   DATABASE_URL  sqlite uri                  (default sqlite://finance_game_production.sqlite?mode=rwc)
#   LOG_LEVEL     log level                   (default info)
set -euo pipefail

APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$APP_DIR/target/release/finance-game-cli"
PIDFILE="$APP_DIR/finance-game.pid"
PORT="${PORT:-5155}"
BINDING="${BINDING:-0.0.0.0}"

log_info() { printf '[deploy] %s\n' "$*"; }
die()      { printf '[deploy] ERROR: %s\n' "$*" >&2; exit 1; }

log_file() {
  if [ -d /var/log ] && [ -w /var/log ]; then
    echo "/var/log/finance-game.log"
  else
    echo "$APP_DIR/finance-game.log"
  fi
}

running_pid() {
  [ -f "$PIDFILE" ] || return 1
  local pid
  pid="$(cat "$PIDFILE" 2>/dev/null || true)"
  [ -n "$pid" ] || return 1
  kill -0 "$pid" 2>/dev/null || return 1
  echo "$pid"
}

build() {
  log_info "building frontend (frontend/ npm run build)"
  (cd "$APP_DIR/frontend" && npm run build)

  log_info "building release binary (cargo build --release)"
  (cd "$APP_DIR" && cargo build --release)

  [ -x "$BIN" ] || die "binary not found at $BIN"
  log_info "binary ready: $BIN"
}

start() {
  if pid="$(running_pid)"; then
    die "already running with PID $pid (run ./deploy.sh stop first)"
  fi
  [ -x "$BIN" ] || die "binary missing; run ./deploy.sh build first"
  if command -v ss >/dev/null 2>&1 && ss -ltn 2>/dev/null | grep -q ":$PORT "; then
    die "port $PORT is already in use (another server? try PORT=<other> ./deploy.sh)"
  fi
  rm -f "$PIDFILE"

  local log pid
  log="$(log_file)"

  cd "$APP_DIR"
  PORT="$PORT" BINDING="$BINDING" \
    nohup "$BIN" start -e production -b "$BINDING" -p "$PORT" >>"$log" 2>&1 &
  pid=$!
  echo "$pid" > "$PIDFILE"

  sleep 1
  if ! kill -0 "$pid" 2>/dev/null; then
    rm -f "$PIDFILE"
    die "server exited immediately; see $log"
  fi

  log_info "started"
  log_info "  pid:       $pid (pidfile: $PIDFILE)"
  log_info "  listening: http://$BINDING:$PORT"
  log_info "  health:    http://127.0.0.1:$PORT/_health"
  log_info "  log:       $log"

  if command -v curl >/dev/null 2>&1; then
    local i
    for i in $(seq 1 15); do
      if curl -fsS -m 2 "http://127.0.0.1:$PORT/_health" >/dev/null 2>&1; then
        log_info "health check: OK"
        return 0
      fi
      kill -0 "$pid" 2>/dev/null || die "server died during startup; see $log"
      sleep 1
    done
    log_info "health check: not OK yet (process is up; check $log)"
  fi
}

stop() {
  local pid
  if ! pid="$(running_pid)"; then
    log_info "not running (no live process in $PIDFILE)"
    rm -f "$PIDFILE"
    return 0
  fi
  log_info "stopping PID $pid"
  kill "$pid" 2>/dev/null || true
  local i
  for i in $(seq 1 10); do
    kill -0 "$pid" 2>/dev/null || break
    sleep 1
  done
  if kill -0 "$pid" 2>/dev/null; then
    log_info "still alive after 10s, sending SIGKILL"
    kill -9 "$pid" 2>/dev/null || true
    sleep 1
  fi
  rm -f "$PIDFILE"
  log_info "stopped"
}

status() {
  if pid="$(running_pid)"; then
    log_info "running: PID $pid"
    ps -o pid,etime,cmd -p "$pid" 2>/dev/null || true
  else
    log_info "not running"
    return 1
  fi
}

case "${1:-start}" in
  build)   build ;;
  start)   build; start ;;
  stop)    stop ;;
  restart) stop; build; start ;;
  status)  status ;;
  *)
    echo "usage: $0 {build|start|stop|restart|status}" >&2
    exit 2
    ;;
esac
