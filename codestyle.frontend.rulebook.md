---
status: active
version: 1.0
authority: iron_runtime
applicability: iron_dashboard_frontend
precedence: project_specific
---

# Frontend Codestyle

### Vocabulary
- **Rulebook:** This document, which provides a set of guidelines for formatting and structuring frontend code in the Iron Dashboard project.
- **Rule:** An individual guideline within this rulebook, designed to ensure consistency, readability, and maintainability.
- **SFC:** Single File Component — the `.vue` file format combining `<script>`, `<template>`, and optionally `<style>` in one file.
- **Composable:** A function that encapsulates and reuses stateful logic using the Vue Composition API.
- **Reka UI:** The accessible, unstyled Vue component library this project uses for all interactive UI primitives. Previously named *Radix Vue*; any earlier references to "Radix Vue" refer to the same library.
- **UI Primitive:** A low-level, unstyled component from Reka UI, wrapped with project-specific Tailwind styling.

### Governing Principles
This rulebook provides a set of guidelines for formatting and structuring frontend code to ensure consistency, readability, and maintainability across the Iron Dashboard project. The foundational principle is that all rules apply universally to all frontend code, regardless of its location.

This project follows **Prettier** for automatic formatting and **ESLint** (flat config) for linting. Any aspect of formatting not covered in this document defaults to the Prettier and ESLint configuration. TypeScript strict mode is mandatory.

### Structure
Most rules in this document follow a consistent structure for clarity:
- The rule statement itself, stated as the opening paragraph of each section.
- **Rationale:** An explanation of why the rule exists and the benefits of following it.
- **Examples:** `Good` and `Bad` examples illustrating correct and incorrect application of the rule.

### Quick Reference Summary

