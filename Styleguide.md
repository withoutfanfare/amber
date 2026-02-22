# Spool Design System

> Practical reference for building Spool UI. Documents what is actually in the codebase — verified against component source files. Dark mode only. Warm greys. Glassmorphic panels.

---

## 1. Colour Palette

Design tokens live in `src/assets/main.css` inside `@theme {}`. Reference them in Tailwind as `text-text-primary`, `bg-surface-raised`, `border-border-subtle`, etc.

### Surfaces

| Token | Value | Tailwind class | Usage |
|---|---|---|---|
| `--color-surface-base` | `#171717` | `bg-surface-base` | App background, deepest layer |
| `--color-surface-sidebar` | `#1A1A1A` | `bg-surface-sidebar` | Sidebar (used at 42% with blur) |
| `--color-surface-raised` | `#262626` | `bg-surface-raised` | Cards, panels, dropdowns |
| `--color-surface-overlay` | `#303030` | `bg-surface-overlay` | Hover states, elevated surfaces |
| `--color-surface-elevated` | `#3F3F46` | `bg-surface-elevated` | Modals, highest elevation |

### Text

| Token | Value | Tailwind class | Usage |
|---|---|---|---|
| `--color-text-primary` | `#EDEDED` | `text-text-primary` | Headings, body, active labels |
| `--color-text-secondary` | `#999999` | `text-text-secondary` | Descriptions, metadata |
| `--color-text-tertiary` | `#737373` | `text-text-tertiary` | Placeholders, timestamps, hints |
| `--color-text-muted` | `#525252` | `text-text-muted` | Disabled text, decorative elements |

### Accent

| Token | Value | Tailwind class | Usage |
|---|---|---|---|
| `--color-accent` | `#60A5FA` | `text-accent`, `bg-accent` | Primary actions, active states, links |
| `--color-accent-strong` | `#2563EB` | `text-accent-strong` | Emphasis, focus rings |
| `--color-accent-subtle` | `rgba(96,165,250,0.1)` | `bg-accent-subtle` | Tinted backgrounds |

### Status

| Token | Value | Tailwind class | Usage |
|---|---|---|---|
| `--color-success` | `#22C55E` | `text-success`, `bg-success` | Completed, healthy |
| `--color-warning` | `#EAB308` | `text-warning`, `bg-warning` | Queued, pending, cancelled |
| `--color-danger` | `#EF4444` | `text-danger`, `bg-danger` | Failed, errors, destructive |
| `--color-info` | `#60A5FA` | `text-info` | Informational (same value as accent) |

### Borders

| Token | Value | Tailwind class | Usage |
|---|---|---|---|
| `--color-border` | `#303030` | `border-border` | Default borders |
| `--color-border-subtle` | `rgba(255,255,255,0.04)` | `border-border-subtle` | Dividers, faint separation |
| `--color-border-strong` | `rgba(255,255,255,0.12)` | `border-border-strong` | Selected states, emphasis |

---

## 2. Typography

### Font Stacks

```css
--font-sans: -apple-system, BlinkMacSystemFont, "Inter", system-ui, sans-serif;
--font-mono: "SF Mono", "Fira Code", ui-monospace, monospace;
```

### Type Scale

| Token | Size | Tailwind | Usage |
|---|---|---|---|
| `--text-2xs` | 10px | `text-[10px]` or `text-2xs` | Badge text, table sort labels, section sub-labels |
| `--text-xs` | 12px | `text-xs` | Metadata, timestamps, captions, small labels |
| `--text-sm` | 13px | `text-[13px]` or `text-sm` | Body text, form labels, nav items |
| `--text-base` | 15px | `text-[15px]` or `text-base` | Primary body, input values |
| `--text-lg` | 17px | `text-[17px]` or `text-lg` | Subheadings, card titles |
| `--text-xl` | 20px | `text-xl` | Section headers |
| `--text-2xl` | 24px | `text-2xl` | Page titles |

### Font Weights

- `font-normal` (400) — body text, descriptions
- `font-medium` (500) — labels, nav items, button text, form labels
- `font-semibold` (600) — headings, panel titles, active nav items
- `font-bold` (700) — section micro-labels (uppercase tracking)

### Base Rendering

```css
/* Set in main.css */
html {
  font-size: 15px;
  line-height: 1.6;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
```

### Section Label Pattern

The standard pattern for panel sub-section headings. Used throughout the create form and settings panels:

```html
<p class="text-[10px] font-bold uppercase tracking-wider text-text-muted">
  Section Name
</p>
```

Do not use `SectionHeader.vue` for this — that component renders `text-xs font-medium uppercase tracking-[0.05em] text-text-secondary` (slightly different, used for card-level section titles with an optional action slot).

---

## 3. Spacing and Sizing

Base unit is 4px (standard Tailwind grid).

### Common Spacing

| Context | Tailwind | Notes |
|---|---|---|
| Inline gap (icon + text) | `gap-1` `gap-1.5` `gap-2` | Button content, badge internals |
| Component row gap | `gap-2` `gap-3` | Between chips, between form fields |
| Card section gap | `gap-4` `gap-5` | Between groups within a card |
| Page-level gap | `space-y-5` | Between cards on a page |
| Card padding | `p-4` `p-5` | Internal card content |
| Card header padding | `px-5 py-4` or `px-6 py-4` | Headers with border-b |
| Page content padding | `px-6 pb-6` | Set on `<main>` in App.vue |
| Input padding | `px-3 py-2` (0.5rem 0.75rem) | Applied by form component classes |

### Fixed Sizes

| Element | Size | Applied as |
|---|---|---|
| Titlebar drag region | `h-7` (28px) | `fixed top-0`, layout offset via `pt-7` |
| Sidebar branding row | `h-[3.75rem]` (60px) | Matches `pt-7` + icon clearance |
| Page header | `h-[3.75rem]` (60px) | Set in PageHeader.vue |
| Button `sm` | `min-h-[28px]` | Button.vue |
| Button `md` | `min-h-[32px]` | Button.vue |
| Button `lg` | `min-h-[40px]` | Button.vue |
| Input `sm` | `min-h-[2rem]` (32px) | FormInput `.input-sm` |
| Input `md` | `min-h-[2.5rem]` (40px) | FormInput `.input-md` |

