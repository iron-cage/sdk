# UI Component Inventory

## Purpose

Comprehensive catalog of all Vue components in iron_dashboard, including shadcn-vue UI components, views, layout components, and composables. Serves as a quick reference for component location, purpose, variants, props/emits, and usage patterns.

---

## Scope

**Included:**
- shadcn-vue UI Components (accessible, composable primitives)
- Vue Single-File Components (.vue files)
- Composables (reusable composition functions)
- Component relationships and dependencies
- Props, emits, variants, and usage examples

**Excluded:**
- Component implementation architecture → See `architecture.md` (Component Structure section, shadcn-vue section)
- State management patterns → See `architecture.md` (State Management section)
- API integration details → See `api_integration.md`
- Build configuration → See `architecture.md` (Build Pipeline section)
- shadcn-vue installation guide → See `development_setup.md` (shadcn-vue Components section)

---

## Component Categories

### shadcn-vue UI Components

Location: `src/components/ui/*/`

shadcn-vue components are accessible, customizable Vue components built on Radix Vue primitives. Components are **copied into the project** (not npm dependencies) for full control over styling and behavior. All components use class-variance-authority (CVA) for type-safe variant-based styling and support WCAG 2.1 Level AA accessibility.

**Architecture:**
- **Radix Vue Primitives:** Unstyled accessible components (focus management, keyboard navigation, ARIA attributes)
- **CVA Variants:** Type-safe variant definitions for consistent styling
- **Tailwind CSS:** Utility-first styling with design tokens
- **clsx + tailwind-merge:** Conditional class names with conflict resolution

**Total Components:** 12 component groups, 57 individual .vue files

---

#### 1. Button

**Location:** `src/components/ui/button/`
**Exports:** `Button`, `buttonVariants`
**Base Primitive:** `<Primitive>` (reka-ui)

**Purpose:** Clickable button element with variant-based styling

**Variants:**
- `default` - Primary action button (bg-primary with shadow)
- `destructive` - Dangerous/delete actions (bg-destructive with red styling)
- `outline` - Secondary action with border (border + bg-background)
- `secondary` - Tertiary action (bg-secondary)
- `ghost` - Minimal button without background
- `link` - Text link styled as button

**Sizes:**
- `default` - h-9 px-4 py-2
- `xs` - h-7 px-2 (extra small)
- `sm` - h-8 px-3 (small)
- `lg` - h-10 px-8 (large)
- `icon` - h-9 w-9 (square icon button)
- `icon-sm` - size-8 (small icon button)
- `icon-lg` - size-10 (large icon button)

**Props:**
```typescript
variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'
size?: 'default' | 'xs' | 'sm' | 'lg' | 'icon' | 'icon-sm' | 'icon-lg'
as?: string  // HTML element to render (default: 'button')
asChild?: boolean  // Render as child element (slot delegation)
class?: string  // Additional CSS classes
```

**Usage:**
```vue
<script setup>
import { Button } from '@/components/ui/button'
</script>

<template>
  <!-- Primary action -->
  <Button @click="handleSave">Save Changes</Button>

  <!-- Destructive action -->
  <Button variant="destructive" @click="handleDelete">Delete</Button>

  <!-- Secondary action -->
  <Button variant="outline" @click="handleCancel">Cancel</Button>

  <!-- Icon button -->
  <Button variant="ghost" size="icon">
    <svg><!-- icon --></svg>
  </Button>
</template>
```

**Used In:** All views (LoginView, DashboardView, TokensView, UsageView, LimitsView, TracesView)

---

#### 2. Card

**Location:** `src/components/ui/card/`
**Exports:** `Card`, `CardContent`, `CardDescription`, `CardFooter`, `CardHeader`, `CardTitle`
**Base Primitive:** Native `<div>` elements with styled wrappers

**Purpose:** Container component for grouping related content with consistent spacing and borders

**Subcomponents:**
- `Card` - Root container (rounded border, shadow, bg-card)
- `CardHeader` - Header section (flex column, space-y-1.5, p-6)
- `CardTitle` - Title heading (text-2xl font-semibold)
- `CardDescription` - Subtitle text (text-sm text-muted-foreground)
- `CardContent` - Main content area (p-6 pt-0)
- `CardFooter` - Footer section (flex items-center, p-6 pt-0)

**Props:**
```typescript
// All subcomponents accept:
class?: string  // Additional CSS classes
```

**Usage:**
```vue
<script setup>
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Statistics</CardTitle>
      <CardDescription>Your usage overview</CardDescription>
    </CardHeader>
    <CardContent>
      <!-- Main content -->
    </CardContent>
  </Card>
</template>
```

**Used In:** DashboardView (stat cards), UsageView (analytics cards), TracesView (filter panel)

---

#### 3. Dialog

