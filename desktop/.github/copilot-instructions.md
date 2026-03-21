# Icon Maker - Copilot Instructions

## Project Overview
A Tauri v2 + Vue 3 desktop app for creating icons/logos with live SVG preview. Think "Figma-style icon generator" - not a vector editor, but a preset-driven icon composer with background gradients, icon overlays, and text.

## Architecture

### Frontend-Backend Split
- **Frontend** (`src/`): Vue 3 + Pinia + Shadcn-Vue components
- **Backend** (`src-tauri/`): Rust with Tauri plugins for SVG rasterization, file export, and persistence

### Key Data Flow
1. User edits → `useIconProjectStore` (Pinia) → reactive `IconSettings`
2. `IconSettings` → `IconCanvas.vue` renders live SVG preview
3. Export: `buildExportableSvg()` → Rust `rasterize_svg` command → PNG/WebP/ICO

### Core Files
- [src/stores/iconProject.ts](src/stores/iconProject.ts) - Central state: `IconSettings`, undo/redo, presets
- [src/components/icon/IconCanvas.vue](src/components/icon/IconCanvas.vue) - SVG rendering with gradients, icons, text
- [src/lib/export.ts](src/lib/export.ts) - Export logic invoking Rust commands
- [src-tauri/src/plugins/rasterizer.rs](src-tauri/src/plugins/rasterizer.rs) - SVG-to-image conversion

## Development Commands
```bash
pnpm tauri dev      # Full app (Rust + Vite HMR)
pnpm vite:dev       # Frontend only (web preview)
pnpm lint:fix       # ESLint with auto-fix
pnpm typecheck      # Vue + TypeScript validation
```

## Code Patterns

### UI Components
Uses **Shadcn-Vue** (New York style) with Reka-UI primitives. Add components via:
```bash
npx shadcn-vue@latest add [component]
```
Components live in `src/components/ui/` with barrel exports (`index.ts`).

### Styling
- Tailwind CSS v4 with CSS variables for theming
- Use `cn()` helper from `@/lib/utils` for conditional classes
- Follow [design-system.md](design-system.md): soft shadows, `rounded-md`, Figma-inspired warmth

### State Management
```typescript
// Always use storeToRefs for reactivity
const { settings, canUndo } = storeToRefs(useIconProjectStore())

// Update through store method (auto-persists, auto-history)
iconStore.updateSettings({ iconColor: '#FF0000' })
```

### Composables Pattern
Extract reusable logic to `src/composables/`:
- `useExport()` - Export with toast feedback
- `useCanvasTransform()` - Pan/zoom state
- `useTools()` - Drawing tool state

### Tauri IPC
Frontend calls Rust via `invoke()`:
```typescript
import { invoke } from '@tauri-apps/api/core'
const result = await invoke<RasterizeResponse>('rasterize_svg', { svg, width, height, format })
```
Rust commands registered in [src-tauri/src/lib.rs](src-tauri/src/lib.rs).

### Icon Sources
Multiple icon formats in `settings.icon`:
- Iconify: `ph:star`, `simple-icons:github`
- DiceBear avatars: `dicebear:bottts`
- Custom SVGs: `sudeft:animals/cat`
- Freepik: `freepik:id:url`

Check type with helpers: `isDicebearIconId()`, `isSudeftIcon()`

## Adding Features

### New Setting
1. Add to `IconSettings` interface in `iconProject.ts`
2. Add default in `createDefaultSettings()`
3. Wire UI control in `DesignPanel.vue` or `AdvancedPanel.vue`
4. Use in `IconCanvas.vue` for preview

### New Export Format
1. Add format to `ExportFormat` type in `export.ts`
2. Implement in `rasterizer.rs` with image crate
3. Register command in `lib.rs` invoke_handler

### New Language
1. Create JSON in `src/i18n/locales/[code].json`
2. Add to `supportedLanguages()` in `src/lib/config.ts`

## Persistence
- App settings: Tauri Store plugin → `settings.json`
- Project state: Tauri Store → `icon-projects.json`
- No manual save needed - auto-persists on change

## Testing Builds
```bash
pnpm tauri build              # Production build
pnpm tauri build --debug      # Debug build with devtools
```
Platform configs: `tauri.windows.conf.json`, `tauri.macos.conf.json`, `tauri.linux.conf.json`

#instructions
do not run commands until i tell you to do so 

use rust, tauri for computational heavy task always 

research before coding unconventional things 

be 100% sure before importing a module or icon 

before writing a feature, ask about how you have thought of implementing it and how it might affect other components do not do something that will disrupt other features 

always use border radius 2px

prioritize white space and click to view, pop ups ..etc to avoid cluttering components

no unecessary comments, short and important comments only when needed , a comment should only state what a non obvious function is doing

do not use any black borders, use cololr pallete in the ui we already use , use reuasble components 

no file should be more than 500 lins of code, even 500 is too much


