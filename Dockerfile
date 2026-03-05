# Conquer — Multi-stage Dockerfile (T435)
# Stage 1: Build Rust server
# Stage 2: Build frontend
# Stage 3: Minimal runtime image

# ============================================================
# Stage 1: Build Rust server
# ============================================================
FROM rust:1.84-bookworm AS rust-builder

WORKDIR /build

# Copy workspace manifests first for dependency caching
COPY Cargo.toml Cargo.lock* ./
COPY conquer-core/Cargo.toml conquer-core/Cargo.toml
COPY conquer-engine/Cargo.toml conquer-engine/Cargo.toml
COPY conquer-db/Cargo.toml conquer-db/Cargo.toml
COPY conquer-server/Cargo.toml conquer-server/Cargo.toml
COPY conquer-oracle/Cargo.toml conquer-oracle/Cargo.toml

# Create dummy source files so cargo can resolve dependencies
RUN mkdir -p conquer-core/src && echo "pub fn _dummy() {}" > conquer-core/src/lib.rs && \
    mkdir -p conquer-engine/src && echo "pub fn _dummy() {}" > conquer-engine/src/lib.rs && \
    mkdir -p conquer-db/src && echo "pub fn _dummy() {}" > conquer-db/src/lib.rs && \
    mkdir -p conquer-oracle/src && echo "pub fn _dummy() {}" > conquer-oracle/src/lib.rs && \
    mkdir -p conquer-server/src && echo "fn main() {}" > conquer-server/src/main.rs && \
    echo "pub fn _dummy() {}" > conquer-server/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release --bin conquer-server 2>/dev/null || true

# Copy actual source code
COPY conquer-core/ conquer-core/
COPY conquer-engine/ conquer-engine/
COPY conquer-db/ conquer-db/
COPY conquer-server/ conquer-server/
COPY conquer-oracle/ conquer-oracle/

# Touch source files to invalidate cached builds of our crates
RUN touch conquer-core/src/lib.rs conquer-engine/src/lib.rs \
    conquer-db/src/lib.rs conquer-server/src/main.rs conquer-server/src/lib.rs \
    conquer-oracle/src/lib.rs

# Build release binary
RUN cargo build --release --bin conquer-server

# ============================================================
# Stage 2: Build frontend
# ============================================================
FROM node:22-slim AS frontend-builder

WORKDIR /build/web

COPY web/package.json web/package-lock.json* ./
RUN npm ci

COPY web/ ./
RUN npm run build

# ============================================================
# Stage 3: Runtime image (minimal)
# ============================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash conquer

WORKDIR /app

# Copy server binary
COPY --from=rust-builder /build/target/release/conquer-server /app/conquer-server

# Copy frontend static files
COPY --from=frontend-builder /build/web/dist /app/dist

# Copy database migrations
COPY conquer-db/migrations/ /app/migrations/

# Set ownership
RUN chown -R conquer:conquer /app

USER conquer

# Environment defaults (T438)
ENV PORT=3000 \
    RUST_LOG=info,conquer_server=debug \
    STATIC_DIR=/app/dist \
    JWT_EXPIRY_HOURS=24

EXPOSE 3000

# Health check (T439)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

CMD ["/app/conquer-server"]