**Location:** `src/components/ui/dialog/`
**Exports:** `Dialog`, `DialogClose`, `DialogContent`, `DialogDescription`, `DialogFooter`, `DialogHeader`, `DialogScrollContent`, `DialogTitle`, `DialogTrigger`
**Base Primitive:** `<DialogRoot>`, `<DialogPortal>`, `<DialogOverlay>`, `<DialogContent>` (Radix Vue)

**Purpose:** Modal dialog with focus trapping, backdrop, and accessible keyboard navigation

**Subcomponents:**
- `Dialog` - Root wrapper with v-model:open binding
- `DialogTrigger` - Button/element that opens dialog
- `DialogContent` - Main dialog container (overlay + portal)
- `DialogScrollContent` - Scrollable variant of DialogContent
- `DialogHeader` - Header section (flex column, space-y-1.5, text-center)
- `DialogTitle` - Dialog title (text-lg font-semibold)
- `DialogDescription` - Dialog description (text-sm text-muted-foreground)
- `DialogFooter` - Footer section (flex justify-end, space-x-2)
- `DialogClose` - Close button component

**Accessibility:**
- Focus trapping (focus locked within dialog)
- Escape key closes dialog
- Click outside closes dialog
- ARIA labels from DialogTitle/DialogDescription
- Screen reader announcements

**Props:**
```typescript
// Dialog (root)
open?: boolean  // v-model:open for controlled state

// DialogContent
class?: string  // Additional CSS classes
```

**Usage:**
```vue
<script setup>
import { ref } from 'vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'

const showModal = ref(false)
</script>

<template>
  <Button @click="showModal = true">Open Dialog</Button>

  <Dialog v-model:open="showModal">
    <DialogContent>
      <DialogHeader>
        <DialogTitle>Confirm Action</DialogTitle>
        <DialogDescription>
          This action cannot be undone.
        </DialogDescription>
      </DialogHeader>

      <!-- Content here -->

      <DialogFooter>
        <Button variant="outline" @click="showModal = false">Cancel</Button>
        <Button @click="handleConfirm">Confirm</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
```

**Used In:** TokensView (create token modal, display token modal), LimitsView (create/edit limit modals)

---

#### 4. Select

**Location:** `src/components/ui/select/`
**Exports:** `Select`, `SelectContent`, `SelectGroup`, `SelectItem`, `SelectItemText`, `SelectLabel`, `SelectScrollDownButton`, `SelectScrollUpButton`, `SelectSeparator`, `SelectTrigger`, `SelectValue`
**Base Primitive:** `<SelectRoot>`, `<SelectTrigger>`, `<SelectContent>`, `<SelectItem>` (Radix Vue)

**Purpose:** Accessible dropdown select component with keyboard navigation

**Subcomponents:**
- `Select` - Root wrapper with v-model binding
- `SelectTrigger` - Button that opens dropdown
- `SelectValue` - Displays selected value placeholder
- `SelectContent` - Dropdown content container (portal + positioning)
- `SelectGroup` - Groups related items
- `SelectLabel` - Label for grouped items
- `SelectItem` - Individual selectable option
- `SelectItemText` - Text content of item
- `SelectSeparator` - Visual separator between groups
- `SelectScrollUpButton` - Scroll up indicator
- `SelectScrollDownButton` - Scroll down indicator

**Accessibility:**
- Arrow key navigation (Up/Down)
- Type-ahead search
- Escape to close
- Space/Enter to select
- Screen reader support with ARIA attributes

**Props:**
```typescript
// Select (root)
modelValue?: string | number  // v-model for selected value
disabled?: boolean

// SelectTrigger
class?: string  // Additional CSS classes

// SelectItem
value: string | number  // Required: option value
disabled?: boolean
```

**Usage:**
```vue
<script setup>
import { ref } from 'vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

const selected = ref('option1')
</script>

<template>
  <Select v-model="selected">
    <SelectTrigger>
      <SelectValue placeholder="Choose option" />
    </SelectTrigger>
    <SelectContent>
      <SelectItem value="option1">Option 1</SelectItem>
      <SelectItem value="option2">Option 2</SelectItem>
      <SelectItem value="option3">Option 3</SelectItem>
    </SelectContent>
  </Select>
</template>
```

**Used In:** LimitsView (limit type, period selectors), TracesView (provider, model filters)

---

#### 5. Input

**Location:** `src/components/ui/input/`
**Exports:** `Input`
**Base Primitive:** Native `<input>` element with styled wrapper

**Purpose:** Styled text input field with consistent spacing and borders

**Props:**
```typescript
type?: string  // Input type (text, password, email, number, etc.)
disabled?: boolean
class?: string  // Additional CSS classes
// All native input attributes supported
```

**Styling:**
- Border with focus ring (focus-visible:ring-2)
- Disabled state (disabled:cursor-not-allowed disabled:opacity-50)
- File input variant (file:border-0 file:bg-transparent)
- Placeholder styling (placeholder:text-muted-foreground)

**Usage:**
```vue
<script setup>
import { ref } from 'vue'
import { Input } from '@/components/ui/input'

const username = ref('')
</script>

<template>
  <Input
    v-model="username"
    type="text"
    placeholder="Enter username"
    :disabled="loading"
  />
</template>
```

