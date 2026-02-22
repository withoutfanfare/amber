# Frontend Implementation Plan

> Vue 3 + Tailwind CSS v4 + Spool Design System — for Database Snapshot Manager Phase 1

**Date:** 19 February 2026
**Status:** Draft
**Depends on:** [Phase 1 Full Implementation Design](./2026-02-19-phase1-full-implementation-design.md)

---

## 1. Vue 3 + Vite + Tailwind v4 Setup

### 1.1 Current State

The project is a Tauri 2 app scaffolded with plain TypeScript + Vite (no framework). Key files:

- `package.json` — has `@tauri-apps/api@^2`, `vite@^6`, `typescript@~5.6.2`
- `vite.config.ts` — bare Vite config with Tauri dev server settings
- `index.html` — loads `/src/main.ts`, mounts to `<div id="app">`
- `src/main.ts` — vanilla DOM manipulation (to be replaced entirely)
- `src/styles.css` — light-mode MVP styles (to be replaced entirely)
- `tsconfig.json` — targets ES2020, bundler module resolution

### 1.2 Install Dependencies

```bash
# Vue 3 core + SFC compiler
npm install vue
npm install -D @vitejs/plugin-vue vue-tsc

# Routing + state management
npm install vue-router pinia

# Tailwind CSS v4 (Vite plugin — no PostCSS config needed)
npm install -D tailwindcss @tailwindcss/vite

# Tauri plugin for keyring (already in Cargo.toml, need JS bindings)
npm install @tauri-apps/plugin-keyring
```

Final `dependencies`:
```json
{
  "vue": "^3.5",
  "vue-router": "^4.5",
  "pinia": "^3",
  "@tauri-apps/api": "^2",
  "@tauri-apps/plugin-opener": "^2",
  "@tauri-apps/plugin-keyring": "^2"
}
```

Final `devDependencies`:
```json
{
  "@tauri-apps/cli": "^2",
  "@vitejs/plugin-vue": "^5",
  "tailwindcss": "^4",
  "@tailwindcss/vite": "^4",
  "typescript": "~5.6.2",
  "vue-tsc": "^2",
  "vite": "^6"
}
```

### 1.3 Vite Config Update

Replace `vite.config.ts`:

```ts
// vite.config.ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
  ],

  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },

  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: 'ws', host, port: 1421 }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
```

### 1.4 TypeScript Config Update

Replace `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "skipLibCheck": true,
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

Add `tsconfig.node.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true,
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["vite.config.ts"]
}
```

Add `src/env.d.ts` for Vue SFC type support:

```ts
/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}
```

### 1.5 HTML Entry Point

Update `index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Database Snapshot Manager</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

### 1.6 Build Script Update

Update `package.json` scripts:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  }
}
```

---

## 2. Project Structure

```bash
src/
  assets/
    main.css                    # Tailwind import, @theme tokens, global classes
  components/
    ui/                         # Spool design system primitives
      Button.vue
      Badge.vue
      Card.vue
      FormInput.vue
      FormSelect.vue
      FormTextarea.vue
      ConfirmDialog.vue
      ToastNotification.vue
      EmptyState.vue
      Skeleton.vue
    layout/                     # App shell and navigation
      AppShell.vue              # Root layout: blobs, titlebar, sidebar, main
      AppSidebar.vue            # Nav links, profile selector, status footer
      PageHeader.vue            # Sticky header with title + action slots
    features/                   # Domain-specific composed components
      ProfileCard.vue           # Profile display with test/edit/delete
      ProfileForm.vue           # Shared form for create/edit profile
      SshTunnelConfig.vue       # SSH fields, toggled by checkbox
      ConnectionTestIndicator.vue
      SnapshotTable.vue         # Sortable snapshot list table
      SnapshotRestoreDialog.vue # Confirm restore with details preview
      StorageBreakdown.vue      # Per-project disk usage breakdown
      KpiCard.vue               # Dashboard metric card
  composables/                  # Shared composition functions
    useFocusTrap.ts             # Focus trap for modals
    useToast.ts                 # Toast notification system (provide/inject)
  stores/                       # Pinia stores
    profiles.ts                 # useProfileStore
    snapshots.ts                # useSnapshotStore
    settings.ts                 # useSettingsStore
  views/                        # Route-level page components
    DashboardView.vue
    ProfilesView.vue
    ProfileCreateView.vue
    ProfileEditView.vue
    SnapshotsView.vue
    SnapshotCreateView.vue
    StorageView.vue
    SettingsView.vue
  router/
    index.ts                    # Vue Router configuration
  types/
    index.ts                    # Shared TypeScript interfaces
  App.vue                       # Root component (AppShell wrapper)
  main.ts                       # Vue app bootstrap
  env.d.ts                      # Vite + Vue type declarations
```

---

## 3. Design Token Implementation

### 3.1 Tailwind v4 @theme Configuration

Tailwind CSS v4 uses a CSS-first configuration via `@theme {}` blocks. All tokens from the Styleguide are defined in `src/assets/main.css`. Tailwind v4 automatically generates utility classes from `--color-*`, `--font-*`, `--shadow-*`, etc.

**File:** `src/assets/main.css`

```css
@import "tailwindcss";

/* ============================================================
   Spool Design System — Design Tokens
   ============================================================ */

