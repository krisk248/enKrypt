# WasmPT build orchestration.
# Usage: `just <target>`. Run `just --list` to see all targets.
# (If you don't have `just`, the equivalent commands are shown in the README.)

set shell := ["bash", "-uc"]

# Default: show available targets.
default:
    @just --list

# Build the Rust crypto core to WebAssembly (release, wasm-opt -Oz).
build-wasm:
    cd crypto-core && wasm-pack build --target web --release --out-dir ../web/src/wasm

# Install web dependencies.
install:
    cd web && npm install

# Build the whole app (wasm first, then the web bundle) into web/dist.
build: build-wasm
    cd web && npm run build

# Run the dev server (expects wasm already built once via `just build-wasm`).
dev:
    cd web && npm run dev

# Preview the production build on http://localhost:4173
preview:
    cd web && npm run preview

# Run all tests: Rust crypto core + web (Vitest).
test: test-rust test-web

# Rust unit/integration tests, including GnuPG interop.
test-rust:
    cd crypto-core && cargo test

# Web RPC + utility tests.
test-web:
    cd web && npm test

# Type-check the Svelte/TypeScript app.
check:
    cd web && npm run check

# Build the single self-contained Docker image.
docker-build:
    docker build -t enkrypt:latest .

# Run the container → http://localhost:8080
docker-run: docker-build
    docker run --rm -p 8080:8080 --read-only --tmpfs /tmp \
      --cap-drop ALL --security-opt no-new-privileges enkrypt:latest

# Bring the stack up with docker compose.
up:
    docker compose up -d --build

# Tear the stack down.
down:
    docker compose down

# Remove build artifacts.
clean:
    rm -rf web/dist web/src/wasm crypto-core/target
