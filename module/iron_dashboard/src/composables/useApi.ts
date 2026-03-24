import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3001'

// Prevents concurrent 401 responses from each triggering an independent
// refresh request (token refresh race condition).
let _refreshPromise: Promise<void> | null = null

// Prevents concurrent refresh-failure handlers from each firing an independent
// server-side logout revocation request.
let _logoutPromise: Promise<void> | null = null

interface TokenMetadata {
  id: number
  user_id: string
  agent_id?: number
  provider?: string
  name?: string
  created_at: number
  last_used_at?: number
  is_active: boolean
}

interface CreateTokenRequest {
  user_id: string
  project_id?: string
  description?: string
}

interface CreateTokenResponse {
  id: number
  token: string
  user_id: string
  provider?: string
  description?: string
  created_at: number
}

// AI Provider Key types
type ProviderType = 'openai' | 'anthropic' | 'gemini' | 'xai'

interface ProviderKey {
  id: number
  provider: ProviderType
  alias?: string
  base_url?: string
  description?: string
  is_enabled: boolean
  created_at: number
  last_used_at?: number
  masked_key: string
  assigned_projects: string[]
  total_spend_usd: number
  spending_cap_microdollars: number | null
  spending_used_microdollars: number
}

interface CreateProviderKeyRequest {
  provider: ProviderType
  api_key: string
  alias?: string
  base_url?: string
  description?: string
}

interface UpdateProviderKeyRequest {
  alias?: string
  base_url?: string
  description?: string
  is_enabled?: boolean
}

export interface User {
  id: string
  username: string
  email?: string
  role: string
  is_active: boolean
  created_at: number
  last_login?: number
  suspended_at?: number
  deleted_at?: number
}

export interface CreateUserRequest {
  username: string
  password: string
  email: string
  role?: string
}

export interface Agent {
  id: number
  name: string
  providers: string[]
  created_at: number
  owner_id?: string
  provider_key_ids: number[]
}

export interface AgentBudgetResponse {
  agent_id: number
  total_allocated: number    // microdollars
  total_spent: number        // microdollars
  budget_remaining: number   // microdollars
}

// IC Token types
export interface IcTokenResponse {
  agent_id: number
  ic_token: string
  created_at: number
  warning: string
  old_token_invalidated?: boolean
}

export interface IcTokenStatus {
  agent_id: number
  has_ic_token: boolean
  created_at: number | null
}

// ============================================================================
// Analytics Types (Protocol 012)
// ============================================================================

export type AnalyticsPeriod =
  | 'today'
  | 'yesterday'
  | 'last7-days'
  | 'last30-days'
  | 'this-month'
  | 'last-month'
  | 'all-time'

export interface AnalyticsFilters {
  period?: AnalyticsPeriod
  agent_id?: number
  provider_id?: string
  provider_key_id?: number
  compare?: boolean
}

export interface PaginationParams {
  page?: number
  per_page?: number
}

export interface SpendingTotalComparison {
  total_spend: number
  change_percent: number | null
}

export interface SpendingTotalResponse {
  total_spend: number
  currency: string
  period: string
  filters: { agent_id?: number; provider_id?: string }
  previous_period?: SpendingTotalComparison
  calculated_at: string
}

export interface ProviderSpending {
  provider: string
  provider_key_id?: number
  alias?: string
  spending: number
  request_count: number
  avg_cost_per_request: number
  agent_count: number
}

export interface SpendingByProviderResponse {
  data: ProviderSpending[]
  summary: { total_spend: number; total_requests: number; providers_count: number }
  period: string
  calculated_at: string
}

export interface Pagination {
  page: number
  per_page: number
  total: number
  total_pages: number
}

// Budget status (agent budgets)
export interface BudgetStatusResponse {
  data: BudgetStatus[]
  summary: {
    total_agents: number
    active: number
    exhausted: number
    critical: number
    high: number
    medium: number
    low: number
  }
  pagination: Pagination
  calculated_at: string
}

export interface BudgetStatus {
  agent_id: number
  agent_name: string
  budget: number
  spent: number
  remaining: number
  percent_used: number
  status: string
  risk_level: string
}

export interface RequestUsageComparison {
  total_requests: number
  successful_requests: number
  failed_requests: number
  success_rate: number
  change_percent: number | null
}

