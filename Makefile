# ── La Ruche — Makefile ───────────────────────────────────────────
# Usage: make <target>   (from the project root)

.PHONY: help dev dev-frontend dev-full build build-frontend \
        test test-integration test-all test-frontend \
        lint lint-fix check \
        migrate migrate-test migrate-revert sqlx-prepare seed-dev \
        docker docker-up docker-down docker-prod docker-logs \
        clean install-tools setup setup-hooks commit fmt

# Variables
DB_URL      := postgres://laruche:laruche@localhost:5432/laruche
DB_URL_TEST := postgres://laruche:laruche@localhost:5432/laruche_test
BINARY      := backend/target/release/laruche

help: ## Show this help
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

# ── Development ───────────────────────────────────────────────────
dev: ## Start the dev environment (postgres + minio + backend hot-reload)
	docker compose up -d postgres minio
	cd backend && DATABASE_URL=$(DB_URL) cargo watch -x run

dev-frontend: ## Start the Vue 3 frontend in dev mode
	cd frontend && npm run dev

dev-full: ## Start everything (backend + frontend)
	$(MAKE) -j2 dev dev-frontend

# ── Build ─────────────────────────────────────────────────────────
build: ## Build the release binary
	cd backend && SQLX_OFFLINE=true cargo build --release

build-frontend: ## Build the frontend
	cd frontend && npm run build

# ── Tests ─────────────────────────────────────────────────────────
test: ## Backend unit tests (no DB required)
	cd backend && cargo test --lib

test-integration: ## Backend integration tests (requires postgres)
	cd backend && DATABASE_URL=$(DB_URL_TEST) cargo test --test '*'

test-all: test test-integration ## Run all tests

test-frontend: ## Frontend TypeScript type check
	cd frontend && npm run typecheck

# ── Code quality ──────────────────────────────────────────────────
lint: ## Clippy + fmt check + npm lint
	cd backend && cargo fmt --check
	cd backend && cargo clippy -- -D warnings
	cd frontend && npm run lint

lint-fix: ## Format Rust and Typescript code
	cd backend && cargo fmt 
	cd backend && cargo clippy --fix --allow-dirty
	cd frontend && npm run lint -- --fix

check: ## Quick check (no full build)
	cd backend && SQLX_OFFLINE=true cargo check

# ── Database ──────────────────────────────────────────────────────
migrate: ## Apply migrations
	cd backend && DATABASE_URL=$(DB_URL) sqlx migrate run

migrate-test: ## Apply migrations on the test DB
	cd backend && DATABASE_URL=$(DB_URL_TEST) sqlx migrate run

migrate-revert: ## Revert the last migration
	cd backend && DATABASE_URL=$(DB_URL) sqlx migrate revert

seed-dev: ## ⚠️  Inject dev seed data (resets DB — remove before v1.0)
	docker compose exec -T postgres psql -U laruche -d laruche < backend/seeds/seed_dev.sql

sqlx-prepare: ## Generate SQLx offline metadata (.sqlx/)
	cd backend && DATABASE_URL=$(DB_URL) cargo sqlx prepare

# ── Docker ────────────────────────────────────────────────────────
docker: ## Build the unified image (frontend + backend + Caddy)
	docker build -t laruche:dev .

docker-up: ## Start the full stack via docker compose
	docker compose up -d --build

docker-down: ## Stop the stack
	docker compose down

docker-prod: ## Start in production mode
	docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

docker-logs: ## Tail logs for the laruche service
	docker compose logs -f laruche

# ── Utilities ─────────────────────────────────────────────────────
clean: ## Clean build artifacts
	cd backend && cargo clean
	cd frontend && rm -rf dist node_modules

install-tools: ## Install development tools
	cargo install sqlx-cli --no-default-features --features postgres --locked
	cargo install cargo-watch --locked
	cd frontend && npm ci

setup-hooks: ## Install git hooks (pre-commit: cargo fmt + clippy)
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-commit
	@echo "✅ Git hooks installed (.githooks/)"

setup: ## Full initial setup
	cp backend/.env.example backend/.env
	$(MAKE) setup-hooks
	docker compose up -d postgres minio
	sleep 3
	$(MAKE) migrate
	@echo "✅ La Ruche ready — run 'make dev'"

fmt: ## Format Rust code (cargo fmt)
	cd backend && cargo fmt

commit: ## Safe commit — runs all .githooks hooks then git commit
	@echo "▶ Running pre-commit hooks from .githooks/ ..."
	@for hook in .githooks/*; do \
	  [ -x "$$hook" ] && "$$hook" || true; \
	done
	@echo "✅ Hooks passed — committing"
	git commit $(filter-out $@,$(MAKECMDGOALS))
