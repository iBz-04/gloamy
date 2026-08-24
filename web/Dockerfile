# syntax=docker/dockerfile:1.7

# Gloamy web UI.
#
# Builds the Vue app, then serves `dist/` from nginx. The same nginx also
# reverse-proxies the daemon's API paths, so the browser only ever talks to one
# origin — no CORS preflights and no mixed-content blocking. This mirrors the
# dev-time proxy in vite.config.ts.

# ══════════════════════════════════════════════════════════════════════════════
# builder
# ══════════════════════════════════════════════════════════════════════════════
# Node 22 matches web/.nvmrc.
FROM node:22-bookworm-slim AS builder

ENV PNPM_HOME=/pnpm \
    PATH=/pnpm:$PATH \
    COREPACK_ENABLE_DOWNLOAD_PROMPT=0
# Corepack pins pnpm to the `packageManager` field in package.json.
RUN corepack enable

WORKDIR /app

# Manifests first, so the dependency layer survives source-only edits.
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN --mount=type=cache,id=gloamy-web-pnpm-store,target=/pnpm/store,sharing=locked \
    pnpm install --frozen-lockfile --store-dir /pnpm/store

COPY . .

# Empty means "same origin", which is exactly what the nginx config below
# serves. Override only when the daemon is reached on a different origin.
ARG VITE_GLOAMY_API_BASE=""
ENV VITE_GLOAMY_API_BASE=$VITE_GLOAMY_API_BASE
# Runs vue-tsc then vite build, so a type error fails the image build.
RUN pnpm build

# ══════════════════════════════════════════════════════════════════════════════
# release
# ══════════════════════════════════════════════════════════════════════════════
FROM nginx:1.29-alpine AS release

# The nginx image's entrypoint runs envsubst over /etc/nginx/templates.
# NGINX_ENVSUBST_FILTER restricts it to GLOAMY_* names so nginx's own
# $variables (e.g. $host, $request_uri) are left intact.
ENV NGINX_ENVSUBST_FILTER=^GLOAMY_ \
    GLOAMY_DAEMON_URL=http://gloamy:42617

# Drop the image's stock server block. Without this, a failed template render
# would silently fall back to plain static serving with no API proxy; removing it
# means nginx refuses to start instead.
RUN rm /etc/nginx/conf.d/default.conf

COPY nginx-proxy.conf /etc/nginx/gloamy-proxy.conf
COPY nginx.conf.template /etc/nginx/templates/default.conf.template
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80
