# ---------------------------------------------------------------------------
# Stage 1: Build frontend
# ---------------------------------------------------------------------------
FROM node:26-alpine AS frontend-builder

WORKDIR /build

COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

# ---------------------------------------------------------------------------
# Stage 2: Build Rust backend (only used by the `runtime` target)
# ---------------------------------------------------------------------------
FROM rust:1.100.0-slim AS backend-builder

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
# Stage 3: Runtime base — Debian slim + Caddy, shared by both targets
# ---------------------------------------------------------------------------
FROM debian:trixie-slim AS runtime-base

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
	ca-certificates \
	caddy \
	wget \
	libssl3 \
	&& rm -rf /var/lib/apt/lists/*

# Licences & notices
COPY NOTICE ./NOTICE

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

# ---------------------------------------------------------------------------
# Stage 4: Prebuilt image — no Rust toolchain and no Node toolchain: CI copies
# the release binary and the frontend bundle built once by the backend and
# frontend jobs (same commit), whatever the BuildKit cache state.
# ---------------------------------------------------------------------------
FROM runtime-base AS runtime-prebuilt

# Directory is produced by the workflow (artifact download), never committed.
ARG PREBUILT_DIR=docker-artifacts
ARG FRONTEND_DIR=frontend-artifacts

COPY ${FRONTEND_DIR}/dist /usr/share/caddy
COPY ${PREBUILT_DIR}/laruche ./laruche
COPY backend/migrations ./migrations
COPY backend/seeds ./seeds

# ---------------------------------------------------------------------------
# Stage 5: From-source image (default target: local `docker build`, releases)
# ---------------------------------------------------------------------------
FROM runtime-base AS runtime

# ARG must be re-declared in the new stage to be accessible
ARG CARGO_PROFILE=debug

COPY --from=frontend-builder /build/dist /usr/share/caddy
COPY --from=backend-builder /build/target/${CARGO_PROFILE}/laruche ./laruche
COPY --from=backend-builder /build/migrations ./migrations
COPY --from=backend-builder /build/seeds ./seeds