@theme {
  /* --- Surfaces --- */
  --color-surface-base: #171717;
  --color-surface-sidebar: #1A1A1A;
  --color-surface-raised: #262626;
  --color-surface-overlay: #303030;
  --color-surface-elevated: #3F3F46;

  /* --- Text --- */
  --color-text-primary: #EDEDED;
  --color-text-secondary: #999999;
  --color-text-tertiary: #737373;
  --color-text-muted: #525252;

  /* --- Accent --- */
  --color-accent: #60A5FA;
  --color-accent-strong: #2563EB;
  --color-accent-subtle: rgba(96, 165, 250, 0.1);

  /* --- Status --- */
  --color-success: #22C55E;
  --color-warning: #EAB308;
  --color-danger: #EF4444;
  --color-info: #60A5FA;

  /* --- Borders --- */
  --color-border: #303030;
  --color-border-subtle: rgba(255, 255, 255, 0.04);
  --color-border-strong: rgba(255, 255, 255, 0.12);

  /* --- Typography --- */
  --font-sans: -apple-system, BlinkMacSystemFont, "Inter", system-ui, sans-serif;
  --font-mono: "SF Mono", "Fira Code", ui-monospace, monospace;

  --text-2xs: 10px;
  /* text-xs (12px), text-sm (13px), text-base (15px), text-lg (17px),
     text-xl (20px), text-2xl (24px) use Tailwind defaults or overrides below */

  /* --- Border Radius --- */
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
  --radius-xl: 12px;
  --radius-2xl: 16px;

  /* --- Shadows --- */
  --shadow-inset: inset 0 1px 0 color-mix(in srgb, white 4%, transparent);
  --shadow-inset-strong: inset 0 1px 0 color-mix(in srgb, white 8%, transparent);
  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.35);
  --shadow-lg: 0 14px 30px -10px rgba(0, 0, 0, 0.5);
  --shadow-xl: 0 20px 40px -12px rgba(0, 0, 0, 0.6);
  --shadow-focus: 0 0 0 3px color-mix(in srgb, var(--color-accent) 22%, transparent);

  /* --- Animations --- */
  --animate-fade-in: fade-in 150ms ease-out;
  --animate-content-fade-in: content-fade-in 200ms ease-out;
  --animate-scale-in: scale-in 200ms ease;

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes content-fade-in {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @keyframes scale-in {
    from { opacity: 0; transform: scale(0.95); }
    to { opacity: 1; transform: scale(1); }
  }
}
```

### 3.2 Token-to-Utility Mapping

Tailwind v4 automatically generates utilities from `@theme` variables:

| Token | Generated utilities |
|---|---|
| `--color-surface-base` | `bg-surface-base`, `text-surface-base`, `border-surface-base` |
| `--color-text-primary` | `text-text-primary`, `bg-text-primary` |
| `--color-accent` | `text-accent`, `bg-accent`, `border-accent` |
| `--color-success` | `text-success`, `bg-success`, `border-success` |
| `--color-border` | `border-border` |
| `--font-sans` | `font-sans` |
| `--font-mono` | `font-mono` |
| `--radius-xl` | `rounded-xl` |
| `--shadow-sm` | `shadow-sm` |
| `--animate-fade-in` | `animate-fade-in` |

### 3.3 Type Scale Overrides

The Styleguide uses non-standard sizes for `text-sm` (13px) and `text-base` (15px). In Tailwind v4, override these in the `@theme` block:

```css
@theme {
  --text-sm: 13px;
  --text-base: 15px;
  --text-lg: 17px;
}
```

---

## 4. Global CSS Classes

All global classes go in `src/assets/main.css` below the `@theme` block.

### 4.1 Base Reset

```css
html {
  font-size: 15px;
  line-height: 1.6;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  background: var(--color-surface-base);
  color: var(--color-text-primary);
  font-family: var(--font-sans);
}

* {
  box-sizing: border-box;
}
```

### 4.2 Card / Glass Panel

```css
.card {
  background: color-mix(in srgb, #0e0e0e 42%, transparent);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  border: 1px solid color-mix(in srgb, white 4%, transparent);
  box-shadow: var(--shadow-inset);
}

.card-inset {
  background: color-mix(in srgb, var(--color-surface-base) 60%, var(--color-surface-raised));
  border: 1px solid var(--color-border-subtle);
  padding: 1.25rem;
}
```

### 4.3 Sidebar

```css
.sidebar {
  background: color-mix(in srgb, #0e0e0e 42%, transparent);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  border-right: 1px solid color-mix(in srgb, white 6%, transparent);
  box-shadow: inset -1px 0 0 rgba(255, 255, 255, 0.02);
}
```

### 4.4 Page Header

```css
.page-header {
  position: sticky;
  top: 0;
  z-index: 10;
  background: color-mix(in srgb, var(--color-surface-base) 60%, transparent);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  border-bottom: 1px solid var(--color-border-subtle);
}
```

### 4.5 Backdrop and Modal

```css
.backdrop {
  background: rgba(0, 0, 0, 0.40);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
}

.modal {
  background: var(--color-surface-base);
  border-radius: var(--radius-xl);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-xl);
}
```

### 4.6 Segment Chip

```css
.segment-chip {
  display: inline-flex;
  align-items: center;
  padding: 0.25rem 0.625rem;
  border-radius: var(--radius-md);
  font-size: var(--text-xs);
  font-weight: 500;
  cursor: pointer;
  transition: all 150ms ease;
}
```

### 4.7 Control Field (Small)

```css
.control-field-sm {
  height: 2rem;
  border-radius: 0.5rem;
  border: 1px solid var(--color-border);
  background: var(--color-surface-base);
  font-size: var(--text-xs);
  box-shadow: var(--shadow-sm);
  transition: border-color 150ms ease, background-color 150ms ease, box-shadow 150ms ease;
}

.control-field-sm:hover {
  border-color: var(--color-border-strong);
}

.control-field-sm:focus {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-focus);
  outline: none;
}
```

### 4.8 Ambient Blobs

```css
.ambient-blobs {
  position: fixed;
  inset: 0;
  z-index: 1;
  pointer-events: none;
  overflow: hidden;
}

.ambient-blob {
  position: absolute;
  border-radius: 50%;
  filter: blur(100px);
  opacity: 0.07;
  will-change: transform;
}

.blob-1 {
  width: 800px;
  height: 800px;
  top: -25%;
  right: -10%;
  background: var(--color-accent);
  animation: blob-drift-1 25s ease-in-out infinite alternate;
}

.blob-2 {
  width: 700px;
  height: 700px;
  bottom: -20%;
  left: -10%;
  background: #8B5CF6;
  animation: blob-drift-2 30s ease-in-out infinite alternate;
}

.blob-3 {
  width: 600px;
  height: 600px;
  top: 50%;
  right: 35%;
  background: #06B6D4;
  animation: blob-drift-3 20s ease-in-out infinite alternate;
}