**Used In:** LoginView (username, password), TokensView (project_id, description), LimitsView (limit value), TracesView (token ID filter)

---

#### 6. Label

**Location:** `src/components/ui/label/`
**Exports:** `Label`
**Base Primitive:** `<Label>` (Radix Vue)

**Purpose:** Accessible form label with proper click association

**Props:**
```typescript
for?: string  // Associates label with input id
class?: string  // Additional CSS classes
```

**Accessibility:**
- Clicking label focuses associated input
- Screen reader association via for/id
- Disabled state handling (peer-disabled:cursor-not-allowed peer-disabled:opacity-70)

**Usage:**
```vue
<script setup>
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
</script>

<template>
  <div class="space-y-2">
    <Label for="email">Email</Label>
    <Input id="email" type="email" />
  </div>
</template>
```

**Used In:** LoginView, TokensView, LimitsView, TracesView (all form inputs)

---

#### 7. Alert

**Location:** `src/components/ui/alert/`
**Exports:** `Alert`, `AlertDescription`, `AlertTitle`, `alertVariants`
**Base Primitive:** Native `<div>` elements with styled wrappers

**Purpose:** Contextual feedback message with variant-based styling

**Variants:**
- `default` - Informational alert (bg-background text-foreground)
- `destructive` - Error/warning alert (border-destructive text-destructive)

**Subcomponents:**
- `Alert` - Root container with variant styling
- `AlertTitle` - Title heading (mb-1 font-medium leading-none)
- `AlertDescription` - Description text (text-sm opacity-90)

**Props:**
```typescript
// Alert
variant?: 'default' | 'destructive'
class?: string  // Additional CSS classes
```

**Usage:**
```vue
<script setup>
import { Alert, AlertDescription } from '@/components/ui/alert'
</script>

<template>
  <!-- Error alert -->
  <Alert v-if="error" variant="destructive">
    <AlertDescription>{{ error }}</AlertDescription>
  </Alert>

  <!-- Info alert -->
  <Alert>
    <AlertDescription>Changes saved successfully</AlertDescription>
  </Alert>
</template>
```

**Used In:** LoginView (login errors), TokensView (create errors), LimitsView (create/edit errors)

---

#### 8. Badge

**Location:** `src/components/ui/badge/`
**Exports:** `Badge`, `badgeVariants`
**Base Primitive:** Native `<div>` element with CVA variants

**Purpose:** Small status indicator or label

**Variants:**
- `default` - Primary badge (bg-primary text-primary-foreground)
- `secondary` - Secondary badge (bg-secondary text-secondary-foreground)
- `destructive` - Error/warning badge (bg-destructive text-destructive-foreground)
- `outline` - Bordered badge (border text-foreground)

**Props:**
```typescript
variant?: 'default' | 'secondary' | 'destructive' | 'outline'
class?: string  // Additional CSS classes
```

**Usage:**
```vue
<script setup>
import { Badge } from '@/components/ui/badge'
</script>

<template>
  <Badge>Active</Badge>
  <Badge variant="destructive">Revoked</Badge>
  <Badge variant="outline">Pending</Badge>
</template>
```

**Used In:** DashboardView (token status), TokensView (token status)

---

#### 9. Dropdown Menu

**Location:** `src/components/ui/dropdown-menu/`
**Exports:** Multiple subcomponents (DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem, etc.)
**Base Primitive:** `<DropdownMenuRoot>`, `<DropdownMenuTrigger>`, `<DropdownMenuContent>` (Radix Vue)

**Purpose:** Accessible dropdown menu with keyboard navigation

**Status:** Installed but not currently used in any views

**Accessibility:**
- Arrow key navigation
- Escape to close
- Focus management
- ARIA attributes

---

#### 10. Separator

**Location:** `src/components/ui/separator/`
**Exports:** `Separator`
**Base Primitive:** `<Separator>` (Radix Vue)

**Purpose:** Visual divider between content sections

**Status:** Installed but not currently used in any views

**Props:**
```typescript
orientation?: 'horizontal' | 'vertical'
decorative?: boolean  // ARIA hidden if true
class?: string
```

---

#### 11. Skeleton

**Location:** `src/components/ui/skeleton/`
**Exports:** `Skeleton`
**Base Primitive:** Native `<div>` with pulse animation

**Purpose:** Loading placeholder with shimmer effect

**Status:** Installed but not currently used in any views

**Usage:**
```vue
<script setup>
import { Skeleton } from '@/components/ui/skeleton'
</script>

<template>
  <div v-if="isLoading">
    <Skeleton class="h-12 w-12 rounded-full" />
    <Skeleton class="h-4 w-[250px]" />
  </div>
</template>
```

---

#### 12. Toast

**Location:** `src/components/ui/toast/`
**Exports:** Multiple subcomponents (Toast, ToastAction, ToastClose, ToastDescription, ToastProvider, ToastTitle, ToastViewport, Toaster)
**Base Primitive:** `<ToastRoot>`, `<ToastProvider>` (Radix Vue)