---

## 4. Border Radius

| Token | Value | Tailwind | Usage |
|---|---|---|---|
| `--radius-sm` | 4px | `rounded` | Tiny inline elements |
| `--radius-md` | 6px | `rounded-md` | Segment chips |
| `--radius-lg` | 8px | `rounded-lg` | Buttons, inputs, small cards, dropdowns |
| `--radius-xl` | 12px | `rounded-xl` | Cards (`Card.vue`), modals, toasts |
| `--radius-2xl` | 16px | `rounded-2xl` | Large decorative containers |
| `9999px` | full circle | `rounded-full` | Status dots, filter chips, count badges |

**Standard mappings you should follow:**
- Buttons and inputs → `rounded-lg`
- Cards, modals, toasts → `rounded-xl`
- Badge component → `rounded-md` (set internally)
- Filter/status chips → `rounded-full`

---

## 5. Shadows and Depth

### Shadow Tokens

```css
/* Inset top highlight — applied to cards and inputs to add surface texture */
--shadow-inset:        inset 0 1px 0 color-mix(in srgb, white 4%, transparent);
--shadow-inset-strong: inset 0 1px 0 color-mix(in srgb, white 8%, transparent);

/* Elevation */
--shadow-sm:  0 1px 3px rgba(0, 0, 0, 0.3);
--shadow-md:  0 4px 12px rgba(0, 0, 0, 0.35);
--shadow-lg:  0 14px 30px -10px rgba(0, 0, 0, 0.5);
--shadow-xl:  0 20px 40px -12px rgba(0, 0, 0, 0.6);

/* Focus ring — use on all interactive elements */
--shadow-focus: 0 0 0 3px color-mix(in srgb, var(--color-accent) 22%, transparent);
```

### Vibrancy (the `.card` class)

All panel containers use the `.card` global class from `main.css`:

```css
.card {
  background: color-mix(in srgb, #0e0e0e 42%, transparent);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  border: 1px solid color-mix(in srgb, white 4%, transparent);
  box-shadow: var(--shadow-inset);
}
```

This is identical to `.glass-panel`. Either class works; prefer `.card`.

---

## 6. Layout (App Shell, Sidebar, Page Header)

### App Shell (`App.vue`)

```html
<div class="relative h-screen overflow-hidden text-text-primary antialiased selection:bg-accent selection:text-white">
  <!-- Ambient blobs — z-index 1 via .ambient-blobs class -->
  <div class="ambient-blobs" aria-hidden="true">
    <div class="ambient-blob blob-1"></div>
    <div class="ambient-blob blob-2"></div>
    <div class="ambient-blob blob-3"></div>
  </div>

  <!-- Titlebar drag region — full width, 28px, z-9999 -->
  <div class="titlebar-drag-region fixed top-0 left-0 right-0 z-[9999] h-7 border-b border-border-subtle"></div>

  <!-- Skip link for accessibility -->
  <a href="#main-content" class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:bg-surface-raised focus:px-4 focus:py-2 focus:rounded-lg focus:text-text-primary">
    Skip to content
  </a>

  <!-- Layout — pushed below titlebar with pt-7, z-index 2 -->
  <div class="flex h-full pt-7" style="position: relative; z-index: 2;">
    <AppSidebar />
    <main id="main-content" class="titlebar-no-drag flex flex-1 flex-col overflow-y-auto px-6 pb-6" style="background: transparent;">
      <RouterView />
    </main>
  </div>

  <!-- Global overlays (lazy-loaded, teleported to body) -->
  <SettingsModal v-if="showSettings" />
  <HelpModal v-if="showHelp" />
  <CommandPalette v-if="showCommandPalette" />
</div>
```

Note: `main` has `background: transparent` as an inline style — do not override this with a background class.

### Sidebar (`.sidebar` global class)

```css
.sidebar {
  background: color-mix(in srgb, #0e0e0e 42%, transparent);
  backdrop-filter: blur(24px) saturate(1.4);
  -webkit-backdrop-filter: blur(24px) saturate(1.4);
  border-right: 1px solid color-mix(in srgb, white 6%, transparent);
  box-shadow: inset -1px 0 0 rgba(255, 255, 255, 0.02);
}
```

Applied as `class="relative flex h-full flex-col sidebar transition-[width] select-none z-30"`.

**Sidebar nav link active state** — text colour and weight only, no background, no left border indicator:
```html
<!-- Active -->
class="text-accent font-semibold"

<!-- Inactive -->
class="text-text-muted hover:text-text-primary"
```

### Page Header (`.page-header` global class)

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

Applied in `PageHeader.vue`:
```html
<div class="page-header sticky top-0 z-20 -mx-6 mb-4 flex h-[3.75rem] items-center justify-between px-6 pt-1">
  <div class="flex items-center gap-4">
    <slot name="prepend" />
  </div>
  <div class="flex items-center gap-3">
    <slot name="actions" />
    <!-- "New Task" button hidden automatically when route.path === '/create' -->
  </div>
</div>
```

---

## 7. Cards and Panels

### Primary Container — `.card rounded-xl`

Use this for all major panel containers:

```html
<div class="card rounded-xl">
  <!-- content -->
</div>
```

For panels that should fill their container edge-to-edge (e.g. a table), omit internal padding and manage it per-section:

```html
<div class="card rounded-xl overflow-hidden">
  <div class="border-b border-border-subtle px-5 py-4">
    <h3 class="text-sm font-semibold text-text-primary">Panel Title</h3>
  </div>
  <div class="p-5">
    <!-- content -->
  </div>
</div>
```

### Card.vue Component

Prefer raw `.card rounded-xl` for new work. `Card.vue` is a thin wrapper:

```html
<!-- Default — padding 1.25rem, vibrancy -->
<Card>Content</Card>

<!-- Interactive — transitions border and bg on hover -->
<Card variant="interactive">Content</Card>

<!-- Flush — no padding, useful for tables that fill to edges -->
<Card variant="flush">Content</Card>

<!-- Inset — recessed appearance for nested cards within panels -->
<Card variant="inset">Content</Card>
```

`card-inset` CSS:
```css
background: color-mix(in srgb, var(--color-surface-base) 60%, var(--color-surface-raised));
border: 1px solid var(--color-border-subtle);
padding: 1.25rem;
```

### Panel Header Divider

Simple border — no custom background, no icon box:

```html
<div class="border-b border-border-subtle px-5 py-4">
  <h3 class="text-sm font-semibold text-text-primary">Title</h3>
</div>
```

### Inset Info Blocks

Used within the task preview sidebar and settings panels:

```html
<div class="rounded-lg border border-border-subtle p-3">
  <p class="text-[10px] font-bold uppercase tracking-wider text-text-muted">Section</p>
  <p class="mt-1 text-xs text-text-secondary">Content here.</p>
</div>
```

---

## 8. Buttons

Always use `Button.vue`. Do not recreate button styling inline.

### Props

| Prop | Values | Default |
|---|---|---|
| `variant` | `primary` `secondary` `brand` `danger` `danger-outline` `ghost` `outline` `link` | `primary` |
| `size` | `sm` `md` `lg` `icon` | `md` |
| `to` | Router path | — |
| `type` | `button` `submit` `reset` | `button` |
| `disabled` | Boolean | `false` |
| `loading` | Boolean | `false` |
| `block` | Boolean | `false` |

### All Solid Variants Share the Same Base

`primary`, `secondary`, `brand`, `danger`, `danger-outline`, and `outline` all render with the **same surface treatment**:

```css
/* Scoped in Button.vue */
.btn-primary, .btn-secondary, .btn-danger, .btn-danger-outline, .btn-outline {
  background: var(--color-surface-base);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-sm);
}
/* Hover for all solid variants */
:hover { border-color: var(--color-border-strong); }
```

The visual difference between variants is in the **text colour**:

| Variant | Text colour class |
|---|---|
| `primary` | `text-text-secondary hover:text-text-primary` |
| `secondary` | `text-text-secondary hover:text-text-primary` |
| `brand` | `text-text-secondary hover:text-text-primary` (same as outline) |
| `danger` | `text-danger` |
| `danger-outline` | `text-danger` |
| `outline` | `text-text-secondary hover:text-text-primary` |
| `ghost` | `bg-transparent border-transparent text-text-secondary hover:bg-surface-overlay hover:text-text-primary` |
| `link` | `text-text-secondary hover:text-text-primary hover:underline p-0 h-auto font-normal bg-transparent border-none` |

### Sizes

| Size | Height | Padding | Text |
|---|---|---|---|
| `sm` | `min-h-[28px]` | `px-2.5 py-1.5` | `text-xs` |
| `md` | `min-h-[32px]` | `px-4 py-1.5` | `text-sm` |
| `lg` | `min-h-[40px]` | `px-5 py-2.5` | `text-base` |
| `icon` | `min-h-[32px] min-w-[32px]` | `p-1.5` | — |

All sizes use `rounded-lg`.

### States

```css
/* All buttons */
transition: background-color, color, border-color, box-shadow — 150ms ease;
active: scale-[0.98]
disabled: opacity-50 pointer-events-none
loading: opacity-70 pointer-events-none cursor-wait

/* Focus */
outline: 2px solid color-mix(in srgb, var(--color-accent) 55%, transparent);
outline-offset: 2px;
```

### Usage Examples

```html
<!-- Primary CTA -->
<Button variant="primary" size="md" @click="save">Save Changes</Button>

<!-- Destructive action -->
<Button variant="danger" @click="deleteTask">Delete</Button>

<!-- Icon button (ghost, square) -->
<Button variant="ghost" size="icon" aria-label="Close">
  <svg class="h-4 w-4"><!-- × --></svg>
</Button>

<!-- With router link -->
<Button variant="outline" size="sm" to="/create">New Task</Button>

<!-- Loading state -->
<Button variant="primary" :loading="submitting">Creating...</Button>
```

---

## 9. Badges

Always use `Badge.vue`. Do not create ad-hoc styled `<span>` elements for status or label display.

### Props

| Prop | Values | Default |
|---|---|---|
| `label` | String | required |
| `variant` | `success` `danger` `warning` `info` `accent` `muted` `default` | `default` |

### Structure

```html
<span class="badge inline-flex items-center rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide border">
  Label
</span>
```

### Variant Colours (scoped CSS in Badge.vue)

All variants use `color-mix()` — do not replicate these inline:

| Variant | Background | Border | Text |
|---|---|---|---|
| `success` | success 15% | success 20% | `--color-success` |
| `danger` | danger 15% | danger 20% | `--color-danger` |
| `warning` | warning 15% | warning 20% | `--color-warning` |
| `info` | accent 15% | accent 20% | `--color-accent` |
| `accent` | accent 15% | accent 20% | `--color-accent` |
| `muted` | text-muted 10% | text-muted 15% | `--color-text-muted` |
| `default` | surface-overlay 60% | text-muted 15% | `--color-text-secondary` |

### Specialist Badge Components

**StatusBadge.vue** — wraps Badge, maps task `state` to variant:

| State | Variant | Label |
|---|---|---|
| `queued` | `warning` | Queued |
| `running` | `info` | Running |
| `done` | `success` | Done |
| `failed` | `danger` | Failed |
| `cancelled` | `warning` | Cancelled |
| `archived` | `muted` | Archived |

**PriorityBadge.vue** — maps priority number to variant:

| Priority value | Variant |
|---|---|
| ≤ 1 | `danger` |
| 2–3 | `warning` |
| 4–5 | `default` |
| 6–7 | `accent` |
| 8–10 | `muted` |

**AgentBadge.vue** — coloured dot + agent name + optional overflow count:

| Agent name contains | Dot colour |
|---|---|
| `claude` | `orange-400` |
| `gpt` | `emerald-400` |
| `deepseek` | `blue-400` |
| `gemini` | `violet-400` |
| default | `gray-400` |

Multi-agent overflow: `+N` sub-badge in `text-[9px]` with `bg-surface-elevated`.

### Sidebar Task Count Indicators

These are not `Badge.vue` — they are inline spans specific to the sidebar:

```html
<!-- Running count -->
<span class="flex h-5 min-w-5 items-center justify-center rounded bg-blue-500/10 px-1.5 text-[10px] font-medium text-blue-400">
  {{ count }}
</span>

<!-- Failed count -->
<span class="flex h-5 min-w-5 items-center justify-center rounded bg-rose-500/10 px-1.5 text-[10px] font-medium text-rose-400">
  {{ count }}
</span>

<!-- Queued count (neutral) -->
<span class="flex h-5 min-w-5 items-center justify-center rounded bg-surface-elevated px-1.5 text-[10px] font-medium text-text-muted">
  {{ count }}
</span>
```

---

## 10. Form Controls

Use the shared form components. Do not style raw `<input>`, `<select>`, or `<textarea>` elements directly.

### FormInput.vue

```html
<FormInput
  v-model="value"
  label="Field Name"
  placeholder="Enter a value..."
  type="text"
  size="md"
  :error="errorMessage"
/>
```

Renders:
```html
<div class="form-field"> <!-- flex column, gap-1.5 -->
  <label class="form-label">Field Name</label>
  <!-- text-sm, font-medium (500), text-text-secondary -->
  <input class="form-input input-md" />
</div>
```

`.form-input` CSS (scoped in FormInput.vue):
```css
width: 100%;
padding: 0.5rem 0.75rem;
border-radius: 0.625rem;  /* 10px — intentionally between rounded-lg and rounded-xl */
border: 1px solid var(--color-border);
background: color-mix(in srgb, var(--color-surface-raised) 78%, transparent);
font-size: var(--text-sm);
transition: border-color, background-color, box-shadow — 150ms ease;

:hover  → border-color: color-mix(in srgb, var(--color-accent) 24%, var(--color-border))
          background:   color-mix(in srgb, var(--color-surface-raised) 92%, transparent)

:focus  → border-color: var(--color-accent)
          background:   var(--color-surface-raised)
          box-shadow:   0 0 0 3px color-mix(in srgb, var(--color-accent) 22%, transparent)

:disabled → opacity: 0.6; cursor: not-allowed

.form-input-error  → border-color: var(--color-danger)
                     focus box-shadow uses danger tint
```

### FormSelect.vue

Same layout and label structure as `FormInput`. Key differences:

```css
.form-select {
  appearance: none;
  padding-right: 2.25rem;
  background-image: /* inline SVG chevron, colour #999999 */;
  background-position: right 0.75rem center;
  background-size: 10px 10px;
  cursor: pointer;
}
```

```html
<FormSelect
  v-model="selected"
  label="Priority"
  :options="[{ value: '1', label: 'P1 — Critical' }]"
  size="md"
/>
```

### FormTextarea.vue

Same as `FormInput` plus:
```css
.form-textarea {
  min-height: 5rem;
  resize: vertical;
  line-height: 1.6;
}
```

```html
<FormTextarea v-model="prompt" label="Prompt" :rows="6" placeholder="Describe the task..." />
```

### Small Filter Inputs (`.control-field-sm`)

For compact search/filter inputs in page headers and toolbars — do **not** use `FormInput` here:

```html
<input class="control-field-sm h-8 w-full pl-9 pr-3" placeholder="Search tasks..." />
```

`.control-field-sm` (global, `main.css`):
```css
height: 2rem;
border-radius: 0.5rem;
border: 1px solid var(--color-border);
background: var(--color-surface-base);
font-size: var(--text-xs);
box-shadow: var(--shadow-sm);
transition: border-color, background-color, box-shadow — 150ms ease;

:hover → border-color: var(--color-border-strong)
:focus → border-color: var(--color-accent); box-shadow: var(--shadow-focus)
```

### Checkboxes

```html
<div class="relative flex items-center">
  <input
    type="checkbox"
    v-model="autoCommit"
    class="peer h-4 w-4 appearance-none rounded border border-border bg-surface-base transition-all
           checked:border-accent checked:bg-accent"
  />
  <svg
    class="pointer-events-none absolute left-1/2 top-1/2 h-2.5 w-2.5 -translate-x-1/2 -translate-y-1/2
           text-white opacity-0 peer-checked:opacity-100 transition-opacity duration-200"
    fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3.5"
  >
    <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
  </svg>
</div>
```

When the checkbox is used within a card-style clickable label row:
```html
<label class="group flex cursor-pointer items-center gap-3 rounded-lg border border-border-subtle px-4 py-3 transition-colors hover:border-border hover:bg-surface-raised/50 active:scale-[0.98]">
  <!-- checkbox markup above -->
  <span class="text-xs font-medium text-text-secondary group-hover:text-text-primary transition-colors">Auto Commit</span>
</label>
```

### Inline Number Inputs (Advanced Form)

For fields like Timeout and Retries in the create form where no wrapper component is used:

```html
<input
  type="number"
  class="w-full rounded-lg border border-border bg-surface-base px-3 py-2 text-sm text-text-primary
         transition-colors focus:border-border-strong focus:outline-none hover:border-border-strong"
/>
```

---

## 11. Navigation

### Sidebar Nav Links

Active state uses `text-accent font-semibold` — **no background fill, no left border indicator**:

```html
<RouterLink
  :to="item.path"
  class="group/nav relative flex items-center justify-center transition-colors duration-150 rounded-md"
  :class="[
    isCollapsed ? 'w-9 h-9' : 'w-full gap-3 px-2 py-[5px]',
    isActive(item.path)
      ? 'text-accent font-semibold'
      : 'text-text-muted hover:text-text-primary'
  ]"
>
  <svg class="h-4 w-4"><!-- icon --></svg>
  <span v-if="!isCollapsed" class="whitespace-nowrap text-[13px] font-medium">{{ item.name }}</span>
</RouterLink>
```