@keyframes blob-drift-1 {
  from { transform: translate(0, 0) scale(1); }
  to { transform: translate(-40px, 30px) scale(1.05); }
}

@keyframes blob-drift-2 {
  from { transform: translate(0, 0) scale(1); }
  to { transform: translate(30px, -20px) scale(1.03); }
}

@keyframes blob-drift-3 {
  from { transform: translate(0, 0) scale(1); }
  to { transform: translate(-20px, -30px) scale(1.07); }
}

@media (prefers-reduced-motion: reduce) {
  .ambient-blob { animation: none !important; }
}

.page-hidden .ambient-blob {
  animation-play-state: paused;
}
```

### 4.9 Stagger Fade-In

```css
.stagger-fade-in > * {
  opacity: 0;
  animation: fade-in 200ms ease-out forwards;
}
.stagger-fade-in > *:nth-child(1) { animation-delay: 0ms; }
.stagger-fade-in > *:nth-child(2) { animation-delay: 30ms; }
.stagger-fade-in > *:nth-child(3) { animation-delay: 60ms; }
.stagger-fade-in > *:nth-child(4) { animation-delay: 90ms; }
.stagger-fade-in > *:nth-child(5) { animation-delay: 120ms; }
.stagger-fade-in > *:nth-child(n+6) { animation-delay: 150ms; }
```

### 4.10 Titlebar Drag Region

```css
.titlebar-drag-region {
  -webkit-app-region: drag;
}

.titlebar-no-drag {
  -webkit-app-region: no-drag;
}
```

### 4.11 Scrollbars

```css
::-webkit-scrollbar { width: 10px; height: 10px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--color-accent) 45%, transparent);
  border: 2px solid transparent;
  background-clip: content-box;
  border-radius: 999px;
}
::-webkit-scrollbar-thumb:hover { background: color-mix(in srgb, var(--color-accent) 66%, transparent); }
::-webkit-scrollbar-thumb:active { background: color-mix(in srgb, var(--color-accent) 80%, transparent); }

