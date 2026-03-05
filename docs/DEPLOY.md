# Conquer — Deployment Guide

Complete guide to deploying Conquer from scratch.

## Table of Contents

- [Quick Start (Docker)](#quick-start-docker)
- [Local Development](#local-development)
- [Production (Docker + Caddy)](#production-docker--caddy)
- [Railway Deployment](#railway-deployment)
- [Environment Variables](#environment-variables)
- [Database Setup](#database-setup)
- [Domain & SSL](#domain--ssl)
- [Monitoring](#monitoring)
- [Troubleshooting](#troubleshooting)

---

## Quick Start (Docker)

```bash
# Clone and run locally
git clone https://github.com/your-org/conquer.git
cd conquer
docker compose up --build
```

Open http://localhost:3000 — the game is running.

---

## Local Development

### Prerequisites

- **Rust** 1.84+ (install via [rustup](https://rustup.rs))
- **Node.js** 22+ (for frontend build)
- **PostgreSQL** 16+ (optional — server runs with in-memory store by default)

### Without Docker

```bash
# Terminal 1: Build and run the server
cd conquer
cargo run --release --bin conquer-server

# Terminal 2: Run the frontend dev server (hot reload)
cd conquer/web
npm install
npm run dev
```

- Backend: http://localhost:3000
- Frontend dev: http://localhost:5173 (proxies API to :3000)

### With Docker (recommended)

```bash
docker compose up --build
```

This starts:
- **Conquer server** on port 3000 (serves API + frontend)
- **PostgreSQL** on port 5432
- **pgAdmin** on port 5050 (optional, use `docker compose --profile tools up`)

---

## Production (Docker + Caddy)

### 1. Create environment file

```bash
cp .env.example .env
# Edit with production values:
```

```env
# .env — Production configuration
DOMAIN=conquer.yourdomain.com
POSTGRES_PASSWORD=<generate-strong-password>
JWT_SECRET=<generate-strong-secret>
```

Generate secrets:
```bash
openssl rand -hex 32  # For JWT_SECRET
openssl rand -hex 24  # For POSTGRES_PASSWORD
```

### 2. Deploy

```bash
docker compose -f docker-compose.prod.yml up -d --build
```

### 3. Verify

```bash
curl https://conquer.yourdomain.com/api/health
# {"status":"ok","version":"0.1.0","timestamp":"..."}
```

### Architecture

```
Internet → Caddy (TLS) → Conquer Server → PostgreSQL
              :443           :3000            :5432
```

Caddy automatically provisions Let's Encrypt TLS certificates.

---

## Railway Deployment

### 1. Prerequisites

- [Railway account](https://railway.app)
- [Railway CLI](https://docs.railway.app/develop/cli) installed

### 2. Create project

```bash
railway login
railway init
```

### 3. Add PostgreSQL

```bash
railway add --plugin postgresql
```

Railway automatically injects `DATABASE_URL` into your environment.

### 4. Set environment variables

```bash
railway variables set JWT_SECRET=$(openssl rand -hex 32)
railway variables set CORS_ORIGIN=https://your-app.up.railway.app
railway variables set RUST_LOG=info
railway variables set STATIC_DIR=/app/dist
```

### 5. Deploy

```bash
railway up
```

Railway detects the `railway.toml` and builds using the Dockerfile.

### 6. Custom domain (optional)

1. Go to Railway dashboard → your service → Settings → Domains
2. Add your custom domain
3. Point your DNS CNAME to the Railway-provided domain
4. TLS is automatic

---

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PORT` | No | `3000` | HTTP server port |
| `DATABASE_URL` | No* | — | PostgreSQL connection string |
| `JWT_SECRET` | **Yes** (prod) | `conquer-dev-secret...` | JWT signing secret (min 32 chars) |
| `JWT_EXPIRY_HOURS` | No | `24` | JWT token lifetime |
| `RUST_LOG` | No | `info,conquer_server=debug` | Log level filter |
| `CORS_ORIGIN` | No | `http://localhost:5173` | Allowed CORS origins (comma-separated) |
| `STATIC_DIR` | No | auto-detect | Path to frontend dist files |
| `RATE_LIMIT_MAX` | No | `100` | Max requests per window per IP |
| `RATE_LIMIT_WINDOW_SECS` | No | `60` | Rate limit window in seconds |

*`DATABASE_URL` is optional — without it, the server uses an in-memory store (data lost on restart).

---

## Database Setup

### Schema migration

The initial schema is at `conquer-db/migrations/001_initial_schema.sql`.

For Docker deployments, migrations run automatically via init scripts.

For manual setup:
```bash
psql $DATABASE_URL < conquer-db/migrations/001_initial_schema.sql
```

### Backups

With Docker:
```bash
docker compose exec postgres pg_dump -U conquer conquer > backup_$(date +%Y%m%d).sql
```

With Railway:
- Railway provides automatic daily backups for Postgres
- Manual: Railway dashboard → Postgres plugin → Backups

---

## Domain & SSL

### With Caddy (docker-compose.prod.yml)

Caddy handles TLS automatically via Let's Encrypt. Just:
1. Point your domain's A record to your server IP
2. Set `DOMAIN=yourdomain.com` in `.env`
3. Start with `docker compose -f docker-compose.prod.yml up -d`

### With Railway

Railway provides automatic TLS on `.up.railway.app` domains and custom domains.

### Manual (nginx/other)

If using your own reverse proxy, forward to port 3000:

```nginx
server {
    listen 443 ssl;
    server_name conquer.yourdomain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## Monitoring

### Health check

```bash
curl http://localhost:3000/api/health
```

### Metrics

```bash
curl http://localhost:3000/api/metrics
```

Returns:
```json
{
  "uptime_secs": 3600,
  "total_requests": 1542,
  "requests_per_minute": 25.7,
  "active_games": 3,
  "connected_players": 8,
  "actions_processed": 234,
  "turns_advanced": 12
}
```

### Logs

Docker:
```bash
docker compose logs -f server
```

Logs are JSON-structured with correlation fields:
```json
{"timestamp":"...","level":"INFO","target":"conquer_server","message":"..."}
```

---

## Troubleshooting

### Server won't start

1. Check `PORT` isn't already in use
2. Check `JWT_SECRET` is set (required in production)
3. Check `DATABASE_URL` is valid (if using Postgres)
4. Check logs: `docker compose logs server`

### WebSocket disconnects

- Ensure your reverse proxy supports WebSocket upgrades
- Check `ws_timeout_secs` (default 60s) — increase if players have slow connections
- Caddy/nginx config must include WebSocket headers

### Frontend not loading

- Verify `STATIC_DIR` points to the built frontend
- Check that `web/dist/index.html` exists in the Docker image
- Try `docker compose exec server ls /app/dist/`

### Database connection issues

- Verify PostgreSQL is running and accessible
- Check `DATABASE_URL` format: `postgresql://user:pass@host:port/dbname`
- For Docker: ensure `depends_on` with health check is configured

### Rate limiting

If getting 429 responses, adjust:
- `RATE_LIMIT_MAX` (default: 100 requests)
- `RATE_LIMIT_WINDOW_SECS` (default: 60 seconds)
