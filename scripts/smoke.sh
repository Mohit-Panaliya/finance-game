#!/usr/bin/env bash
# Finance Forge end-to-end smoke test (read-only against a RUNNING server).
#   usage: scripts/smoke.sh [--persist]
#   env:   BASE_URL   (default http://127.0.0.1:5155)
#          EXPECT_BANK  bank name from the FIRST run; asserts it survived a restart
# Does NOT start/stop/restart or configure the server. Never touches source code.
set -u

BASE_URL="${BASE_URL:-http://127.0.0.1:5155}"
PERSIST=0
for arg in "$@"; do
  case "$arg" in
    --persist) PERSIST=1 ;;
    -h|--help) grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown arg: $arg"; exit 2 ;;
  esac
done

PASS=0; FAIL=0
BODY="$(mktemp /tmp/ff-smoke.XXXXXX)"; trap 'rm -f "$BODY"' EXIT

pass() { PASS=$((PASS + 1)); printf 'PASS  %s\n' "$1"; }
fail() { FAIL=$((FAIL + 1)); printf 'FAIL  %s%s\n' "$1" "${2:+  |  $2}"; }
snippet() { head -c 300 "$BODY" | tr -d '\n'; }

# http METHOD PATH [JSON_BODY] [TOKEN] -> echoes HTTP status, body -> $BODY
http() {
  local method="$1" path="$2" data="${3:-}" token="${4:-}"
  local args=(-sS -m 20 -X "$method" -o "$BODY" -w '%{http_code}')
  [ -n "$token" ] && args+=(-H "Authorization: Bearer $token")
  [ -n "$data" ] && args+=(-H 'Content-Type: application/json' --data "$data")
  curl "${args[@]}" "${BASE_URL}${path}" 2>/dev/null || true   # curl already emits 000 on connect failure
}

# check_status LABEL EXPECTED ACTUAL
check_status() {
  if [ "$3" = "$2" ]; then pass "$1 -> $2"
  else fail "$1" "expected HTTP $2, got $3; body=$(snippet)"; fi
}

# jassert LABEL PYTHON_EXPR   (expr sees json object as `d`)
jassert() {
  if python3 -c "import json,sys;d=json.load(open(sys.argv[1]));sys.exit(0 if ($2) else 1)" "$BODY" 2>/dev/null
  then pass "$1"
  else fail "$1" "json assert '$2' failed; body=$(snippet)"; fi
}

for dep in curl python3; do
  command -v "$dep" >/dev/null 2>&1 || { echo "FAIL  missing dependency: $dep"; exit 1; }
done

echo "== Finance Forge smoke test  BASE_URL=$BASE_URL  $(date -Is) =="

# ---------------------------------------------------------------- 1. health
S=$(http GET /_health)
check_status "1  GET /_health" 200 "$S"
if [ "$S" != "200" ]; then
  echo "SUMMARY: $PASS passed, $FAIL failed  |  SERVER UNREACHABLE at $BASE_URL"
  exit 1
fi

# ---------------------------------------------------------------- 2. auth
TS="$(date +%s)"
EMAIL="smoke-${TS}@t.com"
PASSW="secret1234"
PREFIX="${EMAIL%%@*}"          # smoke-<ts>  -> unique bank name per run
BANK_NAME="SmokeBank-${PREFIX}"

S=$(http POST /api/auth/register "{\"name\":\"Smoke\",\"email\":\"$EMAIL\",\"password\":\"$PASSW\"}")
check_status "2a POST /api/auth/register" 200 "$S"

S=$(http POST /api/auth/login "{\"email\":\"$EMAIL\",\"password\":\"$PASSW\"}")
check_status "2b POST /api/auth/login" 200 "$S"
TOKEN="$(python3 -c "import json,sys;d=json.load(open(sys.argv[1]));t=d.get('token');print(t if isinstance(t,str) else '')" "$BODY" 2>/dev/null || true)"
if [ -n "$TOKEN" ]; then pass "2c login body has non-empty .token"
else fail "2c login body has non-empty .token" "body=$(snippet)"; fi

# ---------------------------------------------------------------- 3. me
if [ -n "$TOKEN" ]; then
  S=$(http GET /api/auth/me '' "$TOKEN")
  check_status "3  GET /api/auth/me" 200 "$S"
