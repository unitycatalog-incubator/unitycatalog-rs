mod dev 'dev/justfile'

set dotenv-load := true

# Show available commands
_default:
    @just --list --justfile {{ justfile() }}

# main code generation command. This will run all generation for unity types.
[group('codegen')]
generate: generate-proto generate-code

# run all code generation for unitycatalog and external types.
[group('codegen')]
generate-full: generate-common-ext generate-build-ext generate-proto generate-code

# run code generation for proto files.
[group('codegen')]
generate-proto:
    buf generate proto
    just generate-openapi
    cargo clippy --fix --allow-dirty --allow-staged

# Update the generated openapi spec with validation extraced from generated jsonschema.
[group('codegen')]
generate-openapi:
    npx -y @redocly/cli bundle --remove-unused-components openapi/openapi.yaml > tmp.yaml
    mv tmp.yaml openapi/openapi.yaml
    buf generate --template '{"version":"v2","plugins":[{"remote":"buf.build/bufbuild/protoschema-jsonschema","opt": ["target=proto-strict-bundle"], "out":"openapi/jsonschema"}]}' proto
    uv run dev/scripts/update_openapi_schemas.py
    rm -rf openapi/jsonschema
    npm run openapi

# generate rest server and client code with build crate.
[group('codegen')]
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
[group('codegen')]
generate-common-ext:
    just crates/common/generate

# generate types for build crate. (google.api and gnostic file extensions)
[group('codegen')]
generate-build-ext:
    just crates/build/generate

# generate types for node client. these are all slow changing external types
[group('codegen')]
generate-node:
    just node/client/generate

[group('dev')]
rest:
    @RUST_LOG=INFO cargo run --bin uc server --rest

[group('dev')]
rest-db:
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo sqlx migrate run --source ./crates/postgres/migrations
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres RUST_LOG=INFO \
        cargo run -p unitycatalog-cli -- server --rest --use-db


docs:
    npm run dev -w docs

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
    docker build -f crates/cli/Dockerfile -t unitycatalog-rs:dev .

# build sqlx quieries to support offline mode
[group('build')]
build-sqlx: _start_pg_sqlx
    # Wait for PostgreSQL to be ready
    sleep 1
    # Run migrations to create tables
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo sqlx migrate run --source ./crates/postgres/migrations
    # Prepare SQLx
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo sqlx prepare --workspace -- --tests
    # Clean up
    @just _stop_pg_sqlx

_start_pg_sqlx:
    docker run -d \
        --name unitycatalog-sqlx-pg \
        -e POSTGRES_PASSWORD=postgres \
        -e POSTGRES_USER=postgres \
        -e POSTGRES_DB=postgres \
        -p 5432:5432 \
        postgres:16

_stop_pg_sqlx:
    docker stop unitycatalog-sqlx-pg && docker rm unitycatalog-sqlx-pg

[group('test')]
test-node:
    npm run test -w @unitycatalog/client

# run integration tests using mocked server responses
[group('test')]
integration:
    UC_INTEGRATION_DIR="{{ justfile_directory() }}/crates/acceptance/recordings" \
    UC_INTEGRATION_STORAGE_ROOT="$DATABRICKS_STORAGE_ROOT" \
    UC_INTEGRATION_RECORD="false" \
    cargo run --bin unitycatalog-acceptance

[group('test')]
integration-record:
    UC_INTEGRATION_URL="$DATABRICKS_HOST" \
    UC_INTEGRATION_TOKEN="$DATABRICKS_TOKEN" \
    UC_INTEGRATION_DIR="{{ justfile_directory() }}/crates/acceptance/recordings" \
    UC_INTEGRATION_STORAGE_ROOT="$DATABRICKS_STORAGE_ROOT" \
    UC_INTEGRATION_RECORD="true" \
    cargo run --bin unitycatalog-acceptance
