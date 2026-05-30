mod dev 'dev/justfile'

set dotenv-load

# Show available commands
_default:
    @just --list --justfile {{ justfile() }}

# main code generation command. This will run all generation for unity types.
[group('codegen')]
generate: generate-proto generate-code fix

# run all code generation for unitycatalog types.
#
# Generation of external types (google.api / gnostic extensions) now lives in
# the `../trestle` codegen repo (`olai-codegen`); see `generate-proto-gen-fixtures`.
[group('codegen')]
generate-full: generate-proto generate-code fix

# run code generation for proto files.
[group('codegen')]
generate-proto:
    buf generate proto/unitycatalog
    just generate-openapi
    buf generate proto/sharing --template {{ justfile_directory() }}/buf.gen.sharing.yaml

# Update the generated openapi spec with validation extracted from generated jsonschema.
[group('codegen')]
generate-openapi:
    buf generate --template '{"version":"v2","plugins":[{"remote":"buf.build/bufbuild/protoschema-jsonschema:v0.5.2","opt": ["target=proto-strict-bundle"], "out":"openapi/jsonschema"}]}' proto
    buf build --output {{ justfile_directory() }}/descriptors.bin proto/unitycatalog
    cargo run --manifest-path ../trestle/crates/trestle/Cargo.toml --bin trestle -- enrich-openapi \
      --jsonschema-dir openapi/jsonschema \
      --descriptors {{ justfile_directory() }}/descriptors.bin
    rm -f {{ justfile_directory() }}/descriptors.bin
    rm -rf openapi/jsonschema
    npx -y @redocly/cli bundle --remove-unused-components openapi/openapi.yaml > tmp.yaml
    mv tmp.yaml openapi/openapi.yaml
    npm run openapi

# generate rest server and client code with build crate.
[group('codegen')]
generate-code:
    buf build --output {{ justfile_directory() }}/descriptors.bin proto/unitycatalog
    cargo run --manifest-path ../trestle/crates/trestle/Cargo.toml --bin trestle -- generate --config proto-gen.yaml \
      --descriptors {{ justfile_directory() }}/descriptors.bin
    rm {{ justfile_directory() }}/descriptors.bin
    just fmt
    mv python/client/src/codegen/_client.pyi python/client/python/unitycatalog_client/_client.pyi
    # Splice in the hand-written PyO3 surface (exceptions, free functions,
    # and the hand-written `#[pymethods]` on `TemporaryCredentialClient`).
    # The codegen-emitted empty `class TemporaryCredentialClient: ...`
    # placeholder is stripped first so the supplement can replace it with
    # the fully-typed declaration. The supplement lives outside the
    # package directory so type-checkers do not validate it standalone
    # (it is a fragment, not a complete stub). Source-of-truth:
    # `python/client/_client_supplement.pyi`.
    grep -v '^class TemporaryCredentialClient: \.\.\.$' \
      python/client/python/unitycatalog_client/_client.pyi \
      > python/client/python/unitycatalog_client/_client.pyi.tmp
    mv python/client/python/unitycatalog_client/_client.pyi.tmp \
      python/client/python/unitycatalog_client/_client.pyi
    cat python/client/_client_supplement.pyi \
      >> python/client/python/unitycatalog_client/_client.pyi

# CURRENTLY not used, but we may need it again come validation ...
[group('codegen')]
generate-common-ext:
    just crates/common/generate

# generate types for node client. these are all slow changing external types
[group('codegen')]
generate-node:
    just node/client/generate

# Regenerate proto-gen test fixture descriptors from proto/ source files.
[group('codegen')]
generate-proto-gen-fixtures:
    buf dep update ../trestle/crates/trestle-codegen/proto
    buf build --output {{ justfile_directory() }}/../trestle/crates/olai-codegen/proto/example.bin \
      ../trestle/crates/olai-codegen/proto/

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

# validate code examples type-check and docs build successfully
[group('test')]
validate-examples:
    cargo check -p unitycatalog-examples
    uvx ty check examples/python/
    npm run build -w @unitycatalog/client
    npx tsc --noEmit -p examples/typescript/tsconfig.json
    npm run build -w docs

# build python bindings
[group('build')]
build-py: build-py-client

# build python client bindings
[group('build')]
build-py-client:
    uv run maturin develop --uv --manifest-path python/client/Cargo.toml

# build python server bindings
[group('build')]
build-py-server:
    uv run maturin develop --uv --manifest-path crates/cli/Cargo.toml

# build node bindings
[group('build')]
build-node:
    npm run build -w @unitycatalog/client

# build node bindings
[group('build')]
build-docker:
    docker build -f crates/cli/Dockerfile -t unitycatalog-rs:dev .

# build sqlx queries to support offline mode
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

# run node integration tests (starts UC server automatically)
[group('test')]
test-node-integration:
    npm run build -w @unitycatalog/client
    npm run test:integration -w @unitycatalog/client

# run integration tests using mocked server responses
[group('test')]
integration:
    UC_INTEGRATION_DIR="{{ justfile_directory() }}/crates/acceptance/recordings" \
    UC_INTEGRATION_STORAGE_ROOT="$DATABRICKS_STORAGE_ROOT" \
    UC_INTEGRATION_RECORD="false" \
    cargo run --bin unitycatalog-acceptance

# run journey tests live against the open-source Java Unity Catalog server.
# Boots the server via docker compose, waits for its healthcheck, then runs
# every OssJava-compatible journey. Tear down with:
#   docker compose -f dev/uc-oss.compose.yaml down -v
[group('test')]
integration-oss-java:
    docker compose -f dev/uc-oss.compose.yaml up -d --wait
    UC_INTEGRATION_PROFILE="oss_java" \
    UC_INTEGRATION_URL="http://localhost:8080" \
    cargo test -p unitycatalog-acceptance -- journey_tests_live --nocapture

# run object-store integration tests against the docker `full` profile
# (UC server + SeaweedFS + Postgres + Azurite). Marks the test crate's
# `#[ignore]` tests as runnable.
[group('test')]
integration-object-store:
    docker compose -f dev/compose.yaml --profile full up -d
    UC_INTEGRATION_URL="http://localhost:8080/api/2.1/unity-catalog/" \
    cargo test -p unitycatalog-object-store --test integration -- --ignored --test-threads=1

[group('test')]
integration-record:
    UC_INTEGRATION_URL="$DATABRICKS_HOST" \
    UC_INTEGRATION_TOKEN="$DATABRICKS_TOKEN" \
    UC_INTEGRATION_DIR="{{ justfile_directory() }}/crates/acceptance/recordings" \
    UC_INTEGRATION_STORAGE_ROOT="$DATABRICKS_STORAGE_ROOT" \
    UC_INTEGRATION_RECORD="true" \
    cargo run --bin unitycatalog-acceptance

# lint nodejs bindings
lint-node:
    npm run lint -w @unitycatalog/client

fix: fix-rust fix-node fix-py
    just fmt

# fix nodejs bindings
fix-node:
    npm run lint-fix -w @unitycatalog/client

# fix rust code
fix-rust:
    cargo clippy --fix --workspace --allow-dirty --allow-staged

fix-py:
    uvx ruff check --fix

fmt:
    cargo fmt
    buf format proto/ --write
    uvx ruff format .
