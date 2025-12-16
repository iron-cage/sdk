import { useAuthStore } from '../stores/auth'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3001'

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

interface UsageRecord {
  id: number
  token_id: number
  provider: string
  model: string
  input_tokens: number
  output_tokens: number
  cost: number
  timestamp: number
}

interface UsageStats {
  total_requests: number
  total_input_tokens: number
  total_output_tokens: number
  total_cost: number
  by_provider: {
    provider: string
    requests: number
    cost: number
  }[]
  by_model: {
    model: string
    requests: number
    cost: number
  }[]
}

interface LimitRecord {
  id: number
  user_id: string
  project_id?: string
  max_tokens_per_day?: number
  max_requests_per_minute?: number
  max_cost_per_month_microdollars?: number  // Backend uses microdollars (1 cent = 10,000 microdollars)
  created_at: number
}

interface CreateLimitRequest {
  user_id: string
  project_id?: string
  max_tokens_per_day?: number
  max_requests_per_minute?: number
  max_cost_per_month_microdollars?: number  // Backend uses microdollars
}

interface UpdateLimitRequest {
  max_tokens_per_day?: number
  max_requests_per_minute?: number
  max_cost_per_month_microdollars?: number  // Backend uses microdollars
}

// AI Provider Key types
type ProviderType = 'openai' | 'anthropic'

interface ProviderKey {
  id: number
  provider: ProviderType
  base_url?: string
  description?: string
  is_enabled: boolean
  created_at: number
  last_used_at?: number
  masked_key: string
  assigned_projects: string[]
}

interface CreateProviderKeyRequest {
  provider: ProviderType
  api_key: string
  base_url?: string
  description?: string
}

interface UpdateProviderKeyRequest {
  base_url?: string
  description?: string
  is_enabled?: boolean
}

interface AssignProviderRequest {
  provider_key_id: number
}

// Budget Request Workflow types
interface BudgetRequest {
  id: string
  agent_id: number
  requester_id: string
  current_budget_usd: number
  requested_budget_usd: number
  justification: string
  status: 'pending' | 'approved' | 'rejected' | 'cancelled'
  created_at: number
  updated_at: number
}

interface CreateBudgetRequestRequest {
  agent_id: number
  requester_id: string
  requested_budget_usd: number
  justification: string
}

interface CreateBudgetRequestResponse {
  request_id: string
  status: string
  created_at: number
}

interface ListBudgetRequestsResponse {
  requests: BudgetRequest[]
}

interface ApproveBudgetRequestResponse {
  request_id: string
  status: string
  approved_at: number
}

interface RejectBudgetRequestRequest {
  rejection_reason: string
}