* {
  scrollbar-width: thin;
  scrollbar-color: color-mix(in srgb, var(--color-accent) 45%, transparent) transparent;
}
```

---

## 5. Shared Component Specs

### 5.1 Button.vue

**File:** `src/components/ui/Button.vue`

**Props:**

```ts
interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'brand' | 'danger' | 'danger-outline' | 'ghost' | 'outline' | 'link'
  size?: 'sm' | 'md' | 'lg' | 'icon'
  to?: string          // Vue Router path — renders <RouterLink> instead of <button>
  type?: 'button' | 'submit' | 'reset'
  disabled?: boolean
  loading?: boolean
  block?: boolean      // Full width
}
```

**Defaults:** `variant="primary"`, `size="md"`, `type="button"`

**Implementation notes:**
- All solid variants (`primary`, `secondary`, `danger`, `danger-outline`, `outline`) share the same scoped background/border CSS: `background: var(--color-surface-base); border: 1px solid var(--color-border); box-shadow: var(--shadow-sm);`
- Hover for all solid variants: `border-color: var(--color-border-strong)`
- Differentiation is via text colour only (see Styleguide section 8)
- `ghost`: transparent background, `hover:bg-surface-overlay`
- `link`: no border/bg, underline on hover, `p-0 h-auto`
- Size classes: `sm` = `min-h-[28px] px-2.5 py-1.5 text-xs`, `md` = `min-h-[32px] px-4 py-1.5 text-sm`, `lg` = `min-h-[40px] px-5 py-2.5 text-base`, `icon` = `min-h-[32px] min-w-[32px] p-1.5`
- All sizes use `rounded-lg`
- States: `active:scale-[0.98]`, `disabled:opacity-50 pointer-events-none`, `loading:opacity-70 pointer-events-none cursor-wait`
- Focus: `outline: 2px solid color-mix(in srgb, var(--color-accent) 55%, transparent); outline-offset: 2px;`
- When `loading`, show an SVG spinner replacing the slot content
- When `to` is provided, render `<RouterLink>` instead of `<button>`
- Emits: `click` (when not disabled/loading)

### 5.2 Badge.vue

**File:** `src/components/ui/Badge.vue`

**Props:**

```ts
interface BadgeProps {
  label: string
  variant?: 'success' | 'danger' | 'warning' | 'info' | 'accent' | 'muted' | 'default'
}
```

**Default:** `variant="default"`

**Implementation notes:**
- Base classes: `inline-flex items-center rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide border`
- Each variant uses `color-mix()` in scoped CSS for background (15%), border (20%), and text colour
- Do not use Tailwind utilities for variant colours — use scoped CSS per Styleguide section 9

### 5.3 Card.vue

**File:** `src/components/ui/Card.vue`

**Props:**

```ts
interface CardProps {
  variant?: 'default' | 'interactive' | 'flush' | 'inset'
}
```

**Default:** `variant="default"`

**Implementation notes:**
- `default`: `.card rounded-xl` with `p-5`
- `interactive`: `.card rounded-xl` with hover border/bg transition
- `flush`: `.card rounded-xl overflow-hidden` with no padding
- `inset`: `.card-inset rounded-xl` (recessed appearance for nesting)
- Uses default slot for content

### 5.4 FormInput.vue

**File:** `src/components/ui/FormInput.vue`

**Props:**

```ts
interface FormInputProps {
  modelValue: string | number
  label?: string
  placeholder?: string
  type?: 'text' | 'number' | 'password' | 'email' | 'url'
  size?: 'sm' | 'md'
  error?: string
  disabled?: boolean
}
```

**Default:** `type="text"`, `size="md"`

**Emits:** `update:modelValue`

**Implementation notes:**
- Wrapper: `<div class="form-field">` (flex column, gap-1.5)
- Label: `<label class="form-label">` (text-sm, font-medium, text-text-secondary)
- Input: scoped `.form-input` CSS per Styleguide section 10 (border-radius 10px, semi-transparent bg, accent hover/focus ring)
- Size `sm`: `min-h-[2rem]`, size `md`: `min-h-[2.5rem]`
- Error state: `.form-input-error` (danger border + focus ring)
- Error message rendered below input as `<p class="text-xs text-danger mt-1">`

### 5.5 FormSelect.vue

**File:** `src/components/ui/FormSelect.vue`

**Props:**

```ts
interface FormSelectProps {
  modelValue: string | number
  label?: string
  options: Array<{ value: string | number; label: string }>
  size?: 'sm' | 'md'
  error?: string
  disabled?: boolean
}
```

**Emits:** `update:modelValue`

**Implementation notes:**
- Same layout and label structure as FormInput
- Scoped `.form-select` with `appearance: none`, custom SVG chevron background-image, `padding-right: 2.25rem`
- `cursor: pointer`

### 5.6 FormTextarea.vue

**File:** `src/components/ui/FormTextarea.vue`

**Props:**

```ts
interface FormTextareaProps {
  modelValue: string
  label?: string
  placeholder?: string
  rows?: number
  error?: string
  disabled?: boolean
}
```

**Default:** `rows=4`

**Emits:** `update:modelValue`

**Implementation notes:**
- Same as FormInput plus `min-height: 5rem`, `resize: vertical`, `line-height: 1.6`

### 5.7 ConfirmDialog.vue

**File:** `src/components/ui/ConfirmDialog.vue`

**Props:**

```ts
interface ConfirmDialogProps {
  open: boolean
  title: string
  message: string
  confirmLabel?: string
  danger?: boolean
  typedConfirmation?: string   // If set, user must type this exact text to enable confirm
}
```

**Default:** `confirmLabel="Confirm"`, `danger=false`

**Emits:** `confirm`, `cancel`

**Implementation notes:**
- Uses `<Teleport to="body">` wrapping a `.backdrop` overlay and `.modal` panel
- Entry animation: `.animate-scale-in` (scale 0.95 + opacity)
- Focus trapped via `useFocusTrap()` composable
- Escape key fires `cancel` emit
- `role="alertdialog"`, `aria-modal="true"`, `aria-label` bound to title
- When `typedConfirmation` set, render a `.control-field-sm` input; confirm button disabled until exact match
- Confirm button uses `variant="danger"` when `danger=true`

### 5.8 ToastNotification.vue

**File:** `src/components/ui/ToastNotification.vue`

**Props:**

```ts
interface ToastProps {
  message: string
  type?: 'success' | 'error' | 'info'
  duration?: number
}
```

**Default:** `type="info"`, `duration=3000`

**Emits:** `dismiss`

**Implementation notes:**
- Scoped `.toast` CSS: `rgba(22, 22, 26, 0.85)` background, `blur(20px) saturate(1.3)`, rounded-xl
- Icon container: `h-7 w-7 rounded-lg` with coloured bg (success/10, danger/10, accent/10)
- Entry: `translate-y-3 opacity-0` -> normal, 300ms ease-out
- Exit: reverse, 200ms ease-in
- Auto-dismiss after `duration` ms
- ARIA: `role="status"` for success/info, `role="alert"` for error
- Positioned fixed bottom-right via a toast container managed by `useToast()` composable

### 5.9 EmptyState.vue

**File:** `src/components/ui/EmptyState.vue`

**Props:**

```ts
interface EmptyStateProps {
  message: string
  actionLabel?: string
  icon?: 'database' | 'snapshot' | 'folder' | 'settings'  // SVG icon key
}
```

**Emits:** `action`

**Implementation notes:**
- Centred flex column with `py-16`
- Icon box: `h-14 w-14 rounded-xl border border-border-subtle bg-surface-raised shadow-[var(--shadow-inset)]`
- Icon: `h-7 w-7 text-text-tertiary stroke-width-1`
- Message: `text-sm text-text-secondary mb-4`
- Action button: `<Button variant="primary" size="sm">` (only rendered when `actionLabel` provided)
- `animate-fade-in` on the container

### 5.10 Skeleton.vue

**File:** `src/components/ui/Skeleton.vue`

**Props:**

```ts
interface SkeletonProps {
  width?: string    // CSS value, e.g. '100%', '60%'
  height?: string   // CSS value, e.g. '1rem'
  rounded?: 'sm' | 'md' | 'lg' | 'full'
}
```

**Default:** `width="100%"`, `height="1rem"`, `rounded="md"`

**Implementation notes:**
- Scoped `.skeleton` CSS: semi-transparent raised bg, subtle border, inset shadow, `pulse` animation (2s)
- Rounded maps to radius tokens: `sm` -> 4px, `md` -> 6px, `lg` -> 8px, `full` -> 9999px

---

## 6. Vue Router Configuration

**File:** `src/router/index.ts`

```ts
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: () => import('@/views/DashboardView.vue'),
    },
    {
      path: '/profiles',
      name: 'profiles',
      component: () => import('@/views/ProfilesView.vue'),
    },
    {
      path: '/profiles/create',
      name: 'profile-create',
      component: () => import('@/views/ProfileCreateView.vue'),
    },
    {
      path: '/profiles/:id/edit',
      name: 'profile-edit',
      component: () => import('@/views/ProfileEditView.vue'),
      props: true,
    },
    {
      path: '/snapshots',
      name: 'snapshots',
      component: () => import('@/views/SnapshotsView.vue'),
    },
    {
      path: '/snapshots/create',
      name: 'snapshot-create',
      component: () => import('@/views/SnapshotCreateView.vue'),
    },
    {
      path: '/storage',
      name: 'storage',
      component: () => import('@/views/StorageView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
    },
  ],
})

export default router
```

All routes use lazy loading via dynamic `import()`. Vite handles code splitting automatically.

---

## 7. Pinia Store Design

### 7.1 Shared Types

**File:** `src/types/index.ts`

```ts
export type DbType = 'mysql' | 'postgresql' | 'sqlite'

export interface Profile {
  id: string
  project: string
  name: string
  db_type: DbType
  host: string | null
  port: number | null
  database_name: string
  username: string | null
  ssh_enabled: boolean
  ssh_host: string | null
  ssh_port: number
  ssh_user: string | null
  notes: string | null
  created_at: string
  updated_at: string
}

