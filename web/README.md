# Gloamy Web

Gloamy Web is the primary GUI for the project. It is a browser app built with:

- [Vue 3](https://vuejs.org/) with [Vite](https://vite.dev/) and TypeScript.
- [Pinia](https://pinia.vuejs.org/) for state, [Vue Router](https://router.vuejs.org/) for navigation.
- [Tailwind CSS v4](https://tailwindcss.com/) + [reka-ui](https://reka-ui.com/) for the component layer.
- [vue-i18n](https://vue-i18n.intlify.dev/) for localization.

It is a thin client over the Gloamy daemon's REST + SSE API. All application
logic lives in the daemon; this app only renders and talks HTTP.

## Prerequisites

- Node.js (see `.nvmrc`) and [pnpm](https://pnpm.io/).
- A running Gloamy daemon. From the repository root:

  ```bash
  cargo run -- serve
  ```

  The daemon listens on `http://127.0.0.1:42617` by default.

## Development

Install dependencies:

```bash
pnpm install
```

Start the dev server:

```bash
pnpm dev
```

The app is served at <http://localhost:1420>.

### How the app reaches the daemon

The client calls the daemon over **same-origin paths** (`/api/*` and `/pair`), so
the browser never needs CORS and HTTPS pages never hit mixed-content errors.
The dev server forwards those paths to the daemon.

Point it at a daemon on a different host or port with:

```bash
GLOAMY_DAEMON_URL=http://192.168.1.20:42617 pnpm dev
```

To bypass the proxy entirely and call a daemon origin directly, set
`VITE_GLOAMY_API_BASE` — that daemon must then allow this app's origin via CORS.
The base URL is also editable at runtime on the authentication screen and is
persisted per browser.

## Pairing

The daemon issues a pairing code; enter it on the authentication screen to
exchange it for a bearer token. The token is stored in the browser's
`localStorage` under the `gloamy:` key prefix, so pairing is per-browser and
signing out on one browser does not affect another.

## Building

```bash
pnpm build      # type-checks, then emits dist/
pnpm preview    # serves dist/ with the same daemon proxy
```

`dist/` is a static bundle. The router uses hash history, so it can be served
from any static host with no rewrite rules. Whatever serves `dist/` in a
deployment must forward `/api/*` and `/pair` to the daemon for the same-origin
contract to hold.

## Other commands

```bash
pnpm lint       # ESLint
pnpm lint:fix   # ESLint with auto-fix
pnpm typecheck  # Vue + TypeScript validation
```