interface RejectBudgetRequestResponse {
  request_id: string
  status: string
  rejected_at: number
}

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

    // Handle empty responses (204 No Content, or empty body)
    const text = await response.text()
    if (!text) {
      return undefined as T
    }
    return JSON.parse(text)
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

  // Usage API methods
  async function getUsage(): Promise<UsageRecord[]> {
    // Backend doesnt have a /api/usage endpoint - return empty array for now
    // TODO: Add backend endpoint or fetch from tokens
    return Promise.resolve([])
  }

  async function getUsageStats(): Promise<UsageStats> {
    // Map backend /api/usage/aggregate to frontend format
    const aggregate = await fetchApi<{
      total_tokens: number
      total_requests: number
      total_cost_cents: number
      providers: Array<{
        provider: string
        tokens: number
        requests: number
        cost_cents: number
      }>
    }>('/api/v1/usage/aggregate')

    return {
      total_requests: aggregate.total_requests,
      total_input_tokens: 0, // Backend doesnt track separately
      total_output_tokens: aggregate.total_tokens,
      total_cost: aggregate.total_cost_cents / 100, // Convert cents to dollars
      by_provider: aggregate.providers.map(p => ({
        provider: p.provider,
        requests: p.requests,
        cost: p.cost_cents / 100, // Convert cents to dollars
      })),
      by_model: [], // Backend doesnt track by model
    }
  }

  async function getUsageByToken(tokenId: number): Promise<UsageRecord[]> {
    return fetchApi<UsageRecord[]>(`/api/v1/usage/token/${tokenId}`)
  }

  // Limits API methods
  async function getLimits(): Promise<LimitRecord[]> {
    return fetchApi<LimitRecord[]>('/api/v1/limits')
  }

  async function getLimit(id: number): Promise<LimitRecord> {
    return fetchApi<LimitRecord>(`/api/v1/limits/${id}`)
  }

  async function createLimit(data: CreateLimitRequest): Promise<LimitRecord> {
    return fetchApi<LimitRecord>('/api/v1/limits', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async function updateLimit(id: number, data: UpdateLimitRequest): Promise<LimitRecord> {
    return fetchApi<LimitRecord>(`/api/v1/limits/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async function deleteLimit(id: number): Promise<void> {
    await fetchApi<void>(`/api/v1/limits/${id}`, {
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

  async function assignProjectProvider(projectId: string, keyId: number): Promise<void> {
    await fetchApi<void>(`/api/v1/projects/${projectId}/provider`, {
      method: 'POST',
      body: JSON.stringify({ provider_key_id: keyId }),
    })
  }

  async function unassignProjectProvider(projectId: string): Promise<void> {
    await fetchApi<void>(`/api/v1/projects/${projectId}/provider`, {
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

  async function updateUserStatus(id: number, isActive: boolean): Promise<User> {
    if (isActive) {
      return activateUser(id)
    } else {
      return suspendUser(id)
    }
  }

  async function suspendUser(id: number, reason?: string): Promise<User> {
    return fetchApi<User>(`/api/v1/users/${id}/suspend`, {
      method: 'PUT',
      body: JSON.stringify({ reason }),
    })
  }

  async function activateUser(id: number): Promise<User> {
    return fetchApi<User>(`/api/v1/users/${id}/activate`, {
      method: 'PUT',
    })
  }

  async function changeUserRole(id: number, role: string): Promise<User> {
    return fetchApi<User>(`/api/v1/users/${id}/role`, {
      method: 'PUT',
      body: JSON.stringify({ role }),
    })
  }

  async function resetUserPassword(id: number, newPassword: string, forceChange: boolean): Promise<User> {
    return fetchApi<User>(`/api/v1/users/${id}/reset-password`, {
      method: 'POST',
      body: JSON.stringify({ new_password: newPassword, force_change: forceChange }),
    })
  }

  async function deleteUser(id: number): Promise<{ success: boolean }> {
    return fetchApi<{ success: boolean }>(`/api/v1/users/${id}`, {
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

  async function createAgent(data: { name: string; providers: string[]; provider_key_id: number; initial_budget_microdollars: number }): Promise<Agent> {
    return fetchApi<Agent>('/api/v1/agents', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async function updateAgent(data: { id: number; name?: string; providers?: string[]; provider_key_id?: number | null }): Promise<Agent> {
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

  async function getAgentTokens(agentId: number): Promise<TokenMetadata[]> {
    return fetchApi<TokenMetadata[]>(`/api/v1/agents/${agentId}/tokens`)
  }

  async function createAgentToken(data: { agent_id: number; user_id: string; provider: string; description?: string }): Promise<CreateTokenResponse> {
    // TODO: Update backend to accept agent_id and provider
    return fetchApi<CreateTokenResponse>('/api/v1/api-tokens', {
      method: 'POST',
      body: JSON.stringify({
        user_id: data.user_id,
        description: data.description,
        // Backend needs to be updated to accept these fields
        agent_id: data.agent_id,
        provider: data.provider,
      }),
    })
  }

  async function updateTokenProvider(tokenId: number, provider: string): Promise<void> {
    // TODO: Add backend endpoint for updating token provider
    // For now, this is a placeholder
    await fetchApi<void>(`/api/v1/api-tokens/${tokenId}`, {
      method: 'PUT',
      body: JSON.stringify({ provider }),
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
    filters?: AnalyticsFilters
  ): Promise<SpendingTotalResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.provider_id) params.append('provider_id', filters.provider_id)
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/spending/total${query ? `?${query}` : ''}`)
  }

  async function getAnalyticsSpendingByProvider(
    filters?: AnalyticsFilters
  ): Promise<SpendingByProviderResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/spending/by-provider${query ? `?${query}` : ''}`)
  }

  async function getAnalyticsUsageRequests(
    filters?: AnalyticsFilters
  ): Promise<RequestUsageResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (filters?.agent_id) params.append('agent_id', String(filters.agent_id))
    if (filters?.provider_id) params.append('provider_id', filters.provider_id)
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/usage/requests${query ? `?${query}` : ''}`)
  }

  async function getAnalyticsUsageModels(
    filters?: AnalyticsFilters,
    pagination?: PaginationParams
  ): Promise<ModelUsageResponse> {
    const params = new URLSearchParams()
    if (filters?.period) params.append('period', filters.period)
    if (pagination?.page) params.append('page', String(pagination.page))
    if (pagination?.per_page) params.append('per_page', String(pagination.per_page))
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/usage/models${query ? `?${query}` : ''}`)
  }

  async function getBudgetStatus(
    page?: number,
    per_page?: number
  ): Promise<BudgetStatusResponse> {
    const params = new URLSearchParams()
    if (page) params.append('page', String(page))
    if (per_page) params.append('per_page', String(per_page))
    const query = params.toString()
    return fetchApi(`/api/v1/analytics/budget/status${query ? `?${query}` : ''}`)
  }

  // ============================================================================
  // Budget Request Workflow API
  // ============================================================================

  async function createBudgetRequest(
    data: CreateBudgetRequestRequest
  ): Promise<CreateBudgetRequestResponse> {
    return fetchApi<CreateBudgetRequestResponse>('/api/v1/budget/requests', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async function getBudgetRequest(requestId: string): Promise<BudgetRequest> {
    return fetchApi<BudgetRequest>(`/api/v1/budget/requests/${requestId}`)
  }

  async function listBudgetRequests(filters?: {
    status?: string
    requester_id?: string
    start_date?: number
    end_date?: number
  }): Promise<ListBudgetRequestsResponse> {
    const params = new URLSearchParams()
    if (filters?.status) params.append('status', filters.status)
    if (filters?.requester_id) params.append('requester_id', filters.requester_id)
    if (filters?.start_date) params.append('start_date', String(filters.start_date))
    if (filters?.end_date) params.append('end_date', String(filters.end_date))
    const query = params.toString()
    return fetchApi<ListBudgetRequestsResponse>(
      `/api/v1/budget/requests${query ? `?${query}` : ''}`
    )
  }

  async function approveBudgetRequest(
    requestId: string
  ): Promise<ApproveBudgetRequestResponse> {
    return fetchApi<ApproveBudgetRequestResponse>(
      `/api/v1/budget/requests/${requestId}/approve`,
      {
        method: 'PATCH',
        body: JSON.stringify({}),
      }
    )
  }

  async function rejectBudgetRequest(
    requestId: string,
    data: RejectBudgetRequestRequest
  ): Promise<RejectBudgetRequestResponse> {
    return fetchApi<RejectBudgetRequestResponse>(
      `/api/v1/budget/requests/${requestId}/reject`,
      {
        method: 'PATCH',
        body: JSON.stringify(data),
      }
    )
  }

  return {
    getTokens,
    getToken,
    createToken,
    rotateToken,
    revokeToken,
    getUsage,
    getUsageStats,
    getUsageByToken,
    getLimits,
    getLimit,
    createLimit,
    updateLimit,
    deleteLimit,
    getProviderKeys,
    getProviderKey,
    createProviderKey,
    updateProviderKey,
    deleteProviderKey,
    assignProjectProvider,
    unassignProjectProvider,
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
    getAgentTokens,
    createAgentToken,
    updateTokenProvider,
    // IC Token methods (agent runtime authentication)
    generateIcToken,
    getIcTokenStatus,
    regenerateIcToken,
    revokeIcToken,
    // Analytics (Protocol 012)
    getAnalyticsSpendingTotal,
    getAnalyticsSpendingByProvider,
    getAnalyticsUsageRequests,
    getAnalyticsUsageModels,
    getBudgetStatus,
    // Budget Request Workflow
    createBudgetRequest,
    getBudgetRequest,
    listBudgetRequests,
    approveBudgetRequest,
    rejectBudgetRequest,
  }
}

export interface User {
  id: number
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
  provider_key_id?: number | null
  has_ic_token?: boolean
  ic_token_created_at?: number
}

export interface AgentBudgetResponse {
  agent_id: number
  total_allocated: number
  total_spent: number
  budget_remaining: number
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

export type {
  TokenMetadata,
  CreateTokenRequest,
  CreateTokenResponse,
  UsageRecord,
  UsageStats,
  LimitRecord,
  CreateLimitRequest,
  UpdateLimitRequest,
  ProviderType,
  ProviderKey,
  CreateProviderKeyRequest,
  UpdateProviderKeyRequest,
  AssignProviderRequest,
  BudgetRequest,
  CreateBudgetRequestRequest,
  CreateBudgetRequestResponse,
  ListBudgetRequestsResponse,
  ApproveBudgetRequestResponse,
  RejectBudgetRequestRequest,
  RejectBudgetRequestResponse,
  AgentBudgetResponse,
  BudgetStatus,
  BudgetStatusResponse,
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
}

export interface PaginationParams {
  page?: number
  per_page?: number
}

export interface SpendingTotalResponse {
  total_spend: number
  currency: string
  period: string
  filters: { agent_id?: number; provider_id?: string }
  calculated_at: string
}

export interface ProviderSpending {
  provider: string
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

export interface RequestUsageResponse {
  total_requests: number
  successful_requests: number
  failed_requests: number
  success_rate: number
  period: string
  filters: { agent_id?: number; provider_id?: string }
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
