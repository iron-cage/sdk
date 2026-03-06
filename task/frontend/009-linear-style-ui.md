# Task: Linear-Style UI

## Goal
Restyle the dashboard to match Linear's light mode visual aesthetic — clean white, minimal, dense. No integration changes, purely visual.

## Relevant Files
- `module/iron_dashboard/src/style.css` — CSS variables / design tokens, dark theme palette
- `module/iron_dashboard/src/components/MainLayout.vue` — sidebar and top header
- `module/iron_dashboard/src/views/DashboardView.vue` — stat cards with colorful icon circles
- `module/iron_dashboard/src/views/LoginView.vue` — login page
- `module/iron_dashboard/src/views/UsageView.vue` — analytics page
- `module/iron_dashboard/src/views/AgentsView.vue`
- `module/iron_dashboard/src/views/UsersView.vue`
- `module/iron_dashboard/src/views/ProvidersView.vue`
- `module/iron_dashboard/src/views/LimitsView.vue`
- `module/iron_dashboard/src/views/TokensView.vue`
- `module/iron_dashboard/src/views/BudgetRequestsView.vue`
- `module/iron_dashboard/src/components/ui/button/` — button variants
- `module/iron_dashboard/src/components/ui/card/` — card components
- `module/iron_dashboard/src/components/ui/input/Input.vue`
- `module/iron_dashboard/src/components/ui/badge/Badge.vue`

## Current State

### Colors
Right now the app uses the default light shadcn theme — white background, `bg-gray-100` page background, `bg-gray-900` sidebar. Cards use box shadows. Accent colors are scattered hardcoded Tailwind classes (`text-blue-600`, `bg-green-100`, `text-purple-600`, etc.).

### Layout
The sidebar is `w-64` with a distinct `bg-gray-800` header block for the logo. There's a sticky white top header (`bg-white shadow-sm`) that holds the username and logout button. Nav items use `rounded-md` with `bg-gray-800` fill on active.

### Components
Shadcn/vue components are locally owned (not a package), so Tailwind classes inside them can be edited directly. The Button, Card, Input, Badge components all carry the default shadcn light-mode styling.

## What needs to be done

- Update the HSL values in `style.css` to Linear's light palette — the token names stay the same (`--background`, `--muted`, `--border`, etc.), just replace the HSL numbers. Tailwind already maps these through `tailwind.config.js` so classes like `bg-background`, `bg-muted`, `border-border` will pick up the new values automatically
- Add Inter as the base font (import from Google Fonts in `index.html`)
- Rework the sidebar in `MainLayout.vue` — light gray background, no distinct logo header block, thin right border separator, compact nav items with small muted icons, grouped sections with tiny uppercase gray section labels (like "Your team", "Featured"), move logout to sidebar footer
- Remove the white sticky top header entirely — page content starts right after the sidebar, no separate header bar
- Replace the colorful icon circles on the dashboard stat cards (the blue/green/purple `rounded-full` divs in `DashboardView.vue`) with small plain monochrome icons
- Update Card components to drop box shadows and use `border-border` instead, keep `bg-card` white background
- Update Button, Input, Badge components to match — use token classes throughout, no heavy shadows
- Sweep all views and replace hardcoded `bg-gray-100`, `bg-gray-900`, `text-gray-900` etc. with the token-based Tailwind classes (`bg-background`, `bg-muted`, `text-foreground`, `text-muted-foreground`, etc.)
- Restyle tables across views (Agents, Users, Providers, Limits, Tokens) — Linear tables are dense with thin `border-border` row dividers, no alternating row backgrounds, small `text-muted-foreground` column headers
- Restyle dialogs — the create/edit modals in AgentsView, UsersView, etc. should match: white background, thin border, no heavy shadow, tight padding
- Restyle the login page — currently a centered card with `bg-gray-100` page background. Should be clean white full-page layout, minimal card or no card at all, just a centered form
- Page titles — replace `text-2xl font-bold text-gray-900` headings with something smaller and lighter to match Linear's low-contrast, dense page headers

## Notes
- All UI components in `src/components/ui/` are local files — edit Tailwind classes in them directly, no library to fight
- The project uses shadcn's token system: CSS variables in `style.css` → `tailwind.config.js` maps them to Tailwind color names → use as `bg-background`, `text-muted-foreground`, `border-border`, etc. Don't add raw hex or arbitrary Tailwind values like `bg-[#f7f7f8]` — update the HSL variables instead
- Linear light mode HSL equivalents: background `0 0% 100%`, muted (sidebar) `0 0% 97%`, border `0 0% 90%`, muted-foreground `0 0% 54%`, accent `235 55% 59%` (violet), radius `0.25rem`
- Nav item active state is very subtle — just `bg-muted` or one shade darker, no color fill, no bold text
- The sidebar in `MainLayout.vue` uses hardcoded `bg-gray-900` / `bg-gray-800` — these won't respond to token changes and need to be replaced manually with `bg-muted` / `bg-background`