export interface RequestUsageResponse {
  total_requests: number
  successful_requests: number
  failed_requests: number
  success_rate: number
  period: string
  filters: { agent_id?: number; provider_id?: string }
  previous_period?: RequestUsageComparison
  calculated_at: string
}

export interface ModelUsage {
  model: string
  provider: string
  request_count: number
  spending: number
  input_tokens: number
  output_tokens: number
}

export interface ModelUsageResponse {
  data: ModelUsage[]
  summary: { unique_models: number; total_requests: number; total_spend: number }
  pagination: { page: number; per_page: number; total: number; total_pages: number }
  period: string
  calculated_at: string
}

export interface AnalyticsEvent {
  event_id: string
  timestamp_ms: number
  event_type: string
  model: string
  provider: string
  input_tokens: number
  output_tokens: number
  cost_micros: number
  agent_id: number
  agent_name: string
  error_code?: string
  error_message?: string
}

export interface EventsListResponse {
  data: AnalyticsEvent[]
  pagination: { page: number; per_page: number; total: number; total_pages: number }
  period: string
  calculated_at: string
}

export interface AgentSpending {
  agent_id: number
  agent_name: string
  /** Total spending in USD */
  spending: number
  /** Allocated budget in USD (NOT microdollars) */
  budget: number
  percent_used: number
  request_count: number
}

export interface SpendingByAgentResponse {
  data: AgentSpending[]
  summary: {
    total_spend: number
    total_budget: number
    total_agents: number
  }
  pagination: Pagination
  period: string
  calculated_at: string
}

export interface AgentTokenUsage {
  agent_id: number
  agent_name: string
  input_tokens: number
  output_tokens: number
  total_tokens: number
  request_count: number
  avg_tokens_per_request: number
}

export interface TokenUsageByAgentResponse {
  data: AgentTokenUsage[]
  summary: {
    total_input_tokens: number
    total_output_tokens: number
    total_tokens: number
  }
  pagination: Pagination
  period: string
  calculated_at: string
}

export interface AvgCostResponse {
  average_cost_per_request: number
  total_requests: number
  total_spend: number
  median_cost_per_request: number
  min_cost_per_request: number
  max_cost_per_request: number
  period: string
  filters: { agent_id: number | null; provider_id: string | null }
  calculated_at: string
}

// ============================================================================
// useApi composable
// ============================================================================