### Service Status Footer

Simple dot + label, no badge component:

```html
<div class="relative border-t border-border p-3">
  <div class="flex items-center gap-2.5">
    <!-- dot: bg-emerald-500 (running) or bg-rose-500 (stopped) -->
    <span class="relative flex h-2 w-2 shrink-0">
      <span class="relative inline-flex h-2 w-2 rounded-full bg-emerald-500" />
    </span>
    <span class="whitespace-nowrap text-xs font-medium text-text-secondary">Scheduler running</span>
  </div>
</div>
```

### Segment Chips (Tab Filters)

Used in TaskListTabs and similar filter bars. Uses the global `.segment-chip` class:

```css
.segment-chip {
  display: inline-flex;
  align-items: center;
  padding: 0.25rem 0.625rem;
  border-radius: var(--radius-md); /* 6px */
  font-size: var(--text-xs);
  font-weight: 500;
  cursor: pointer;
  transition: all 150ms ease;
}
```

Apply state classes to the element alongside `.segment-chip`.

---

## 12. Tables

### Table Wrapper

```html
<div class="w-full overflow-x-auto">
  <table class="w-full" aria-label="Task list">
    <thead>...</thead>
    <tbody :class="{ 'stagger-fade-in': isInitialRender }">...</tbody>
  </table>
</div>
```

### Table Header (`.table-header`)

Applied as a scoped class in `TaskListTable.vue`:

```css
.table-header {
  position: sticky;
  top: 0;
  z-index: 10;
  background: color-mix(in srgb, var(--color-surface-raised) 92%, transparent);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow: inset 0 -1px 0 var(--color-border-subtle);
}
```

Column header cells:
```html
<th class="cursor-pointer whitespace-nowrap px-3 text-left text-2xs font-medium uppercase tracking-wider transition-colors active:bg-surface-overlay/50"
    :class="[
      density === 'compact' ? 'py-1.5' : 'py-2.5',
      sortKey === col.key ? 'text-text-primary' : 'text-text-muted hover:text-text-secondary'
    ]"
    @click="toggleSort(col.key)"
>
  <span class="flex items-center gap-1">
    {{ col.label }}
    <span v-if="sortKey === col.key" class="transition-transform" :class="sortAsc ? '' : 'rotate-180'">↑</span>
  </span>
</th>
```

### Table Rows

Row hover and alternating rows **must use scoped CSS** — Tailwind cannot apply hover styles to `td` child elements:

```html
<tr
  class="task-row group relative cursor-pointer border-b border-border-subtle transition-all focus-visible:outline focus-visible:outline-white/20"
  :class="index % 2 === 1 ? 'task-row--alt' : ''"
  tabindex="0"
  @click="navigateToDetail"
  @keydown.enter="navigateToDetail"
>
  <td class="whitespace-nowrap px-3" :class="density === 'compact' ? 'py-1.5' : 'py-2.5'">
    Content
  </td>
</tr>
```

```css
/* Scoped in TaskListRow.vue */
.task-row { transition: background-color 150ms ease, border-color 150ms ease; }
.task-row:hover td         { background: rgba(0, 0, 0, 0.25); }
.task-row--alt td          { background: rgba(0, 0, 0, 0.06); }
.task-row--alt:hover td    { background: rgba(0, 0, 0, 0.25); }
```

### Density Control

Pass `density` prop down to the row component:

| Value | Padding |
|---|---|
| `comfortable` (default) | `py-2.5` |
| `compact` | `py-1.5` |

### Retry Button in Table Row

```css
/* Scoped in TaskListRow.vue */
.retry-btn {
  border-color: var(--color-border);
  background: var(--color-surface-overlay);
  color: var(--color-text-secondary);
}
.retry-btn:hover:not(:disabled) {
  background: var(--color-surface-elevated);
  color: var(--color-text-primary);
  border-color: color-mix(in srgb, var(--color-accent) 24%, var(--color-border));
}
```

---

## 13. Toasts and Notifications

Always use `ToastNotification.vue`.

### Props

| Prop | Values | Default |
|---|---|---|
| `message` | String | required |
| `type` | `success` `error` `info` | `info` |
| `duration` | Milliseconds | `3000` |

### Structure

```html
<div class="toast pointer-events-auto flex w-96 items-center gap-3 overflow-hidden rounded-xl px-4 py-3">
  <!-- Icon container (h-7 w-7, rounded-lg, coloured bg) -->
  <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg" :class="iconBgClass">
    <!-- success: bg-success/10, error: bg-danger/10, info: bg-accent/10 -->
    <svg class="h-4 w-4 text-success"><!-- checkmark --></svg>
  </div>

  <!-- Message -->
  <span class="flex-1 text-sm font-medium text-text-primary">Task created.</span>

  <!-- Dismiss -->
  <button class="shrink-0 rounded-md p-1 text-text-muted transition-colors hover:text-text-primary" aria-label="Dismiss">
    <svg class="h-3.5 w-3.5"><!-- × --></svg>
  </button>
</div>
```

### Toast CSS (scoped in ToastNotification.vue)

```css
.toast {
  background: rgba(22, 22, 26, 0.85);
  backdrop-filter: blur(20px) saturate(1.3);
  -webkit-backdrop-filter: blur(20px) saturate(1.3);
  border: 1px solid rgba(255, 255, 255, 0.06);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(0, 0, 0, 0.2);
}
```

### Entry/Exit Animation

```html
<Transition
  enter-from-class="translate-y-3 opacity-0"
  enter-active-class="transition-all duration-300 ease-out"
  leave-to-class="translate-y-3 opacity-0"
  leave-active-class="transition-all duration-200 ease-in"
>
```

### ARIA

- `role="status"` for success/info; `role="alert"` for error
- `aria-live="polite"` for success/info; `aria-live="assertive"` for error

---

## 14. Modals and Dialogs

### Global Classes

`.backdrop` and `.modal` are defined in `main.css`:

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

