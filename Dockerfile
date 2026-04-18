# ---------------------------------------------------------------------------
# Stage 1: Build frontend
# ---------------------------------------------------------------------------
FROM node:25-alpine AS frontend-builder

WORKDIR /build

COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

# ---------------------------------------------------------------------------
# Stage 2: Build Rust backend
# ---------------------------------------------------------------------------
FROM rust:1.99.1-slim AS backend-builder

# CARGO_PROFILE=release → optimised prod binary (slower build)
# CARGO_PROFILE=debug   → fast dev build (default)
ARG CARGO_PROFILE=debug

WORKDIR /build

RUN apt-get update && apt-get install -y \
	pkg-config libssl-dev \
	&& rm -rf /var/lib/apt/lists/*

# Cache dependencies by copying manifests first
COPY backend/Cargo.toml backend/Cargo.lock* ./
COPY NOTICE /NOTICE
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
	if [ "$CARGO_PROFILE" = "release" ]; then \
		cargo build --release 2>/dev/null || true; \
	else \
		cargo build 2>/dev/null || true; \
	fi && \
	rm -rf src/main.rs \
	       target/${CARGO_PROFILE}/laruche \
	       target/${CARGO_PROFILE}/.fingerprint/laruche-*

COPY backend/src ./src
COPY backend/migrations ./migrations
COPY backend/seeds ./seeds
COPY backend/assets ./assets
# ⚠️  Requires `make sqlx-prepare` before docker build (creates backend/.sqlx/)
COPY backend/.sqlx ./.sqlx

ENV SQLX_OFFLINE=true
RUN if [ "$CARGO_PROFILE" = "release" ]; then \
		cargo build --release; \
	else \
		cargo build; \
	fi

# ---------------------------------------------------------------------------
# Stage 3: Final image — Debian slim + Caddy + backend binary
# ---------------------------------------------------------------------------
FROM debian:trixie-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
	ca-certificates \
	caddy \
	wget \
	&& rm -rf /var/lib/apt/lists/*

# Licences & notices
COPY NOTICE ./NOTICE

# Copy frontend static bundle
COPY --from=frontend-builder /build/dist /usr/share/caddy

# Copy backend binary and SQL migrations
# ARG must be re-declared in the new stage to be accessible
ARG CARGO_PROFILE=debug
COPY --from=backend-builder /build/target/${CARGO_PROFILE}/laruche ./laruche
COPY --from=backend-builder /build/migrations ./migrations
COPY --from=backend-builder /build/seeds ./seeds

# Copy Caddy configuration
COPY docker/Caddyfile /etc/caddy/Caddyfile

# Script de démarrage (backend en arrière-plan + caddy au premier plan)
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# 80 : HTTP  |  443 : HTTPS automatique (Caddy Let's Encrypt)
EXPOSE 80 443

# Healthcheck backend: Caddy depends on Rust API availability on :8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=5 \
	CMD wget -qO- http://127.0.0.1:8080/notice >/dev/null || exit 1

ENTRYPOINT ["/entrypoint.sh"]