export function useApi() {
  const authStore = useAuthStore()
  const router = useRouter()

  async function fetchApi<T>(path: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...((options.headers as Record<string, string>) || {}),
    }

    const authHeader = authStore.getAuthHeader()

    if (authHeader) {
      headers['Authorization'] = authHeader
    }

    let response = await fetch(`${API_BASE_URL}${path}`, {
      ...options,
      headers,
    })

    // Attempt token refresh on 401, retry once
    if (response.status === 401 && authStore.refreshToken) {
      try {
        if (!_refreshPromise) {
          _refreshPromise = authStore.refresh().finally(() => { _refreshPromise = null })
        }
        await _refreshPromise
        const newAuth = authStore.getAuthHeader()
        if (newAuth) headers['Authorization'] = newAuth
        response = await fetch(`${API_BASE_URL}${path}`, { ...options, headers })
        // Refresh succeeded but retried request is still 401 — treat as full session expiry
        if (response.status === 401) {
          if (!_logoutPromise) {
            _logoutPromise = authStore.logout()
              .then(() => { router.replace('/login') })
              .finally(() => { _logoutPromise = null })
          }
          await _logoutPromise
          throw new Error('Session expired')
        }
      } catch (err) {
        if (err instanceof Error && err.message === 'Session expired') throw err
        if (err instanceof TypeError) throw err
        if (!_logoutPromise) {
          _logoutPromise = authStore.logout()
            .then(() => { router.replace('/login') })
            .finally(() => { _logoutPromise = null })
        }
        await _logoutPromise
        throw new Error('Session expired')
      }
    } else if (response.status === 401) {
      if (!_logoutPromise) {
        _logoutPromise = authStore.logout()
          .then(() => { router.replace('/login') })
          .finally(() => { _logoutPromise = null })
      }
      await _logoutPromise
      throw new Error('Session expired')
    }

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Request failed' }))
      throw new Error(error.error || `HTTP ${response.status}`)
    }

    // Handle empty responses (204 No Content, or empty body)
    const text = await response.text()
    if (!text) {
      return undefined as T
    }
    try {
      return JSON.parse(text) as T
    } catch {
      throw new Error('Invalid response from server (expected JSON)')
    }
  }

  // Health API
  async function getHealth(): Promise<{ status: string; timestamp: number }> {
    return fetchApi('/api/health')
  }

  // Token API methods
  async function getTokens(): Promise<TokenMetadata[]> {
    return fetchApi<TokenMetadata[]>('/api/v1/api-tokens')
  }

  async function getToken(id: number): Promise<TokenMetadata> {
    return fetchApi<TokenMetadata>(`/api/v1/api-tokens/${id}`)
  }

  async function createToken(data: CreateTokenRequest): Promise<CreateTokenResponse> {
    return fetchApi<CreateTokenResponse>('/api/v1/api-tokens', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async function rotateToken(id: number): Promise<CreateTokenResponse> {
    return fetchApi<CreateTokenResponse>(`/api/v1/api-tokens/${id}/rotate`, {
      method: 'POST',
      body: JSON.stringify({}),
    })
  }

  async function revokeToken(id: number): Promise<void> {
    await fetchApi<void>(`/api/v1/api-tokens/${id}`, {
      method: 'DELETE',
    })
  }

  // Provider Key API methods
  async function getProviderKeys(): Promise<ProviderKey[]> {
    return fetchApi<ProviderKey[]>('/api/v1/providers')
  }

  async function getProviderKey(id: number): Promise<ProviderKey> {
    return fetchApi<ProviderKey>(`/api/v1/providers/${id}`)
  }

  async function createProviderKey(data: CreateProviderKeyRequest): Promise<ProviderKey> {
    return fetchApi<ProviderKey>('/api/v1/providers', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async function updateProviderKey(id: number, data: UpdateProviderKeyRequest): Promise<ProviderKey> {
    return fetchApi<ProviderKey>(`/api/v1/providers/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async function deleteProviderKey(id: number): Promise<void> {
    await fetchApi<void>(`/api/v1/providers/${id}`, {
      method: 'DELETE',
    })
  }

  // User API methods
  async function getUsers(params?: { role?: string; is_active?: boolean; search?: string; page?: number; page_size?: number }): Promise<{ users: User[]; total: number; page: number; page_size: number }> {
    const query = new URLSearchParams()
    if (params?.role) query.append('role', params.role)
    if (params?.is_active !== undefined) query.append('is_active', String(params.is_active))
    if (params?.search) query.append('search', params.search)
    if (params?.page) query.append('page', String(params.page))
    if (params?.page_size) query.append('page_size', String(params.page_size))

    return fetchApi<{ users: User[]; total: number; page: number; page_size: number }>(`/api/v1/users?${query.toString()}`)
  }

  async function createUser(data: CreateUserRequest): Promise<User> {
    return fetchApi<User>('/api/v1/users', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async function updateUserStatus(id: string, isActive: boolean): Promise<User> {
    if (isActive) {
      return activateUser(id)
    } else {
      return suspendUser(id)
    }
  }

  async function suspendUser(id: string, reason?: string): Promise<User> {
    return fetchApi<User>(`/api/v1/users/${id}/suspend`, {
      method: 'PUT',
      body: JSON.stringify({ reason }),
    })
  }

  async function activateUser(id: string): Promise<User> {
    return fetchApi<User>(`/api/v1/users/${id}/activate`, {
      method: 'PUT',
    })
  }

  async function changeUserRole(id: string, role: string): Promise<User> {
    return fetchApi<User>(`/api/v1/users/${id}/role`, {
      method: 'PUT',
      body: JSON.stringify({ role }),
    })
  }

  async function resetUserPassword(id: string, newPassword: string, forceChange: boolean): Promise<User> {
    return fetchApi<User>(`/api/v1/users/${id}/reset-password`, {
      method: 'POST',
      body: JSON.stringify({ new_password: newPassword, force_change: forceChange }),
    })
  }

  async function deleteUser(id: string): Promise<void> {
    await fetchApi<void>(`/api/v1/users/${id}`, {
      method: 'DELETE',
    })
  }

  // Agent API methods
  async function getAgents(): Promise<Agent[]> {
    return fetchApi<Agent[]>('/api/v1/agents')
  }

  async function getAgent(id: number): Promise<Agent> {
    return fetchApi<Agent>(`/api/v1/agents/${id}`)
  }

  async function createAgent(data: {
    name: string
    providers: string[]
    provider_key_ids: number[]
    initial_budget_microdollars: number
    owner_id?: string  // Admins can assign to other users
  }): Promise<Agent> {
    return fetchApi<Agent>('/api/v1/agents', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async function updateAgent(data: {
    id: number
    name?: string
    providers?: string[]
    provider_key_ids?: number[]
    owner_id?: string  // Admins can reassign to other users
  }): Promise<Agent> {
    const { id, ...updateData } = data
    return fetchApi<Agent>(`/api/v1/agents/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updateData),
    })
  }

  async function updateAgentBudget(agentId: number, total_allocated_microdollars: number): Promise<AgentBudgetResponse> {
    return fetchApi<AgentBudgetResponse>(`/api/v1/agents/${agentId}/budget`, {
      method: 'PUT',
      body: JSON.stringify({ total_allocated_microdollars }),
    })
  }

  async function deleteAgent(id: number): Promise<void> {
    await fetchApi<void>(`/api/v1/agents/${id}`, {
      method: 'DELETE',
    })
  }

  // ============================================================================
  // IC Token API (Agent Runtime Authentication)
  // ============================================================================

  async function generateIcToken(agentId: number): Promise<IcTokenResponse> {
    return fetchApi<IcTokenResponse>(`/api/v1/agents/${agentId}/ic-token`, {
      method: 'POST',
    })
  }

  async function getIcTokenStatus(agentId: number): Promise<IcTokenStatus> {
    return fetchApi<IcTokenStatus>(`/api/v1/agents/${agentId}/ic-token`)
  }

  async function regenerateIcToken(agentId: number): Promise<IcTokenResponse> {
    return fetchApi<IcTokenResponse>(`/api/v1/agents/${agentId}/ic-token/regenerate`, {
      method: 'POST',
    })
  }

  async function revokeIcToken(agentId: number): Promise<void> {
    await fetchApi<void>(`/api/v1/agents/${agentId}/ic-token`, {
      method: 'DELETE',
    })
  }

  // ============================================================================
  // Analytics API (Protocol 012)
  // ============================================================================

  async function getAnalyticsSpendingTotal(
    filters?: AnalyticsFilters,
    signal?: AbortSignal
  ): Promise<SpendingTotalResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.provider_id) params.append('provider_id', filters.provider_id)
    if (filters?.provider_key_id) params.append('provider_key_id', String(filters.provider_key_id))
    if (filters?.compare) params.append('compare', 'true')
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/spending/total${query ? `?${query}` : ''}`, { signal })
  }

  async function getAnalyticsSpendingByProvider(
    filters?: AnalyticsFilters,
    signal?: AbortSignal
  ): Promise<SpendingByProviderResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.provider_id) params.append('provider_id', filters.provider_id)
    if (filters?.provider_key_id) params.append('provider_key_id', String(filters.provider_key_id))
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/spending/by-provider${query ? `?${query}` : ''}`, { signal })
  }

  async function getAnalyticsUsageRequests(
    filters?: AnalyticsFilters,
    signal?: AbortSignal
  ): Promise<RequestUsageResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.provider_id) params.append('provider_id', filters.provider_id)
    if (filters?.provider_key_id) params.append('provider_key_id', String(filters.provider_key_id))
    if (filters?.compare) params.append('compare', 'true')
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/usage/requests${query ? `?${query}` : ''}`, { signal })
  }

  async function getAnalyticsUsageModels(
    filters?: AnalyticsFilters,
    pagination?: PaginationParams,
    signal?: AbortSignal
  ): Promise<ModelUsageResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.provider_id) params.append('provider_id', filters.provider_id)
    if (filters?.provider_key_id) params.append('provider_key_id', String(filters.provider_key_id))
    if (pagination?.page) params.append('page', String(pagination.page))
    if (pagination?.per_page) params.append('per_page', String(pagination.per_page))
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/usage/models${query ? `?${query}` : ''}`, { signal })
  }

  async function getAnalyticsEventsList(
    filters?: AnalyticsFilters,
    pagination?: PaginationParams,
    signal?: AbortSignal
  ): Promise<EventsListResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.provider_id) params.append('provider_id', filters.provider_id)
    if (filters?.provider_key_id) params.append('provider_key_id', String(filters.provider_key_id))
    if (pagination?.page) params.append('page', String(pagination.page))
    if (pagination?.per_page) params.append('per_page', String(pagination.per_page))
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/events/list${query ? `?${query}` : ''}`, { signal })
  }

  async function getBudgetStatus(filters?: {
    status?: string
    threshold?: number
    agent_id?: number
    page?: number
    per_page?: number
  }, signal?: AbortSignal): Promise<BudgetStatusResponse> {
    const params = new URLSearchParams()
    if (filters?.status) params.append('status', filters.status)
    if (filters?.threshold != null) params.append('threshold', String(filters.threshold))
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.page) params.append('page', String(filters.page))
    if (filters?.per_page) params.append('per_page', String(filters.per_page))
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/budget/status${query ? `?${query}` : ''}`, { signal })
  }

  async function getAnalyticsSpendingByAgent(
    filters?: AnalyticsFilters,
    pagination?: PaginationParams,
    signal?: AbortSignal
  ): Promise<SpendingByAgentResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.provider_id) params.append('provider_id', filters.provider_id)
    if (filters?.provider_key_id) params.append('provider_key_id', String(filters.provider_key_id))
    if (pagination?.page) params.append('page', String(pagination.page))
    if (pagination?.per_page) params.append('per_page', String(pagination.per_page))
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/spending/by-agent${query ? `?${query}` : ''}`, { signal })
  }

  async function getAnalyticsSpendingAvgPerRequest(
    filters?: AnalyticsFilters,
    signal?: AbortSignal
  ): Promise<AvgCostResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.provider_id) params.append('provider_id', filters.provider_id)
    if (filters?.provider_key_id) params.append('provider_key_id', String(filters.provider_key_id))
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/spending/avg-per-request${query ? `?${query}` : ''}`, { signal })
  }

  async function getAnalyticsUsageTokensByAgent(
    filters?: AnalyticsFilters,
    pagination?: PaginationParams,
    signal?: AbortSignal
  ): Promise<TokenUsageByAgentResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.provider_id) params.append('provider_id', filters.provider_id)
    if (filters?.provider_key_id) params.append('provider_key_id', String(filters.provider_key_id))
    if (pagination?.page) params.append('page', String(pagination.page))
    if (pagination?.per_page) params.append('per_page', String(pagination.per_page))
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/usage/tokens/by-agent${query ? `?${query}` : ''}`, { signal })
  }

  return {
    getHealth,
    getTokens,
    getToken,
    createToken,
    rotateToken,
    revokeToken,
    getProviderKeys,
    getProviderKey,
    createProviderKey,
    updateProviderKey,
    deleteProviderKey,
    getUsers,
    createUser,
    updateUserStatus,
    suspendUser,
    activateUser,
    changeUserRole,
    resetUserPassword,
    deleteUser,
    // Agent methods
    getAgents,
    getAgent,
    createAgent,
    updateAgent,
    updateAgentBudget,
    deleteAgent,
    // IC Token methods (agent runtime authentication)
    generateIcToken,
    getIcTokenStatus,
    regenerateIcToken,
    revokeIcToken,
    // Analytics (Protocol 012)
    getAnalyticsSpendingTotal,
    getAnalyticsSpendingByProvider,
    getAnalyticsSpendingByAgent,
    getAnalyticsSpendingAvgPerRequest,
    getAnalyticsUsageTokensByAgent,
    getAnalyticsUsageRequests,
    getAnalyticsUsageModels,
    getAnalyticsEventsList,
    getBudgetStatus,
  }
}

export type {
  TokenMetadata,
  CreateTokenRequest,
  CreateTokenResponse,
  ProviderType,
  ProviderKey,
  CreateProviderKeyRequest,
  UpdateProviderKeyRequest,
}