### Standard Modal Pattern

```html
<Teleport to="body">
  <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center">
    <Transition name="backdrop-fade" appear>
      <div class="backdrop absolute inset-0" @click="$emit('cancel')"></div>
    </Transition>
    <div ref="dialogRef" class="modal animate-scale-in relative w-full max-w-md p-6"
         role="alertdialog" aria-modal="true" :aria-label="title">
      <!-- Modal content -->
    </div>
  </div>
</Teleport>
```

`.animate-scale-in` — `scale(0.95) opacity(0)` → `scale(1) opacity(1)`, 200ms ease.

### ConfirmDialog.vue

```html
<ConfirmDialog
  :open="showConfirm"
  title="Delete task?"
  message="This action cannot be undone."
  confirm-label="Delete"
  :danger="true"
  @confirm="deleteTask"
  @cancel="showConfirm = false"
/>
```

With `typedConfirmation` set, a `.control-field-sm h-9 w-full px-3 text-sm` input is rendered requiring an exact text match before confirm activates.

### Vue Transition Names

| Name | Enter | Leave |
|---|---|---|
| `backdrop-fade` | `opacity 200ms ease` | `opacity 150ms ease` |
| `modal` | `all 200ms ease`, `scale(0.95) + opacity` | `all 200ms ease`, same |
| `dropdown` | `all 120ms ease-out`, `scale(0.98) translateY(-4px) + opacity` | `all 100ms ease-in` |

### Focus Management

All modals use `useFocusTrap()`. Escape key fires the cancel emit. Do not open a modal without focus trapping.

---

## 15. Empty States and Loading

### EmptyState.vue

```html
<EmptyState message="No tasks found." action-label="Create Task" @action="goToCreate" />
```

Renders:
```html
<div class="animate-fade-in flex flex-col items-center justify-center py-16 text-center">
  <div class="mb-4 flex h-14 w-14 items-center justify-center rounded-xl border border-border-subtle bg-surface-raised shadow-[var(--shadow-inset)]">
    <svg class="h-7 w-7 text-text-tertiary" stroke-width="1"><!-- icon --></svg>
  </div>
  <p class="mb-4 text-sm text-text-secondary">No tasks found.</p>
  <Button variant="primary" size="sm">Create Task</Button>
</div>
```

### Inline Empty State (not using EmptyState.vue)

Used in `RecentActivityFeed.vue` and similar places where the container has its own background:

```html
<div class="flex flex-col items-center gap-3 px-5 py-16 text-center">
  <div class="flex h-12 w-12 items-center justify-center rounded-xl bg-surface-overlay ring-1 ring-border-strong shadow-inner">
    <svg class="h-6 w-6 text-text-muted opacity-50"><!-- icon --></svg>
  </div>
  <div class="space-y-1">
    <p class="text-sm font-medium text-text-secondary">No activity yet</p>
    <p class="text-xs text-text-muted">Start queueing tasks to see logs here.</p>
  </div>
</div>
```

### Skeleton.vue

```html
<Skeleton width="100%" height="1rem" rounded="md" />
<Skeleton width="60%" height="0.75rem" rounded="md" />
```

CSS (scoped in Skeleton.vue):
```css
.skeleton {
  border: 1px solid color-mix(in srgb, var(--color-border) 74%, transparent);
  background: color-mix(in srgb, var(--color-surface-raised) 92%, var(--color-surface-base));
  box-shadow: inset 0 1px 0 color-mix(in srgb, white 4%, transparent);
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
```

`rounded` prop: `sm` → `var(--radius-sm)`, `md` → `var(--radius-md)`, `lg` → `var(--radius-lg)`, `full` → `9999px`.

### Loading Spinners

Button.vue handles the loading spinner internally when `loading` prop is `true`. For standalone spinners:

```html
<svg class="h-4 w-4 animate-spin text-text-muted"><!-- spinner paths --></svg>
```

### Live Pulse Dot (running tasks)

```html
<span class="relative flex h-2 w-2 shrink-0">
  <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-success opacity-75"></span>
  <span class="relative inline-flex h-2 w-2 rounded-full bg-success"></span>
</span>
```

---

## 16. Animations and Transitions

### Global Animation Classes (`main.css`)

| Class | Effect | Duration |
|---|---|---|
| `.animate-fade-in` | opacity 0 → 1 | 150ms ease-out |
| `.animate-content-fade-in` | opacity 0 → 1 + translateY(4px → 0) | 200ms ease-out |
| `.animate-scale-in` | scale(0.95) + opacity 0 → scale(1) + opacity 1 | 200ms ease |
| `.stagger-fade-in > *` | Each child fades in with staggered delays (0–150ms, 30ms steps) | 200ms ease-out per child |

### Standard Durations

| Interaction | Duration | Easing |
|---|---|---|
| All interactive elements (buttons, inputs, links) | 150ms | ease |
| Dropdown appear | 120ms enter / 100ms leave | ease-out / ease-in |
| Modal appear | 200ms | ease |
| Backdrop | 200ms enter / 150ms leave | ease |
| Sidebar width | 200ms | implied by `transition-[width]` |

### Keyframes

```css
@keyframes fade-in {
  from { opacity: 0; }
  to   { opacity: 1; }
}

@keyframes content-fade-in {
  from { opacity: 0; transform: translateY(4px); }
  to   { opacity: 1; transform: translateY(0); }
}

@keyframes scale-in {
  from { opacity: 0; transform: scale(0.95); }
  to   { opacity: 1; transform: scale(1); }
}
```

### Reduced Motion

All blob animations are suppressed:
```css
@media (prefers-reduced-motion: reduce) {
  .ambient-blob { animation: none !important; }
}
```

Blob animations are also paused when the tab loses focus, via `document.documentElement.classList.toggle('page-hidden', document.hidden)` in `App.vue`.

---

## 17. Ambient Background

Three large blurred colour circles animate slowly behind the entire UI at `z-index: 1`. All panel surfaces are semi-transparent so the blob glow passes through.

### Structure

