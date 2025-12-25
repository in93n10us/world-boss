# StudyFlow Development Makefile
# Common commands for development, testing, and deployment

.PHONY: help setup install-tools db-up db-down db-create db-migrate db-reset \
        build test test-unit test-integration lint fmt check clean run dev \
        docker-build docker-up docker-down logs watch coverage audit

# Default target - show help
help:
	@echo "StudyFlow Development Commands"
	@echo "=============================="
	@echo ""
	@echo "Setup & Installation:"
	@echo "  make setup          - Complete project setup (tools, deps, db)"
	@echo "  make install-tools  - Install required Rust tools (sqlx-cli, etc.)"
	@echo ""
	@echo "Database:"
	@echo "  make db-up          - Start PostgreSQL container"
	@echo "  make db-down        - Stop PostgreSQL container"
	@echo "  make db-create      - Create database"
	@echo "  make db-migrate     - Run database migrations"
	@echo "  make db-reset       - Drop, recreate, and migrate database"
	@echo ""
	@echo "Development:"
	@echo "  make build          - Build all workspace crates"
	@echo "  make run            - Run the API server"
	@echo "  make dev            - Run with auto-reload (cargo-watch)"
	@echo "  make watch          - Watch and run tests on file changes"
	@echo ""
	@echo "Testing:"
	@echo "  make test           - Run all tests"
	@echo "  make test-unit      - Run unit tests only"
	@echo "  make test-integration - Run integration tests only"
	@echo "  make coverage       - Generate test coverage report"
	@echo ""
	@echo "Code Quality:"
	@echo "  make lint           - Run clippy linter"
	@echo "  make fmt            - Format code with rustfmt"
	@echo "  make check          - Run all checks (fmt, lint, test)"
	@echo "  make audit          - Security audit of dependencies"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build   - Build Docker image"
	@echo "  make docker-up      - Start all services with docker-compose"
	@echo "  make docker-down    - Stop all services"
	@echo "  make logs           - View docker-compose logs"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean          - Remove build artifacts"

# Complete project setup
setup: install-tools db-up db-create db-migrate
	@echo "✓ Setup complete! Run 'make run' to start the server."

# Install required development tools
install-tools:
	@echo "Installing Rust toolchain components..."
	rustup component add rustfmt clippy
	@echo "Installing sqlx-cli..."
	cargo install sqlx-cli --no-default-features --features postgres --locked
	@echo "Installing cargo-watch (optional, for auto-reload)..."
	cargo install cargo-watch || true
	@echo "Installing cargo-tarpaulin (optional, for coverage)..."
	cargo install cargo-tarpaulin || true
	@echo "✓ Tools installed"

# Database management
db-up:
	@echo "Starting PostgreSQL container..."
	docker-compose up -d postgres
	@echo "Waiting for PostgreSQL to be ready..."
	@sleep 3
	@echo "✓ PostgreSQL is running"

db-down:
	@echo "Stopping PostgreSQL container..."
	docker-compose down
	@echo "✓ PostgreSQL stopped"

db-create:
	@echo "Creating database..."
	sqlx database create
	@echo "✓ Database created"

db-migrate:
	@echo "Running migrations..."
	sqlx migrate run --source crates/db/migrations
	@echo "✓ Migrations applied"

db-reset:
	@echo "Resetting database..."
	sqlx database drop -y || true
	sqlx database create
	sqlx migrate run --source crates/db/migrations
	@echo "✓ Database reset complete"

# Build commands
build:
	@echo "Building workspace..."
	cargo build --workspace
	@echo "✓ Build complete"

build-release:
	@echo "Building release binaries..."
	cargo build --workspace --release
	@echo "✓ Release build complete"

# Run commands
run:
	@echo "Starting API server..."
	cargo run --package api

dev:
	@echo "Starting development server with auto-reload..."
	cargo watch -x 'run --package api'

watch:
	@echo "Watching for changes and running tests..."
	cargo watch -x test

# Testing commands
test:
	@echo "Running all tests..."
	cargo test --workspace

test-unit:
	@echo "Running unit tests..."
	cargo test --workspace --lib

test-integration:
	@echo "Running integration tests..."
	cargo test --workspace --test '*'

coverage:
	@echo "Generating test coverage report..."
	cargo tarpaulin --workspace --out Html --output-dir coverage --skip-clean
	@echo "✓ Coverage report generated in coverage/index.html"

# Code quality commands
lint:
	@echo "Running clippy..."
	cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	@echo "Formatting code..."
	cargo fmt --all

fmt-check:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

check: fmt-check lint test
	@echo "✓ All checks passed"

audit:
	@echo "Running security audit..."
	cargo audit

# Docker commands
docker-build:
	@echo "Building Docker image..."
	docker build -t studyflow-api:latest .
	@echo "✓ Docker image built"

docker-up:
	@echo "Starting all services..."
	docker-compose up -d
	@echo "✓ Services started"

docker-down:
	@echo "Stopping all services..."
	docker-compose down
	@echo "✓ Services stopped"

logs:
	@echo "Showing logs (Ctrl+C to exit)..."
	docker-compose logs -f

# Cleanup
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "✓ Clean complete"

# Development workflow shortcuts
quick-check: fmt lint
	@echo "✓ Quick check complete"

pre-commit: fmt lint test
	@echo "✓ Pre-commit checks passed"

# Database utilities
db-shell:
	@echo "Connecting to database..."
	docker-compose exec postgres psql -U studyflow -d studyflow_dev

db-backup:
	@echo "Creating database backup..."
	@mkdir -p backups
	docker-compose exec -T postgres pg_dump -U studyflow studyflow_dev > backups/backup_$$(date +%Y%m%d_%H%M%S).sql
	@echo "✓ Backup created in backups/"

db-restore:
	@echo "Restoring database from backup..."
	@read -p "Enter backup file path: " backup_file; \
	docker-compose exec -T postgres psql -U studyflow studyflow_dev < $$backup_file
	@echo "✓ Database restored"

# Dependency management
update-deps:
	@echo "Updating dependencies..."
	cargo update
	@echo "✓ Dependencies updated"

outdated:
	@echo "Checking for outdated dependencies..."
	cargo outdated

# Documentation
docs:
	@echo "Building documentation..."
	cargo doc --workspace --no-deps --open

docs-private:
	@echo "Building documentation (including private items)..."
	cargo doc --workspace --no-deps --document-private-items --open

# CI/CD simulation
ci: fmt-check lint test
	@echo "✓ CI checks passed"

# Environment setup
env-setup:
	@if [ ! -f .env ]; then \
		echo "Creating .env from .env.example..."; \
		cp .env.example .env; \
		echo "✓ .env created - please update with your configuration"; \
	else \
		echo ".env already exists"; \
	fi