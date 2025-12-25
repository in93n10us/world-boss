# World Boss

A Rust-based backend for the StudyFlow application - a study planning and time-blocking platform for students.

## Overview

World Boss is built with a clean, modular architecture using Rust's workspace feature to separate concerns:

- **API Layer** (`crates/api`): HTTP server, routes, middleware, and request handling
- **Core Layer** (`crates/core`): Business logic, domain models, and use cases
- **Database Layer** (`crates/db`): Data persistence, migrations, and repository implementations

## Architecture

This project follows hexagonal (ports and adapters) architecture principles:

- **Domain models** are pure Rust structs with business logic
- **Ports** define interfaces for external dependencies
- **Adapters** implement ports for specific technologies (PostgreSQL, HTTP, etc.)

## Tech Stack

- **Web Framework**: Axum 0.7 (ergonomic, type-safe routing)
- **Async Runtime**: Tokio (full-featured async runtime)
- **Database**: PostgreSQL with SQLx (compile-time checked queries)
- **Logging**: Tracing with structured JSON output
- **Configuration**: Environment-based with validation
- **Testing**: Integration and unit tests with test utilities

## Prerequisites

- Rust 1.75+ (stable toolchain)
- Docker and Docker Compose (for PostgreSQL)
- SQLx CLI for migrations

## Quick Start

### 1. Install Rust (if not already installed)

bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add rustfmt clippy


### 2. Install SQLx CLI

bash
cargo install sqlx-cli --no-default-features --features postgres


### 3. Clone and Setup

bash
# Start PostgreSQL database
docker-compose up -d

# Copy environment template
cp .env.example .env

# Edit .env with your configuration (defaults should work for local dev)
# DATABASE_URL=postgres://worldboss:worldboss@localhost:5432/worldboss_dev

# Create database
sqlx database create

# Run migrations (when available)
sqlx migrate run

# Build the project
cargo build --workspace


### 4. Run the Application

bash
# Using cargo
cargo run --bin api

# Or using make
make run


The API server will start on `http://localhost:8000`

### 5. Verify Installation

bash
# Check health endpoint
curl http://localhost:8000/health

# Expected response:
# {"status":"ok","timestamp":"2024-01-15T10:30:00Z"}


## Development

### Common Commands

bash
# Run all tests
make test
# or
cargo test --workspace

# Run with logging
RUST_LOG=debug cargo run --bin api

# Format code
make fmt
# or
cargo fmt --all

# Lint code
make lint
# or
cargo clippy --workspace -- -D warnings

# Check code without building
cargo check --workspace

# Run specific crate tests
cargo test -p api
cargo test -p core
cargo test -p db


### Database Management

bash
# Create database
sqlx database create

# Drop database
sqlx database drop

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Add new migration
sqlx migrate add <migration_name>


### Running Tests

bash
# All tests
cargo test --workspace

# Integration tests only
cargo test --test '*'

# Unit tests only
cargo test --lib

# With output
cargo test -- --nocapture

# Specific test
cargo test test_health_check


## Project Structure


world-boss/
├── Cargo.toml                 # Workspace root
├── .env.example              # Environment template
├── docker-compose.yml        # Local PostgreSQL setup
├── Makefile                  # Development commands
├── rust-toolchain.toml       # Rust version pinning
├── rustfmt.toml             # Code formatting rules
├── clippy.toml              # Linting configuration
│
├── crates/
│   ├── api/                 # HTTP API server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs      # Application entry point
│   │   │   ├── lib.rs       # Library exports
│   │   │   ├── config.rs    # Configuration management
│   │   │   ├── error.rs     # Error types and handling
│   │   │   ├── routes/      # HTTP route handlers
│   │   │   │   ├── mod.rs
│   │   │   │   └── health.rs
│   │   │   └── middleware/  # HTTP middleware
│   │   │       ├── mod.rs
│   │   │       └── logging.rs
│   │   └── tests/           # Integration tests
│   │       ├── common/
│   │       │   └── mod.rs
│   │       └── health_check.rs
│   │
│   ├── core/                # Business logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── domain/      # Domain models
│   │       │   └── mod.rs
│   │       └── ports/       # Interface definitions
│   │           └── mod.rs
│   │
│   └── db/                  # Database layer
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   └── connection.rs
│       ├── migrations/      # SQL migrations
│       │   └── .gitkeep
│       └── tests/
│           └── connection_test.rs


