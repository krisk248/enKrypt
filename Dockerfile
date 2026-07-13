# syntax=docker/dockerfile:1
#
# enKrypt — single self-contained static image.
#
# Three stages:
#   1. wasm   — compile the Rust crypto core to WebAssembly (rpgp).
#   2. web    — build the Svelte/Vite bundle and pre-gzip the assets.
#   3. runtime — nginx:alpine serving the static files (tiny, multi-arch).
#
# The result is one small image (~55 MB) with zero runtime dependencies on the
# host. All cryptography runs client-side in the browser; nginx only serves
# files. Build:  docker build -t enkrypt:latest .

# ---------------------------------------------------------------------------
# Stage 1 — build the WebAssembly crypto core
# ---------------------------------------------------------------------------
FROM rust:1-bookworm AS wasm

# wasm-pack + the wasm target. Pin wasm-pack for reproducible builds.
RUN rustup target add wasm32-unknown-unknown \
 && curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh

WORKDIR /src
COPY crypto-core ./crypto-core
RUN wasm-pack build crypto-core --target web --release --out-dir /wasm-pkg

# ---------------------------------------------------------------------------
# Stage 2 — build the web app
# ---------------------------------------------------------------------------
FROM node:20-bookworm-slim AS web

WORKDIR /app
COPY web/package.json web/package-lock.json ./
RUN npm ci

COPY web ./
# Drop in the freshly built wasm package from stage 1.
COPY --from=wasm /wasm-pkg ./src/wasm
RUN npm run build

# Pre-gzip static assets so nginx can serve them with gzip_static (zero runtime
# CPU for the 2.4 MB wasm, which compresses to ~0.8 MB).
RUN find dist -type f \
      \( -name '*.js' -o -name '*.css' -o -name '*.wasm' -o -name '*.html' \
         -o -name '*.svg' -o -name '*.json' -o -name '*.webmanifest' \) \
      -exec gzip -9 -k {} \;

# ---------------------------------------------------------------------------
# Stage 3 — runtime
# ---------------------------------------------------------------------------
# nginx-unprivileged runs as a non-root user (uid 101) and listens on 8080 —
# no capabilities, no root, ideal for homelab / Kubernetes / rootless Docker.
FROM nginxinc/nginx-unprivileged:1.27-alpine AS runtime

LABEL org.opencontainers.image.title="enKrypt" \
      org.opencontainers.image.description="100% client-side OpenPGP encryption (self-hostable)" \
      org.opencontainers.image.source="https://github.com/kannan/enkrypt" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"

COPY deploy/nginx.conf /etc/nginx/nginx.conf
COPY --from=web /app/dist /usr/share/nginx/html

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
  CMD wget -qO- http://127.0.0.1:8080/ >/dev/null 2>&1 || exit 1

# The base image already runs as the unprivileged uid 101; just launch nginx.
CMD ["nginx", "-g", "daemon off;"]
