# Development Environment Setup

## Purpose

Comprehensive guide for setting up iron_dashboard development environment, including installation, configuration, troubleshooting, and tooling recommendations. Expands on `readme.md` quick start with detailed rationale and advanced workflows.

---

## Scope

**Included:**
- Node.js and npm installation
- Project dependency installation
- Environment variable configuration
- Development server setup
- IDE/editor configuration
- Troubleshooting common issues

**Excluded:**
- Production deployment → See deployment guide (TBD)
- Backend setup → See `iron_control_api/readme.md`
- Architecture details → See `docs/architecture.md`
- API integration → See `docs/api_integration.md`

---

## Prerequisites

### Required Software

**Node.js (≥20.0.0)**

**Why:** Vite 7 requires Node.js 20+, Vue 3 Composition API benefits from modern JavaScript features

**Installation:**
```bash
# Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# macOS (Homebrew)
brew install node@20

# Verify installation
node --version  # Should be ≥20.0.0
npm --version   # Should be ≥10.0.0
```

**Alternative (nvm - recommended for multiple projects):**
```bash
# Install nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash

# Install Node.js 20
nvm install 20
nvm use 20
nvm alias default 20
```

**Rationale for nvm:** Allows switching Node versions per project, avoids system-wide version conflicts

---

### Backend Server (iron_control_api)

**Required:** Frontend cannot function without running backend

**Setup:**
```bash
# Terminal 1: Start iron_control_api backend
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_control_api
cargo run --release

# Wait for: "Server listening on http://0.0.0.0:3000"
```

**Why Running Locally:** Frontend makes REST API calls to `http://localhost:3000` for all data operations

**Health Check:**
```bash
curl http://localhost:3000/health
# Expected: {"status":"ok"}
```

**Troubleshooting:**
- Port 3000 already in use → Kill process: `sudo lsof -t -i:3000 | xargs kill -9`
- Database connection error → Ensure PostgreSQL running
- Compilation errors → Run `cargo clean && cargo build`

---

## Project Setup

### 1. Clone Repository

```bash
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_dashboard
```

**Note:** Repository already exists, this is for reference only

---

### 2. Install Dependencies

```bash
npm install
```

**What This Does:**
- Downloads all dependencies from `package.json` to `node_modules/`
- Generates `package-lock.json` (lockfile for reproducible builds)
- Typically takes 30-60 seconds on first install

**Expected Output:**
```
added 345 packages, and audited 346 packages in 42s

68 packages are looking for funding
  run `npm fund` for details

found 0 vulnerabilities
```

**Troubleshooting:**
- `EACCES` permission error → Avoid `sudo npm install`, use nvm instead
- Network timeout → Check firewall, try `npm install --verbose`
- Package conflict → Delete `node_modules/` and `package-lock.json`, retry

---

### 3. Environment Configuration

**File:** `.env` (create in project root)

```bash
# API Backend URL (default: http://localhost:3000)
VITE_API_URL=http://localhost:3000
```

**Usage in Code:**
```typescript
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000'
```

**Why `VITE_` Prefix:** Vite only exposes environment variables starting with `VITE_` to client code (security: prevents leaking server-side secrets)

**Other Environments:**
- **Production:** `VITE_API_URL=https://api.iron-cage.com`
- **Staging:** `VITE_API_URL=https://staging-api.iron-cage.com`
- **Local with custom port:** `VITE_API_URL=http://localhost:8080`

**Note:** `.env` file is git-ignored (never commit secrets)

---

## Development Workflow

### Start Development Server

```bash
npm run dev
```

**What This Does:**
- Starts Vite dev server on `http://localhost:5173`
- Enables Hot Module Replacement (HMR) - instant updates without full reload
- Serves source maps for debugging
- Watches files for changes

**Expected Output:**
```
  VITE v7.2.4  ready in 423 ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: http://192.168.1.100:5173/
  ➜  press h + enter to show help
```

**Access Application:**
Open browser to **http://localhost:5173**

**Default Login:**
- Username: `test` (or your configured user)
- Password: `test` (or your configured password)

**HMR Behavior:**
- Edit `.vue` file → Component reloads instantly (preserves state)
- Edit `.ts` file → Module reloads (resets state)
- Edit `vite.config.ts` → Requires full server restart