**Purpose:** Non-blocking notification system

**Status:** Installed but not currently used in any views

**Accessibility:**
- Screen reader announcements
- Dismissible
- Auto-dismiss timeout
- Focus management

---

### Views (Route-Level Components)

Location: `src/views/`

Views are route-level components bound to specific URLs via Vue Router. They orchestrate data fetching, state management, and layout composition.

#### 1. LoginView.vue

**Route:** `/login`
**Purpose:** User authentication interface

**Functionality:**
- Username/password form submission
- JWT token acquisition and storage
- Redirect to `/dashboard` on success
- Error display for failed login

**shadcn-vue Components Used:**
- `Card`, `CardContent`, `CardDescription`, `CardHeader`, `CardTitle` - Form container
- `Input` - Username and password fields
- `Label` - Form field labels
- `Button` - Submit button
- `Alert`, `AlertDescription` - Error messages

**Props:** None
**Emits:** None
**State:** Uses `useAuthStore()` for authentication logic

**Usage:**
```vue
<template>
  <LoginView /> <!-- Rendered when route is /login -->
</template>
```

---

#### 2. DashboardView.vue

**Route:** `/dashboard`
**Purpose:** Overview of token statistics and quick actions

**Data Display:**
- Total tokens count
- Active tokens count
- Revoked tokens count
- Recent tokens list (last 5)

**shadcn-vue Components Used:**
- `Card`, `CardContent`, `CardHeader`, `CardTitle` - Stat cards and content containers
- `Button` - Navigation buttons (Manage Tokens, View Usage Analytics, Configure Limits)
- `Badge` - Token status indicators (Active/Revoked)

**API Calls:**
- `useQuery(['tokens'], api.getTokens)` - Fetches all tokens

**Dependencies:**
- `useApi` composable
- TanStack Vue Query for data fetching

**Usage:**
```vue
<template>
  <DashboardView /> <!-- Rendered when route is /dashboard -->
</template>
```

---

#### 3. TokensView.vue

**Route:** `/tokens`
**Purpose:** Token management interface (create, rotate, revoke)

**Functionality:**
- Display all tokens in table format
- Create new token modal with project_id/description fields
- Rotate token (revokes old, generates new)
- Revoke token with confirmation dialog
- Copy token to clipboard
- Display newly created token (one-time display)

**shadcn-vue Components Used:**
- `Dialog`, `DialogContent`, `DialogDescription`, `DialogFooter`, `DialogHeader`, `DialogTitle` - Create/display token modals
- `Button` - Action buttons (Create, Rotate, Revoke, Copy, Close)
- `Input` - Project ID and description fields
- `Label` - Form field labels
- `Alert`, `AlertDescription` - Error messages
- `Badge` - Token status indicators

**State:**
```typescript
showCreateModal: Ref<boolean>  // Controls create token modal
showTokenModal: Ref<boolean>   // Controls "token created" modal
newTokenData: Ref<CreateTokenResponse | null>  // Stores newly created token
projectId: Ref<string>         // Form field for project_id
description: Ref<string>       // Form field for description
createError: Ref<string>       // Error message display
```

**Mutations:**
- `createMutation` - POST /api/tokens
- `rotateMutation` - POST /api/tokens/:id/rotate
- `revokeMutation` - POST /api/tokens/:id/revoke

**Query Invalidation:**
All mutations invalidate `['tokens']` query to trigger refetch

**Usage:**
```vue
<template>
  <TokensView /> <!-- Rendered when route is /tokens -->
</template>
```

---

#### 4. UsageView.vue

**Route:** `/usage`
**Purpose:** Cost and usage analytics visualization

**Data Display:**
- Total requests count
- Total input/output tokens
- Total cost (USD)
- Cost breakdown by provider
- Cost breakdown by model
- Recent usage records (last 10)

**shadcn-vue Components Used:**
- `Card`, `CardContent`, `CardHeader`, `CardTitle` - Summary stat cards, provider/model breakdown cards, recent usage card

**API Calls:**
- `useQuery(['usage'], api.getUsage)` - All usage records
- `useQuery(['usage-stats'], api.getUsageStats)` - Aggregated statistics

**Usage:**
```vue
<template>
  <UsageView /> <!-- Rendered when route is /usage -->
</template>
```

---

#### 5. LimitsView.vue

**Route:** `/limits`
**Purpose:** Budget limit configuration interface

**Functionality:**
- Display all budget limits (daily/weekly/monthly/yearly)
- Create new limit with type/value/period
- Update existing limit
- Delete limit with confirmation

**shadcn-vue Components Used:**
- `Dialog`, `DialogContent`, `DialogDescription`, `DialogFooter`, `DialogHeader`, `DialogTitle` - Create/edit limit modals
- `Select`, `SelectContent`, `SelectItem`, `SelectTrigger`, `SelectValue` - Limit type and period dropdowns
- `Button` - Action buttons (Create, Edit, Delete, Cancel)
- `Input` - Project ID and limit value fields
- `Label` - Form field labels
- `Alert`, `AlertDescription` - Error messages

