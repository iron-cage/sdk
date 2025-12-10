# api_integration

Backend integration guide for iron_dashboard ↔ iron_control_api.

## Purpose

This document describes how iron_dashboard integrates with iron_control_api backend, including REST API endpoints, TypeScript type definitions, WebSocket integration, error handling patterns, and authentication flow. Intended for developers adding new API endpoints or modifying integration logic.

---

## Table of Contents

1. [REST API Integration](#rest-api-integration)
2. [TypeScript Type Safety](#typescript-type-safety)
3. [WebSocket Integration](#websocket-integration)
4. [Error Handling](#error-handling)
5. [Authentication Flow](#authentication-flow)
6. [Adding New Endpoints](#adding-new-endpoints)

---

## REST API Integration

### Base Configuration

**Backend:** iron_control_api (Axum Rust server)
**Base URL:** `http://localhost:3000` (dev), configurable via `VITE_API_URL`
**Content-Type:** `application/json`
**Authentication:** `Authorization: Bearer <jwt_token>` header

---

### API Client (useApi Composable)

**Location:** `src/composables/useApi.ts`

**Structure:**

```typescript
// src/composables/useApi.ts
import { useAuthStore } from '../stores/auth'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000'

export function useApi() {
  const authStore = useAuthStore()

  async function fetchApi<T>(path: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...((options.headers as Record<string, string>) || {}),
    }

    const authHeader = authStore.getAuthHeader()
    if (authHeader) {
      headers['Authorization'] = authHeader
    }

    const response = await fetch(`${API_BASE_URL}${path}`, {
      ...options,
      headers,
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Request failed' }))
      throw new Error(error.error || `HTTP ${response.status}`)
    }

    return response.json()
  }

  // Token API methods
  async function getTokens(): Promise<TokenMetadata[]> {
    return fetchApi<TokenMetadata[]>('/api/tokens')
  }

  async function createToken(data: CreateTokenRequest): Promise<CreateTokenResponse> {
    return fetchApi<CreateTokenResponse>('/api/tokens', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  // ... more methods

  return {
    getTokens,
    createToken,
    // ... exported methods
  }
}
```

**Key Features:**
- Automatic auth header injection (from Pinia auth store)
- Centralized error handling (parse error response, throw Error)
- Type-safe responses (generic `fetchApi<T>`)
- Base URL configuration (environment variable)

---

### API Endpoint Reference

#### Authentication

**POST /api/auth/login**
- Request: `{ username: string, password: string }`
- Response: `{ token: string, user: User }`
- Usage: Login flow (LoginView.vue)

**POST /api/auth/logout**
- Request: None
- Response: `{ success: boolean }`
- Usage: Logout flow (AppHeader.vue)

---

#### Tokens

**GET /api/tokens**
- Request: None
- Response: `TokenMetadata[]`
- Usage: Token list view (TokensView.vue)

**GET /api/tokens/:id**
- Request: None
- Response: `TokenMetadata`
- Usage: Token detail view (TokenDetailModal.vue)

**POST /api/tokens**
- Request: `CreateTokenRequest`
- Response: `CreateTokenResponse`
- Usage: Token creation (CreateTokenModal.vue)

**POST /api/tokens/:id/rotate**
- Request: None (empty body `{}`)
- Response: `CreateTokenResponse` (new token)
- Usage: Token rotation (TokenTable.vue)

**POST /api/tokens/:id/revoke**
- Request: None (empty body `{}`)
- Response: `{ success: boolean }`
- Usage: Token revocation (TokenTable.vue)

---

#### Usage

**GET /api/usage**
- Request: None
- Response: `UsageRecord[]`
- Usage: Usage history view (UsageView.vue)

**GET /api/usage/stats**
- Request: None
- Response: `UsageStats` (totals + breakdowns)
- Usage: Usage overview cards (UsageView.vue)

**GET /api/usage/token/:id**
- Request: None
- Response: `UsageRecord[]`
- Usage: Per-token usage view (TokenDetailView.vue)

---

#### Limits

**GET /api/limits**
- Request: None
- Response: `LimitRecord[]`
- Usage: Limits list view (LimitsView.vue)

**GET /api/limits/:id**
- Request: None
- Response: `LimitRecord`
- Usage: Limit detail view (LimitDetailModal.vue)

**POST /api/limits**
- Request: `CreateLimitRequest`
- Response: `LimitRecord`
- Usage: Limit creation (CreateLimitModal.vue)

**PUT /api/limits/:id**
- Request: `UpdateLimitRequest`
- Response: `LimitRecord`
- Usage: Limit update (EditLimitModal.vue)

**DELETE /api/limits/:id**
- Request: None
- Response: `{ success: boolean }`
- Usage: Limit deletion (LimitsView.vue)

---

#### Traces

**GET /api/traces**
- Request: None
- Response: `TraceRecord[]`
- Usage: Traces list view (TracesView.vue)

**GET /api/traces/:id**
- Request: None
- Response: `TraceRecord`
- Usage: Trace detail view (TraceDetailModal.vue)

---

## TypeScript Type Safety

### Type Mapping Strategy

**Rule:** Frontend TypeScript types MUST match backend Rust schemas exactly.

**Example Mapping:**

**Backend (Rust):**
```rust
// iron_control_api/src/models/token.rs
#[derive(Serialize, Deserialize)]
pub struct TokenMetadata {
    pub id: i64,
    pub user_id: String,
    pub project_id: Option<String>,
    pub name: Option<String>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub is_active: bool,
}
```

**Frontend (TypeScript):**
```typescript
// src/composables/useApi.ts
export interface TokenMetadata {
  id: number          // Rust i64 → TypeScript number
  user_id: string     // Rust String → TypeScript string
  project_id?: string // Rust Option<String> → TypeScript string | undefined
  name?: string       // Rust Option<String> → TypeScript string | undefined
  created_at: number  // Rust i64 → TypeScript number
  last_used_at?: number // Rust Option<i64> → TypeScript number | undefined
  is_active: boolean  // Rust bool → TypeScript boolean
}
```

**Type Conversion Rules:**

| Rust Type | TypeScript Type |
|-----------|----------------|
| `i32`, `i64`, `u32`, `u64` | `number` |
| `String` | `string` |
| `bool` | `boolean` |
| `Option<T>` | `T \| undefined` (optional field `T?`) |
| `Vec<T>` | `T[]` (array) |
| `HashMap<K, V>` | `Record<K, V>` |

---

### Type Definitions Location

**All API types defined in:** `src/composables/useApi.ts`

**Rationale:**
- Single source of truth (types co-located with API methods)
- Easy to update (change type + method in same file)
- No circular dependencies (composable has no dependencies)

**Exported Types:**

```typescript
// src/composables/useApi.ts
export type {
  TokenMetadata,
  CreateTokenRequest,
  CreateTokenResponse,
  UsageRecord,
  UsageStats,
  LimitRecord,
  CreateLimitRequest,
  UpdateLimitRequest,
  TraceRecord,
}
```

**Usage in Components:**

```vue
<script setup lang="ts">
import type { TokenMetadata } from '@/composables/useApi'

interface Props {
  token: TokenMetadata
}
defineProps<Props>()
</script>
```

---

### Type Validation (Runtime)

**Problem:** TypeScript types erased at runtime, no guarantee JSON matches types.

**Solution:** Validate critical API responses with type guards.

**Example:**

```typescript
function isTokenMetadata(obj: unknown): obj is TokenMetadata {
  if (typeof obj !== 'object' || obj === null) return false
  const token = obj as Record<string, unknown>
  return (
    typeof token.id === 'number' &&
    typeof token.user_id === 'string' &&
    typeof token.is_active === 'boolean' &&
    (token.project_id === undefined || typeof token.project_id === 'string') &&
    (token.name === undefined || typeof token.name === 'string')
  )
}

async function getTokens(): Promise<TokenMetadata[]> {
  const data = await fetchApi<TokenMetadata[]>('/api/tokens')
  if (!Array.isArray(data)) {
    throw new Error('Invalid response: expected array')
  }
  // Validate each item (optional, for critical endpoints)
  data.forEach((item, idx) => {
    if (!isTokenMetadata(item)) {
      throw new Error(`Invalid token at index ${idx}`)
    }
  })
  return data
}
```

**When to Use:**
- Critical data (authentication, tokens)
- User-provided data (form submissions)
- External APIs (not iron_control_api, which we control)

**When NOT to Use:**
- Internal iron_control_api responses (trust backend types)
- Non-critical data (usage stats, traces)

---

## WebSocket Integration

### WebSocket Configuration

**WebSocket URL:** `ws://localhost:8080/ws` (dev), configurable via `VITE_WS_URL`
**Protocol:** JSON messages
**Reconnect Strategy:** Exponential backoff (1s, 2s, 4s, 8s, max 30s)

---

### WebSocket Client (useWebSocket Composable)

**Location:** `src/composables/useWebSocket.ts` (future implementation)

**Example Implementation:**

```typescript
// src/composables/useWebSocket.ts
import { ref, onMounted, onUnmounted } from 'vue'

interface WebSocketMessage {
  type: 'trace' | 'agent_status' | 'budget_update'
  data: unknown
}

export function useWebSocket(url: string) {
  const socket = ref<WebSocket | null>(null)
  const connected = ref(false)
  const reconnectAttempts = ref(0)
  const maxReconnectDelay = 30000 // 30 seconds

  function connect() {
    socket.value = new WebSocket(url)

    socket.value.onopen = () => {
      console.log('WebSocket connected')
      connected.value = true
      reconnectAttempts.value = 0
    }

    socket.value.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data)
        handleMessage(message)
      } catch (error) {
        console.error('WebSocket message parse error:', error)
      }
    }

    socket.value.onerror = (error) => {
      console.error('WebSocket error:', error)
    }

    socket.value.onclose = () => {
      console.log('WebSocket disconnected')
      connected.value = false
      scheduleReconnect()
    }
  }

  function scheduleReconnect() {
    const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.value), maxReconnectDelay)
    console.log(`Reconnecting in ${delay}ms (attempt ${reconnectAttempts.value + 1})`)
    reconnectAttempts.value++
    setTimeout(() => {
      connect()
    }, delay)
  }

  function handleMessage(message: WebSocketMessage) {
    switch (message.type) {
      case 'trace':
        // Emit trace event (handle in component)
        break
      case 'agent_status':
        // Update agent status
        break
      case 'budget_update':
        // Update budget metrics
        break
      default:
        console.warn('Unknown message type:', message.type)
    }
  }

  function disconnect() {
    if (socket.value) {
      socket.value.close()
      socket.value = null
    }
  }

  onMounted(() => {
    connect()
  })

  onUnmounted(() => {
    disconnect()
  })

  return {
    connected,
    socket,
    disconnect,
  }
}
```

---

### WebSocket Message Format

**Message Structure:**

```typescript
interface WebSocketMessage {
  type: 'trace' | 'agent_status' | 'budget_update'
  data: unknown
}
```

**Example Messages:**

**Trace Message:**
```json
{
  "type": "trace",
  "data": {
    "id": 123,
    "token_id": 1,
    "request_id": "req_abc",
    "provider": "openai",
    "model": "gpt-4o",
    "input_tokens": 1234,
    "output_tokens": 567,
    "cost": 0.12,
    "timestamp": 1733404800
  }
}
```

**Agent Status Message:**
```json
{
  "type": "agent_status",
  "data": {
    "agent_id": "agent_001",
    "status": "RUNNING",
    "cost": 12.34,
    "duration_secs": 154
  }
}
```

**Budget Update Message:**
```json
{
  "type": "budget_update",
  "data": {
    "limit": 50.0,
    "spent": 23.14,
    "remaining": 26.86,
    "percentage_used": 0.4628
  }
}
```

---

### WebSocket Usage in Components

**Example (DashboardView.vue):**

```vue
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const wsUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:8080/ws'
let socket: WebSocket | null = null

const traces = ref<TraceRecord[]>([])
const agentStatus = ref<AgentStatus | null>(null)

function connectWebSocket() {
  socket = new WebSocket(wsUrl)

  socket.onmessage = (event) => {
    const message = JSON.parse(event.data)
    if (message.type === 'trace') {
      traces.value.unshift(message.data) // Add to top
    } else if (message.type === 'agent_status') {
      agentStatus.value = message.data
    }
  }

  socket.onerror = (error) => {
    console.error('WebSocket error:', error)
  }
}

onMounted(() => {
  connectWebSocket()
})

onUnmounted(() => {
  if (socket) {
    socket.close()
  }
})
</script>
```

---

## Error Handling

### Error Types

**1. Network Errors**
- Cause: Backend offline, network issues
- Status: N/A (fetch throws)
- Handling: Show "Connection error" message, retry button

**2. HTTP Errors**
- Cause: Invalid request, server error
- Status: 400-599
- Handling: Parse error response JSON, display error message

**3. Parse Errors**
- Cause: Invalid JSON response
- Status: 200 (but body invalid)
- Handling: Show "Invalid response" message, log to console

---

### Error Handling Pattern

**API Client Error Handling:**

```typescript
async function fetchApi<T>(path: string, options: RequestInit = {}): Promise<T> {
  try {
    const response = await fetch(`${API_BASE_URL}${path}`, {
      ...options,
      headers,
    })

    if (!response.ok) {
      // HTTP error (400-599)
      const error = await response.json().catch(() => ({ error: 'Request failed' }))
      throw new Error(error.error || `HTTP ${response.status}`)
    }

    // Success (200-299)
    return response.json()
  } catch (error) {
    // Network error or parse error
    if (error instanceof Error) {
      throw error
    } else {
      throw new Error('Unknown error occurred')
    }
  }
}
```

**Component Error Handling:**

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { useApi } from '@/composables/useApi'

const api = useApi()
const error = ref<string | null>(null)
const loading = ref(false)

async function createToken(data: CreateTokenRequest) {
  loading.value = true
  error.value = null

  try {
    const response = await api.createToken(data)
    console.log('Token created:', response.token)
    // Success handling
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Unknown error'
    console.error('Failed to create token:', err)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div>
    <p v-if="error" class="text-red-600">{{ error }}</p>
    <button @click="createToken(formData)" :disabled="loading">
      {{ loading ? 'Creating...' : 'Create Token' }}
    </button>
  </div>
</template>
```

---

### TanStack Query Error Handling

**Automatic Error States:**

```vue
<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { useApi } from '@/composables/useApi'

const api = useApi()

const { data, isLoading, error } = useQuery({
  queryKey: ['tokens'],
  queryFn: () => api.getTokens(),
  retry: 3, // Retry 3 times before giving up
  retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
})
</script>

<template>
  <div>
    <p v-if="isLoading">Loading...</p>
    <p v-else-if="error" class="text-red-600">{{ error.message }}</p>
    <ul v-else>
      <li v-for="token in data" :key="token.id">{{ token.name }}</li>
    </ul>
  </div>
</template>
```

---

## Authentication Flow

### JWT Token Flow

**1. Login:**
```
User → LoginView → POST /api/auth/login
                    ↓
Backend returns { token: "jwt_...", user: {...} }
                    ↓
Frontend stores token in localStorage (key: "auth_token")
                    ↓
Frontend stores user in Pinia auth store
                    ↓
Frontend redirects to /dashboard
```

**2. Authenticated Requests:**
```
Component → useApi().getTokens()
            ↓
fetchApi() reads token from Pinia auth store
            ↓
fetchApi() adds Authorization: Bearer <token> header
            ↓
Backend validates JWT, returns data
```

**3. Token Expiration:**
```
Backend returns 401 Unauthorized
            ↓
Frontend detects 401 in error handler
            ↓
Frontend clears token from localStorage
            ↓
Frontend redirects to /login
```

---

### Auth Store Integration

**Pinia Auth Store:**

```typescript
// src/stores/auth.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('auth_token'))

  const isAuthenticated = computed(() => !!token.value)

  function login(newToken: string) {
    token.value = newToken
    localStorage.setItem('auth_token', newToken)
  }

  function logout() {
    token.value = null
    localStorage.removeItem('auth_token')
  }

  function getAuthHeader(): string | null {
    return token.value ? `Bearer ${token.value}` : null
  }

  return { token, isAuthenticated, login, logout, getAuthHeader }
})
```

**Router Guard:**

```typescript
// src/router/index.ts
router.beforeEach((to, from, next) => {
  const authStore = useAuthStore()

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    next('/login')
  } else {
    next()
  }
})
```

---

## Adding New Endpoints

### Step-by-Step Guide

**1. Add Backend Endpoint (iron_control_api)**

```rust
// iron_control_api/src/routes/example.rs
#[derive(Serialize, Deserialize)]
pub struct ExampleData {
    pub id: i64,
    pub name: String,
}

pub async fn get_example(Path(id): Path<i64>) -> Result<Json<ExampleData>, AppError> {
    // Implementation
    Ok(Json(ExampleData { id, name: "Example".to_string() }))
}
```

**2. Add TypeScript Type (Frontend)**

```typescript
// src/composables/useApi.ts
export interface ExampleData {
  id: number
  name: string
}
```

**3. Add API Method**

```typescript
// src/composables/useApi.ts
export function useApi() {
  // ... existing code

  async function getExample(id: number): Promise<ExampleData> {
    return fetchApi<ExampleData>(`/api/example/${id}`)
  }

  return {
    // ... existing methods
    getExample,
  }
}
```

**4. Export Type**

```typescript
// src/composables/useApi.ts
export type {
  // ... existing types
  ExampleData,
}
```

**5. Use in Component**

```vue
<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { useApi } from '@/composables/useApi'

const api = useApi()

const { data, isLoading } = useQuery({
  queryKey: ['example', 1],
  queryFn: () => api.getExample(1),
})
</script>

<template>
  <div>
    <p v-if="isLoading">Loading...</p>
    <p v-else>{{ data?.name }}</p>
  </div>
</template>
```

---

### Checklist

When adding new endpoint:

- [ ] Backend endpoint implemented (iron_control_api)
- [ ] Backend types defined (Rust structs with Serialize/Deserialize)
- [ ] Frontend types defined (TypeScript interfaces in useApi.ts)
- [ ] Types match exactly (Rust ↔ TypeScript)
- [ ] API method added to useApi composable
- [ ] Type exported from useApi.ts
- [ ] Component uses TanStack Query for data fetching
- [ ] Error handling implemented (display error message)
- [ ] Loading state implemented (show loading indicator)

---

## Integration Testing

### Manual Testing Checklist

For new endpoints, verify:

- [ ] **Request Success**: Endpoint returns 200 with valid data
- [ ] **Request Failure**: Endpoint returns 4xx/5xx with error message
- [ ] **Type Safety**: Response matches TypeScript type (no runtime errors)
- [ ] **Authentication**: 401 returned when token missing/invalid
- [ ] **Loading State**: UI shows loading indicator during request
- [ ] **Error Display**: UI shows error message on failure
- [ ] **Data Display**: UI correctly displays response data

---

## Known Integration Issues

**Current Issues (as of migration):**

1. **No WebSocket reconnection UI** - No visible indicator when connection drops
   - Workaround: Refresh page manually
   - Fix: Add "Offline" badge in header

2. **No loading states for API calls** - Tables appear empty during loading
   - Workaround: Fast API responses on localhost
   - Fix: Add skeleton loaders (Radix Vue Skeleton)

3. **No retry for failed requests** - Single failure = error displayed
   - Workaround: User manually retries (refresh page)
   - Fix: TanStack Query retry configuration (already supports, needs UI)

---

**End of API Integration Documentation**
