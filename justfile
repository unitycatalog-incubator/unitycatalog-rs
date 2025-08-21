set dotenv-load := true

# Show available commands
default:
    @just --list --justfile {{ justfile() }}

# main code generation command. This will run all generation for unity types.
[group('generate')]
generate: generate-proto generate-code

# run all code generation for unitycatalog and external types.
[group('generate')]
generate-full: generate-common-ext generate-build-ext generate-proto generate-code

# run code generation for proto files.
[group('generate')]
generate-proto:
    buf generate proto
    npx -y @redocly/cli bundle --remove-unused-components openapi/openapi.yaml > tmp.yaml
    mv tmp.yaml openapi/openapi.yaml
    cargo clippy --fix --allow-dirty --allow-staged

# generate rest server and client code with build crate.
[group('generate')]
generate-code:
    buf build --output {{ justfile_directory() }}/descriptors.bin proto
    cargo run --bin unitycatalog-build -- \
      --output-common crates/common/src/codegen \
      --output-server crates/server/src/codegen \
      --output-client crates/client/src/codegen \
      --descriptors {{ justfile_directory() }}/descriptors.bin
    rm {{ justfile_directory() }}/descriptors.bin
    cargo clippy --fix --allow-dirty --allow-staged --all-features
    cargo fmt

# generate auxiliary types in common crate. (custom google.protobuf build)
[group('generate')]
generate-common-ext:
    just crates/common/generate

# generate types for build crate. (google.api and gnostic file extensions)
[group('generate')]
generate-build-ext:
    just crates/build/generate

# generate types for node client. these are all slow changing external types
[group('generate')]
generate-node:
    just node/client/generate

sqlx-prepare: start_pg
    # Wait for PostgreSQL to be ready
    sleep 1
    # Run migrations to create tables
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo sqlx migrate run --source ./crates/postgres/migrations
    # Prepare SQLx
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo sqlx prepare --workspace -- --tests
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
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo sqlx migrate run --source ./crates/postgres/migrations
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres RUST_LOG=INFO \
        cargo run -p unitycatalog-cli -- server --rest --use-db

# Run local docker emvironment
compose:
    docker-compose -p unitycatalog-rs -f compose/local.compose.yaml up -d

compose-full:
    docker-compose -p unitycatalog-rs -f compose/sandbox.compose.yaml up -d

docs:
    npm run dev -w docs

update-openapi:
    just app/update-openapi
    npx -y @redocly/cli bundle --remove-unused-components openapi/openapi.yaml > tmp.yaml
    mv tmp.yaml openapi/openapi.yaml

# build python bindings
[group('build')]
build-py: build-py-client

# build python client bindings
[group('build')]
build-py-client:
    uv run maturin develop --uv \
      --manifest-path python/client/Cargo.toml

# build python server bindings
[group('build')]
build-py-server:
    uv run maturin develop --uv \
      --manifest-path crates/cli/Cargo.toml

# build node bindings
[group('build')]
build-node:
    npm run build -w @unitycatalog/client

# build node bindings
[group('build')]
build-docker:
    docker build -f docker/Dockerfile -t unitycatalog-rs:dev .

# run marimo notebook server
notebook:
    uv run --directory notebooks marimo edit client.py

test-api:
    UC_SERVER_URL="http://localhost:8080/api/2.1/unity-catalog/" cargo run -p unitycatalog-cli -- test

[group('test')]
record-integration:
    UC_INTEGRATION_URL="$DATABRICKS_HOST" \
    UC_INTEGRATION_TOKEN="$DATABRICKS_TOKEN" \
    UC_INTEGRATION_DIR="{{ justfile_directory() }}/test_data/recordings" \
    UC_INTEGRATION_STORAGE_ROOT="$DATABRICKS_STORAGE_ROOT" \
    UC_INTEGRATION_RECORD="true" \
    cargo run --bin unitycatalog-acceptance