```html
<!-- In App.vue — always present, always behind all content -->
<div class="ambient-blobs" aria-hidden="true">
  <div class="ambient-blob blob-1"></div>
  <div class="ambient-blob blob-2"></div>
  <div class="ambient-blob blob-3"></div>
</div>
```

### Specifications

| Blob | Colour | Size | Position | Duration |
|---|---|---|---|---|
| `blob-1` | `--color-accent` (#60A5FA) | 800×800px | top -25%, right -10% | 25s |
| `blob-2` | `#8B5CF6` (violet) | 700×700px | bottom -20%, left -10% | 30s |
| `blob-3` | `#06B6D4` (cyan) | 600×600px | top 50%, right 35% | 20s |

```css
.ambient-blob {
  position: absolute;
  border-radius: 50%;
  filter: blur(100px);
  opacity: 0.07;
  will-change: transform;
}
```

**Do not use opaque backgrounds on cards or panels.** Panels must remain semi-transparent (`color-mix(in srgb, #0e0e0e 42%, transparent)`) so the blob glow shows through.

---

## 18. Scrollbars

Styled globally in `main.css`. No per-component scrollbar CSS is needed.

```css
::-webkit-scrollbar { width: 10px; height: 10px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--color-accent) 45%, transparent);
  border: 2px solid transparent;
  background-clip: content-box;
  border-radius: 999px;
}
::-webkit-scrollbar-thumb:hover  { background: color-mix(in srgb, var(--color-accent) 66%, transparent); }
::-webkit-scrollbar-thumb:active { background: color-mix(in srgb, var(--color-accent) 80%, transparent); }

/* Firefox */
* {
  scrollbar-width: thin;
  scrollbar-color: color-mix(in srgb, var(--color-accent) 45%, transparent) transparent;
}
```

---

## 19. Icons

All icons are inline SVG. There is no external icon library.

### Standard Attributes

```html
<svg
  fill="none"
  viewBox="0 0 24 24"
  stroke="currentColor"
  stroke-width="2"
  aria-hidden="true"
>
  <!-- path -->
</svg>
```

Use `stroke-width="1.5"` for larger decorative icons; `stroke-width="2"` for small interactive icons; `stroke-width="2.5"` for tiny icons (12px or less) where legibility needs reinforcing.

### Size Classes

| Tailwind | Size | Usage |
|---|---|---|
| `h-3 w-3` | 12px | Sort arrows, separators |
| `h-3.5 w-3.5` | 14px | Dismiss buttons, toolbar icons |
| `h-4 w-4` | 16px | Navigation, button icons, standard actions |
| `h-5 w-5` | 20px | Prominent header icons |
| `h-6 w-6` | 24px | Empty state inline icons |
| `h-7 w-7` | 28px | EmptyState.vue icon |

### Colour

Icons use `currentColor` — set the text colour on the icon or its container:

- Active/prominent → `text-text-primary`
- Inactive/secondary → `text-text-secondary`
- Muted/decorative → `text-text-muted` or `text-text-tertiary`
- Accent action → `text-accent`
- Status → `text-success` / `text-danger` / `text-warning`

Always add `aria-label` to icon-only interactive elements:

```html
<button aria-label="Dismiss notification">
  <svg aria-hidden="true" class="h-3.5 w-3.5"><!-- × --></svg>
</button>
```

---

## 20. Accessibility

- **Focus rings** — all interactive elements use `var(--shadow-focus)` (3px accent ring) via `focus-visible`. Never suppress focus outlines without a replacement.
- **ARIA labels** — all icon-only buttons have `aria-label`. Decorative SVGs have `aria-hidden="true"`.
- **Keyboard nav** — full tab order throughout. Escape closes modals and dropdowns.
- **Focus trap** — modals use `useFocusTrap()`. Do not open overlays without trapping focus.
- **Contrast** — all text tokens meet WCAG AA against their respective backgrounds in dark mode.
- **Reduced motion** — blob animations suppressed via `prefers-reduced-motion: reduce`.
- **Semantic HTML** — `<button>` for actions, `<a>` / `<RouterLink>` for navigation, explicit `<label>` associations on all form fields.
- **Screen reader announcements** — toasts use `role="status"` (success/info) or `role="alert"` (error) with matching `aria-live`.
- **Skip link** — `<a href="#main-content">` at the top of `App.vue`, visible only on focus.
- **`aria-current="page"`** — applied to active sidebar nav links.

---

## 21. Scoped CSS Reference

These patterns exist in component `<style scoped>` blocks because Tailwind cannot express them cleanly (mainly pseudo-selectors across parent→child boundaries).

### Row Hover (via scoped CSS — do not use Tailwind group-hover on `td`)

```css
/* TaskListRow.vue */
.task-row:hover td         { background: rgba(0, 0, 0, 0.25); }
.task-row--alt td          { background: rgba(0, 0, 0, 0.06); }
.task-row--alt:hover td    { background: rgba(0, 0, 0, 0.25); }
```

```css
/* RecentActivityFeed.vue */
.feed-row:hover { background: rgba(0, 0, 0, 0.25); }
```

### Divider Patterns

```css
/* Feed row divider — applied to header and date separators */
.feed-divider {
  border-bottom: 1px solid rgba(255, 255, 255, 0.04); /* = border-border-subtle */
}
```

Internal section dividers use `border-t border-white/5` (inline Tailwind):
```html
<div class="border-t border-white/5 pt-6 mt-8">
```

### Common `rgba()` Values

| Purpose | Value | CSS var equivalent |
|---|---|---|
| Row hover bg | `rgba(0, 0, 0, 0.25)` | — |
| Alternating row stripe | `rgba(0, 0, 0, 0.06)` | — |
| Subtle section divider | `rgba(255, 255, 255, 0.04)` | `--color-border-subtle` |
| Ghost/hover bg (links) | `rgba(255, 255, 255, 0.04)` → `0.08` on hover | — |
| Very faint divider | `rgba(255, 255, 255, 0.025)` | below border-subtle |
| Toast border | `rgba(255, 255, 255, 0.06)` | — |

### Button Variant CSS (Button.vue scoped)

```css
.btn-primary, .btn-secondary, .btn-danger, .btn-danger-outline, .btn-outline {
  background: var(--color-surface-base);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-sm);
}
/* All solid variants share one hover rule */
.btn-primary:hover, .btn-secondary:hover, .btn-danger:hover,
.btn-danger-outline:hover, .btn-outline:hover {
  border-color: var(--color-border-strong);
}
```

### Badge Variant CSS (Badge.vue scoped)

```css
.badge-success  { background: color-mix(in srgb, var(--color-success) 15%, transparent);  border-color: color-mix(in srgb, var(--color-success) 20%, transparent);  color: var(--color-success); }
.badge-danger   { background: color-mix(in srgb, var(--color-danger)  15%, transparent);  border-color: color-mix(in srgb, var(--color-danger)  20%, transparent);  color: var(--color-danger); }
.badge-warning  { background: color-mix(in srgb, var(--color-warning) 15%, transparent);  border-color: color-mix(in srgb, var(--color-warning) 20%, transparent);  color: var(--color-warning); }
.badge-info,
.badge-accent   { background: color-mix(in srgb, var(--color-accent)  15%, transparent);  border-color: color-mix(in srgb, var(--color-accent)  20%, transparent);  color: var(--color-accent); }
.badge-muted    { background: color-mix(in srgb, var(--color-text-muted) 10%, transparent); border-color: color-mix(in srgb, var(--color-text-muted) 15%, transparent); color: var(--color-text-muted); }
.badge-default  { background: color-mix(in srgb, var(--color-surface-overlay) 60%, transparent); border-color: color-mix(in srgb, var(--color-text-muted) 15%, transparent); color: var(--color-text-secondary); }
```

### Table Header (TaskListTable.vue scoped)

```css
.table-header {
  position: sticky;
  top: 0;
  z-index: 10;
  background: color-mix(in srgb, var(--color-surface-raised) 92%, transparent);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow: inset 0 -1px 0 var(--color-border-subtle);
}
```

### Retry Button (TaskListRow.vue scoped)

```css
.retry-btn {
  border-color: var(--color-border);
  background: var(--color-surface-overlay);
  color: var(--color-text-secondary);
}
.retry-btn:hover:not(:disabled) {
  background: var(--color-surface-elevated);
  color: var(--color-text-primary);
  border-color: color-mix(in srgb, var(--color-accent) 24%, var(--color-border));
}
```

### Toast Background (ToastNotification.vue scoped)

```css
.toast {
  background: rgba(22, 22, 26, 0.85);
  backdrop-filter: blur(20px) saturate(1.3);
  -webkit-backdrop-filter: blur(20px) saturate(1.3);
  border: 1px solid rgba(255, 255, 255, 0.06);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(0, 0, 0, 0.2);
}
```

### Sticky Footer Bar (TaskCreateForm.vue)

```html
<div class="sticky bottom-0 z-20 -mx-4 -mb-5 border-t border-border-subtle px-6 py-3 backdrop-blur-xl">
  <div class="flex w-full items-center justify-between gap-4">
    <!-- Left: summary Badge chips -->
    <div class="flex flex-wrap items-center gap-2 overflow-hidden">
      <Badge :label="primarySummary" variant="accent" />
      <Badge v-for="part in otherParts" :label="part" variant="default" />
    </div>
    <!-- Right: Cancel + Submit -->
    <div class="flex items-center gap-3">
      <Button variant="ghost" @click="cancel">Cancel</Button>
      <Button variant="primary" type="submit" :loading="submitting">Create Task</Button>
    </div>
  </div>
</div>
```

Background is transparent — `backdrop-blur-xl` on the container lets the card below show through.

---

## Quick Reference

### Most Common Patterns

```html
<!-- Card container -->
<div class="card rounded-xl">...</div>

<!-- Card with header divider -->
<div class="card rounded-xl overflow-hidden">
  <div class="border-b border-border-subtle px-5 py-4">
    <h3 class="text-sm font-semibold text-text-primary">Title</h3>
  </div>
  <div class="p-5">...</div>
</div>

<!-- Section micro-label -->
<p class="text-[10px] font-bold uppercase tracking-wider text-text-muted">Section</p>

<!-- Icon button -->
<button class="flex h-8 w-8 items-center justify-center rounded-md text-text-muted
               hover:text-text-primary hover:bg-surface-overlay transition-colors
               active:scale-[0.94]" aria-label="Action">
  <svg class="h-4 w-4" aria-hidden="true">...</svg>
</button>

<!-- Status dot -->
<span class="h-2 w-2 rounded-full bg-success"></span>

<!-- Status dot with live pulse (running) -->
<span class="relative flex h-2 w-2">
  <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-success opacity-75"></span>
  <span class="relative inline-flex h-2 w-2 rounded-full bg-success"></span>
</span>

<!-- Stagger list entry animation -->
<ul class="stagger-fade-in space-y-1">
  <li>...</li>
</ul>
```

### Do / Don't

| Do | Don't |
|---|---|
| Use `Button.vue` with a `variant` prop | Write custom button CSS inline |
| Use `Badge.vue` for all status/label chips | Create ad-hoc `<span>` badges |
| Use `.card rounded-xl` for containers | Use hardcoded `background: #262626` on panels |
| Use `border-border-subtle` for dividers | Use `border-gray-700` or similar named Tailwind greys |
| Apply row hover via scoped CSS on `td` | Use `hover:bg-black/25` on `<tr>` (doesn't target cells) |
| Use `text-accent font-semibold` for active nav | Add background tints or left border indicators to nav |
| Keep panel backgrounds semi-transparent | Use `bg-surface-raised` (solid) on panels — breaks blob effect |
| Set `aria-hidden="true"` on decorative SVGs | Leave all SVGs without ARIA attributes |
| Use `var(--shadow-focus)` on all focused elements | Suppress `:focus-visible` outlines |
| Use `FormInput`, `FormSelect`, `FormTextarea` components | Style raw form elements inline |