**Limit Types:**
- `budget` - Maximum cost in USD
- `tokens` - Maximum tokens consumed
- `requests` - Maximum number of requests

**Periods:**
- `daily`
- `weekly`
- `monthly`
- `yearly`

**Mutations:**
- `createLimit` - POST /api/limits
- `updateLimit` - PUT /api/limits/:id
- `deleteLimit` - DELETE /api/limits/:id

**Usage:**
```vue
<template>
  <LimitsView /> <!-- Rendered when route is /limits -->
</template>
```

---

#### 6. TracesView.vue

**Route:** `/traces`
**Purpose:** Request-level trace inspection with filtering

**Data Display:**
- Trace list with timestamp, request ID, token ID, provider, model, cost
- Input/output token counts
- Filters: token ID, provider, model
- Filtered count display

**shadcn-vue Components Used:**
- `Card`, `CardContent`, `CardHeader`, `CardTitle` - Filter panel container
- `Select`, `SelectContent`, `SelectItem`, `SelectTrigger`, `SelectValue` - Provider and model filter dropdowns
- `Input` - Token ID filter field
- `Label` - Filter field labels
- `Button` - Clear filters, Retry buttons

**API Calls:**
- `useQuery(['traces'], api.getTraces)` - Fetch all traces

**Usage:**
```vue
<template>
  <TracesView /> <!-- Rendered when route is /traces -->
</template>
```

---

### Layout Components

Location: `src/components/`

#### 7. MainLayout.vue

**Purpose:** Authenticated page layout with sidebar navigation and header

**Structure:**
```
┌──────────────────────────────────────┐
│ Sidebar (left)   │ Header (top)      │
│ - Dashboard      │ - Username        │
│ - Tokens         │ - Logout button   │
│ - Usage          │                   │
│ - Limits         │                   │
│ - Traces         │                   │
│                  │                   │
│                  │ Main Content      │
│                  │ <router-view />   │
└──────────────────────────────────────┘
```

**State:**
```typescript
sidebarOpen: Ref<boolean>  // Sidebar visibility (default: true)
```

**Methods:**
```typescript
handleLogout(): void  // Calls authStore.logout(), redirects to /login
```

**Responsive Behavior:**
- **Desktop (≥1024px):** Sidebar visible, content offset by 256px
- **Mobile (<1024px):** Sidebar hidden, overlay when opened

**Navigation Links:**
All sidebar links use `<router-link>` with active state detection

**Usage:**
```vue
<template>
  <MainLayout>
    <DashboardView />  <!-- Slot content -->
  </MainLayout>
</template>
```

**Note:** App.vue conditionally renders MainLayout based on `route.meta.requiresAuth`

---

#### 8. App.vue

**Purpose:** Root component with conditional layout rendering

**Logic:**
```vue
<MainLayout v-if="requiresAuth">
  <router-view />
</MainLayout>
<router-view v-else />  <!-- LoginView renders without layout -->
```

**Computed:**
```typescript
requiresAuth: ComputedRef<boolean>  // Based on route.meta.requiresAuth
```

**Usage:**
This is the root component mounted in `main.ts`:
```typescript
createApp(App).mount('#app')
```

---

### Utility Components

#### 9. HelloWorld.vue

**Status:** Legacy Vue template component (not used in production)
**Purpose:** Original Vite + Vue template demo component
**Action:** Should be removed in future cleanup

---

## Composables (Composition Functions)

Location: `src/composables/`

### useApi.ts

**Purpose:** Centralized REST API client with authentication

**Exports:**
```typescript
function useApi(): {
  // Token methods
  getTokens(): Promise<TokenMetadata[]>
  getToken(id: number): Promise<TokenMetadata>
  createToken(data: CreateTokenRequest): Promise<CreateTokenResponse>
  rotateToken(id: number): Promise<CreateTokenResponse>
  revokeToken(id: number): Promise<void>

  // Usage methods
  getUsage(): Promise<UsageRecord[]>
  getUsageStats(): Promise<UsageStats>
  getUsageByToken(tokenId: number): Promise<UsageRecord[]>

  // Limits methods
  getLimits(): Promise<LimitRecord[]>
  getLimit(id: number): Promise<LimitRecord>
  createLimit(data: CreateLimitRequest): Promise<LimitRecord>
  updateLimit(id: number, data: UpdateLimitRequest): Promise<LimitRecord>
  deleteLimit(id: number): Promise<void>

  // Traces methods
  getTraces(): Promise<TraceRecord[]>
  getTrace(id: number): Promise<TraceRecord>
}
```

**Type Exports:**
```typescript
TokenMetadata
CreateTokenRequest
CreateTokenResponse
UsageRecord
UsageStats
LimitRecord
CreateLimitRequest
UpdateLimitRequest
TraceRecord
```

