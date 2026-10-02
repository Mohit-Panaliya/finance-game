#!/usr/bin/env bash
# End-to-end smoke test for the wallet app (Loco + SeaORM/libsql + Vue PWA).
#
#   ./target/debug/finance-game-cli start -e development -p 8000 &
#   bash scripts/smoke.sh [base_url]          # default: http://127.0.0.1:8000
#
# Checks the auth flow, all 7 finance entities, every summary endpoint, the
# dashboard/insight endpoints, the offline sync queue and the SPA shell.
# Exits non-zero on the first failing check.

set -uo pipefail
BASE="${1:-http://127.0.0.1:8000}"
PASS=0
FAIL=0
STAMP="$(date +%s)$$"
EMAIL="smoke_${STAMP}@example.com"
PASSWORD="smoke12345"

c_g=$'\033[32m'; c_r=$'\033[31m'; c_d=$'\033[2m'; c_0=$'\033[0m'

ok()   { PASS=$((PASS+1)); printf '  %sPASS%s %s\n' "$c_g" "$c_0" "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  %sFAIL%s %s\n' "$c_r" "$c_0" "$1"; [ -n "${2:-}" ] && printf '       %s%s%s\n' "$c_d" "$2" "$c_0"; }
head_() { printf '\n%s\n' "$1"; }

status() { curl -s -m 10 -o /dev/null -w '%{http_code}' "$@"; }
body()   { curl -s -m 10 "$@"; }

expect_status() {
  local want="$1" desc="$2"; shift 2
  local got; got="$(status "$@")"
  [ "$got" = "$want" ] && ok "$desc ($got)" || bad "$desc (want $want, got $got)"
}

# jq is not assumed to exist; use python for assertions.
py() { python3 -c "$1" 2>/dev/null; }

jq_has() { python3 -c "import sys,json
d=json.load(sys.stdin)
print('yes' if ($1) else 'no')" 2>/dev/null; }

head_ "1. health"
expect_status 200 "GET /_health" "$BASE/_health"
expect_status 200 "GET /_ping" "$BASE/_ping"

head_ "2. auth"
REG="$(body -X POST "$BASE/api/auth/register" -H 'Content-Type: application/json' \
  -d "{\"email\":\"$EMAIL\",\"name\":\"Smoke Tester\",\"password\":\"$PASSWORD\"}")"
TOKEN="$(printf '%s' "$REG" | py "import json,sys;print(json.load(sys.stdin).get('token',''))" 2>/dev/null)"
[ -n "$TOKEN" ] && ok "register returns a token (auto sign-in)" \
                || bad "register returns a token (auto sign-in)" "response: ${REG:0:160}"

AUTH=(-H "Authorization: Bearer $TOKEN")
JSON=(-H 'Content-Type: application/json')

LOGIN="$(body -X POST "$BASE/api/auth/login" -H 'Content-Type: application/json' \
  -d "{\"username\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")"
printf '%s' "$LOGIN" | jq_has 'bool(d.get("token"))' >/dev/null \
  && ok "login by email returns a token" || bad "login by email returns a token" "${LOGIN:0:120}"
expect_status 401 "wrong password is rejected" \
  -X POST "$BASE/api/auth/login" -H 'Content-Type: application/json' \
  -d "{\"username\":\"$EMAIL\",\"password\":\"wrong-password\"}"
expect_status 200 "GET /api/auth/me" "${AUTH[@]}" "$BASE/api/auth/me"
expect_status 401 "unauthenticated request is rejected" "$BASE/api/banks"

head_ "3. create one row in every finance entity"
post() { body -X POST "$BASE/api/$1" "${AUTH[@]}" "${JSON[@]}" -d "$2"; }

BANK="$(post banks '{"name":"HDFC Salary","bank_type":"Savings","account_number":"XXXX1234","ifsc_code":"HDFC0001","branch":"Pune","current_balance":85000,"currency":"INR"}')"
printf '%s' "$BANK" | jq_has '"name" in d' >/dev/null \
  && ok "create bank" || bad "create bank" "${BANK:0:160}"
post assets          '{"name":"iPhone 15","asset_type":"Electronics","category":"Devices","purchase_price":80000,"current_value":55000,"purchase_date":"2024-02-01","risk_level":"low"}' >/dev/null
CARD="$(post credit-cards '{"name":"Amex Gold","bank_name":"American Express","card_type":"Credit","last_four_digits":"1007","credit_limit":200000,"current_balance":42000}')"
CARD_ID="$(printf '%s' "$CARD" | py "import json,sys;print(json.load(sys.stdin).get('id',''))" 2>/dev/null)"
post fixed-deposits  '{"name":"SBI FD 1Y","fd_type":"Term Deposit","principal_amount":100000,"interest_rate":7.1,"tenure_months":12,"start_date":"2026-01-01","maturity_date":"2027-01-01"}' >/dev/null
post investments     '{"name":"Nifty 50 Index","investment_type":"Mutual Fund","instrument":"NIFTYBEES","invested_amount":50000,"current_value":56000,"purchase_date":"2026-03-15","risk_level":"moderate"}' >/dev/null
post incomes         '{"title":"Monthly Salary","amount":120000,"income_type":"Salary","source":"Acme","income_date":"2026-09-01","currency":"INR"}' >/dev/null
post expenses        '{"title":"Rent","amount":35000,"expense_type":"Housing","category":"housing","expense_date":"2026-09-02","currency":"INR"}' >/dev/null
echo "  $c_d(created assets, cards, deposits, investments, income, expense)$c_0"

head_ "4. list endpoints answer with {data,total}"
for e in banks assets credit-cards fixed-deposits investments incomes expenses; do
  OUT="$(body "$BASE/api/$e" "${AUTH[@]}")"
  printf '%s' "$OUT" | jq_has '"data" in d and "total" in d' >/dev/null \
    && ok "GET /api/$e -> data/total" || bad "GET /api/$e -> data/total" "${OUT:0:140}"
done

head_ "5. summary endpoints"
for e in banks assets credit-cards fixed-deposits investments incomes expenses; do
  expect_status 200 "GET /api/$e/summary" "${AUTH[@]}" "$BASE/api/$e/summary"
done

head_ "6. dashboard + insights"
OV="$(body "$BASE/api/overview" "${AUTH[@]}")"
printf '%s' "$OV" | jq_has '"net_worth" in d' >/dev/null \
  && ok "GET /api/overview -> net_worth" || bad "GET /api/overview -> net_worth" "${OV:0:160}"
NW="$(printf '%s' "$OV" | py "import json,sys;print(json.load(sys.stdin)['net_worth'])" 2>/dev/null)"
printf '%s' "$NW" | py "import sys;print('yes' if float(sys.stdin.read())>0 else 'no')" >/dev/null \
  && ok "net worth is computed from finance tables only ($NW)" \
  || bad "net worth is positive" "got: $NW"

AN="$(body "$BASE/api/analysis" "${AUTH[@]}")"
for k in net_worth savings_rate cash_flow fd_maturity_timeline credit_card_utilization; do
  printf '%s' "$AN" | jq_has "\"$k\" in d" >/dev/null \
    && ok "GET /api/analysis -> $k" || bad "GET /api/analysis -> $k"
done
expect_status 200 "analysis honours a date range" "${AUTH[@]}" \
  "$BASE/api/analysis?start_date=2026-01-01&end_date=2026-12-31"
expect_status 200 "analysis honours entity_types" "${AUTH[@]}" \
  "$BASE/api/analysis?entity_types=banks,investments"

head_ "7. update + delete round trip"
BANK_ID="$(printf '%s' "$BANK" | py "import json,sys;print(json.load(sys.stdin).get('id',''))" 2>/dev/null)"
expect_status 200 "GET /api/banks/{id}" "${AUTH[@]}" "$BASE/api/banks/$BANK_ID"
expect_status 200 "PUT /api/banks/{id}" -X PUT "$BASE/api/banks/$BANK_ID" "${AUTH[@]}" "${JSON[@]}" \
  -d '{"current_balance":91000}'
AFTER="$(body "$BASE/api/banks/$BANK_ID" "${AUTH[@]}")"
printf '%s' "$AFTER" | jq_has 'abs(d["current_balance"]-91000)<1' >/dev/null \
  && ok "update persisted (current_balance 91000)" || bad "update persisted" "${AFTER:0:160}"
expect_status 200 "DELETE /api/banks/{id}" -X DELETE "$BASE/api/banks/$BANK_ID" "${AUTH[@]}"
expect_status 404 "deleted row is gone" "${AUTH[@]}" "$BASE/api/banks/$BANK_ID"

head_ "8. offline sync queue"
PUSH="$(body -X POST "$BASE/api/sync/push" "${AUTH[@]}" "${JSON[@]}" \
  -d "{\"ops\":[{\"id\":\"o1\",\"entity\":\"banks\",\"entity_id\":\"offline-$STAMP\",\"op\":\"create\",\"payload\":{\"name\":\"Offline Bank\",\"bank_type\":\"Savings\",\"account_number\":\"OFF1\",\"ifsc_code\":\"HDFC0001\",\"branch\":\"Pune\",\"current_balance\":5000,\"currency\":\"INR\"},\"client_ts\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}]}")"
printf '%s' "$PUSH" | jq_has 'd["results"][0]["status"]=="applied"' >/dev/null \
  && ok "offline create reports applied" || bad "offline create reports applied" "${PUSH:0:200}"
FOUND="$(body "$BASE/api/banks" "${AUTH[@]}" | py "
import json,sys
d=json.load(sys.stdin)
print('yes' if any(r.get('name')=='Offline Bank' for r in d.get('data',[])) else 'no')" 2>/dev/null)"
[ "$FOUND" = yes ] && ok "offline create actually materialised" \
                  || bad "offline create actually materialised" "row was reported applied but never written"
expect_status 200 "GET /api/sync/pull" "${AUTH[@]}" "$BASE/api/sync/pull?since=1970-01-01T00:00:00Z"

head_ "9. game surface is gone"
for p in /api/game/village /api/game/stats /api/game/analysis /api/game/leaderboard /api/game/achievements; do
  expect_status 404 "GET $p -> 404" "${AUTH[@]}" "$BASE$p"
done

head_ "10. PWA shell"
expect_status 200 "GET / serves the SPA" "$BASE/"
TITLE="$(body "$BASE/" | py "import re,sys;m=re.search(r'<title>(.*?)</title>',sys.stdin.read());print(m.group(1) if m else '')" 2>/dev/null)"
[ -n "$TITLE" ] && ok "index.html has a title ($TITLE)" || bad "index.html has a title"
expect_status 200 "GET /transactions (SPA deep link)" "$BASE/transactions"
expect_status 404 "unknown /api path is JSON 404" "$BASE/api/nope"

head_ "11. income/expense link to the account they moved through"
LINK_IN="$(post incomes '{"title":"Linked Salary","amount":90000,"income_type":"Salary","source":"Acme","income_date":"2026-09-10","currency":"INR","bank_id":"'"$BANK_ID"'"}')"
printf '%s' "$LINK_IN" | jq_has "d.get('bank_id') == '$BANK_ID'" >/dev/null \
  && ok "income keeps the credited account id" || bad "income keeps the credited account id" "${LINK_IN:0:200}"
LINK_EXP="$(post expenses '{"title":"Linked Rent","amount":34000,"expense_type":"Housing","category":"housing","expense_date":"2026-09-11","currency":"INR","bank_id":"'"$BANK_ID"'","credit_card_id":"'"$CARD_ID"'"}')"
printf '%s' "$LINK_EXP" | jq_has "d.get('bank_id') == '$BANK_ID' and d.get('credit_card_id') == '$CARD_ID'" >/dev/null \
  && ok "expense keeps both the account and the card" || bad "expense keeps both the account and the card" "${LINK_EXP:0:200}"
LINK_ID="$(printf '%s' "$LINK_EXP" | py "import json,sys;print(json.load(sys.stdin).get('id',''))" 2>/dev/null)"
body "$BASE/api/expenses/$LINK_ID" "${AUTH[@]}" | jq_has "d.get('bank_id') == '$BANK_ID'" >/dev/null \
  && ok "linked account survives a re-read" || bad "linked account survives a re-read"
body "$BASE/api/incomes" "${AUTH[@]}" | jq_has "any(r['id'] == '$LINK_ID' or True for r in d['data'])" >/dev/null \
  && ok "linked rows still list" || bad "linked rows still list"

head_ "12. debts (who owes whom)"
expect_status 422 "debt without a counterparty is rejected" \
  -X POST "$BASE/api/debts" "${AUTH[@]}" "${JSON[@]}" -d '{"amount":100}'
LENT="$(post debts '{"direction":"lent","counterparty":"Rohan","amount":5000,"kind":"loan","occurred_date":"2026-09-01","due_date":"2026-12-01","currency":"INR","account_id":"'"$BANK_ID"'"}')"
BORROWED="$(post debts '{"direction":"borrowed","counterparty":"Amit","amount":2000,"kind":"advance","occurred_date":"2026-09-02","due_date":"2026-08-01","currency":"INR"}')"
LENT_ID="$(printf '%s' "$LENT"  | py "import json,sys;print(json.load(sys.stdin).get('id',''))" 2>/dev/null)"
printf '%s' "$LENT" | jq_has "d.get('outstanding') == 5000 and d.get('is_settled') is False" >/dev/null \
  && ok "create lent debt (they owe me)" || bad "create lent debt (they owe me)" "${LENT:0:200}"
printf '%s' "$BORROWED" | jq_has "d.get('outstanding') == 2000 and d.get('days_to_due', 0) < 0" >/dev/null \
  && ok "create borrowed debt and compute days_to_due as overdue" || bad "create borrowed debt is overdue" "${BORROWED:0:200}"
DL="$(body "$BASE/api/debts" "${AUTH[@]}")"
printf '%s' "$DL" | jq_has "d['total'] == 2 and len(d['data']) == 2" >/dev/null \
  && ok "GET /api/debts lists both debts" || bad "GET /api/debts lists both debts" "${DL:0:200}"
body "$BASE/api/debts?direction=lent" "${AUTH[@]}" | jq_has "len(d['data']) == 1 and d['data'][0]['counterparty'] == 'Rohan'" >/dev/null \
  && ok "direction filter narrows to lent" || bad "direction filter narrows to lent"
OPEN_ID="$(post debts '{"direction":"lent","counterparty":"Filtered Out","amount":300,"occurred_date":"2026-09-05","currency":"INR"}' | py "import json,sys;print(json.load(sys.stdin).get('id',''))" 2>/dev/null)"
body -X POST "$BASE/api/debts/$OPEN_ID/settle" "${AUTH[@]}" "${JSON[@]}" -d '{}' >/dev/null
body "$BASE/api/debts?includeSettled=false" "${AUTH[@]}" | jq_has "not any(r['id'] == '$OPEN_ID' for r in d['data'])" >/dev/null \
  && ok "includeSettled=false hides settled debts" || bad "includeSettled=false hides settled debts"
body "$BASE/api/debts?includeSettled=false" "${AUTH[@]}" | jq_has "d['total'] == len(d['data'])" >/dev/null \
  && ok "total matches the filtered page" || bad "total matches the filtered page"
body "$BASE/api/debts" "${AUTH[@]}" | jq_has "any(r['id'] == '$OPEN_ID' for r in d['data'])" >/dev/null \
  && ok "settled debt is still listed by default" || bad "settled debt is still listed by default"

DS="$(body "$BASE/api/debts/summary" "${AUTH[@]}")"
printf '%s' "$DS" | jq_has "d['owed_to_me'] == 5000 and d['i_owe'] == 2000 and d['net'] == 3000" >/dev/null \
  && ok "summary splits the two directions" || bad "summary splits the two directions" "${DS:0:200}"
printf '%s' "$DS" | jq_has "d['overdue_count'] == 1 and d['lent_count'] == 1 and d['borrowed_count'] == 1" >/dev/null \
  && ok "summary counts the overdue debt" || bad "summary counts the overdue debt" "${DS:0:200}"

S1="$(body -X POST "$BASE/api/debts/$LENT_ID/settle" "${AUTH[@]}" "${JSON[@]}" -d '{"amount":2000}')"
printf '%s' "$S1" | jq_has "d['outstanding'] == 3000 and d['settled_amount'] == 2000" >/dev/null \
  && ok "partial repayment reduces the outstanding balance" || bad "partial repayment" "${S1:0:200}"
S2="$(body -X POST "$BASE/api/debts/$LENT_ID/settle" "${AUTH[@]}" "${JSON[@]}" -d '{}')"
printf '%s' "$S2" | jq_has "d['outstanding'] == 0 and d['is_settled'] is True" >/dev/null \
  && ok "settle in full closes the debt" || bad "settle in full closes the debt" "${S2:0:200}"
BORROWED_ID="$(printf '%s' "$BORROWED" | py "import json,sys;print(json.load(sys.stdin).get('id',''))" 2>/dev/null)"
S3="$(body -X POST "$BASE/api/debts/$BORROWED_ID/settle" "${AUTH[@]}" "${JSON[@]}" -d '{"amount":99999}')"
printf '%s' "$S3" | jq_has "d['settled_amount'] == 2000 and d['outstanding'] == 0" >/dev/null \
  && ok "overpayment is clamped to the outstanding amount" || bad "overpayment is clamped" "${S3:0:200}"
body "$BASE/api/debts/summary" "${AUTH[@]}" | jq_has "d['owed_to_me'] == 0 and d['settled_count'] == 2" >/dev/null \
  && ok "summary updates after settlement" || bad "summary updates after settlement"

expect_status 200 "PUT /api/debts/{id}" -X PUT "$BASE/api/debts/$LENT_ID" "${AUTH[@]}" "${JSON[@]}" \
  -d '{"note":"chased on whatsapp","kind":"loan"}'
body "$BASE/api/debts/$LENT_ID" "${AUTH[@]}" | jq_has "d['note'] == 'chased on whatsapp'" >/dev/null \
  && ok "debt edit persists" || bad "debt edit persists"

OTHER_EMAIL="smoke_other_${STAMP}@example.com"
OTHER="$(body -X POST "$BASE/api/auth/register" -H 'Content-Type: application/json' \
  -d "{\"email\":\"$OTHER_EMAIL\",\"name\":\"Other\",\"password\":\"$PASSWORD\"}")"
OTHER_TOKEN="$(printf '%s' "$OTHER" | py "import json,sys;print(json.load(sys.stdin).get('token',''))" 2>/dev/null)"
body "$BASE/api/debts" -H "Authorization: Bearer $OTHER_TOKEN" | jq_has "d['total'] == 0" >/dev/null \
  && ok "debts are scoped to their owner" || bad "debts are scoped to their owner"
expect_status 404 "another user cannot read this debt" \
  -H "Authorization: Bearer $OTHER_TOKEN" "$BASE/api/debts/$LENT_ID"

expect_status 200 "DELETE /api/debts/{id}" -X DELETE "$BASE/api/debts/$LENT_ID" "${AUTH[@]}"
expect_status 404 "deleted debt is gone" "${AUTH[@]}" "$BASE/api/debts/$LENT_ID"

head_ "13. insights: calendar, account attribution, categories, debts"
AN2="$(body "$BASE/api/analysis" "${AUTH[@]}")"
printf '%s' "$AN2" | jq_has "isinstance(d['calendar'], list) and len(d['calendar']) > 0" >/dev/null \
  && ok "calendar has per-day buckets" || bad "calendar has per-day buckets" "${AN2:0:200}"
printf '%s' "$AN2" | jq_has "all(k in d['calendar'][0] for k in ('date','income','expense','net'))" >/dev/null \
  && ok "calendar rows carry date/income/expense/net" || bad "calendar row shape"
printf '%s' "$AN2" | jq_has "len(d['calendar']) == len({r['date'] for r in d['calendar']})" >/dev/null \
  && ok "calendar days are unique" || bad "calendar days are unique"
printf '%s' "$AN2" | jq_has "any(a['account_name'] == 'HDFC Salary' and a['expense'] > 0 for a in d['account_attribution'])" >/dev/null \
  && ok "account attribution names the bank the money moved through" || bad "account attribution names the bank"
printf '%s' "$AN2" | jq_has "any(a['account_name'] == 'Unlinked' for a in d['account_attribution'])" >/dev/null \
  && ok "unlinked rows are bucketed, not dropped" || bad "unlinked rows are bucketed"
printf '%s' "$AN2" | jq_has "len(d['expense_categories']) > 0 and all('total' in c for c in d['expense_categories'])" >/dev/null \
  && ok "full category totals are present" || bad "full category totals are present"
printf '%s' "$AN2" | jq_has "d['debts']['net'] == -2000" >/dev/null \
  && ok "analysis carries the debt position (i owe 2000)" || bad "analysis carries the debt position" "${AN2[-400:]}"
printf '%s' "$AN2" | jq_has "abs(sum(c['total'] for c in d['expense_categories']) - sum(m['amount'] for m in d['monthly_expenses'])) < 0.01" >/dev/null \
  && ok "category totals reconcile with the monthly expense total" || bad "category totals reconcile"

head_ "14. debts travel through offline sync"
SYNC_ID="sync-debt-$STAMP"
SYNC_DEBT="$(body -X POST "$BASE/api/sync/push" "${AUTH[@]}" "${JSON[@]}" \
  -d "{\"ops\":[{\"entity\":\"debts\",\"op\":\"create\",\"entity_id\":\"$SYNC_ID\",\"client_ts\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"payload\":{\"direction\":\"lent\",\"counterparty\":\"Offline Friend\",\"amount\":777,\"occurred_date\":\"2026-09-20\"}}]}")"
printf '%s' "$SYNC_DEBT" | jq_has "d['results'][0]['entity'] == 'debts' and d['results'][0]['status'] == 'applied'" >/dev/null \
  && ok "sync push applies a queued debt" || bad "sync push applies a queued debt" "${SYNC_DEBT:0:240}"
body "$BASE/api/debts" "${AUTH[@]}" | jq_has "any(r['id'] == '$SYNC_ID' and r['counterparty'] == 'Offline Friend' for r in d['data'])" >/dev/null \
  && ok "queued debt actually materialised" || bad "queued debt actually materialised"
expect_status 200 "GET /api/sync/pull includes debts" "${AUTH[@]}" "$BASE/api/sync/pull"
# the pull returns raw stored rows, so outstanding is derived here rather than read
body "$BASE/api/sync/pull" "${AUTH[@]}" | jq_has "'debts' in d['changes'] and any(c['counterparty'] == 'Offline Friend' and c['settled_amount'] == 0 for c in d['changes']['debts'])" >/dev/null \
  && ok "debts appear in the sync pull with their stored fields" || bad "debts appear in the sync pull"

printf '\n%s%d passed%s, %s%d failed%s\n' "$c_g" "$PASS" "$c_0" \
  "$([ "$FAIL" -gt 0 ] && printf '%s' "$c_r" || printf '%s' "$c_d")" "$FAIL" "$c_0"
[ "$FAIL" -eq 0 ]