**Troubleshooting:**
- Port 5173 already in use → Kill process: `lsof -t -i:5173 | xargs kill -9`
- Blank page → Check browser console for errors (likely API connection failure)
- Slow HMR → Large dependency imported in hot path (check network tab)

---

### Build for Production

```bash
npm run build
```

**What This Does:**
1. Runs TypeScript compiler (`vue-tsc -b`) to check types
2. Bundles JavaScript/CSS with Vite (Rollup under the hood)
3. Minifies assets (Terser for JS, cssnano for CSS)
4. Outputs to `dist/` directory

**Build Output:**
```
dist/
├── index.html                 # Entry point
├── assets/
│   ├── index-abc123.js        # Main bundle (~180 KB)
│   ├── index-def456.css       # Styles (~15 KB)
│   ├── DashboardView-ghi789.js  # Lazy-loaded chunk
│   ├── TokensView-jkl012.js     # Lazy-loaded chunk
│   └── ...                      # Other route chunks
```

**Bundle Size Expectations:**
- **Main bundle:** ~180 KB (includes Vue, Pinia, TanStack Query, Radix Vue)
- **Gzipped:** ~60 KB
- **Lazy chunks:** 10-30 KB each (per route)

**Type Checking:**
- Build fails if TypeScript errors exist
- Strict mode enabled (`tsconfig.json:strict: true`)
- No `any` types allowed

**Troubleshooting:**
- Type errors → Fix TypeScript issues before build
- Build slow (>10s) → Check for large dependencies, consider code splitting
- Large bundle (>300 KB) → Run bundle analyzer (see below)

---

### Preview Production Build

```bash
npm run preview
```

**What This Does:**
- Serves `dist/` directory on `http://localhost:4173`
- Simulates production environment locally
- Does NOT rebuild (run `npm run build` first)

**Use Case:**
- Verify production build works before deployment
- Test lazy loading behavior
- Check bundle sizes in network tab

**Access Application:**
Open browser to **http://localhost:4173**

---

### Type Checking (Without Build)

```bash
npx vue-tsc --noEmit
```

**What This Does:**
- Runs TypeScript compiler on all `.vue` and `.ts` files
- Validates types without generating output files
- Faster than full build (~5s vs ~15s)

**Use Case:**
- Quick type validation during development
- CI/CD pre-commit hook
- Editor integration (e.g., VSCode)

**Expected Output (no errors):**
```
# No output = success
```

**Expected Output (with errors):**
```
src/views/DashboardView.vue:42:7 - error TS2322: Type 'string' is not assignable to type 'number'.

42   const count: number = "invalid"
         ~~~~~

Found 1 error in src/views/DashboardView.vue:42
```

---

## IDE Configuration

### VSCode (Recommended)

**Required Extensions:**
1. **Vue Language Features (Volar)** - `Vue.volar`
   - Vue 3 SFC syntax highlighting, type checking
   - Replaces Vetur (Vue 2 extension)

2. **TypeScript Vue Plugin (Volar)** - `Vue.vscode-typescript-vue-plugin`
   - Enables TypeScript support in `.vue` files
   - Required for `<script setup lang="ts">`

**Recommended Extensions:**
- **ESLint** - `dbaeumer.vscode-eslint` (auto-fix on save)
- **Prettier** - `esbenp.prettier-vscode` (code formatting)
- **Tailwind CSS IntelliSense** - `bradlc.vscode-tailwindcss` (class autocomplete)

**Workspace Settings (`.vscode/settings.json`):**
```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "tailwindCSS.experimental.classRegex": [
    ["class:\\s*['\"]([^'\"]*)['\"]"]
  ]
}
```

**Disable Vetur (if installed):**
Vetur conflicts with Volar, must be disabled for Vue 3 projects

---

### WebStorm / IntelliJ IDEA

**Built-in Support:**
- Vue 3 support enabled by default (2023.1+)
- TypeScript integration native
- No additional plugins required

**Recommended Settings:**
- Enable ESLint: `Settings → Languages & Frameworks → JavaScript → Code Quality Tools → ESLint`
- Enable Prettier: `Settings → Languages & Frameworks → JavaScript → Prettier`
- Enable Tailwind CSS: `Settings → Languages & Frameworks → Style Sheets → Tailwind CSS`

---

## Advanced Development Tools

### Bundle Analyzer

**Install:**
```bash
npm install -D rollup-plugin-visualizer
```