**Authentication:**
- Automatically injects `Authorization: Bearer <token>` header
- Uses `useAuthStore().getAuthHeader()` for token retrieval
- Throws error on HTTP 401/403

**Base URL:**
```typescript
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3001'
```

**Error Handling:**
```typescript
if (!response.ok) {
  const error = await response.json().catch(() => ({ error: 'Request failed' }))
  throw new Error(error.error || `HTTP ${response.status}`)
}
```

**Usage Example:**
```vue
<script setup>
import { useApi } from '@/composables/useApi'
const api = useApi()

const { data: tokens } = useQuery({
  queryKey: ['tokens'],
  queryFn: () => api.getTokens()
})
</script>
```

---

## Component Dependency Graph

```
App.vue
├── MainLayout.vue (if authenticated)
│   ├── useRouter (Vue Router)
│   ├── useAuthStore (Pinia store)
│   └── router-view (slot)
│       ├── DashboardView.vue
│       │   ├── useApi
│       │   ├── useQuery (TanStack)
│       │   ├── useRouter
│       │   ├── Card, CardContent, CardHeader, CardTitle (shadcn-vue)
│       │   ├── Button (shadcn-vue)
│       │   └── Badge (shadcn-vue)
│       ├── TokensView.vue
│       │   ├── useApi
│       │   ├── useQuery (TanStack)
│       │   ├── useMutation (TanStack)
│       │   ├── useQueryClient (TanStack)
│       │   ├── useAuthStore
│       │   ├── Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle (shadcn-vue)
│       │   ├── Button (shadcn-vue)
│       │   ├── Input, Label (shadcn-vue)
│       │   ├── Alert, AlertDescription (shadcn-vue)
│       │   └── Badge (shadcn-vue)
│       ├── UsageView.vue
│       │   ├── useApi
│       │   ├── useQuery (TanStack)
│       │   └── Card, CardContent, CardHeader, CardTitle (shadcn-vue)
│       ├── LimitsView.vue
│       │   ├── useApi
│       │   ├── useQuery (TanStack)
│       │   ├── useMutation (TanStack)
│       │   ├── Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle (shadcn-vue)
│       │   ├── Select, SelectContent, SelectItem, SelectTrigger, SelectValue (shadcn-vue)
│       │   ├── Button (shadcn-vue)
│       │   ├── Input, Label (shadcn-vue)
│       │   └── Alert, AlertDescription (shadcn-vue)
│       └── TracesView.vue
│           ├── useApi
│           ├── useQuery (TanStack)
│           ├── Card, CardContent, CardHeader, CardTitle (shadcn-vue)
│           ├── Select, SelectContent, SelectItem, SelectTrigger, SelectValue (shadcn-vue)
│           ├── Input, Label (shadcn-vue)
│           └── Button (shadcn-vue)
└── router-view (if not authenticated)
    └── LoginView.vue
        ├── useAuthStore
        ├── useRouter
        ├── Card, CardContent, CardDescription, CardHeader, CardTitle (shadcn-vue)
        ├── Input, Label (shadcn-vue)
        ├── Button (shadcn-vue)
        └── Alert, AlertDescription (shadcn-vue)
```

---

## Shared Patterns

### Data Fetching Pattern

All views except LoginView use TanStack Vue Query for data fetching:

```vue
<script setup>
import { useQuery } from '@tanstack/vue-query'
import { useApi } from '@/composables/useApi'

const api = useApi()
const { data, isLoading, error } = useQuery({
  queryKey: ['unique-key'],
  queryFn: () => api.getSomeData()
})
</script>

<template>
  <div v-if="isLoading">Loading...</div>
  <div v-else-if="error">Error: {{ error.message }}</div>
  <div v-else>
    <!-- Render data -->
  </div>
</template>
```

**Rationale:** Automatic caching, background refetching, request deduplication

---

### Mutation Pattern

Components with data modification (TokensView, LimitsView) use TanStack mutations:

```vue
<script setup>
import { useMutation, useQueryClient } from '@tanstack/vue-query'

const queryClient = useQueryClient()
const mutation = useMutation({
  mutationFn: (data) => api.createSomething(data),
  onSuccess: () => {
    // Invalidate cache to trigger refetch
    queryClient.invalidateQueries({ queryKey: ['something'] })
  }
})

function handleCreate() {
  mutation.mutate({ field: 'value' })
}
</script>
```

**Rationale:** Optimistic updates, error handling, cache invalidation

---

### shadcn-vue Component Patterns

#### Variant-Based Styling Pattern

shadcn-vue components use class-variance-authority (CVA) for type-safe variant styling:

```vue
<script setup>
import { Button } from '@/components/ui/button'
</script>

<template>
  <!-- Primary action (default variant) -->
  <Button>Save</Button>

  <!-- Destructive action -->
  <Button variant="destructive">Delete</Button>

  <!-- Secondary action with custom class -->
  <Button variant="outline" class="mt-4">Cancel</Button>
</template>
```