**Formatting & Whitespace**
* [Prettier as the Single Formatter](#formatting--whitespace--prettier-as-the-single-formatter)

**Component Structure**
* [Mandatory Script Setup with TypeScript](#component-structure--mandatory-script-setup-with-typescript)
* [SFC Section Order](#component-structure--sfc-section-order)
* [Strict Prop Typing with defineProps](#component-structure--strict-prop-typing-with-defineprops)
* [Typed Emits with defineEmits](#component-structure--typed-emits-with-defineemits)
* [Loading, Error, and Empty State Pattern](#component-structure--loading-error-and-empty-state-pattern)

**Naming Conventions**
* [File Naming](#naming-conventions--file-naming)
* [Directory Naming](#naming-conventions--directory-naming)
* [Component Naming](#naming-conventions--component-naming)
* [Composable Naming](#naming-conventions--composable-naming)
* [Variable and Function Naming](#naming-conventions--variable-and-function-naming)
* [Type and Interface Naming](#naming-conventions--type-and-interface-naming)
* [Constant Naming](#naming-conventions--constant-naming)

**Imports & Module Organization**
* [Import Order](#imports--module-organization--import-order)
* [Path Aliases](#imports--module-organization--path-aliases)
* [Barrel Exports for UI Primitives](#imports--module-organization--barrel-exports-for-ui-primitives)

**State Management**
* [Pinia Composition API Stores](#state-management--pinia-composition-api-stores)
* [TanStack Vue Query for Server State](#state-management--tanstack-vue-query-for-server-state)
* [Mutation and Cache Invalidation Pattern](#state-management--mutation-and-cache-invalidation-pattern)

**API Layer**
* [Centralized API Composable](#api-layer--centralized-api-composable)
* [Type Definitions at the API Boundary](#api-layer--type-definitions-at-the-api-boundary)
* [Error Propagation](#api-layer--error-propagation)

**Styling**
* [Tailwind Utility-First](#styling--tailwind-utility-first)
* [CSS Variables for Design Tokens](#styling--css-variables-for-design-tokens)
* [Dynamic Class Composition with cn()](#styling--dynamic-class-composition-with-cn)
* [CVA for Component Variants](#styling--cva-for-component-variants)

**UI Primitives**
* [Reka UI Base with Tailwind Wrapper](#ui-primitives--reka-ui-base-with-tailwind-wrapper)
* [Icon Components](#ui-primitives--icon-components)

**Routing & Navigation Guards**
* [Lazy-Loaded Route Components](#routing--navigation-guards--lazy-loaded-route-components)
* [Route Meta for Auth and Role Gating](#routing--navigation-guards--route-meta-for-auth-and-role-gating)
* [Centralized Navigation Guard](#routing--navigation-guards--centralized-navigation-guard)

**Testing**
* [Vitest as the Single Test Runner](#testing--vitest-as-the-single-test-runner)
* [Colocated Test Files](#testing--colocated-test-files)
* [Test Structure and Conventions](#testing--test-structure-and-conventions)

**Comments & Task Markers**
* [Comment Content](#comments--task-markers--comment-content)
* [Task Marker Compatibility](#comments--task-markers--task-marker-compatibility)

**Security**
* [v-html Prohibition and XSS Prevention](#security--v-html-prohibition-and-xss-prevention)
* [Global Error Boundary](#security--global-error-boundary)

**Accessibility**
* [Custom Component Requirements](#accessibility--custom-component-requirements)

**Project Structure**
* [Canonical Directory Layout](#project-structure--canonical-directory-layout)

---

### Formatting & Whitespace : Prettier as the Single Formatter

Prettier is the **sole** authority for code formatting. All formatting decisions — line width, brace placement, trailing commas, quote style, indentation — are delegated to Prettier via the `.prettierrc` configuration file. It is **strictly forbidden** to override Prettier's formatting decisions with manual formatting, ESLint formatting rules, or editor-specific settings.

The `.prettierrc` at the project root enforces:

```json
{
  "semi": false,
  "singleQuote": true,
  "trailingComma": "es5",
  "printWidth": 100,
  "tabWidth": 2
}
```

Key settings:
- **Semicolons:** Omitted. Statements **must not** end with a semicolon.
- **Quotes:** Single quotes for all string literals. Double quotes are only permitted in HTML template attributes.
- **Indentation:** 2 spaces. Tabs are **strictly forbidden**.
- **Line width:** 100 characters.
- **Trailing commas:** Required in multi-line constructs (ES5-compatible positions).

**Rationale:** A single, deterministic formatter eliminates all formatting debates and guarantees that every file in the project is formatted identically regardless of the developer's editor.

---

### Component Structure : Mandatory Script Setup with TypeScript

All Vue components **must** use the `<script setup lang="ts">` syntax. It is **strictly forbidden** to use the Options API, the non-setup Composition API (`setup()` function), or plain JavaScript in `.vue` files.

**Rationale:**
- **Conciseness:** `<script setup>` eliminates boilerplate (no `export default`, no explicit `return`).
- **Type Safety:** `lang="ts"` enforces TypeScript throughout.
- **Performance:** `<script setup>` compiles to more efficient code.

> Bad (Options API)

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 }
  },
  methods: {
    increment() { this.count++ }
  }
}
</script>
```

> Bad (Non-setup Composition API)

```vue
<script lang="ts">
import { ref, defineComponent } from 'vue'

export default defineComponent({
  setup() {
    const count = ref(0)
    return { count }
  }
})
</script>
```

> Good

```vue
<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)
function increment() {
  count.value++
}
</script>
```

---

### Component Structure : SFC Section Order

Single File Components **must** order their sections as follows:

1. `<script setup lang="ts">` — **always first**
2. `<template>` — **always second**
3. `<style>` — **optional, always last** (prefer Tailwind utilities over `<style>` blocks)

It is **strictly forbidden** to place `<template>` before `<script>` or to place `<style>` between `<script>` and `<template>`.

**Rationale:** Consistent ordering allows developers to find logic, markup, and styles in predictable locations. Script-first ordering ensures that props, state, and computed values are defined before the template that references them.

> Bad

```vue
<template>
  <div>{{ message }}</div>
</template>

<script setup lang="ts">
const message = 'hello'
</script>
```

> Good

```vue
<script setup lang="ts">
const message = 'hello'
</script>

<template>
  <div>{{ message }}</div>
</template>
```

---

### Component Structure : Strict Prop Typing with defineProps

All component props **must** be defined using `defineProps<T>()` with a TypeScript interface. It is **strictly forbidden** to use the runtime declaration syntax (`defineProps({ ... })`) or to leave props untyped.

When default values are needed, `withDefaults(defineProps<T>(), { ... })` **must** be used.

**Rationale:** Type-only prop declarations provide compile-time safety and eliminate the need for manual runtime validation.

> Bad (Runtime declaration)

```vue
<script setup lang="ts">
const props = defineProps({
  title: { type: String, required: true },
  isLoading: { type: Boolean, default: false },
})
</script>
```

> Good (Type-only declaration)

```vue
<script setup lang="ts">
defineProps<{
  title: string
  isLoading?: boolean
}>()
</script>
```

> Good (With defaults)

```vue
<script setup lang="ts">
const props = withDefaults(defineProps<{
  variant?: 'default' | 'destructive'
  size?: 'sm' | 'md' | 'lg'
}>(), {
  variant: 'default',
  size: 'md',
})
</script>
```

---

### Component Structure : Typed Emits with defineEmits

All component emits **must** be defined using `defineEmits<T>()` with a TypeScript interface specifying event names and their payload types. It is **strictly forbidden** to use the runtime array syntax (`defineEmits(['click'])`) or to leave emits untyped.

**Rationale:** Typed emits provide compile-time validation that parent components pass the correct handlers and that child components emit the correct payloads.

> Bad

```vue
<script setup lang="ts">
const emit = defineEmits(['update:modelValue', 'confirm'])
</script>
```

> Good

```vue
<script setup lang="ts">
const emit = defineEmits<{
  'update:modelValue': [value: string]
  'confirm': []
}>()
</script>
```

---

### Component Structure : Loading, Error, and Empty State Pattern

All data-driven components **must** handle three states explicitly in the template: loading, error, and empty. These states **must** appear in this exact order using `v-if` / `v-else-if` / `v-else` chains, before the main content.

**Rationale:** Guarantees that every data view has consistent, user-visible feedback for all possible data states, preventing blank screens or confusing behavior.

> Good

```vue
<template>
  <!-- Loading -->
  <div v-if="isLoading" class="p-4">
    <p class="text-muted-foreground">Loading...</p>
  </div>

  <!-- Error -->
  <div v-else-if="error" class="p-4">
    <p class="text-destructive">{{ error.message }}</p>
    <Button v-if="onRetry" variant="outline" class="mt-4" @click="onRetry">Retry</Button>
  </div>

  <!-- Empty -->
  <div v-else-if="isEmpty" class="p-4 text-center">
    <p class="text-muted-foreground">No data</p>
  </div>

  <!-- Content -->
  <template v-else>
    <!-- Main content here -->
  </template>
</template>
```

---

### Naming Conventions : File Naming

File naming **must** follow these rules based on file type:

| File Type | Convention | Example |
|-----------|-----------|---------|
| Vue components | PascalCase | `MainLayout.vue`, `StatCard.vue` |
| Composables | camelCase, prefixed with `use` | `useApi.ts`, `useConfirm.ts` |
| Stores | lowercase | `auth.ts` |
| Utilities / helpers | lowercase | `utils.ts`\*, `formatters.ts`, `providers.ts` |
| Test files | Same as source, suffixed with `.test` | `auth.test.ts`, `formatters.test.ts` |
| Config files | Standard tooling names | `vite.config.ts`, `tailwind.config.js` |

It is **strictly forbidden** to use kebab-case or UPPER_CASE for source files.

> **Cross-rulebook note:** Vue file naming conventions (PascalCase for `.vue` files, kebab-case for directories) differ from the project-wide `lowercase_snake_case` standard defined in CLAUDE.md Rule 3. This is a tooling/ecosystem exception per the "when possible" qualifier in that rule. The Vue/Vite ecosystem universally expects PascalCase component files and kebab-case directories.

**Rationale:** PascalCase for components mirrors the PascalCase tag names used in templates. camelCase for composables matches the function naming convention. Lowercase for utilities avoids ambiguity.

---

### Naming Conventions : Directory Naming

Directories **must** use lowercase for single-word names and kebab-case for multi-word names.

| Type | Convention | Example |
|------|-----------|---------|
| Single-word | lowercase | `components/`, `stores/`, `composables/`, `lib/` |
| Multi-word | kebab-case | `dropdown-menu/`, `scroll-area/` |

It is **strictly forbidden** to use PascalCase, camelCase, or snake_case for directory names.

**Rationale:** Lowercase and kebab-case are the universal conventions in the JavaScript ecosystem for directory naming and prevent case-sensitivity issues across operating systems.

---

### Naming Conventions : Component Naming

Vue component names **must** follow PascalCase. Additionally, the following naming patterns are **mandatory**:

| Pattern | Prefix/Suffix | Example |
|---------|--------------|---------|
| Page-level views | Suffix `View` | `DashboardView.vue`, `UsersView.vue` |
| Icon components | Prefix `Icon` | `IconHome.vue`, `IconChevronDown.vue` |
| Layout components | Suffix `Layout` | `MainLayout.vue`, `PageLayout.vue` |
| UI primitives | Bare noun | `Button.vue`, `Card.vue`, `Dialog.vue` |
| Domain components | Descriptive PascalCase | `ProviderBadge.vue`, `AvatarInitial.vue` |

**Rationale:** Consistent prefixes and suffixes allow instant identification of a component's role in the architecture.

---

### Naming Conventions : Composable Naming

All composable functions **must** be prefixed with `use` and written in camelCase. The file containing a composable **must** be named identically to the primary exported function.

**Rationale:** The `use` prefix is the universal Vue convention for composables. It signals that the function uses reactive state and must be called within a setup context.

> Bad

```typescript
// composables/api.ts
export function api() { /* ... */ }
```

> Good

```typescript
// composables/useApi.ts
export function useApi() { /* ... */ }
```

---

### Naming Conventions : Variable and Function Naming

Variables and functions **must** use camelCase. The following conventions are **mandatory**:

| Category | Convention | Example |
|----------|-----------|---------|
| Boolean refs | Prefix with `is`, `has`, `show` | `isLoading`, `hasError`, `showModal` |
| Event handlers | Prefix with `handle` or `on` | `handleLogout`, `onClickOutside` |
| Computed properties | Descriptive noun/adjective | `totalSpend`, `isAuthenticated` |
| Template refs | Suffix with `Ref` | `userMenuRef`, `inputRef` |

**Rationale:** Consistent naming conventions communicate intent immediately. Boolean prefixes make conditionals read naturally (`v-if="isLoading"`). Handler prefixes distinguish event callbacks from other functions.

> Bad

```typescript
const loading = ref(false)
const modal = ref(false)
const menu = ref<HTMLElement | null>(null)
function logout() { /* ... */ }
```

> Good

```typescript
const isLoading = ref(false)
const showModal = ref(false)
const menuRef = ref<HTMLElement | null>(null)
function handleLogout() { /* ... */ }
```

---

### Naming Conventions : Type and Interface Naming

All TypeScript types and interfaces **must** use PascalCase. Request and response types **must** use descriptive suffixes:

| Category | Pattern | Example |
|----------|---------|---------|
| Domain entities | Bare noun | `User`, `Agent`, `ProviderKey` |
| API request payloads | Suffix `Request` | `CreateUserRequest`, `UpdateProviderKeyRequest` |
| API response payloads | Suffix `Response` | `CreateTokenResponse` |
| Union types | PascalCase | `ProviderType` |
| Component prop interfaces | `Props` (local) | `interface Props extends PrimitiveProps { ... }` |

Interfaces and types should not be prefixed with `I` (e.g., `IUser`) or `T` (e.g., `TUser`). Bare PascalCase names are preferred.

**Rationale:** PascalCase types are the TypeScript standard. Descriptive suffixes distinguish entities from their API transport shapes.

---

### Naming Conventions : Constant Naming

Module-level constants **must** use `UPPER_SNAKE_CASE` when they represent static lookup tables, configuration values, or environment-derived values. Enum-like constant objects follow the same pattern.

**Rationale:** `UPPER_SNAKE_CASE` visually distinguishes immutable module-level data from reactive state and function-scoped variables.

> Good

```typescript
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3001'

const PROVIDER_LABELS: Record<string, string> = {
  openai: 'OpenAI',
  anthropic: 'Anthropic',
  gemini: 'Gemini',
  xai: 'xAI',
}
```

---

### Imports & Module Organization : Import Order

Imports **must** be organized in the following order, with a blank line separating each group:

1. **Vue core and external libraries** — `vue`, `vue-router`, `@tanstack/vue-query`, `@vueuse/core`, `vue-sonner`, `pinia`, etc.
2. **Stores, composables, and utilities** — `@/stores/*`, `@/composables/*`, `@/lib/*`
3. **Components** — `@/components/ui/*`, `@/components/*` (UI primitives first, then project components)
4. **Types** (type-only imports)
5. **Assets and styles**

**Rationale:** Consistent import ordering makes dependencies scannable at a glance and prevents merge conflicts in import blocks.

> Good

```typescript
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { refDebounced } from '@vueuse/core'
import { toast } from 'vue-sonner'

import { useApi } from '@/composables/useApi'
import { useAuthStore } from '@/stores/auth'
import { useConfirm } from '@/composables/useConfirm'
import { formatCostUsd, formatTimestamp } from '@/lib/formatters'

import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import PageLayout from '@/components/PageLayout.vue'
import DataTable from '@/components/DataTable.vue'
import IconPlus from '@/components/icons/IconPlus.vue'

import type { User, CreateUserRequest } from '@/composables/useApi'
```

---

### Imports & Module Organization : Path Aliases

All imports from within the `src/` directory **must** use the `@/` path alias. Relative imports (`../`, `./`) are **only permitted** within the same immediate directory (e.g., a barrel `index.ts` importing from `./Button.vue` in the same folder).

The `tsconfig.json` alias:

```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}
```

**Rationale:** Absolute `@/` imports are immune to directory restructuring and are easier to read than deeply nested relative paths.

> Bad

```typescript
import { useApi } from '../../composables/useApi'
import { Button } from '../../components/ui/button'
```

> Good

```typescript
import { useApi } from '@/composables/useApi'
import { Button } from '@/components/ui/button'
```

> Permitted (same-directory relative import in a barrel file)

```typescript
// components/ui/button/index.ts
export { default as Button } from './Button.vue'
```

---

### Imports & Module Organization : Barrel Exports for UI Primitives

Every UI primitive directory under `components/ui/` **must** contain an `index.ts` barrel file that re-exports all public components, variant configurations, and types from that directory. Multi-part primitives (e.g., Dialog with DialogContent, DialogHeader, DialogTitle, etc.) **must** export all parts from the single barrel. Consumers **must** import from the barrel, never from individual `.vue` files within the primitive directory.

**Rationale:** Barrel exports provide a stable public API for UI primitives, allowing internal file restructuring without breaking consumers. Clean destructured imports mirror the component's compositional structure.

> Bad (Direct import from internal file)

```typescript
import Button from '@/components/ui/button/Button.vue'
```

> Good (Import from barrel)

```typescript
import { Button } from '@/components/ui/button'
```

> Good (Barrel file structure)

```typescript
// components/ui/button/index.ts
import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export { default as Button } from './Button.vue'

export const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-xs font-medium ...',
  {
    variants: {
      variant: {
        default: 'text-foreground hover:bg-primary/10',
        destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
        outline: 'bg-background hover:bg-secondary hover:text-secondary-foreground',
        ghost: 'hover:bg-secondary hover:text-secondary-foreground',
      },
      size: {
        default: 'h-9 px-4 py-2',
        sm: 'h-8 rounded-md px-3 text-xs',
        lg: 'h-10 rounded-md px-8',
        icon: 'h-9 w-9',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

export type ButtonVariants = VariantProps<typeof buttonVariants>
```

> Good (Multi-part primitive barrel)

```typescript
// components/ui/dialog/index.ts
export { default as Dialog } from './Dialog.vue'
export { default as DialogContent } from './DialogContent.vue'
export { default as DialogDescription } from './DialogDescription.vue'
export { default as DialogFooter } from './DialogFooter.vue'
export { default as DialogHeader } from './DialogHeader.vue'
export { default as DialogTitle } from './DialogTitle.vue'
```

---

### State Management : Pinia Composition API Stores

All Pinia stores **must** use the Composition API syntax (function-based `defineStore`). It is **strictly forbidden** to use the Options API syntax for stores. Stores **must** follow this internal structure:

1. **Refs** for state
2. **Computed** for derived state
3. **Functions** for mutations and actions
4. **Initialization** calls (if needed)
5. **Return** statement exposing the public API

**Rationale:** Composition API stores provide full TypeScript inference without additional type annotations and are consistent with the `<script setup>` component pattern used throughout the project.

> Bad (Options API store)

```typescript
export const useAuthStore = defineStore('auth', {
  state: () => ({ token: null as string | null }),
  getters: {
    isAuthenticated: (state) => !!state.token,
  },
  actions: {
    login(credentials: LoginCredentials) { /* ... */ },
  },
})
```

> Good (Composition API store)

```typescript
export const useAuthStore = defineStore('auth', () => {
  // State
  const accessToken = ref<string | null>(null)
  const role = ref<string | null>(null)

  // Derived state
  const isAuthenticated = computed(() => !!accessToken.value)
  const isAdmin = computed(() => role.value === 'admin')

  // Actions
  async function login(credentials: LoginCredentials) {
    const response = await fetch(/* ... */)
    const tokens: AuthTokens = await response.json()
    accessToken.value = tokens.user_token
  }

  function clearTokens() {
    accessToken.value = null
    role.value = null
  }

  // Initialize
  loadTokens()

  // Public API
  return {
    accessToken,
    role,
    isAuthenticated,
    isAdmin,
    login,
    clearTokens,
  }
})
```

---

### State Management : TanStack Vue Query for Server State

All server-side data fetching **must** use TanStack Vue Query (`useQuery`). It is **strictly forbidden** to use raw `fetch` or `axios` calls in components for data that should be cached, refetched, or shared across components. Raw `fetch` is **only permitted** inside the centralized API composable and inside Pinia store actions that manage authentication (login, refresh, logout).

**Query keys must** be arrays that include all reactive dependencies used in the `queryFn`, so that Vue Query refetches automatically when inputs change.

**Rationale:** Vue Query provides automatic caching, deduplication, background refetching, and loading/error state management — eliminating an entire category of bugs related to manual data fetching.

> Bad (Raw fetch in a component)

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'

const users = ref([])
const isLoading = ref(true)

onMounted(async () => {
  const res = await fetch('/api/v1/users')
  users.value = await res.json()
  isLoading.value = false
})
</script>
```

> Good (Vue Query)

```vue
<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { useApi } from '@/composables/useApi'

const api = useApi()

const { data: users, isLoading, error } = useQuery({
  queryKey: ['users', page, pageSize, searchDebounced],
  queryFn: () => api.getUsers({
    page: page.value,
    page_size: pageSize.value,
    search: searchDebounced.value || undefined,
  }),
})
</script>
```

---

### State Management : Mutation and Cache Invalidation Pattern

All write operations (create, update, delete) **must** use `useMutation` from TanStack Vue Query. After a successful mutation, the relevant query caches **must** be invalidated using `queryClient.invalidateQueries()`. User feedback **must** be provided via `vue-sonner` toast notifications in the `onSuccess` and `onError` callbacks.

**Rationale:** Centralizing mutations through `useMutation` provides consistent loading states, error handling, and cache synchronization. Toast notifications give immediate user feedback.

> Good

```typescript
const queryClient = useQueryClient()

const createMutation = useMutation({
  mutationFn: (data: CreateUserRequest) => api.createUser(data),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['users'] })
    toast.success('User created successfully')
    showCreateModal.value = false
  },
  onError: (err) => {
    toast.error(err instanceof Error ? err.message : 'Failed to create user')
  },
})
```

---

### API Layer : Centralized API Composable

All backend API calls **must** be defined in the `useApi()` composable (`composables/useApi.ts`). It is **strictly forbidden** for components, stores, or other composables to construct HTTP requests directly. The `useApi()` composable is the single source of truth for:

1. Base URL configuration
2. Authorization header injection
3. 401 response handling and token refresh
4. Request/response type definitions
5. Error response parsing

**Rationale:** A single API layer prevents duplication of auth logic, ensures consistent error handling, and provides a clear inventory of all backend endpoints.

---

### API Layer : Type Definitions at the API Boundary

All request and response types **must** be defined at the top of the API composable file, before the composable function itself. Types that are consumed by components **must** be exported using `export type` or `export interface` so they are importable alongside the composable.

**Rationale:** Colocating API types with the API layer ensures they stay synchronized with the actual endpoint contracts. Placing them before the composable function makes them importable without circular dependencies.

> Good

```typescript
// composables/useApi.ts

// Types defined before the composable
export interface User {
  id: string
  username: string
  email?: string
  role: string
  is_active: boolean
  created_at: number
}

export interface CreateUserRequest {
  username: string
  password: string
  email: string
  role?: string
}

// Composable defined after types
export function useApi() {
  // ...
}
```

---

### API Layer : Error Propagation

API errors **must** propagate as `Error` instances with human-readable messages. The API composable **must** parse error response bodies and extract server-provided error messages. Components **must** handle errors through the Vue Query `error` ref or `onError` callback — never with try/catch around query functions.

**Rationale:** Consistent error objects allow the UI layer to display meaningful messages without knowledge of HTTP response formats.

> Good (In the API layer)

```typescript
async function fetchApi<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${url}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      Authorization: authStore.getAuthHeader() ?? '',
      ...options?.headers,
    },
  })

  if (!response.ok) {
    let msg: string
    try {
      const body = await response.json()
      msg = body?.error?.message ?? body?.error ?? body?.message ?? 'Request failed'
    } catch {
      msg = 'Request failed'
    }
    throw new Error(msg)
  }

  return response.json()
}
```

> Good (In the component)

```vue
<template>
  <div v-if="error" class="p-4">
    <p class="text-destructive">{{ error.message }}</p>
  </div>
</template>
```

---

### Styling : Tailwind Utility-First

All styling **must** use Tailwind CSS utility classes applied directly in templates. It is **strictly forbidden** to create `<style>` blocks in `.vue` files for styling that can be achieved with Tailwind utilities. The **only** permitted use of `<style>` is for third-party library overrides or CSS features that Tailwind cannot express (e.g., complex animations, pseudo-element content).

**Rationale:** Tailwind's utility-first approach colocates styling with markup, eliminates CSS naming debates, and produces smaller CSS bundles through automatic purging.

> Bad

```vue
<template>
  <div class="card">
    <h1 class="card-title">Dashboard</h1>
  </div>
</template>

<style scoped>
.card {
  padding: 1rem
  background: white
  border-radius: 0.5rem
}
.card-title {
  font-size: 1.25rem
  font-weight: 600
}
</style>
```

> Good

```vue
<template>
  <div class="p-4 bg-card rounded-lg">
    <h1 class="text-xl font-semibold">Dashboard</h1>
  </div>
</template>
```

---

### Styling : CSS Variables for Design Tokens

All color values, border radii, and other themeable properties **must** be defined as CSS custom properties (variables) in `style.css` and referenced through the Tailwind theme configuration. It is **strictly forbidden** to use hardcoded hex, RGB, or HSL values directly in Tailwind classes or inline styles for any property that participates in theming.

Color variables **must** use the HSL format without the `hsl()` wrapper, allowing Tailwind to apply opacity modifiers (e.g., `bg-primary/80`).

**Rationale:** CSS variables enable theme switching without class duplication and provide a single source of truth for the design system.

> Bad (Hardcoded color)

```vue
<div class="bg-[#5c5fea] text-white">Accent section</div>
```

> Good (Design token reference)

```vue
<div class="bg-accent text-accent-foreground">Accent section</div>
```

---

### Styling : Dynamic Class Composition with cn()

When a component needs to merge external class props with internal classes, or conditionally apply classes, the `cn()` utility **must** be used. `cn()` wraps `clsx` and `tailwind-merge` to resolve Tailwind class conflicts correctly. It is **strictly forbidden** to use manual string concatenation or template literals for class merging when Tailwind conflicts are possible.

**Rationale:** `tailwind-merge` resolves specificity conflicts (e.g., `p-4` vs `p-2`) that manual concatenation would not, ensuring the last-specified class wins.

> Bad (Manual concatenation)

```vue
<div :class="'p-4 bg-card ' + (isActive ? 'bg-primary' : '')">
```

> Good

```vue
<div :class="cn('p-4 bg-card', { 'bg-primary': isActive })">
```

> Good (Accepting external class prop)

```vue
<script setup lang="ts">
import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: string
}>()
</script>

<template>
  <div :class="cn('flex flex-col h-full', props.class)">
    <slot />
  </div>
</template>
```

---

### Styling : CVA for Component Variants

When a component has multiple visual variants (e.g., different sizes, colors, or styles), the `class-variance-authority` (CVA) library **must** be used to define and manage variant classes. CVA configurations **must** be exported from the component's barrel `index.ts` file alongside the component itself.

**Rationale:** CVA provides type-safe variant management with auto-complete support, eliminating error-prone ternary chains for class selection.

> Good

```typescript
// components/ui/button/index.ts
import { cva } from 'class-variance-authority'
import type { VariantProps } from 'class-variance-authority'

export const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 rounded-md text-xs font-medium transition-colors',
  {
    variants: {
      variant: {
        default: 'text-foreground hover:bg-primary/10',
        destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
      },
      size: {
        default: 'h-9 px-4 py-2',
        sm: 'h-8 rounded-md px-3',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

export type ButtonVariants = VariantProps<typeof buttonVariants>
```

---

### UI Primitives : Reka UI Base with Tailwind Wrapper

All interactive UI primitives (buttons, dialogs, dropdowns, selects, etc.) **must** be built on top of Reka UI unstyled components. Project-specific styling **must** be applied in a thin wrapper component that uses Tailwind utilities.

It is **strictly forbidden** to:
1. Build interactive primitives from scratch when a Reka UI component exists
2. Import Reka UI components directly in views or page components — always use the project's wrapper

**Rationale:** Reka UI provides accessible, keyboard-navigable, WAI-ARIA compliant primitives. Wrapping them with Tailwind styling maintains accessibility guarantees while applying project-specific design tokens.

> Good (Wrapper component)

```vue
<!-- components/ui/button/Button.vue -->
<script setup lang="ts">
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import type { ButtonVariants } from '.'
import { Primitive } from 'reka-ui'
import { cn } from '@/lib/utils'
import { buttonVariants } from '.'

interface Props extends PrimitiveProps {
  variant?: ButtonVariants['variant']
  size?: ButtonVariants['size']
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  as: 'button',
  variant: 'default',
  size: 'default',
  class: '',
})
</script>

<template>
  <Primitive
    :as="as"
    :as-child="asChild"
    :class="cn(buttonVariants({ variant, size }), props.class)"
  >
    <slot />
  </Primitive>
</template>
```

---

### UI Primitives : Icon Components

All reusable icons **must** be implemented as individual Vue SFC components in the `components/icons/` directory, prefixed with `Icon`. Icons **must** render inline SVG elements and accept standard SVG attributes via prop passthrough.

It is **strictly forbidden** to:
1. Use icon fonts
2. Import SVG files as static assets (via `<img src="icon.svg">`)

For one-off decorative SVGs that are not reused across components, inline `<svg>` markup in the template is acceptable. However, if the same icon appears in more than one location, it **must** be extracted into an `Icon*` component.

**Rationale:** Vue SFC icons are tree-shakeable, allow Tailwind class styling (size, color), and provide a consistent API across all icons.

> Good

```vue
<!-- components/icons/IconHome.vue -->
<template>
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"
    stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
    <polyline points="9 22 9 12 15 12 15 22" />
  </svg>
</template>
```

```vue
<!-- Usage in a component -->
<IconHome class="w-4 h-4 mr-2 flex-shrink-0" />
```

---

### Routing & Navigation Guards : Lazy-Loaded Route Components

All route components except the login page **must** use dynamic imports (`() => import(...)`) for lazy loading. Only the login page — as the entry point for unauthenticated users — **may** be statically imported.

**Rationale:** Lazy loading splits each view into its own chunk, reducing the initial bundle size and improving first-load performance.

> Good

```typescript
const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: LoginView, // Static import — entry point
      meta: { requiresAuth: false },
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      component: () => import('../views/DashboardView.vue'), // Lazy-loaded
      meta: { requiresAuth: true },
    },
  ],
})
```

---

### Routing & Navigation Guards : Route Meta for Auth and Role Gating

Every route **must** declare `requiresAuth` in its `meta` object. Admin-only routes **must** additionally declare `requiresAdmin: true`. Page-level access control **must** be handled by the centralized navigation guard, not by individual view components. However, conditional UI rendering based on role (e.g., `v-if="authStore.isAdmin"` to show/hide a button) is permitted within views — this is presentation logic, not access control.

**Rationale:** Declarative route meta makes access requirements visible at the route definition level and centralizes enforcement in a single guard. Role-based UI visibility within an already-authorized page is a normal presentation concern.

> Good

```typescript
{
  path: '/providers',
  name: 'providers',
  component: () => import('../views/ProvidersView.vue'),
  meta: { requiresAuth: true, requiresAdmin: true },
}
```

---

### Routing & Navigation Guards : Centralized Navigation Guard

A single `router.beforeEach` guard **must** enforce all authentication and authorization logic. The guard **must** handle three cases:

1. **Unauthenticated user accessing a protected route** — redirect to `/login`
2. **Non-admin user accessing an admin route** — redirect to `/dashboard`
3. **Authenticated user accessing `/login`** — redirect to `/dashboard`

It is **strictly forbidden** to add additional `beforeEach`, `beforeResolve`, or per-route guards for authentication or role-based access control. Additional guards for non-auth concerns (e.g., analytics tracking, page transitions) are permitted.

**Rationale:** A single guard provides a complete, auditable access control policy in one location.

> Good

```typescript
router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore()
  const requiresAuth = to.meta.requiresAuth !== false

  if (requiresAuth && !authStore.isAuthenticated) {
    next('/login')
  } else if (to.meta.requiresAdmin && !authStore.isAdmin) {
    next('/dashboard')
  } else if (to.path === '/login' && authStore.isAuthenticated) {
    next('/dashboard')
  } else {
    next()
  }
})
```

---

### Testing : Vitest as the Single Test Runner

Vitest is the **sole** test framework for the frontend project. It is **strictly forbidden** to use Jest, Mocha, Jasmine, or any other test runner. The Vitest configuration uses `jsdom` as the DOM environment.

**Rationale:** Vitest is natively integrated with Vite, shares the same transformation pipeline, and supports Vue SFC testing without additional configuration.

---

### Testing : Colocated Test Files

Test files **must** be colocated alongside the source files they test, in the same directory, with the `.test.ts` suffix. This is a deliberate departure from the Rust codestyle rulebook's centralized `tests/` directory rule.

| Source File | Test File |
|-------------|-----------|
| `stores/auth.ts` | `stores/auth.test.ts` |
| `lib/formatters.ts` | `lib/formatters.test.ts` |
| `composables/useConfirm.ts` | `composables/useConfirm.test.ts` |

It is **strictly forbidden** to place test files in a top-level `tests/` or `__tests__/` directory.

**Rationale:** Colocation makes it immediately obvious whether a module has tests and reduces the cognitive overhead of navigating between source and test files. This is the standard convention in the Vue/Vite ecosystem.

---

### Testing : Test Structure and Conventions

Tests **must** follow these conventions:

1. **Describe blocks** group tests by function or behavior
2. **Nested describe blocks** group tests by scenario (e.g., `describe('login', () => { ... })`)
3. **`beforeEach`** resets state (Pinia store, localStorage, mocks)
4. **`vi.fn()`** for mock functions, **`vi.stubGlobal()`** for global overrides
5. **Assertions** use `expect()` with specific matchers (`.toBe()`, `.toBeNull()`, `.rejects.toThrow()`)

Prefer `it()` over `test()` for consistency with the existing test suite.

> **Mocking exception:** The global no-mocking policy (CLAUDE.md, `test_organization.rulebook.md`, `iron_runtime.rulebook.md`) applies to Rust backend code. Frontend tests run in `jsdom`, which lacks browser APIs — the following stubs are **permitted**: (1) `vi.stubGlobal('fetch', ...)` for HTTP calls, (2) `vi.fn()` for event handler verification, (3) `vi.stubGlobal('localStorage', ...)` for storage. Mocking Vue component internals, Pinia store actions, or Vue Router navigation is **strictly forbidden**.

**Rationale:** Consistent structure makes tests self-documenting and easy to navigate.

> Good

```typescript
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useAuthStore } from './auth'

const mockFetch = vi.fn()
vi.stubGlobal('fetch', mockFetch)

describe('useAuthStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
    mockFetch.mockReset()
  })

  describe('loadTokens', () => {
    it('restores session when access_token is present', () => {
      localStorage.setItem('access_token', makeJwt({ role: 'admin' }))
      const store = useAuthStore()
      expect(store.isAuthenticated).toBe(true)
      expect(store.role).toBe('admin')
    })
  })
})
```

---

### Comments & Task Markers : Comment Content

Comments **must** primarily explain the "why" or clarify non-obvious aspects of the current code. It is **strictly forbidden** to add comments that merely state what change was made or serve as a historical log. Security-relevant decisions (e.g., storing tokens in localStorage) **must** include a `SECURITY NOTE:` comment explaining the accepted risk and required mitigations.

**Rationale:** Comments that explain intent age well; comments that describe changes become stale immediately.

> Good

```typescript
// Derive role from the JWT payload — do not trust the localStorage value
role.value = decodeJwtRole(storedAccessToken)

// SECURITY NOTE: localStorage is XSS-accessible. Access and refresh tokens are
// stored here for SPA session persistence. The accepted risk is that an XSS
// vulnerability would expose these tokens. Mitigations: strict CSP, short token
// TTLs, and refresh token rotation are required to limit blast radius.
localStorage.setItem('access_token', tokens.user_token)

// Prevents concurrent 401 responses from each triggering an independent
// refresh request (token refresh race condition).
let _refreshPromise: Promise<void> | null = null
```

---

### Comments & Task Markers : Task Marker Compatibility

The task marker system defined in the Rust codestyle rulebook (`xxx:`, `qqq:`, `aaa:`, `zzz:`) **must** be used identically in frontend code. All rules from the Rust rulebook regarding task markers — including the prohibition on removing existing task comments and the requirement to annotate addressed tasks with `aaa:` — apply without modification.

**Rationale:** A unified task marker system across the entire codebase (Rust and TypeScript) enables project-wide task tracking with a single grep command.

---

### Project Structure : Canonical Directory Layout

The frontend project **must** follow this directory structure. **Any of these directories may be absent** if not needed, but when present, they **must** serve exactly the described purpose.

```
src/
├── main.ts                     # App initialization (plugins, router, mount)
├── App.vue                     # Root component with route-aware layout
├── style.css                   # Tailwind imports + CSS variable tokens
│
├── components/                 # Reusable UI components
│   ├── cards/                  # Card-type display components
│   ├── icons/                  # SVG icon components (Icon* prefix)
│   └── ui/                     # Shadcn-style primitives (Reka UI wrappers)
│       ├── button/
│       │   ├── Button.vue
│       │   └── index.ts        # Barrel export + CVA config
│       ├── dialog/
│       ├── input/
│       └── [primitive]/
│
├── composables/                # Composition API hooks
│   ├── useApi.ts               # Centralized API client + types
│   └── useConfirm.ts           # Confirmation dialog state
│
├── lib/                        # Pure utility functions (no Vue reactivity)
│   ├── utils.ts                # cn() utility (*)
│   ├── formatters.ts           # Display formatting helpers
│   └── providers.ts            # Provider labels, colors, placeholders
│
├── stores/                     # Pinia stores (Composition API)
│   └── auth.ts                 # Authentication state
│
├── router/                     # Vue Router configuration
│   └── index.ts                # Routes + navigation guards
│
├── views/                      # Page-level components (*View suffix)
│   ├── LoginView.vue
│   ├── DashboardView.vue
│   └── [Feature]View.vue
│
└── assets/                     # Static images and files
```

**ABSOLUTE File Placement Rules (No Exceptions):**
- **Page components:** MUST be in `views/` with the `View` suffix
- **Reusable components:** MUST be in `components/`
- **UI primitives:** MUST be in `components/ui/` with barrel exports
- **Icon components:** MUST be in `components/icons/` with `Icon` prefix
- **API logic:** MUST be in `composables/useApi.ts`
- **Reactive state hooks:** MUST be in `composables/`
- **Pure utility functions:** MUST be in `lib/`
- **Global state:** MUST be in `stores/`
- **Route definitions:** MUST be in `router/`
- **CSS tokens and Tailwind imports:** MUST be in `style.css`

> **(\*) `utils.ts` naming exception:** The global CLAUDE.md and `files_structure.rulebook.md` prohibit generic filenames like `utils.*`. The file `lib/utils.ts` is an explicit exception because it is the standard filename generated by the `shadcn-vue` toolchain and contains only the `cn()` class-merge utility — a single, well-defined responsibility. Renaming it would break the `shadcn-vue` code generation workflow.

---

### Security : v-html Prohibition and XSS Prevention

It is **strictly forbidden** to use `v-html` in any component. If rendering user-generated or server-provided HTML is unavoidable, the content **must** be sanitized with DOMPurify before binding. All API requests **must** set `Content-Type: application/json` (enforced in the centralized `useApi` composable). Client-side validation is for UX only — the server is always authoritative for access control and data integrity.

It is **strictly forbidden** to:
1. Use `v-html` without DOMPurify sanitization
2. Interpolate user input into template strings that are rendered as HTML
3. Trust client-side role checks as a security boundary (they are presentation logic only)

**Rationale:** `v-html` is Vue's primary XSS vector. For a control panel that manages LLM provider keys and authentication tokens, preventing XSS is critical. The auth store already documents the localStorage XSS risk — this rule codifies the prevention side.

---

### Security : Global Error Boundary

The application **must** register a global error handler via `app.config.errorHandler` to catch unhandled exceptions in the Vue component tree. Additionally, a `window.addEventListener('unhandledrejection', ...)` listener **must** be registered in `main.ts` to catch unhandled promise rejections. Both handlers **must** log the error and display a user-facing toast notification.

**Rationale:** Without a global error boundary, unhandled exceptions silently break the UI. A global handler provides a last line of defense for unexpected errors.

---

### Accessibility : Custom Component Requirements

All interactive elements not built on Reka UI primitives **must** meet these accessibility requirements:

1. **Keyboard accessibility:** All interactive elements **must** be reachable and operable via keyboard (`Tab`, `Enter`, `Space`, `Escape` as appropriate)
2. **ARIA attributes:** Custom interactive components **must** include appropriate ARIA roles and states (`aria-label`, `aria-expanded`, `aria-describedby`, etc.)
3. **Focus management:** Route transitions **must** not trap or lose focus. After modal close, focus **must** return to the triggering element
4. **Images:** All `<img>` elements **must** have an `alt` attribute. Decorative images use `alt=""`

**Rationale:** Reka UI primitives handle accessibility automatically, but custom interactive elements (e.g., clickable text, custom cards) must meet the same standard. Accessibility is a legal requirement in many jurisdictions and improves usability for all users.
