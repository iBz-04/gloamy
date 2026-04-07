# Desktop UI Design System

This document outlines the core principles, styles, and components that define the visual identity of the desktop application.

## Philosophy: Visual Language

- **Clean & Approachable**: We use generous whitespace, but the UI should not feel sterile or empty. The interface should feel breathable and balanced—not too rounded to seem childish, and not too square to feel harsh.
- **Guided Typography**: Headers are clear, readable, and appropriately sized. Section labels are subtle but visible. We avoid aggressive uppercase tracking in favor of natural, readable text.
- **Subtle Depth**: Soft box shadows (`shadow-sm`, `shadow-md`) provide a gentle lift to elements. Interactive controls have a slight tint (`bg-muted/40`) to feel touchable without being heavy.
- **Interactive Feedback**: Hover states are smooth (`transition-all`). Buttons have a slight background shift, and selected items scale slightly with a shadow to provide clear, responsive feedback.
- **Logical Grouping**: Sections are clearly separated but flow naturally. Controls are grouped logically, and input fields maintain consistent heights and padding.

## Typography

- **Primary Font**: The UI uses the **Inter** typeface for all text elements, loaded from Google Fonts.
- **Font Weights**: Regular (400), Medium (500), and Semibold (600) weights are used to create a clear visual hierarchy.
- **Implementation**: The font is applied globally in `src/assets/css/tailwind.css`.

## Color Palette

The application uses a dual-theme system (light and dark) defined with CSS custom properties in `src/assets/css/tailwind.css`.

### Light Theme (Premium Minimalism - Cream & Pure White)

| Variable | Hex | Description |
| --- | --- | --- |
| `--background` | `#F3F1EE` | Main background color |
| `--foreground` | `#171717` | Main text color |
| `--card` | `#FFFFFF` | Card and popover background |
| `--primary` | `#171717` | Primary interactive elements |
| `--primary-foreground` | `#FFFFFF` | Text on primary elements |
| `--secondary` | `#F3F1EE` | Secondary interactive elements |
| `--muted` | `#E8E6E3` | Muted backgrounds |
| `--muted-foreground` | `#737373` | Muted text color |
| `--accent` | `#F3F1EE` | Accent color for highlights |
| `--destructive` | `#EF4444` | Destructive action elements |
| `--border` | `rgba(0,0,0,0.06)` | Default border color |

### Dark Theme (Premium Minimalism - Warm Charcoal & Soft Ivory)

| Variable | Hex | Description |
| --- | --- | --- |
| `--background` | `#1C1B1A` | Main background color |
| `--foreground` | `#E8E6E3` | Main text color |
| `--card` | `#252423` | Card and popover background |
| `--primary` | `#E8E6E3` | Primary interactive elements |
| `--primary-foreground` | `#1C1B1A` | Text on primary elements |
| `--secondary` | `#2D2C2A` | Secondary interactive elements |
| `--muted` | `#333231` | Muted backgrounds |
| `--muted-foreground` | `#9C9A97` | Muted text color |
| `--accent` | `#2D2C2A` | Accent color for highlights |
| `--destructive` | `#DC4545` | Destructive action elements |
| `--border` | `rgba(255,255,255,0.08)` | Default border color |

## Border Radius

The base border radius is defined as `2px`, with variations for different component sizes:

- **Base**: `var(--radius)` -> `2px`
- **Small**: `calc(var(--radius) - 4px)` -> `-2px` (effectively `0px` or sharp corners for some elements)
- **Medium**: `calc(var(--radius) - 2px)` -> `0px`
- **Large**: `var(--radius)` -> `2px`
- **Extra Large**: `calc(var(--radius) + 4px)` -> `6px`

## Iconography

The project uses two primary icon libraries to ensure a consistent and high-quality visual language:

- **Lucide Icons**: Provided via the `lucide-vue-next` package for general-purpose iconography.
- **Huge Icons**: Provided via `@hugeicons/vue` for more specific or illustrative icons.

## Component System

UI components are built using a combination of Vue, Tailwind CSS, and `class-variance-authority` (CVA). CVA allows us to define component variants (e.g., `primary`, `secondary`, `destructive` buttons) in a clean, reusable, and type-safe manner.

- **Location**: Core UI components are located in `src/components/ui/`.
- **Styling**: Component styles and variants are defined in the `index.ts` file within each component's directory (e.g., `src/components/ui/button/index.ts`).
- **Utilities**: `clsx` and `tailwind-merge` are used to intelligently merge and apply Tailwind classes.