**Key Benefits:**
- Type-safe variant props (TypeScript autocomplete)
- Consistent styling across application
- Easy customization via additional classes

---

#### Dialog State Management Pattern

Dialogs use v-model:open for controlled state:

```vue
<script setup>
import { ref } from 'vue'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'

const showModal = ref(false)

function handleSubmit() {
  // Process form
  showModal.value = false  // Close dialog
}
</script>

<template>
  <!-- Trigger -->
  <Button @click="showModal = true">Open</Button>

  <!-- Dialog -->
  <Dialog v-model:open="showModal">
    <DialogContent>
      <DialogHeader>
        <DialogTitle>Dialog Title</DialogTitle>
      </DialogHeader>
      <!-- Content -->
      <Button @click="handleSubmit">Submit</Button>
    </DialogContent>
  </Dialog>
</template>
```

**Benefits:**
- Controlled state (programmatic open/close)
- Automatic focus trapping
- Escape key and click-outside handling

---

#### Form Input Pattern with Label

Labels are properly associated with inputs for accessibility:

```vue
<script setup>
import { ref } from 'vue'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'

const email = ref('')
</script>

<template>
  <div class="space-y-2">
    <Label for="email">Email Address</Label>
    <Input
      id="email"
      v-model="email"
      type="email"
      placeholder="user@example.com"
    />
  </div>
</template>
```

**Benefits:**
- Clicking label focuses input
- Screen reader association
- Consistent spacing via Tailwind utilities

---

#### Select with v-model Pattern

Selects use v-model for two-way binding:

```vue
<script setup>
import { ref } from 'vue'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

const selectedOption = ref('option1')
</script>

<template>
  <Select v-model="selectedOption">
    <SelectTrigger>
      <SelectValue placeholder="Choose option" />
    </SelectTrigger>
    <SelectContent>
      <SelectItem value="option1">Option 1</SelectItem>
      <SelectItem value="option2">Option 2</SelectItem>
      <SelectItem value="option3">Option 3</SelectItem>
    </SelectContent>
  </Select>
</template>
```

**Benefits:**
- Automatic keyboard navigation (arrow keys, type-ahead)
- ARIA attributes for screen readers
- Portal rendering (avoids z-index issues)

---

#### Conditional Alerts Pattern

Alerts display contextual feedback based on state:

```vue
<script setup>
import { ref } from 'vue'
import { Alert, AlertDescription } from '@/components/ui/alert'

const error = ref('')
const success = ref('')
</script>

<template>
  <!-- Error alert -->
  <Alert v-if="error" variant="destructive">
    <AlertDescription>{{ error }}</AlertDescription>
  </Alert>

  <!-- Success alert -->
  <Alert v-if="success">
    <AlertDescription>{{ success }}</AlertDescription>
  </Alert>
</template>
```

**Benefits:**
- Clear visual distinction (destructive variant)
- Conditional rendering
- Consistent spacing and styling

---

#### Card Composition Pattern

Cards are composed from multiple subcomponents:

```vue
<script setup>
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Statistics</CardTitle>
      <CardDescription>Your monthly overview</CardDescription>
    </CardHeader>
    <CardContent>
      <div class="text-3xl font-bold">1,234</div>
    </CardContent>
  </Card>
</template>
```

**Benefits:**
- Consistent spacing and borders
- Composable structure
- Semantic hierarchy (header, content, footer)

---

### Date Formatting Pattern

All views use consistent date formatting:

```typescript
function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString()
}
```

**Input:** Unix timestamp (seconds)
**Output:** Localized date/time string (e.g., "12/5/2024, 2:30:45 PM")

---

## Component File Structure

All Vue SFCs follow this structure:

```vue
<script setup lang="ts">
// 1. Imports
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useQuery } from '@tanstack/vue-query'

// 2. Composables
const router = useRouter()
const api = useApi()

// 3. Reactive state
const showModal = ref(false)

// 4. Computed properties
const stats = computed(() => { /* ... */ })

// 5. Methods
function handleAction() { /* ... */ }
</script>

<template>
  <!-- Template with Tailwind classes -->
</template>
```

**No `<style>` blocks** - All styling via Tailwind utility classes

---

## Component Metrics

| Category | Count | Files |
|----------|-------|-------|
| **shadcn-vue UI Components** | **12 groups** | **57 .vue files total** |
| ├─ Button | 1 | Button.vue |
| ├─ Card | 6 | Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle |
| ├─ Dialog | 9 | Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogScrollContent, DialogTitle, DialogTrigger |
| ├─ Select | 11 | Select, SelectContent, SelectGroup, SelectItem, SelectItemText, SelectLabel, SelectScrollDownButton, SelectScrollUpButton, SelectSeparator, SelectTrigger, SelectValue |
| ├─ Input | 1 | Input.vue |
| ├─ Label | 1 | Label.vue |
| ├─ Alert | 3 | Alert, AlertDescription, AlertTitle |
| ├─ Badge | 1 | Badge.vue |
| ├─ Dropdown Menu | ~10 | DropdownMenu (multiple subcomponents, not currently used) |
| ├─ Separator | 1 | Separator.vue (not currently used) |
| ├─ Skeleton | 1 | Skeleton.vue (not currently used) |
| └─ Toast | ~12 | Toast (multiple subcomponents, not currently used) |
| **Views** | **6** | LoginView, DashboardView, TokensView, UsageView, LimitsView, TracesView |
| **Layout Components** | **2** | App, MainLayout |
| **Utility Components** | **1** | HelloWorld (legacy, unused) |
| **Composables** | **1** | useApi |
| **Total** | **79** | 66 .vue files (57 UI + 9 views/layout/utility), 1 .ts file, 12 index.ts exports |

