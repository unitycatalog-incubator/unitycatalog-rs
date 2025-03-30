set dotenv-load := true

# Show available commands
default:
    @just --list --justfile {{ justfile() }}

# run code generation for proto files.
generate:
    buf generate proto
    npx -y @redocly/cli bundle --remove-unused-components openapi/openapi.yaml > tmp.yaml
    mv tmp.yaml openapi/openapi.yaml
    cargo clippy --fix --allow-dirty --allow-staged

generate-app:
    just app/generate

generate-types:
    just unitycatalog/common/generate

sqlx-prepare: start_pg
    # Wait for PostgreSQL to be ready
    sleep 1
    # Run migrations to create tables
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo sqlx migrate run --source ./unitycatalog/postgres/migrations
    # Prepare SQLx
    cargo sqlx prepare --workspace -- --tests
    # Clean up
    @just stop_pg

# Start PostgreSQL container to prepare SQLx or to run tests
start_pg:
    docker run -d \
        --name unitycatalog-pg \
        -e POSTGRES_PASSWORD=postgres \
        -e POSTGRES_USER=postgres \
        -e POSTGRES_DB=postgres \
        -p 5432:5432 \
        postgres:16

# Stop PostgreSQL container
stop_pg:
    docker stop unitycatalog-pg && docker rm unitycatalog-pg

rest:
    @RUST_LOG=INFO cargo run --bin uc server --rest

rest-db:
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres RUST_LOG=INFO \
        cargo run --bin uc server --rest --use-db

# Run local docker emvironment
compose:
    docker-compose -p unitycatalog-rs -f compose/local.compose.yaml up -d

# run local app
app:
  cd app && npm run tauri dev

generate-py:
  uv run scripts/prepare_jsonschema.py
  uv run datamodel-codegen \
    --input ./tmp_schemas/ \
    --input-file-type jsonschema \
    --output python/models/
  rm -rf tmp_schemas