**Usage:**
```bash
npm run build
# Opens stats.html in browser with interactive treemap
```

**What It Shows:**
- Bundle size by dependency
- Tree-shaking effectiveness
- Duplicate dependencies
- Largest imports

**Use Case:**
- Identify large dependencies to lazy-load
- Find duplicate packages (version conflicts)
- Optimize bundle size

---

### Vue Devtools (Browser Extension)

**Install:**
- Chrome: [Vue.js devtools](https://chrome.google.com/webstore/detail/vuejs-devtools/nhdogjmejiglipccpnnnanhbledajbpd)
- Firefox: [Vue.js devtools](https://addons.mozilla.org/en-US/firefox/addon/vue-js-devtools/)

**Features:**
- Component tree inspection
- Pinia store state viewer
- TanStack Query cache inspector
- Performance profiling
- Event timeline

**Usage:**
1. Open browser DevTools (F12)
2. Navigate to "Vue" tab
3. Inspect components, stores, and queries

---

## Troubleshooting Common Issues

### Issue: "Cannot find module 'vue'"

**Cause:** Dependencies not installed

**Solution:**
```bash
rm -rf node_modules package-lock.json
npm install
```

---

### Issue: "Port 5173 already in use"

**Cause:** Previous dev server still running

**Solution:**
```bash
# Find process using port 5173
lsof -t -i:5173

# Kill process
lsof -t -i:5173 | xargs kill -9

# Restart dev server
npm run dev
```

**Alternative:** Change port in `vite.config.ts`:
```typescript
export default defineConfig({
  server: {
    port: 3001
  }
})
```

---

### Issue: "Network: ERR_CONNECTION_REFUSED"

**Cause:** Backend (iron_control_api) not running

**Solution:**
```bash
# Terminal 1: Start backend
cd /home/user1/pro/lib/wip_iron/iron_cage/dev/module/iron_control_api
cargo run

# Wait for server to start, then retry frontend
```

**Verify Backend Running:**
```bash
curl http://localhost:3000/health
# Expected: {"status":"ok"}
```

---

### Issue: TypeScript Errors in Vue Files

**Cause:** Volar not installed or enabled

**Solution (VSCode):**
1. Install "Vue Language Features (Volar)" extension
2. Disable "Vetur" extension (if installed)
3. Reload VSCode window (`Cmd/Ctrl + Shift + P` → "Reload Window")

**Solution (Command Line):**
```bash
npx vue-tsc --noEmit
# Shows all TypeScript errors
```

---

### Issue: Slow HMR (Hot Module Replacement)

**Cause:** Large dependency imported in frequently-updated file

**Solution:**
1. Identify slow import:
   - Check browser DevTools → Network tab → Filter by "JS"
   - Look for large files (>500 KB) loading on every change

2. Lazy-load heavy dependencies:
   ```typescript
   // Before (slow HMR)
   import Chart from 'chart.js/auto'

   // After (fast HMR)
   const Chart = await import('chart.js/auto')
   ```

3. Move static imports to separate file

---

### Issue: CORS Errors in Browser Console

**Cause:** Backend not configured to allow frontend origin

**Solution (Backend):**
Add CORS middleware in `iron_control_api`:
```rust
// In iron_control_api/src/main.rs
.layer(
  CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
)
```

**Temporary Workaround (Frontend):**
Use Vite proxy in `vite.config.ts`:
```typescript
export default defineConfig({
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true
      }
    }
  }
})
```

---

## Development Best Practices

### 1. Run Type Checking Before Committing

```bash
npx vue-tsc --noEmit && npm run build
```

**Why:** Catches type errors before CI/CD, prevents broken builds

---

### 2. Use HMR Effectively

- Keep component state local (use `ref()`, not global variables)
- Avoid side effects in top-level `<script setup>` (use `onMounted()`)
- Test full page reload periodically (HMR can mask issues)

---

### 3. Monitor Bundle Size

```bash
npm run build
# Check dist/assets/*.js sizes
```

**Target:** Main bundle <200 KB uncompressed, <70 KB gzipped

**Action if exceeded:** Lazy-load routes, remove unused dependencies

---

### 4. Test Production Build Locally

```bash
npm run build && npm run preview
```

**Why:** Production build behaves differently (minification, lazy loading, tree shaking)

---

## shadcn-vue Component Installation

### Overview

iron_dashboard uses shadcn-vue for UI components. Components are **copied** to `src/components/ui/` (not installed via npm as dependencies).

**Architecture:** Copy-paste (components owned by project, customizable)

### Currently Installed Components (12)

```bash
# List installed components
ls src/components/ui/
# Output: button/, dialog/, card/, input/, label/, badge/, select/, separator/, skeleton/, alert/, toast/, dropdown-menu/
```

### Installing Additional Components

**Command:**
```bash
npx shadcn-vue@latest add <component-name>
```

**Available Components:**
- Accordion, Avatar, Checkbox, Collapsible, Command
- ContextMenu, DataTable, DatePicker, Form, HoverCard
- Menubar, NavigationMenu, Popover, Progress, RadioGroup
- ScrollArea, Sheet, Slider, Switch, Table, Tabs
- Textarea, Toggle, Tooltip

**Installation Process:**
1. Run command (e.g., `npx shadcn-vue@latest add tooltip`)
2. CLI copies component files to `src/components/ui/<component>/`
3. Component TypeScript types auto-generated
4. Import and use: `import { Tooltip } from '@/components/ui/tooltip'`

**Example:**
```bash
# Install Tooltip component
npx shadcn-vue@latest add tooltip

# Verify installation
ls src/components/ui/tooltip/
# Output: Tooltip.vue, TooltipContent.vue, TooltipProvider.vue, TooltipTrigger.vue, index.ts
```

### Customizing Components

**Two approaches:**

1. **Edit component files directly:**
   ```bash
   # Edit Button component
   vim src/components/ui/button/Button.vue

   # Change variant colors, sizes, etc.
   ```

2. **Use className prop:**
   ```vue
   <Button :class="'custom-class'">Click Me</Button>
   ```

### Troubleshooting

**Issue:** `npx shadcn-vue@latest add <component>` fails with "Invalid components.json"
**Solution:** Verify `components.json` exists and is valid JSON

**Issue:** Component import fails (`Cannot find module '@/components/ui/button'`)
**Solution:**
1. Verify component directory exists: `ls src/components/ui/button/`
2. Check TypeScript path alias in `tsconfig.json` (should have `"@/*": ["./src/*"]`)
3. Restart Vite dev server: `npm run dev`

**Issue:** Component styling doesn't work
**Solution:**
1. Verify Tailwind CSS is running: `npm run dev` should show Tailwind processing
2. Check `tailwind.config.js` has correct content paths
3. Verify CSS variables in `src/style.css`

### CLI Reference

```bash
# List all available components
npx shadcn-vue@latest add

# Install specific component
npx shadcn-vue@latest add button

# Install multiple components at once
npx shadcn-vue@latest add button dialog card

# Force reinstall (overwrites existing)
npx shadcn-vue@latest add button --overwrite

# Check CLI version
npx shadcn-vue@latest --version
```

---

## Environment Variables Reference

| Variable | Default | Purpose |
|----------|---------|---------|
| `VITE_API_URL` | `http://localhost:3000` | Backend API base URL |
| `NODE_ENV` | `development` | Build environment (auto-set by Vite) |

**Future Variables:**
- `VITE_WS_URL` - WebSocket server URL (when WebSocket implemented)
- `VITE_SENTRY_DSN` - Error tracking (when monitoring implemented)
- `VITE_ANALYTICS_ID` - Analytics tracking (if needed)

---

## Decision Rationale

**Why Vite (not Webpack)?**
- 10x faster cold start (~400ms vs ~4s)
- Native ESM during development (no bundling)
- Simpler configuration (zero-config for most use cases)
- Better HMR performance

**Why npm (not yarn/pnpm)?**
- Standard Node.js package manager (no additional install)
- Lockfile (`package-lock.json`) committed to repo
- Consistent with iron_cage tooling conventions

**Why Node 20+ (not LTS 18)?**
- Vite 7 requires Node 20+
- Performance improvements (native Fetch API, faster regex)
- Future-proofing (18 EOL April 2025)

---

## References

- **Quick Start:** `readme.md` (Development section)
- **Architecture Details:** `docs/architecture.md`
- **API Integration:** `docs/api_integration.md`
- **Vite Documentation:** https://vitejs.dev/guide/
- **Vue 3 Documentation:** https://vuejs.org/guide/quick-start.html
- **TypeScript Documentation:** https://www.typescriptlang.org/docs/

---

**End of Development Setup Guide**