---

## Design Patterns Summary

**Component Types:**
- **shadcn-vue UI Components** - Accessible primitives with variant-based styling (CVA)
- **Views** - Route-level, data orchestration, business logic
- **Layout** - Structure and navigation, no business logic
- **Composables** - Reusable logic, stateless functions

**State Management:**
- **Server State:** TanStack Vue Query (caching, refetching)
- **Global UI State:** Pinia stores (auth state)
- **Local Component State:** `ref()`, `reactive()`
- **UI Component State:** v-model bindings for shadcn-vue components (Dialog open state, Select values)

**TypeScript:**
- All components use `<script setup lang="ts">`
- Strict type checking enabled
- Type imports from `useApi.ts` for API contracts
- CVA variant types for shadcn-vue components (`ButtonVariants`, `AlertVariants`)

**Styling:**
- **shadcn-vue Components:** CVA variants + Tailwind utility classes
- **Custom Classes:** clsx for conditional classes, tailwind-merge for conflict resolution
- **No `<style>` blocks** - Pure Tailwind CSS

**Accessibility:**
- **Radix Vue primitives** (all shadcn-vue components): Focus management, keyboard navigation, ARIA attributes
- **WCAG 2.1 Level AA compliance** via shadcn-vue
- **Semantic HTML** (`<button>`, `<nav>`, `<header>`)
- **Keyboard navigation** in all dialogs, selects, and interactive components
- **Screen reader support** with proper ARIA labels and announcements

---

## Future Component Additions

**When adding new shadcn-vue components:**

1. **Install via CLI:**
   ```bash
   npx shadcn-vue@latest add [component-name]
   ```
   - Components are copied to `src/components/ui/[component-name]/`
   - See `development_setup.md` for detailed installation guide

2. **Import and use:**
   ```vue
   <script setup lang="ts">
   import { Component } from '@/components/ui/component'
   </script>
   ```

3. **Customize if needed:**
   - Edit component files directly (they're owned by the project)
   - Modify CVA variants in `index.ts` for new styling options
   - Update Tailwind classes for design system changes

**When adding new custom components:**

1. **Choose correct location:**
   - Route-level → `src/views/[ComponentName]View.vue`
   - Reusable UI → `src/components/[ComponentName].vue`
   - Logic → `src/composables/use[Feature].ts`

2. **Follow naming conventions:**
   - Views: PascalCase + "View" suffix
   - Components: PascalCase
   - Composables: camelCase + "use" prefix

3. **Use TypeScript:**
   - `<script setup lang="ts">`
   - Import types from `useApi.ts`
   - Define props/emits with TypeScript interfaces

4. **Prefer shadcn-vue components:**
   - Use existing shadcn-vue components for UI elements
   - Only create custom components for business logic or complex compositions

5. **Follow data fetching pattern:**
   - Use TanStack Query for server data
   - Use Pinia stores for global state
   - Use local refs for component-specific state

6. **Update this inventory:**
   - Add component to relevant category
   - Document props, emits, purpose, shadcn-vue components used
   - Update dependency graph
   - Update component metrics

---

## References

**Internal Documentation:**
- **shadcn-vue Architecture:** `docs/architecture.md:124-279` (shadcn-vue section)
- **shadcn-vue Installation:** `docs/development_setup.md:570-667` (installation guide)
- **Component Source Code:** `src/views/*.vue`, `src/components/*.vue`
- **shadcn-vue UI Components:** `src/components/ui/*/` (12 component groups)
- **Composables Source Code:** `src/composables/*.ts`
- **API Integration:** `docs/api_integration.md`
- **Type Definitions:** `src/composables/useApi.ts:1-93` (interfaces)
- **Routing Configuration:** `src/router/index.ts`

**External Documentation:**
- **shadcn-vue:** https://www.shadcn-vue.com/
- **Radix Vue:** https://www.radix-vue.com/ (primitive components)
- **class-variance-authority:** https://cva.style/docs (CVA variants)
- **Tailwind CSS:** https://tailwindcss.com/docs
- **Vue 3 Composition API:** https://vuejs.org/api/composition-api.html
- **TanStack Query:** https://tanstack.com/query/latest/docs/vue/overview

---

**End of Component Inventory**
