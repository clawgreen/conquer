#!/usr/bin/env bash
# Conquer — End-to-End Integration Test (T457-T460)
#
# Tests the full game flow: register → create game → join → take turns → verify
# Requires a running server at $BASE_URL (default: http://localhost:3000)
#
# Usage:
#   ./tests/e2e_test.sh                          # against localhost
#   BASE_URL=https://conquer.example.com ./tests/e2e_test.sh

set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:3000}"
PASS=0
FAIL=0

log() { echo "  $1"; }
pass() { PASS=$((PASS + 1)); echo "✅ $1"; }
fail() { FAIL=$((FAIL + 1)); echo "❌ $1"; }

check_status() {
    local label="$1" expected="$2" actual="$3"
    if [ "$actual" -eq "$expected" ]; then
        pass "$label (HTTP $actual)"
    else
        fail "$label (expected $expected, got $actual)"
    fi
}

json_field() {
    echo "$1" | python3 -c "import sys,json; print(json.load(sys.stdin)$2)"
}

echo "========================================"
echo " Conquer E2E Test Suite"
echo " Server: $BASE_URL"
echo "========================================"
echo ""

# ── Health Check ──
echo "── Health Check ──"
STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/health")
check_status "Health endpoint" 200 "$STATUS"

HEALTH=$(curl -s "$BASE_URL/api/health")
log "Response: $HEALTH"

# ── Register 3 Players ──
echo ""
echo "── Register Players ──"

register_player() {
    local username="$1" email="$2"
    local RESP
    RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/register" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$username\",\"email\":\"$email\",\"password\":\"testpass123\"}")
    local CODE=$(echo "$RESP" | tail -1)
    local BODY=$(echo "$RESP" | head -1)
    check_status "Register $username" 200 "$CODE"
    json_field "$BODY" "['token']"
}

TOKEN1=$(register_player "player1_$$" "p1_$$@test.com")
TOKEN2=$(register_player "player2_$$" "p2_$$@test.com")
TOKEN3=$(register_player "player3_$$" "p3_$$@test.com")

log "Got 3 player tokens"

# ── Create Game ──
echo ""
echo "── Create Game ──"

RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/games" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN1" \
    -d '{"name":"E2E Test Game"}')
CODE=$(echo "$RESP" | tail -1)
BODY=$(echo "$RESP" | head -1)
check_status "Create game" 200 "$CODE"

GAME_ID=$(json_field "$BODY" "['id']")
log "Game ID: $GAME_ID"

# ── Join Game (3 players) ──
echo ""
echo "── Join Game ──"

join_game() {
    local token="$1" name="$2" leader="$3" race="$4" mark="$5"
    local RESP CODE
    RESP=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/api/games/$GAME_ID/join" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $token" \
        -d "{\"nation_name\":\"$name\",\"leader_name\":\"$leader\",\"race\":\"$race\",\"class\":1,\"mark\":\"$mark\"}")
    check_status "Join as $name" 200 "$RESP"
}

join_game "$TOKEN1" "Gondor" "Aragorn" "H" "G"
join_game "$TOKEN2" "Rohan" "Theoden" "H" "R"
join_game "$TOKEN3" "Mordor" "Sauron" "O" "M"

# ── Verify Nation State ──
echo ""
echo "── Verify State ──"

RESP=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/games/$GAME_ID/nation" \
    -H "Authorization: Bearer $TOKEN1")
CODE=$(echo "$RESP" | tail -1)
check_status "Get nation (player 1)" 200 "$CODE"

RESP=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/games/$GAME_ID/map" \
    -H "Authorization: Bearer $TOKEN1")
CODE=$(echo "$RESP" | tail -1)
check_status "Get map" 200 "$CODE"

RESP=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/games/$GAME_ID/nations" \
    -H "Authorization: Bearer $TOKEN1")
CODE=$(echo "$RESP" | tail -1)
check_status "Get all nations" 200 "$CODE"

# ── Scores ──
echo ""
echo "── Scores ──"

RESP=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/games/$GAME_ID/scores" \
    -H "Authorization: Bearer $TOKEN1")
CODE=$(echo "$RESP" | tail -1)
check_status "Get scores" 200 "$CODE"

# ── Chat ──
echo ""
echo "── Chat ──"

RESP=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/games/$GAME_ID/chat?channel=public" \
    -H "Authorization: Bearer $TOKEN1")
CODE=$(echo "$RESP" | tail -1)
check_status "Get chat history" 200 "$CODE"

# ── Metrics ──
echo ""
echo "── Metrics ──"

RESP=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/metrics")
CODE=$(echo "$RESP" | tail -1)
BODY=$(echo "$RESP" | head -1)
check_status "Metrics endpoint" 200 "$CODE"
log "Metrics: $BODY"

# ── Summary ──
echo ""
echo "========================================"
echo " Results: $PASS passed, $FAIL failed"
echo "========================================"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
