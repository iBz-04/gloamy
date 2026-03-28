# Gloamy Desktop

Gloamy Desktop is the primary GUI for the project. It is built with:

- Rust
  - [Tauri v2](https://beta.tauri.app/start/) as the desktop application framework.
  - [Tauri Store Plugin](https://v2.tauri.app/plugin/store/) for persistence.
  - [Tauri Log Plugin](https://v2.tauri.app/plugin/logging/) for logging.
  - [CrabNebula DevTools](https://v2.tauri.app/develop/debug/crabnebula-devtools/) for development.

- Vue.js 3
  - [Shadcn Vue + Tailwind CSS v4](https://www.shadcn-vue.com/) for components.
  - [Vue Router](https://router.vuejs.org/) for application routing.
  - [Vue I18n](https://vue-i18n.intlify.dev/) for internationalization.
  - [Pinia](https://pinia.vuejs.org/introduction.html) for state management.



## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Prerequisites

Follow [Tauri's prerequisites guide](https://v2.tauri.app/start/prerequisites/) to setup your development environment.

## Installation

Install dependencies with `pnpm install`

## Development

### Background image CDN / manifest

The app can load background-image thumbnails from a JSON manifest.

By default, it will try same-origin: `GET /logo/manifest.v1.json` (served from `public/logo/manifest.v1.json`).

To use a remote CDN/Worker endpoint, configure either:

- `VITE_BACKGROUND_CDN_BASE` (base URL that contains `manifest.v1.json`)
- `VITE_BACKGROUND_MANIFEST_URL` (full URL to the manifest)

Note: the `*.r2.cloudflarestorage.com` hostname is an R2 S3 API endpoint and usually requires Authorization,
so it won’t work for browser `fetch()` unless you put a public/custom domain or a Worker in front of it.

For **Desktop** development, run: `pnpm tauri dev`

The legacy browser dashboard has been removed. Use this desktop app for the primary UI.

### Background Images (CDN)

The app can optionally load background image thumbnails from a CDN using a JSON manifest.

- `VITE_BACKGROUND_CDN_BASE`: Base URL that contains `manifest.v1.json` and the folders (e.g. `blur/`, `glass/`).
- `VITE_BACKGROUND_MANIFEST_URL`: Full override URL for the manifest (optional).
- `VITE_BACKGROUND_PROXY_TARGET`: Dev-only Vite proxy target (optional). If set, you can use a same-origin base like `VITE_BACKGROUND_CDN_BASE=/__bg/logo`.

Note: the `*.r2.cloudflarestorage.com` endpoint is the R2 S3 API endpoint and typically requires auth; for browser access you usually need a public/custom domain (or `r2.dev`) with CORS enabled.

### Internationalization (i18n)

[Vue I18n](https://vue-i18n.intlify.dev/)

#### Adding a new language

Create a new JSON file in the [locales](./src/i18n/locales/) directory with the appropriate translations and the locale as the filename.

Then update the `supportedLanguages` function in [lib/config.ts](./src/lib/config.ts) to include the new language.

### Helpful Tips

Tauri Store Plugin stores `settings.json` at:

**macOS**: `~/Library/Application Support/com.gloamy.desktop`

#### App Icon

Tauri CLI provides an [icon command](https://v2.tauri.app/reference/cli/#icon) `pnpm tauri icon` which takes an image path and generates icon files for your application.

## Contributing

Contributions are welcome! Feel free to:

- Open issues for bugs or feature requests
- Submit pull requests for improvements
- Provide feedback on existing features
- Suggest documentation improvements
- Help with translations

## Deployment & Release

[Tauri v2 Deployment Guide](https://v2.tauri.app/distribute/)

To build a binary, run:

```bash
pnpm tauri build
```

This repo is currently setup to create a release on Github when you merge to the `release` branch. See [https://github.com/tauri-apps/tauri-action/tree/dev](https://github.com/tauri-apps/tauri-action/tree/dev).

### Caveats

If you are using the [signing identity](./src-tauri/tauri.macos.conf.json#L20) `-` for **macOS**, when you first download and run the application, you will have to go to `System Settings` > `Privacy & Security` and allow your app to run.
See [tauri-apps/tauri-action/issues/824](https://github.com/tauri-apps/tauri-action/issues/824) & [support.apple.com/open-a-mac-app-from-an-unidentified-developer](https://support.apple.com/guide/mac-help/open-a-mac-app-from-an-unidentified-developer-mh40616/mac).
