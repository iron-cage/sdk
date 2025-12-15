# architecture

Technical architecture guide for iron_dashboard.

## Purpose

This document describes the frontend architecture, including Vue 3 patterns, component organization, state management strategies, routing configuration, and build pipeline. Intended for developers adding features or modifying existing components.

---

## Table of Contents

1. [Technology Stack](#technology-stack)
2. [Vue 3 Architecture](#vue-3-architecture)
3. [Component Structure](#component-structure)
4. [State Management](#state-management)
5. [Routing Strategy](#routing-strategy)
6. [Build Pipeline](#build-pipeline)
7. [Type Safety](#type-safety)
8. [Performance Optimization](#performance-optimization)

---

## Technology Stack

**Responsibility:** This section is the **authoritative source** for iron_dashboard's technology choices, including versions, selection rationale, and architectural implications. For quick reference see `readme.md`; for requirements context see `spec.md`.

---

### Core Framework

#### Vue 3.5.24

**Why Vue 3:**
- **Composition API:** Better TypeScript integration, improved code reusability through composables
- **Performance:** ~20-40% faster than Vue 2, smaller bundle size with tree-shaking
- **Modern Features:** Teleport, Suspense, Fragments, `<script setup>` syntax sugar
- **Ecosystem Maturity:** Comprehensive tooling (Vite, Pinia, Vue Router, Devtools)

**Alternatives Considered:**
- **React 18:** More verbose (JSX + hooks), larger ecosystem but steeper learning curve
- **Svelte 4:** Smaller bundle, but less mature ecosystem for enterprise features
- **Angular 17:** Over-engineered for pilot scope, requires TypeScript decorators

**Trade-offs:**
- ✅ Pros: Gentle learning curve, excellent TypeScript support, fast HMR
- ⚠️ Cons: Smaller talent pool than React, fewer third-party component libraries

---

#### TypeScript 5.9.3

**Why TypeScript:**
- **Type Safety:** Catch errors at compile-time (strict mode enabled)
- **Developer Experience:** IntelliSense, refactoring, auto-completion in VS Code
- **API Contract Enforcement:** Frontend types match Rust backend schemas exactly
- **Maintainability:** Self-documenting code, easier refactoring

**Configuration:**
```json
{
  "strict": true,              // Enable all strict type-checking
  "noUncheckedIndexedAccess": true,  // Safer array/object access
  "noImplicitReturns": true    // Explicit return types
}
```

**Alternatives Considered:**
- **JavaScript with JSDoc:** No compile-time guarantees, poor IDE support
- **Flow:** Deprecated by Meta, moving to TypeScript

**Trade-offs:**
- ✅ Pros: Prevents ~30% of runtime errors, better refactoring confidence
- ⚠️ Cons: Slightly slower development (type annotations), build step required

---

### Build Tool

#### Vite 7.2.4

**Why Vite:**
- **Speed:** 10-100x faster cold start than Webpack (~400ms vs ~4s)
- **Native ESM:** No bundling during development, instant server start
- **Hot Module Replacement:** Sub-50ms HMR updates preserve component state
- **Rollup Production Build:** Optimized bundles with tree-shaking, code splitting

**Alternatives Considered:**
- **Webpack 5:** Mature but slow, complex configuration, slower HMR
- **Parcel 2:** Zero-config but less control, smaller ecosystem
- **Turbopack:** Experimental (Next.js only), not production-ready

**Trade-offs:**
- ✅ Pros: Best DX, fast iteration, modern defaults
- ⚠️ Cons: Dev/prod parity issues (ESM vs bundled), edge cases with CommonJS deps

---

### UI Framework

#### Radix Vue 1.9.17

**Why Radix Vue:**
- **Accessibility:** WCAG 2.1 Level AA compliant out-of-the-box (keyboard nav, ARIA, focus management)
- **Unstyled Primitives:** No design opinions, full control with Tailwind
- **Composable:** Built on Vue Composition API, fits naturally into `<script setup>`
- **Future-Proof:** Will be used for modals, dropdowns, tooltips when added

**Current Usage:**
- Planned for future components (not yet implemented in pilot)
- Pre-integrated in `package.json` for rapid expansion

**Alternatives Considered:**
- **Headless UI Vue:** Limited component set, less active development
- **PrimeVue:** Opinionated styling, harder to customize with Tailwind
- **Element Plus:** Material Design aesthetic conflicts with Iron Cage branding

**Trade-offs:**
- ✅ Pros: Best-in-class a11y, flexible styling, active maintenance
- ⚠️ Cons: Additional dependency (~50 KB), requires Tailwind integration

---

#### shadcn-vue 2.4.2

**Why shadcn-vue:**
- **Production-Ready Components:** Built on Radix Vue with consistent styling
- **Copy-Paste Architecture:** Components copied to src/components/ui/ (not npm dependency)
  - Full ownership: Can modify components without ejecting
  - No version conflicts: Components are part of project source
  - Customizable: Edit component files directly or via className prop
- **Accessibility:** Inherits Radix Vue primitives (WCAG 2.1 AA compliant)
  - Focus trapping in Dialog
  - Keyboard navigation in Select (Arrow keys, Enter, Escape)
  - ARIA attributes (role, aria-labelledby, aria-describedby)
  - Screen reader support
- **Variant-Based Styling:** class-variance-authority (CVA) for type-safe variants
  - Button: default, secondary, outline, ghost, link, destructive
  - Badge: default, secondary, destructive, outline
  - TypeScript autocomplete for variant props
- **Minimal Bundle Impact:** Components tree-shaken, only used components bundled
  - Button: ~2 KB (incl. CVA)
  - Dialog: ~8 KB (incl. Radix Dialog primitive)
  - Total added: ~8 KB gzipped for 12 components (dependencies already installed)
- **Developer Experience:** CLI for installation, examples in docs, TypeScript support

**How It Works:**
1. Run: `npx shadcn-vue@latest add button`
2. CLI copies Button.vue to src/components/ui/button/
3. Component uses Radix Vue primitive + CVA variants
4. Import and use: `import { Button } from '@/components/ui/button'`
5. Customize: Edit Button.vue or add className prop

**Copy-Paste Architecture Example:**
```typescript
// src/components/ui/button/Button.vue (owned by project)
<script setup lang="ts">
import { cn } from '@/lib/utils'
import { type HTMLAttributes, computed } from 'vue'
import { Primitive, type PrimitiveProps } from 'radix-vue'
import { type ButtonVariants, buttonVariants } from '.'

// Variants defined using class-variance-authority (CVA)
export const buttonVariants = cva(
  'inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
        outline: 'border border-input hover:bg-accent hover:text-accent-foreground',
        // ... more variants
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-9 rounded-md px-3',
        lg: 'h-11 rounded-md px-8',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  }
)
</script>
```

**Alternatives Considered:**
- **Vuetify:** Material Design only, opinionated, large bundle (~500 KB)
- **PrimeVue:** Enterprise-focused, less modern styling, steep learning curve
- **Naive UI:** Good components but less accessible, no CVA-style variants
- **Custom components:** Full control but 10x more work, accessibility challenging

**Trade-offs:**
- ✅ Pros: Production-ready, accessible, customizable, minimal bundle, type-safe
- ✅ Pros: Copy-paste architecture (no npm lock-in)
- ⚠️ Cons: Must update components manually (not auto-updated via npm)
- ⚠️ Cons: Requires Tailwind CSS (can't use other CSS frameworks)

**Architectural Decision:**
shadcn-vue provides the best balance of accessibility, customizability, and developer experience. Copy-paste architecture allows full customization without ejecting, while Radix Vue primitives ensure WCAG compliance. CVA provides type-safe variants without CSS-in-JS overhead.

**Compatibility Note:**
shadcn-vue requires Tailwind CSS v3 (not v4). Project downgraded from `@tailwindcss/postcss` 4.1.17 to `tailwindcss` 3.4.17 for compatibility. No v4-specific features were used, so downgrade was safe.

---

#### class-variance-authority (CVA) 0.7.1

**Why CVA:**
- Type-safe variant props (autocomplete in IDE)
- Conditional styling without CSS-in-JS
- Better performance than styled-components (compile-time)
- Used by shadcn-vue for Button, Badge variants

**Example:**
```typescript
const buttonVariants = cva('base-classes', {
  variants: {
    variant: {
      default: 'bg-blue-600',
      secondary: 'bg-gray-600',
    },
    size: {
      sm: 'h-9',
      lg: 'h-11',
    },
  },
})

// TypeScript autocomplete for variant/size
<Button variant="default" size="sm" />
```

**Alternatives:**
- CSS-in-JS (emotion, styled-components) → Runtime overhead
- Stitches → React-only
- Vanilla CSS classes → No type safety, verbose

**Trade-offs:**
- ✅ Pros: Type-safe, compile-time, zero runtime overhead
- ⚠️ Cons: Requires setup, learning curve for variant syntax

---

#### clsx & tailwind-merge

**Why Both:**
- **clsx 2.1.1:** Conditional class names
  - `clsx('base', isActive && 'active')` → 'base active' or 'base'
  - Small (0.3 KB gzipped), fast, widely used
- **tailwind-merge 3.4.0:** Merge conflicting Tailwind classes
  - Prevents: `<div class="px-4 px-2">` (both px-4 and px-2 apply)
  - Merges: `twMerge('px-4', 'px-2')` → 'px-2' (last wins)

**Combined in cn() Helper:**
```typescript
// src/lib/utils.ts
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// Usage
<Button :class="cn('base-class', isActive && 'active-class', className)" />
```

**Alternatives:**
- Manual class merging → Error-prone, verbose
- classnames library → Doesn't merge Tailwind conflicts

**Trade-offs:**
- ✅ Pros: Prevents class conflicts, clean conditional logic
- ⚠️ Cons: Adds ~2.8 KB gzipped (acceptable for utility)

---

#### Tailwind CSS 3.4.17

**Why Tailwind:**
- **Utility-First:** Rapid prototyping without context-switching to CSS files
- **Consistency:** Design tokens enforce spacing, colors, typography standards
- **Bundle Size:** PurgeCSS removes unused classes (~15 KB gzipped final CSS)
- **Developer Experience:** IntelliSense in VS Code, responsive modifiers (`md:`, `lg:`)

**Configuration:**
```javascript
// tailwind.config.js
module.exports = {
  content: ['./index.html', './src/**/*.{vue,js,ts}'],
  theme: {
    extend: {
      colors: {
        iron: { /* custom palette */ }
      }
    }
  }
}
```

**Alternatives Considered:**
- **CSS Modules:** Manual class naming, no design system enforcement
- **Styled Components (Vue):** Runtime CSS-in-JS overhead, larger bundle
- **Bootstrap 5:** Opinionated components, harder to customize

**Trade-offs:**
- ✅ Pros: Fast iteration, consistent design, small bundle
- ⚠️ Cons: Verbose HTML classes, learning curve for utility naming

---

### State Management

#### Pinia 3.0.4

**Why Pinia:**
- **Official Vue Store:** Recommended by Vue core team (replaces Vuex)
- **TypeScript Native:** Full type inference without manual typing
- **Composition API:** Uses `ref()`, `computed()` internally (familiar API)
- **DevTools:** Vue Devtools integration for time-travel debugging

**Usage Pattern:**
```typescript
// src/stores/auth.ts
export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(null)
  const isAuthenticated = computed(() => !!token.value)

  function login(credentials: Credentials) { /* ... */ }
  function logout() { token.value = null }

  return { token, isAuthenticated, login, logout }
})
```

**Scope:** Global UI state (authentication, theme, sidebar state)

**Alternatives Considered:**
- **Vuex 4:** Options API style, verbose, deprecated in favor of Pinia
- **Vue Context (provide/inject):** No DevTools, manual reactivity management
- **Zustand/Jotai (React ports):** Not Vue-native, poor TypeScript inference

**Trade-offs:**
- ✅ Pros: Simple API, great TypeScript, minimal boilerplate
- ⚠️ Cons: Another dependency (~10 KB), learning curve for Vuex users

---

#### TanStack Vue Query 5.92.0

**Why TanStack Query:**
- **Server State Specialization:** Automatic caching, refetching, invalidation
- **Request Deduplication:** Multiple components request same data → single fetch
- **Background Refetching:** Keep UI fresh without manual polling
- **Optimistic Updates:** Instant UI updates while mutations process
- **DevTools:** Inspect cache, queries, mutations in real-time

**Usage Pattern:**
```typescript
const { data: tokens, isLoading, error } = useQuery({
  queryKey: ['tokens'],
  queryFn: () => api.getTokens(),
  staleTime: 5000  // Cache for 5 seconds
})

const createMutation = useMutation({
  mutationFn: (data) => api.createToken(data),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['tokens'] })
  }
})
```

**Scope:** All server data (tokens, usage stats, limits, traces)

**Alternatives Considered:**
- **Apollo Client:** GraphQL-specific, overkill for REST API
- **SWR Vue:** Less mature, fewer features than TanStack Query
- **Manual fetch + Pinia:** Duplicates caching logic, error-prone

**Trade-offs:**
- ✅ Pros: Eliminates boilerplate, automatic cache management, great DX
- ⚠️ Cons: Learning curve (~40 KB bundle), overkill for simple apps

---

### Routing

#### Vue Router 4.6.3

**Why Vue Router:**
- **Official Vue Router:** First-party support, designed for Vue 3
- **Type-Safe Routes:** TypeScript route definitions with params/query validation
- **Navigation Guards:** Authentication, authorization, analytics hooks
- **Code Splitting:** Lazy-load routes for faster initial load

**Configuration:**
```typescript
const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/login', component: LoginView },
    { path: '/dashboard', component: () => import('./views/DashboardView.vue') }
  ]
})

router.beforeEach((to, from, next) => {
  const authStore = useAuthStore()
  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    next('/login')
  } else {
    next()
  }
})
```

**Alternatives Considered:**
- **React Router (Vue port):** Not Vue-native, poor ecosystem fit
- **Vanilla History API:** Manual routing, no guards, no lazy loading

**Trade-offs:**
- ✅ Pros: Official support, comprehensive features, great TypeScript
- ⚠️ Cons: Minimal (only viable option for Vue 3 SPA)

---

### Development Dependencies

| Tool | Version | Purpose | Why This Choice |
|------|---------|---------|-----------------|
| **vue-tsc** | 2.0.x | TypeScript checking for `.vue` files | Official Vue TypeScript compiler, required for SFC type-checking |
| **@vitejs/plugin-vue** | 6.0.x | Vue SFC support in Vite | Official Vite plugin, enables `<script setup>` HMR |
| **autoprefixer** | 10.4.x | CSS vendor prefixes | Ensures cross-browser CSS compatibility (flexbox, grid) |
| **postcss** | 8.4.x | CSS processing | Required by Tailwind and autoprefixer |

---

### Version Policy

**Locked Versions:**
- All production dependencies use exact versions in `package.json` (no `^` or `~`)
- Lock file (`package-lock.json`) committed to ensure reproducible builds

**Update Strategy:**
- **Patch updates:** Applied immediately (security fixes)
- **Minor updates:** Quarterly review (new features, deprecation warnings)
- **Major updates:** Requires spec update + testing (breaking changes)

**Rationale:** Pilot project prioritizes stability over latest features. Conference demo cannot afford runtime surprises.

---

### Bundle Size Targets

**Production Build Goals:**
- **Main bundle:** <200 KB uncompressed, <70 KB gzipped
- **Lazy chunks:** 10-30 KB each (per route)
- **Total initial load:** <100 KB gzipped
- **CSS:** <20 KB gzipped

**Current Metrics (as of last build):**
- Main bundle: ~180 KB uncompressed, ~60 KB gzipped ✅
- Largest chunk: DashboardView (~25 KB) ✅
- CSS: ~15 KB gzipped ✅

**Monitoring:** Run `npm run build` to verify bundle sizes before commits.

---

### Architecture Decision Records (ADRs)

Full ADRs for specific patterns documented in sections below:
- **ADL-1:** Composition API over Options API → See [Vue 3 Architecture](#vue-3-architecture)
- **ADL-2:** TanStack Query for Server State → See [State Management](#state-management)
- **ADL-3:** Radix Vue for UI Primitives → See [Component Structure](#component-structure)

---

## Vue 3 Architecture

### Composition API with `<script setup>`

**Decision:** Use Composition API exclusively, no Options API.

**Rationale:**
- Better TypeScript inference
- Improved code organization (group by feature, not by option)
- Smaller bundle size (tree-shaking friendly)
- More explicit reactivity (ref, reactive, computed)

**Example:**

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useApi } from '@/composables/useApi'

// Props
interface Props {
  tokenId: number
}
const props = defineProps<Props>()

// State
const tokens = ref<TokenMetadata[]>([])
const loading = ref(false)

// Computed
const activeTokens = computed(() =>
  tokens.value.filter(t => t.is_active)
)

// Composables
const api = useApi()

// Methods
async function fetchTokens() {
  loading.value = true
  try {
    tokens.value = await api.getTokens()
  } catch (error) {
    console.error('Failed to fetch tokens:', error)
  } finally {
    loading.value = false
  }
}

// Lifecycle
onMounted(() => {
  fetchTokens()
})

// Emits
const emit = defineEmits<{
  tokenCreated: [token: TokenMetadata]
}>()
</script>

<template>
  <div>
    <p v-if="loading">Loading...</p>
    <ul v-else>
      <li v-for="token in activeTokens" :key="token.id">
        {{ token.name }}
      </li>
    </ul>
  </div>
</template>
```

---

### Single-File Components (SFC)

**Structure:**

```vue
<script setup lang="ts">
// 1. Imports (Vue, libraries, components, composables, types)
// 2. Props definition (defineProps)
// 3. Emits definition (defineEmits)
// 4. State (ref, reactive)
// 5. Composables (useApi, useAuth, etc.)
// 6. Computed properties
// 7. Methods (async functions, event handlers)
// 8. Lifecycle hooks (onMounted, onUnmounted, etc.)
// 9. Watchers (watch, watchEffect)
</script>

<template>
  <!-- Single root element (not required in Vue 3, but recommended) -->
  <div class="container">
    <!-- Template content -->
  </div>
</template>

<style scoped>
/* Component-specific styles (Tailwind utilities preferred) */
</style>
```

**Best Practices:**
- Keep `<script setup>` under 150 lines (extract composables if larger)
- Use TypeScript for all props, emits, and state
- Prefer Tailwind utilities over custom CSS
- Use `scoped` styles sparingly (most styling via Tailwind)

---

## Component Structure

### Component Hierarchy

```
src/
├── App.vue                 # Root component (router-view, global layout)
├── views/                  # Page-level components (routes)
│   ├── LoginView.vue       # /login
│   ├── DashboardView.vue   # /dashboard
│   ├── TokensView.vue      # /tokens
│   ├── UsageView.vue       # /usage
│   ├── LimitsView.vue      # /limits
│   └── TracesView.vue      # /traces
└── components/             # Reusable UI components
    ├── AppHeader.vue       # Global header (navigation, logout)
    ├── BudgetCard.vue      # Budget status widget
    ├── AgentCard.vue       # Agent execution widget
    ├── TokenTable.vue      # Token list table
    ├── CreateTokenModal.vue# Token creation dialog
    ├── UsageStatsCard.vue  # Usage statistics card
    ├── LimitForm.vue       # Limit creation/edit form
    └── TraceDetailModal.vue# Trace detail view dialog
```

**Naming Conventions:**
- Views: `[Feature]View.vue` (e.g., `TokensView.vue`)
- Components: `[Feature][Type].vue` (e.g., `TokenTable.vue`, `CreateTokenModal.vue`)
- Use PascalCase for component names (matches import syntax)

---

### View Components (Route-Level)

**Responsibilities:**
- Fetch data (composables, TanStack Query)
- Manage page-level state
- Coordinate multiple child components
- Handle routing (navigation, query params)

**Example (`TokensView.vue`):**

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi } from '@/composables/useApi'
import TokenTable from '@/components/TokenTable.vue'
import CreateTokenModal from '@/components/CreateTokenModal.vue'

const api = useApi()
const queryClient = useQueryClient()

// Fetch tokens (TanStack Query)
const { data: tokens, isLoading } = useQuery({
  queryKey: ['tokens'],
  queryFn: () => api.getTokens(),
  staleTime: 5 * 60 * 1000, // 5 minutes
})

// Create token mutation
const createTokenMutation = useMutation({
  mutationFn: api.createToken,
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['tokens'] })
    showCreateModal.value = false
  },
})

// Modal state
const showCreateModal = ref(false)

function handleCreateToken(data: CreateTokenRequest) {
  createTokenMutation.mutate(data)
}
</script>

<template>
  <div class="container mx-auto p-6">
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold">API Tokens</h1>
      <button @click="showCreateModal = true" class="btn-primary">
        Create Token
      </button>
    </div>

    <TokenTable
      :tokens="tokens ?? []"
      :loading="isLoading"
      @rotate="handleRotateToken"
      @revoke="handleRevokeToken"
    />

    <CreateTokenModal
      v-if="showCreateModal"
      @create="handleCreateToken"
      @close="showCreateModal = false"
    />
  </div>
</template>
```

---

### Reusable Components

**Responsibilities:**
- Display UI (no direct API calls)
- Receive data via props
- Emit events for parent handling
- Focus on single responsibility

**Example (`TokenTable.vue`):**

```vue
<script setup lang="ts">
import type { TokenMetadata } from '@/composables/useApi'

interface Props {
  tokens: TokenMetadata[]
  loading: boolean
}
defineProps<Props>()

interface Emits {
  rotate: [tokenId: number]
  revoke: [tokenId: number]
}
const emit = defineEmits<Emits>()

function handleRotate(tokenId: number) {
  emit('rotate', tokenId)
}

function handleRevoke(tokenId: number) {
  emit('revoke', tokenId)
}
</script>

<template>
  <div class="overflow-x-auto">
    <table class="min-w-full divide-y divide-gray-200">
      <thead class="bg-gray-50">
        <tr>
          <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">ID</th>
          <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Name</th>
          <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Status</th>
          <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">Actions</th>
        </tr>
      </thead>
      <tbody class="bg-white divide-y divide-gray-200">
        <tr v-if="loading">
          <td colspan="4" class="px-6 py-4 text-center text-gray-500">Loading...</td>
        </tr>
        <tr v-else-if="tokens.length === 0">
          <td colspan="4" class="px-6 py-4 text-center text-gray-500">No tokens found</td>
        </tr>
        <tr v-else v-for="token in tokens" :key="token.id">
          <td class="px-6 py-4 whitespace-nowrap">{{ token.id }}</td>
          <td class="px-6 py-4 whitespace-nowrap">{{ token.name ?? 'Unnamed' }}</td>
          <td class="px-6 py-4 whitespace-nowrap">
            <span :class="token.is_active ? 'text-green-600' : 'text-red-600'">
              {{ token.is_active ? 'Active' : 'Revoked' }}
            </span>
          </td>
          <td class="px-6 py-4 whitespace-nowrap text-right">
            <button @click="handleRotate(token.id)" class="btn-sm mr-2">Rotate</button>
            <button @click="handleRevoke(token.id)" class="btn-sm-danger">Revoke</button>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
```

---

## State Management

### Three Types of State

**1. Server State (TanStack Vue Query)**
- Data from API (tokens, usage, limits, traces)
- Cached with automatic invalidation
- Background refetching
- Loading/error states

**2. Global UI State (Pinia)**
- Authentication status (logged in/out, user info)
- Theme (dark/light mode)
- Toast notifications
- Global modals

**3. Local Component State (ref, reactive)**
- Form inputs
- Modal open/close
- Local filters/sorting
- UI-only state (dropdowns, tooltips)

---

### TanStack Vue Query (Server State)

**Installation:**
```typescript
// src/main.ts
import { createApp } from 'vue'
import { VueQueryPlugin } from '@tanstack/vue-query'
import App from './App.vue'

const app = createApp(App)

app.use(VueQueryPlugin, {
  queryClientConfig: {
    defaultOptions: {
      queries: {
        staleTime: 5 * 60 * 1000, // 5 minutes
        cacheTime: 10 * 60 * 1000, // 10 minutes
        refetchOnWindowFocus: false,
      },
    },
  },
})

app.mount('#app')
```

**Usage:**

```vue
<script setup lang="ts">
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useApi } from '@/composables/useApi'

const api = useApi()
const queryClient = useQueryClient()

// Query (fetch data)
const { data, isLoading, error } = useQuery({
  queryKey: ['tokens'],
  queryFn: () => api.getTokens(),
})

// Mutation (modify data)
const createMutation = useMutation({
  mutationFn: api.createToken,
  onSuccess: () => {
    // Invalidate cache to refetch
    queryClient.invalidateQueries({ queryKey: ['tokens'] })
  },
})

function createToken(data: CreateTokenRequest) {
  createMutation.mutate(data)
}
</script>
```

**Benefits:**
- Automatic caching (no manual cache management)
- Background refetching (data stays fresh)
- Loading/error states (no manual state tracking)
- Request deduplication (multiple components, single request)

---

### Pinia (Global UI State)

**Store Structure:**

```typescript
// src/stores/auth.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAuthStore = defineStore('auth', () => {
  // State
  const token = ref<string | null>(localStorage.getItem('auth_token'))
  const user = ref<User | null>(null)

  // Getters
  const isAuthenticated = computed(() => !!token.value)

  // Actions
  function login(newToken: string) {
    token.value = newToken
    localStorage.setItem('auth_token', newToken)
  }

  function logout() {
    token.value = null
    user.value = null
    localStorage.removeItem('auth_token')
  }

  function getAuthHeader(): string | null {
    return token.value ? `Bearer ${token.value}` : null
  }

  return {
    token,
    user,
    isAuthenticated,
    login,
    logout,
    getAuthHeader,
  }
})
```

**Usage:**

```vue
<script setup lang="ts">
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>

<template>
  <div>
    <p v-if="authStore.isAuthenticated">Welcome!</p>
    <button @click="handleLogout">Logout</button>
  </div>
</template>
```

---

### Local Component State

**Use `ref` for primitives:**
```typescript
const count = ref(0)
const name = ref('')
const isOpen = ref(false)
```

**Use `reactive` for objects (rare, prefer `ref`):**
```typescript
const form = reactive({
  username: '',
  password: '',
})
```

**Prefer `ref` over `reactive`:**
- Better TypeScript inference
- Can reassign entire object (`form.value = newForm`)
- Consistent API (always use `.value`)

---

## Routing Strategy

### Vue Router Configuration

**Route Definition:**

```typescript
// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/LoginView.vue'),
      meta: { requiresAuth: false },
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      component: () => import('@/views/DashboardView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/tokens',
      name: 'tokens',
      component: () => import('@/views/TokensView.vue'),
      meta: { requiresAuth: true },
    },
    // ... more routes
    {
      path: '/',
      redirect: '/dashboard',
    },
  ],
})

// Global navigation guard
router.beforeEach((to, from, next) => {
  const authStore = useAuthStore()

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    next('/login')
  } else if (to.path === '/login' && authStore.isAuthenticated) {
    next('/dashboard')
  } else {
    next()
  }
})

export default router
```

**Route Guards:**
- `beforeEach` - Global guard (authentication check)
- `meta.requiresAuth` - Route-level metadata
- Redirect unauthenticated users to `/login`
- Redirect authenticated users from `/login` to `/dashboard`

**Lazy Loading:**
- All views lazy-loaded (`() => import(...)`)
- Reduces initial bundle size
- Vite creates separate chunks per route

---

## Build Pipeline

### Vite Configuration

```typescript
// vite.config.ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
  plugins: [vue()],

  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },

  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:3001',
        changeOrigin: true,
      },
    },
  },

  build: {
    target: 'es2020',
    outDir: 'dist',
    sourcemap: 'hidden',
    rollupOptions: {
      output: {
        manualChunks: {
          'vue-vendor': ['vue', 'vue-router', 'pinia'],
          'query-vendor': ['@tanstack/vue-query'],
          'ui-vendor': ['radix-vue'],
        },
      },
    },
  },
})
```

**Key Features:**
- `@` alias points to `src/` (e.g., `import Foo from '@/components/Foo.vue'`)
- Proxy `/api` requests to backend (avoids CORS in dev)
- Manual chunks for better caching (vendor bundles separate from app code)
- Hidden source maps (generated but not served)

---

### Build Optimization

**Code Splitting:**
- Route-level splitting (each view = separate chunk)
- Vendor splitting (Vue, TanStack Query, Radix Vue = separate chunks)
- Automatic dynamic import splitting

**Tree Shaking:**
- Composition API functions tree-shaken automatically
- Unused components excluded from bundle
- Unused Tailwind classes purged in production

**Asset Optimization:**
- Images optimized (if using vite-plugin-imagemin)
- CSS minified (cssnano via PostCSS)
- JavaScript minified (terser via Rollup)

---

## Type Safety

### TypeScript Configuration

```json
// tsconfig.json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "moduleResolution": "bundler",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "skipLibCheck": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}
```

**Strict Mode Features:**
- `strict: true` - All strict checks enabled
- `noUnusedLocals` - Unused variables error
- `noUnusedParameters` - Unused function parameters error
- `noFallthroughCasesInSwitch` - Switch cases must have break/return

---

### Type Definitions

**Props and Emits:**

```typescript
// Props
interface Props {
  tokenId: number
  name?: string
}
const props = defineProps<Props>()

// Emits
interface Emits {
  created: [token: TokenMetadata]
  updated: [tokenId: number, changes: Partial<TokenMetadata>]
}
const emit = defineEmits<Emits>()
```

**API Types:**

```typescript
// src/composables/useApi.ts
export interface TokenMetadata {
  id: number
  user_id: string
  project_id?: string
  name?: string
  created_at: number
  last_used_at?: number
  is_active: boolean
}

export interface CreateTokenRequest {
  user_id: string
  project_id?: string
  description?: string
}

export interface CreateTokenResponse {
  id: number
  token: string
  user_id: string
  project_id?: string
  description?: string
  created_at: number
}
```

**Type Guards:**

```typescript
function isTokenMetadata(obj: unknown): obj is TokenMetadata {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    'id' in obj &&
    'user_id' in obj &&
    'is_active' in obj
  )
}
```

---

## Performance Optimization

### Virtual Scrolling (Future)

For long lists (>1000 items), use virtual scrolling:

```vue
<script setup lang="ts">
import { useVirtualizer } from '@tanstack/vue-virtual'
import { ref } from 'vue'

const parentRef = ref<HTMLElement | null>(null)
const traces = ref<TraceRecord[]>([/* 10000 items */])

const virtualizer = useVirtualizer({
  count: traces.value.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 50, // 50px per row
})
</script>

<template>
  <div ref="parentRef" class="h-[600px] overflow-auto">
    <div :style="{ height: `${virtualizer.getTotalSize()}px` }">
      <div
        v-for="item in virtualizer.getVirtualItems()"
        :key="item.index"
        :style="{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: `${item.size}px`,
          transform: `translateY(${item.start}px)`,
        }"
      >
        {{ traces[item.index].request_id }}
      </div>
    </div>
  </div>
</template>
```

---

### Debounced Inputs

For search/filter inputs, debounce API calls:

```vue
<script setup lang="ts">
import { ref, watch } from 'vue'
import { useDebouncedRef } from '@/composables/useDebouncedRef'

const searchInput = ref('')
const debouncedSearch = useDebouncedRef(searchInput, 300) // 300ms delay

watch(debouncedSearch, (newValue) => {
  // API call here (only after 300ms of no typing)
  fetchResults(newValue)
})
</script>
```

---

### Lazy Component Loading

For heavy components (charts, editors), lazy load:

```vue
<script setup lang="ts">
import { defineAsyncComponent } from 'vue'

const HeavyChart = defineAsyncComponent(() =>
  import('@/components/HeavyChart.vue')
)
</script>

<template>
  <Suspense>
    <HeavyChart v-if="showChart" />
    <template #fallback>
      <div>Loading chart...</div>
    </template>
  </Suspense>
</template>
```

---

## Architecture Decision Log

### ADL-1: Composition API over Options API

**Decision:** Use Composition API exclusively (no Options API).

**Context:** Vue 3 supports both APIs, but Composition API offers better TypeScript support and code organization.

**Consequences:**
- Pros: Better type inference, smaller bundles, easier to extract composables
- Cons: Steeper learning curve for Vue 2 developers

---

### ADL-2: TanStack Query for Server State

**Decision:** Use TanStack Vue Query for all server state (API data).

**Context:** Pinia can manage server state, but lacks caching, background refetching, and loading states.

**Consequences:**
- Pros: Automatic caching, request deduplication, background refetching
- Cons: Additional dependency, new API to learn

---

### ADL-3: Radix Vue for UI Primitives

**Decision:** Use Radix Vue for accessible UI components (modals, dropdowns, tabs).

**Context:** Need WCAG 2.1 Level AA compliance without custom ARIA implementation.

**Consequences:**
- Pros: Accessible by default, unstyled (Tailwind-compatible), well-maintained
- Cons: Additional dependency, requires Tailwind integration

---

**End of Architecture Documentation**