export interface ProfileCreatePayload {
  project: string
  name: string
  db_type: DbType
  host?: string
  port?: number
  database_name: string
  username?: string
  password?: string
  ssh_enabled?: boolean
  ssh_host?: string
  ssh_port?: number
  ssh_user?: string
  ssh_key_path?: string
  ssh_password?: string
  notes?: string
}

export interface ProfileUpdatePayload extends ProfileCreatePayload {
  id: string
}

export interface Snapshot {
  id: string
  profile_id: string
  name: string
  note: string | null
  file_path: string
  size_bytes: number
  db_version: string | null
  dump_tool_version: string | null
  created_at: string
  restored_at: string | null
}

export interface SnapshotCreatePayload {
  profile_id: string
  name: string
  note?: string
}

export interface StorageUsage {
  project: string
  total_bytes: number
  snapshot_count: number
}

export interface ConnectionTestResult {
  success: boolean
  message: string
  latency_ms?: number
}
```

### 7.2 useProfileStore

**File:** `src/stores/profiles.ts`

```ts
import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Profile, ProfileCreatePayload, ProfileUpdatePayload, ConnectionTestResult } from '@/types'

export const useProfileStore = defineStore('profiles', () => {
  // --- State ---
  const profiles = ref<Profile[]>([])
  const activeProfileId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const testResults = ref<Map<string, ConnectionTestResult>>(new Map())

  // --- Getters ---
  const activeProfile = computed(() =>
    profiles.value.find(p => p.id === activeProfileId.value) ?? null
  )

  const profilesByProject = computed(() => {
    const grouped = new Map<string, Profile[]>()
    for (const profile of profiles.value) {
      const list = grouped.get(profile.project) ?? []
      list.push(profile)
      grouped.set(profile.project, list)
    }
    return grouped
  })

  // --- Actions ---
  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      profiles.value = await invoke<Profile[]>('profile_list')
      // Auto-select first profile if none active
      if (!activeProfileId.value && profiles.value.length > 0) {
        activeProfileId.value = profiles.value[0].id
      }
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function create(payload: ProfileCreatePayload): Promise<Profile> {
    const profile = await invoke<Profile>('profile_create', { payload })
    profiles.value.push(profile)
    activeProfileId.value = profile.id
    return profile
  }

  async function update(payload: ProfileUpdatePayload): Promise<Profile> {
    const updated = await invoke<Profile>('profile_update', { payload })
    const index = profiles.value.findIndex(p => p.id === updated.id)
    if (index >= 0) profiles.value[index] = updated
    return updated
  }

  async function remove(id: string): Promise<void> {
    await invoke('profile_delete', { id })
    profiles.value = profiles.value.filter(p => p.id !== id)
    if (activeProfileId.value === id) {
      activeProfileId.value = profiles.value[0]?.id ?? null
    }
  }

  async function testConnection(id: string): Promise<ConnectionTestResult> {
    const result = await invoke<ConnectionTestResult>('profile_test_connection', { id })
    testResults.value.set(id, result)
    return result
  }

  function setActive(id: string) {
    activeProfileId.value = id
  }

  return {
    profiles, activeProfileId, loading, error, testResults,
    activeProfile, profilesByProject,
    fetchAll, create, update, remove, testConnection, setActive,
  }
})
```

### 7.3 useSnapshotStore

**File:** `src/stores/snapshots.ts`

```ts
import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Snapshot, SnapshotCreatePayload } from '@/types'
import { useProfileStore } from './profiles'