## Configuration

Configuration is managed through environment variables. See `.env.example` for all available options:

- `DATABASE_URL`: PostgreSQL connection string
- `SERVER_HOST`: API server bind address (default: 0.0.0.0)
- `SERVER_PORT`: API server port (default: 8000)
- `RUST_LOG`: Logging level (debug, info, warn, error)
- `JWT_SECRET`: Secret key for JWT token signing (required for auth)

## API Endpoints

### Health Check


GET /health


Returns server health status and timestamp.

**Response:**
json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z"
}


## Testing Strategy

### Unit Tests

Located inline with source code using `#[cfg(test)]` modules. Test business logic, domain models, and pure functions.

bash
cargo test --lib


### Integration Tests

Located in `tests/` directories. Test HTTP endpoints, database operations, and component interactions.

bash
cargo test --test '*'


### Test Utilities

Common test utilities are in `crates/api/tests/common/mod.rs`:
- Test server spawning
- Test database setup and cleanup
- HTTP client helpers

## Code Quality

### Formatting

Code is formatted using `rustfmt` with project-specific rules in `rustfmt.toml`.

bash
cargo fmt --all


### Linting

Code is linted using `clippy` with strict rules in `clippy.toml`.

bash
cargo clippy --workspace -- -D warnings


### Pre-commit Checklist

Before committing code:

1. ✅ Format: `cargo fmt --all`
2. ✅ Lint: `cargo clippy --workspace -- -D warnings`
3. ✅ Test: `cargo test --workspace`
4. ✅ Build: `cargo build --workspace --release`

## Security Considerations

- **Environment Variables**: Never commit `.env` files. Use `.env.example` as template.
- **Database Credentials**: Use strong passwords in production. Rotate regularly.
- **JWT Secrets**: Generate cryptographically secure secrets. Never use defaults in production.
- **SQL Injection**: SQLx provides compile-time query checking. Always use parameterized queries.
- **Input Validation**: Validate all user input using the `validator` crate.
- **Error Messages**: Don't leak sensitive information in error responses.

## Production Deployment

### Build Release Binary

bash
cargo build --release --workspace


Binaries will be in `target/release/`.

### Environment Variables

Ensure all required environment variables are set:
- `DATABASE_URL` with production database credentials
- `JWT_SECRET` with a strong, random secret
- `RUST_LOG=info` for production logging

### Database Migrations

Run migrations before deploying new versions:

bash
sqlx migrate run


### Health Checks

Configure your orchestrator (Kubernetes, Docker Swarm, etc.) to use the `/health` endpoint for readiness and liveness probes.

## Troubleshooting

### Database Connection Issues

bash
# Check PostgreSQL is running
docker-compose ps

# Check database exists
sqlx database create

# Verify connection string in .env
echo $DATABASE_URL


### Compilation Errors

bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check Rust version
rustc --version


### Test Failures

bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name -- --nocapture

# Check test database
docker-compose logs postgres


## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting (`make test && make lint`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Commit Message Convention

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `test:` Test additions or changes
- `refactor:` Code refactoring
- `chore:` Maintenance tasks

## License

[Specify your license here]

## Support

For issues and questions:
- Open an issue on GitHub
- Check existing documentation
- Review test files for usage examples

## Roadmap

- [ ] User authentication and authorization
- [ ] Task management endpoints
- [ ] Study session tracking
- [ ] Progress analytics
- [ ] Notification system
- [ ] Mobile app integration
- [ ] Real-time updates with WebSockets