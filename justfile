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