export const useSnapshotStore = defineStore('snapshots', () => {
  // --- State ---
  const snapshots = ref<Snapshot[]>([])
  const loading = ref(false)
  const creating = ref(false)
  const restoring = ref(false)
  const error = ref<string | null>(null)

  // --- Getters ---
  const sortedByDate = computed(() =>
    [...snapshots.value].sort(
      (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    )
  )

  const totalSizeBytes = computed(() =>
    snapshots.value.reduce((sum, s) => sum + s.size_bytes, 0)
  )

  const recentSnapshots = computed(() => sortedByDate.value.slice(0, 5))

  // --- Actions ---
  async function fetchForProfile(profileId: string) {
    loading.value = true
    error.value = null
    try {
      snapshots.value = await invoke<Snapshot[]>('snapshot_list', { profileId })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      snapshots.value = await invoke<Snapshot[]>('snapshot_list', { profileId: null })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function create(payload: SnapshotCreatePayload): Promise<Snapshot> {
    creating.value = true
    error.value = null
    try {
      const snapshot = await invoke<Snapshot>('snapshot_create', { payload })
      snapshots.value.unshift(snapshot)
      return snapshot
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      creating.value = false
    }
  }

  async function restore(id: string): Promise<void> {
    restoring.value = true
    error.value = null
    try {
      await invoke('snapshot_restore', { id })
      // Update restored_at timestamp locally
      const snapshot = snapshots.value.find(s => s.id === id)
      if (snapshot) {
        snapshot.restored_at = new Date().toISOString()
      }
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      restoring.value = false
    }
  }

  async function remove(id: string): Promise<void> {
    await invoke('snapshot_delete', { id })
    snapshots.value = snapshots.value.filter(s => s.id !== id)
  }

  return {
    snapshots, loading, creating, restoring, error,
    sortedByDate, totalSizeBytes, recentSnapshots,
    fetchForProfile, fetchAll, create, restore, remove,
  }
})
```

### 7.4 useSettingsStore

**File:** `src/stores/settings.ts`

```ts
import { ref } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useSettingsStore = defineStore('settings', () => {
  // --- State ---
  const settings = ref<Map<string, string>>(new Map())
  const loading = ref(false)

  // --- Convenience Getters ---
  function get(key: string, defaultValue: string = ''): string {
    return settings.value.get(key) ?? defaultValue
  }

  // --- Actions ---
  async function fetchAll() {
    loading.value = true
    try {
      const entries = await invoke<Array<{ key: string; value: string }>>('settings_list')
      settings.value = new Map(entries.map(e => [e.key, e.value]))
    } finally {
      loading.value = false
    }
  }

  async function set(key: string, value: string) {
    await invoke('settings_set', { key, value })
    settings.value.set(key, value)
  }

  return {
    settings, loading,
    get, fetchAll, set,
  }
})
```

---

## 8. Tauri IPC from Vue

### 8.1 Pattern

All Tauri command calls are encapsulated within Pinia store actions. Vue components never call `invoke()` directly.

```text
Component  -->  Store Action  -->  invoke('command_name', { args })  -->  Rust #[tauri::command]
                                            |
                                   Returns Promise<T>
                                   Errors are strings
```

### 8.2 Import

```ts
import { invoke } from '@tauri-apps/api/core'
```

The project has `"app": { "withGlobalTauri": true }` in `tauri.conf.json`, so `window.__TAURI__` is also available. However, prefer the npm import for type safety.

### 8.3 Argument Convention

Tauri 2 expects arguments as a JSON object with **camelCase** keys on the JS side, which are automatically converted to **snake_case** on the Rust side:

```ts
// JS
await invoke('profile_create', { payload: { project: 'my-app', name: 'local', dbType: 'mysql' } })

// Rust
#[tauri::command]
fn profile_create(payload: ProfileCreatePayload) -> Result<Profile, String> { ... }
```

### 8.4 Error Handling Pattern

All Rust commands return `Result<T, String>`. In Pinia actions:

```ts
async function create(payload: ProfileCreatePayload): Promise<Profile> {
  try {
    const profile = await invoke<Profile>('profile_create', { payload })
    profiles.value.push(profile)
    return profile
  } catch (e) {
    // e is the error string from Rust
    error.value = String(e)
    throw e  // Re-throw so the component can show a toast
  }
}
```

In view components:

```ts
async function handleCreate() {
  try {
    await profileStore.create(formData.value)
    toast.success('Profile created successfully.')
    router.push('/profiles')
  } catch (e) {
    toast.error(`Failed to create profile: ${e}`)
  }
}
```

### 8.5 Event Listening (Progress)

For snapshot create/restore progress, Tauri can emit events from Rust. Listen in the component:

```ts
import { listen } from '@tauri-apps/api/event'

const unlisten = await listen<{ progress: number; message: string }>('snapshot-progress', (event) => {
  progress.value = event.payload.progress
  statusMessage.value = event.payload.message
})

onUnmounted(() => {
  unlisten()
})
```

---

## 9. Vue Application Bootstrap

**File:** `src/main.ts`

```ts
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from '@/router'
import App from '@/App.vue'
import '@/assets/main.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
```

**File:** `src/App.vue`

```vue
<script setup lang="ts">
import AppShell from '@/components/layout/AppShell.vue'

// Pause blob animations when tab is hidden
document.addEventListener('visibilitychange', () => {
  document.documentElement.classList.toggle('page-hidden', document.hidden)
})
</script>

<template>
  <AppShell />
</template>
```

---

## 10. View Component Specs

### 10.1 DashboardView

**File:** `src/views/DashboardView.vue`

**Layout:**
- `PageHeader` with title "Dashboard" (no actions)
- Grid of 4 `KpiCard` components: total profiles, total snapshots, total storage, last restore
- "Recent Snapshots" card with a mini table (last 5 snapshots across all profiles)
- "Quick Actions" card with buttons: Create Snapshot, Create Profile, View Storage

**Data flow:**
- On mount: `profileStore.fetchAll()`, `snapshotStore.fetchAll()`
- KPI data derived from store getters
- Recent snapshots from `snapshotStore.recentSnapshots`

### 10.2 ProfilesView

**File:** `src/views/ProfilesView.vue`

**Layout:**
- `PageHeader` with title "Profiles", action slot: `<Button to="/profiles/create">New Profile</Button>`
- List of `ProfileCard` components grouped by project
- `EmptyState` when no profiles exist

**Data flow:**
- On mount: `profileStore.fetchAll()`
- Profiles listed from `profileStore.profilesByProject`

### 10.3 ProfileCreateView

**File:** `src/views/ProfileCreateView.vue`

**Layout:**
- `PageHeader` with title "New Profile", back button to `/profiles`
- `ProfileForm` component in create mode
- `SshTunnelConfig` section (toggled by checkbox)
- Sticky footer bar with Cancel + Create buttons

**Data flow:**
- Form state local to view (`ref` for each field)
- On submit: `profileStore.create(formData)` -> toast -> redirect to `/profiles`
- Connection test via `profileStore.testConnection()` with `ConnectionTestIndicator`

### 10.4 ProfileEditView

**File:** `src/views/ProfileEditView.vue`

**Layout:**
- Same as ProfileCreateView but pre-populated from `profileStore.profiles.find(p => p.id === route.params.id)`
- `PageHeader` title: "Edit Profile"
- Delete button in header actions (opens `ConfirmDialog`)

**Data flow:**
- On mount: fetch profile by ID (or use store cache)
- On submit: `profileStore.update(formData)` -> toast -> redirect
- On delete: `ConfirmDialog` -> `profileStore.remove(id)` -> redirect to `/profiles`

### 10.5 SnapshotsView

**File:** `src/views/SnapshotsView.vue`

**Layout:**
- `PageHeader` with title "Snapshots", action: `<Button to="/snapshots/create">New Snapshot</Button>`
- Profile selector dropdown (filter by profile)
- `SnapshotTable` — sortable columns: name, note, created_at, size_bytes, actions
- Each row has Restore and Delete actions
- `EmptyState` when no snapshots for selected profile

**Data flow:**
- On mount / profile change: `snapshotStore.fetchForProfile(profileStore.activeProfileId)`
- Restore: opens `SnapshotRestoreDialog` -> `snapshotStore.restore(id)` -> toast
- Delete: `ConfirmDialog` -> `snapshotStore.remove(id)` -> toast

### 10.6 SnapshotCreateView

**File:** `src/views/SnapshotCreateView.vue`

**Layout:**
- `PageHeader` with title "Create Snapshot", back button to `/snapshots`
- Profile selector (defaults to active profile)
- `FormInput` for snapshot name
- `FormTextarea` for note
- Progress bar (shown during creation)
- Sticky footer with Cancel + Create buttons

**Data flow:**
- On submit: `snapshotStore.create({ profile_id, name, note })` -> listen for progress events -> toast -> redirect to `/snapshots`

### 10.7 StorageView

**File:** `src/views/StorageView.vue`

**Layout:**
- `PageHeader` with title "Storage"
- Total storage KPI at top
- `StorageBreakdown` component showing per-project usage (list with progress bars)
- Pruning controls: delete snapshots older than N days
- Per-project deletion actions

**Data flow:**
- On mount: `invoke('storage_usage')` to get per-project breakdown
- Delete actions call `snapshotStore.remove()` or bulk prune via a Rust command

### 10.8 SettingsView

**File:** `src/views/SettingsView.vue`

**Layout:**
- `PageHeader` with title "Settings"
- Card with form fields for app preferences:
  - Default snapshot directory (display only, set by system)
  - Auto-prune toggle + days threshold
  - Compression preference (gzip level)
- Each setting saved individually via `settingsStore.set(key, value)`

**Data flow:**
- On mount: `settingsStore.fetchAll()`
- On change: debounced `settingsStore.set()` calls

---

## 11. Composables

### 11.1 useFocusTrap

**File:** `src/composables/useFocusTrap.ts`

```ts
import { ref, onMounted, onUnmounted, type Ref } from 'vue'

export function useFocusTrap(containerRef: Ref<HTMLElement | null>) {
  const previouslyFocused = ref<HTMLElement | null>(null)

  function getFocusableElements(): HTMLElement[] {
    if (!containerRef.value) return []
    return Array.from(
      containerRef.value.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
      )
    )
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key !== 'Tab') return

    const focusable = getFocusableElements()
    if (focusable.length === 0) return

    const first = focusable[0]
    const last = focusable[focusable.length - 1]

    if (e.shiftKey) {
      if (document.activeElement === first) {
        e.preventDefault()
        last.focus()
      }
    } else {
      if (document.activeElement === last) {
        e.preventDefault()
        first.focus()
      }
    }
  }

  function activate() {
    previouslyFocused.value = document.activeElement as HTMLElement
    document.addEventListener('keydown', handleKeyDown)
    // Focus first focusable element
    const focusable = getFocusableElements()
    if (focusable.length > 0) focusable[0].focus()
  }

  function deactivate() {
    document.removeEventListener('keydown', handleKeyDown)
    previouslyFocused.value?.focus()
  }

  return { activate, deactivate }
}
```

### 11.2 useToast

**File:** `src/composables/useToast.ts`

```ts
import { ref, type InjectionKey } from 'vue'

export interface Toast {
  id: string
  message: string
  type: 'success' | 'error' | 'info'
  duration: number
}

export interface ToastContext {
  toasts: Ref<Toast[]>
  success: (message: string) => void
  error: (message: string) => void
  info: (message: string) => void
  dismiss: (id: string) => void
}

export const ToastKey: InjectionKey<ToastContext> = Symbol('toast')

export function useToastProvider(): ToastContext {
  const toasts = ref<Toast[]>([])

  function add(message: string, type: Toast['type'], duration = 3000) {
    const id = crypto.randomUUID()
    toasts.value.push({ id, message, type, duration })
    setTimeout(() => dismiss(id), duration)
  }

  function dismiss(id: string) {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }

  return {
    toasts,
    success: (msg) => add(msg, 'success'),
    error: (msg) => add(msg, 'error', 5000),
    info: (msg) => add(msg, 'info'),
    dismiss,
  }
}

export function useToast(): ToastContext {
  const ctx = inject(ToastKey)
  if (!ctx) throw new Error('useToast() called without ToastProvider')
  return ctx
}
```

The `ToastProvider` is set up in `AppShell.vue` via `provide(ToastKey, useToastProvider())`. Toast rendering happens in a fixed container at the bottom-right of the app.

---

## 12. Layout Component Details

### 12.1 AppShell.vue

**File:** `src/components/layout/AppShell.vue`

```vue
<script setup lang="ts">
import { provide } from 'vue'
import { RouterView } from 'vue-router'
import AppSidebar from './AppSidebar.vue'
import ToastNotification from '@/components/ui/ToastNotification.vue'
import { ToastKey, useToastProvider } from '@/composables/useToast'

const toast = useToastProvider()
provide(ToastKey, toast)
</script>

<template>
  <div class="relative h-screen overflow-hidden text-text-primary antialiased selection:bg-accent selection:text-white">
    <!-- Ambient blobs -->
    <div class="ambient-blobs" aria-hidden="true">
      <div class="ambient-blob blob-1"></div>
      <div class="ambient-blob blob-2"></div>
      <div class="ambient-blob blob-3"></div>
    </div>

    <!-- Titlebar drag region -->
    <div class="titlebar-drag-region fixed top-0 left-0 right-0 z-[9999] h-7 border-b border-border-subtle"></div>

    <!-- Skip link -->
    <a href="#main-content" class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:bg-surface-raised focus:px-4 focus:py-2 focus:rounded-lg focus:text-text-primary">
      Skip to content
    </a>

    <!-- Layout -->
    <div class="flex h-full pt-7" style="position: relative; z-index: 2;">
      <AppSidebar />
      <main id="main-content" class="titlebar-no-drag flex flex-1 flex-col overflow-y-auto px-6 pb-6" style="background: transparent;">
        <RouterView />
      </main>
    </div>

    <!-- Toasts -->
    <div class="fixed bottom-4 right-4 z-[9999] flex flex-col gap-2">
      <TransitionGroup
        enter-from-class="translate-y-3 opacity-0"
        enter-active-class="transition-all duration-300 ease-out"
        leave-to-class="translate-y-3 opacity-0"
        leave-active-class="transition-all duration-200 ease-in"
      >
        <ToastNotification
          v-for="t in toast.toasts.value"
          :key="t.id"
          :message="t.message"
          :type="t.type"
          :duration="t.duration"
          @dismiss="toast.dismiss(t.id)"
        />
      </TransitionGroup>
    </div>
  </div>
</template>
```

### 12.2 AppSidebar.vue

**File:** `src/components/layout/AppSidebar.vue`

Structure:
- Fixed-width sidebar (`w-56`) with `.sidebar` global class
- Branding row at top: app icon + "DSM" text (`h-[3.75rem]`)
- Navigation links using `<RouterLink>` with active state: `text-accent font-semibold` (no bg, no left border)
- Nav items: Dashboard, Profiles, Snapshots, Storage, Settings
- Profile selector dropdown near the bottom (uses `FormSelect` or custom dropdown)
- Service status footer with dot indicator

Navigation items:

```ts
const navItems = [
  { name: 'Dashboard', path: '/', icon: 'home' },
  { name: 'Profiles', path: '/profiles', icon: 'database' },
  { name: 'Snapshots', path: '/snapshots', icon: 'camera' },
  { name: 'Storage', path: '/storage', icon: 'hard-drive' },
  { name: 'Settings', path: '/settings', icon: 'settings' },
]
```

Active detection: `route.path === item.path` or `route.path.startsWith(item.path)` for nested routes.

### 12.3 PageHeader.vue

**File:** `src/components/layout/PageHeader.vue`

**Props:**

```ts
interface PageHeaderProps {
  // No props — uses slots only
}
```

**Slots:** `prepend` (left side: title, back button), `actions` (right side: buttons)

```vue
<template>
  <div class="page-header sticky top-0 z-20 -mx-6 mb-4 flex h-[3.75rem] items-center justify-between px-6 pt-1">
    <div class="flex items-center gap-4">
      <slot name="prepend" />
    </div>
    <div class="flex items-center gap-3">
      <slot name="actions" />
    </div>
  </div>
</template>
```

Usage:

```vue
<PageHeader>
  <template #prepend>
    <h1 class="text-lg font-semibold">Profiles</h1>
  </template>
  <template #actions>
    <Button variant="primary" size="sm" to="/profiles/create">New Profile</Button>
  </template>
</PageHeader>
```

---

## 13. Feature Component Details

### 13.1 ProfileCard.vue

**Props:** `profile: Profile`
**Emits:** `edit`, `delete`, `test`

Shows: project name, profile name, db_type badge, host:port/database, SSH indicator, connection test button + result. Edit and delete icon buttons in header.

### 13.2 ProfileForm.vue

**Props:** `initialData?: Partial<Profile>`, `submitting: boolean`
**Emits:** `submit(payload)`, `cancel`

Shared form used by both ProfileCreateView and ProfileEditView. Fields: project, name, db_type (select), host, port, database_name, username, password (never pre-filled), notes. Includes `SshTunnelConfig` sub-component toggled by checkbox.

### 13.3 SshTunnelConfig.vue

**Props:** `modelValue: { enabled: boolean; host: string; port: number; user: string; keyPath: string; password: string }`
**Emits:** `update:modelValue`

Fields: SSH host, SSH port (default 22), SSH user, SSH key path (file picker), SSH password. All fields hidden until the "Enable SSH tunnel" checkbox is ticked.

### 13.4 ConnectionTestIndicator.vue

**Props:** `result: ConnectionTestResult | null`, `testing: boolean`
**Emits:** none (display only)

Shows: spinner when `testing`, green check + latency when `result.success`, red X + error message when `!result.success`.

### 13.5 SnapshotTable.vue

**Props:** `snapshots: Snapshot[]`, `profiles: Profile[]`
**Emits:** `restore(id)`, `delete(id)`

Sortable columns: Name, Note, Created, Size, Actions. Row hover uses scoped CSS. Size displayed in human-readable format (KB/MB/GB). Actions column has Restore and Delete buttons.

### 13.6 SnapshotRestoreDialog.vue

**Props:** `open: boolean`, `snapshot: Snapshot | null`
**Emits:** `confirm`, `cancel`

Wraps `ConfirmDialog` with snapshot details preview: name, profile, created date, size. Warning message about destructive operation.

### 13.7 StorageBreakdown.vue

**Props:** `usage: StorageUsage[]`
**Emits:** `delete-project(project)`

Lists each project with snapshot count, total size, and a horizontal progress bar showing proportion of total storage. Delete button per project.

### 13.8 KpiCard.vue

**Props:** `label: string`, `value: string | number`, `icon?: string`

Simple display card: icon + label + large value. Uses `.card rounded-xl p-4`.

---

## 14. Implementation Order

Following the vertical slices from the design doc:

### Slice 1: Foundation
1. Install all dependencies
2. Configure Vite + Vue + Tailwind
3. Create `src/assets/main.css` with all tokens and global classes
4. Create `App.vue`, `AppShell.vue`, `AppSidebar.vue`, `PageHeader.vue`
5. Create all UI primitives: Button, Badge, Card, FormInput, FormSelect, FormTextarea, ConfirmDialog, ToastNotification, EmptyState, Skeleton
6. Set up Vue Router with all routes (views can be placeholder stubs)
7. Set up Pinia with empty stores
8. Verify: ambient blobs render, sidebar navigates, page headers display

### Slice 2: Connection Profiles
1. Implement `useProfileStore` with Tauri invoke calls
2. Build `ProfileForm.vue`, `SshTunnelConfig.vue`, `ConnectionTestIndicator.vue`
3. Build `ProfileCard.vue`
4. Build `ProfilesView.vue`, `ProfileCreateView.vue`, `ProfileEditView.vue`
5. Wire up sidebar profile selector

### Slice 3: Snapshot Creation
1. Implement `useSnapshotStore` with create action + progress listening
2. Build `SnapshotCreateView.vue`
3. Progress bar integration with Tauri event stream

### Slice 4: Snapshot Browser + Restore
1. Build `SnapshotTable.vue`, `SnapshotRestoreDialog.vue`
2. Build `SnapshotsView.vue`
3. Wire up sort, filter by profile, restore, delete flows

### Slice 5: Storage Management
1. Build `StorageBreakdown.vue`
2. Build `StorageView.vue`
3. Pruning controls

### Slice 6: Dashboard
1. Build `KpiCard.vue`
2. Build `DashboardView.vue` with KPIs, recent snapshots, quick actions
3. Implement `useSettingsStore`
4. Build `SettingsView.vue`

---

*Plan drafted 19 February 2026*