else
  fail "3  GET /api/auth/me" "skipped: no token"
fi

# ---------------------------------------------------------------- 4. CRUD
if [ -n "$TOKEN" ]; then
  S=$(http POST /api/banks "{\"name\":\"$BANK_NAME\",\"bank_type\":\"savings\",\"account_number\":\"1\",\"current_balance\":42}" "$TOKEN")
  check_status "4a POST /api/banks" 200 "$S"
  jassert "4a response has .id" "isinstance(d.get('id'),str) and len(d['id'])>0"
  BANK_ID="$(python3 -c "import json,sys;d=json.load(open(sys.argv[1]));print(d.get('id') or '')" "$BODY" 2>/dev/null || true)"

  S=$(http GET /api/banks '' "$TOKEN")
  check_status "4b GET /api/banks" 200 "$S"
  jassert "4b .total >= 1" "isinstance(d.get('total'),int) and d['total']>=1"

  if [ -n "$BANK_ID" ]; then
    S=$(http PUT "/api/banks/$BANK_ID" '{"current_balance":99}' "$TOKEN")
    check_status "4c PUT /api/banks/{id}" 200 "$S"
    jassert "4c .current_balance == 99" "d.get('current_balance')==99"
  else
    fail "4c PUT /api/banks/{id}" "skipped: no bank id from 4a"
    fail "4c .current_balance == 99" "skipped: no bank id from 4a"
  fi

  S=$(http POST /api/incomes '{"title":"Pay","amount":100,"income_type":"salary","source":"Co","income_date":"2026-09-01","is_recurring":true,"recurrence":"monthly","frequency_multiplier":1}' "$TOKEN")
  check_status "4d POST /api/incomes" 200 "$S"
  jassert "4d response has .id" "isinstance(d.get('id'),str) and len(d['id'])>0"

  S=$(http POST /api/expenses '{"title":"Food","amount":50,"expense_type":"variable","category":"food","expense_date":"2026-09-02"}' "$TOKEN")
  check_status "4e POST /api/expenses" 200 "$S"
  jassert "4e response has .id" "isinstance(d.get('id'),str) and len(d['id'])>0"

  echo "     this run's bank name: $BANK_NAME"

  # ---- optional persistence assertion (EXPECT_BANK = FIRST run's bank name)
  if [ -n "${EXPECT_BANK:-}" ]; then
    FIRST_EMAIL="${EXPECT_BANK#SmokeBank-}@t.com"   # bank name embeds the email prefix
    FS=$(http POST /api/auth/login "{\"email\":\"$FIRST_EMAIL\",\"password\":\"$PASSW\"}")
    FTOKEN="$(python3 -c "import json,sys;d=json.load(open(sys.argv[1]));t=d.get('token');print(t if isinstance(t,str) else '')" "$BODY" 2>/dev/null || true)"
    if [ "$FS" = "200" ] && [ -n "$FTOKEN" ]; then
      pass "4f re-login as first-run user $FIRST_EMAIL"
      S=$(http GET '/api/banks?perPage=200' '' "$FTOKEN")
      check_status "4f GET /api/banks (first-run user)" 200 "$S"
      jassert "4f GET /api/banks still lists EXPECT_BANK='$EXPECT_BANK'" "(d.get('total') or 0)>=0 and any(b.get('name')=='$EXPECT_BANK' for b in (d.get('data') or []))"
    else
      fail "4f re-login as first-run user $FIRST_EMAIL" "login HTTP=$FS body=$(snippet)"
      fail "4f GET /api/banks still lists EXPECT_BANK='$EXPECT_BANK'" "skipped: cannot log in as first-run user"
    fi
  fi
else
  for i in 4a 4b 4c 4d 4e; do fail "$i CRUD" "skipped: no token"; done
fi

# ---------------------------------------------------------------- 5. game
VILLAGE_ID=""; FIRST_BUILDING=""
if [ -n "$TOKEN" ]; then
  S=$(http GET /api/game/village '' "$TOKEN")
  check_status "5a GET /api/game/village" 200 "$S"
  jassert "5a .village.id present" "isinstance(d.get('village'),dict) and bool(d['village'].get('id'))"
  VILLAGE_ID="$(python3 -c "import json,sys;d=json.load(open(sys.argv[1]));print((d.get('village') or {}).get('id') or '')" "$BODY" 2>/dev/null || true)"
  FIRST_BUILDING="$(python3 -c "
