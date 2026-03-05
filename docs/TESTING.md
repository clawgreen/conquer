# Conquer — Testing Guide

## Test Suites

### Unit Tests (Rust)

```bash
cargo test --all
```

194 tests across 5 crates:
- **conquer-core** (36 tests): data types, constants, RNG, actions
- **conquer-engine** (110 tests): worldgen, combat, movement, economy, diplomacy, turns
- **conquer-db** (28 tests): store operations, auth, game lifecycle
- **conquer-oracle** (5 tests): C oracle JSON loading, cross-validation
- **conquer-server** (10+5 tests): API routes, WebSocket protocol, JWT

### Frontend

```bash
cd web && npx tsc --noEmit  # Type checking
cd web && npm run build      # Build verification
```

### E2E Integration Test (T457-T460)

```bash
# Start server first, then:
./tests/e2e_test.sh

# Against a remote server:
BASE_URL=https://conquer.example.com ./tests/e2e_test.sh
```

Tests the full flow:
1. Health check
2. Register 3 players
3. Create game
4. All 3 join with different nations/races
5. Verify nation state, map, scores
6. Verify chat endpoints
7. Verify metrics endpoint

---

## Concurrent Games Test (T458)

The in-memory store is fully concurrent via `Arc<RwLock<>>`. The `test_full_game_flow` test in `conquer-server` can be run in parallel:

```bash
# Run multiple E2E tests simultaneously
for i in {1..5}; do
    ./tests/e2e_test.sh &
done
wait
```

Each run creates unique usernames (using PID) to avoid collisions.

---

## Persistence Test (T459)

With PostgreSQL configured:

1. Start server with `DATABASE_URL` set
2. Create a game, join players, submit actions
3. Stop the server (`Ctrl+C` — triggers graceful shutdown)
4. Restart the server
5. Verify game state is preserved via API

```bash
# 1. Start
DATABASE_URL=postgresql://conquer:pass@localhost/conquer cargo run --release

# 2. Run E2E test
./tests/e2e_test.sh
# Note the GAME_ID from output

# 3. Stop (Ctrl+C)

# 4. Restart
DATABASE_URL=postgresql://conquer:pass@localhost/conquer cargo run --release

# 5. Verify
curl http://localhost:3000/api/games/$GAME_ID
```

---

## Browser Compatibility (T461)

The Canvas 2D terminal renderer uses standard APIs supported by all modern browsers:

| Browser | Status | Notes |
|---------|--------|-------|
| **Chrome** 100+ | ✅ Full support | Primary development browser |
| **Firefox** 100+ | ✅ Full support | Canvas 2D + WebSocket work identically |
| **Safari** 16+ | ✅ Full support | WebSocket requires secure context (HTTPS) in some versions |
| **Edge** 100+ | ✅ Full support | Chromium-based, same as Chrome |

**Key APIs used:**
- `Canvas 2D` (`getContext('2d')`) — character grid rendering
- `WebSocket` — real-time game updates
- `fetch` — REST API calls
- `localStorage` — JWT token storage

No WebGL, WebAssembly, or exotic APIs required.

---

## Performance Test (T462)

### 35-Nation Max Game

The original C game supports up to `NTOTAL=35` nations. Performance considerations:

| Metric | Expectation | Notes |
|--------|-------------|-------|
| Map render (100x50) | <16ms per frame | Canvas 2D character grid, no sprites |
| Turn advance (35 nations) | <500ms | All NPC + player actions resolved |
| API response (game state) | <50ms | In-memory store, O(1) lookups |
| WebSocket broadcast | <10ms | Tokio broadcast channel to all clients |
| Map data size | ~100KB | 5000 sectors × ~20 bytes/sector JSON |

**Bottleneck:** Turn processing with 35 nations, 50 armies each = 1750 army movements. The Rust engine handles this easily in a single thread.

**Frontend rendering:** At 100×50 characters with 12px monospace font, the canvas is 1200×600px — trivial for any GPU.

---

## Security Tests (T465)

### Fog of War API Bypass

The server enforces fog of war server-side. Test:

```bash
# Get map as player 1 — should only see own territory + adjacent
TOKEN=$(curl -s -X POST http://localhost:3000/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"player1","password":"pass"}' | jq -r .token)

MAP=$(curl -s http://localhost:3000/api/games/$GAME_ID/map \
    -H "Authorization: Bearer $TOKEN")

# Verify sectors outside vision have designation=null or are masked
echo "$MAP" | jq '.sectors[] | select(.owner != null and .owner != 1) | .people' | head
# Should be null/0 for enemy sectors outside vision radius
```

### Password Leak Check

```bash
# Get nations list — verify no password fields
curl -s http://localhost:3000/api/games/$GAME_ID/nations \
    -H "Authorization: Bearer $TOKEN" | jq 'keys'
# Should NOT contain "password", "password_hash", "pass"

# Get user profile
curl -s http://localhost:3000/api/users/me \
    -H "Authorization: Bearer $TOKEN" | jq 'keys'
# Should NOT contain "password_hash"
```

### Rate Limit Check

```bash
# Send 200 rapid requests
for i in $(seq 1 200); do
    curl -s -o /dev/null -w "%{http_code}\n" http://localhost:3000/api/health
done | sort | uniq -c
# After RATE_LIMIT_MAX (default 100), should see 429 responses
```

---

## CI Integration

All tests run in GitHub Actions on every push/PR:

```yaml
# .github/workflows/ci.yml
- cargo fmt --check
- cargo clippy -- -D warnings
- cargo test --all
- cd web && npm run build
- docker build .
```

See `.github/workflows/ci.yml` for the full workflow.