import json,sys
d=json.load(open(sys.argv[1]))
bs=d.get('buildings') or []
print(bs[0].get('id','') if bs else '')" "$BODY" 2>/dev/null || true)"

  if [ -n "$FIRST_BUILDING" ]; then
    S=$(http POST /api/game/buildings/collect "{\"building_id\":\"$FIRST_BUILDING\"}" "$TOKEN")
    check_status "5b POST /api/game/buildings/collect (first building)" 200 "$S"
    jassert "5b response has .collected (number)" "isinstance(d.get('collected'),(int,float))"
  else
    fail "5b POST /api/game/buildings/collect (first building)" "no buildings in village response"
  fi

  S=$(http POST /api/game/battle '{"battle_type":"raid"}' "$TOKEN")
  check_status "5c POST /api/game/battle" 200 "$S"
  jassert "5c .rewards present" "isinstance(d.get('rewards'),dict)"

  S=$(http GET /api/game/analysis '' "$TOKEN")
  check_status "5d GET /api/game/analysis" 200 "$S"
  jassert "5d .net_worth.total is number" "isinstance((d.get('net_worth') or {}).get('total'),(int,float)) and not isinstance((d.get('net_worth') or {}).get('total'),bool)"

  S=$(http GET /api/game/achievements '' "$TOKEN")
  check_status "5e GET /api/game/achievements" 200 "$S"
  jassert "5e achievements array length >= 10" "isinstance(d,list) and len(d)>=10"
else
  for i in 5a 5b 5c 5d 5e; do fail "$i game" "skipped: no token"; done
fi

# ------------------------------------------------- 5f. persistence reminder
if [ "$PERSIST" = "1" ]; then
  echo "----- --persist: persistence reminder -----"
  echo "This script did NOT restart the server (and never will)."
  echo "Persistence check = run this script twice against a RESTARTED server:"
  echo "  run 1:  ./scripts/smoke.sh"
  echo "  then restart the server yourself (keep finance_game_development.sqlite)"
  echo "  run 2:  EXPECT_BANK='$BANK_NAME' ./scripts/smoke.sh"
  echo "  -> check 4f asserts GET /api/banks still lists the FIRST run's bank."
  echo "  (done automatically for you when EXPECT_BANK is exported; see 4f above)"
  echo "-------------------------------------------"
fi

# ---------------------------------------------------------------- 6. sync
if [ -n "$TOKEN" ]; then
  S=$(http POST /api/sync/push '{"ops":[]}' "$TOKEN")
  check_status "6a POST /api/sync/push" 200 "$S"
  jassert "6a .ok == true" "d.get('ok') is True"

  S=$(http GET /api/sync/pull '' "$TOKEN")
  check_status "6b GET /api/sync/pull" 200 "$S"
else
  fail "6a POST /api/sync/push" "skipped: no token"
  fail "6b GET /api/sync/pull" "skipped: no token"
fi

# ---------------------------------------------------------------- 7. frontend
S=$(http GET /)
check_status "7a GET /" 200 "$S"
if grep -qi 'Finance Forge' "$BODY" 2>/dev/null; then pass "7a index HTML contains 'Finance Forge'"
else fail "7a index HTML contains 'Finance Forge'" "body=$(snippet)"; fi

S=$(http GET /analysis)
check_status "7b GET /analysis (SPA)" 200 "$S"
if grep -qi '<!doctype html\|<html' "$BODY" 2>/dev/null; then pass "7b /analysis is HTML"
else fail "7b /analysis is HTML" "body=$(snippet)"; fi

S=$(http GET /api/nope)
check_status "7c GET /api/nope" 404 "$S"

# ---------------------------------------------------------------- summary
echo "==========================================="
if [ "$FAIL" -eq 0 ]; then
  echo "SUMMARY: ALL $PASS checks PASSED, 0 failed  (bank: $BANK_NAME, base: $BASE_URL)"
  exit 0
else
  echo "SUMMARY: $PASS passed, $FAIL FAILED  (bank: $BANK_NAME, base: $BASE_URL)"
  exit 1
fi
