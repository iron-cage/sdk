import { useAuthStore } from '../stores/auth'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000'

interface TokenMetadata {
  id: number
  user_id: string
  project_id?: string
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
  project_id?: string
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
  max_cost_per_month_cents?: number
  created_at: number
}

interface CreateLimitRequest {
  user_id: string
  project_id?: string
  max_tokens_per_day?: number
  max_requests_per_minute?: number
  max_cost_per_month_cents?: number
}

interface UpdateLimitRequest {
  max_tokens_per_day?: number
  max_requests_per_minute?: number
  max_cost_per_month_cents?: number
}

interface TraceRecord {
  id: number
  token_id: number
  request_id: string
  provider: string
  model: string
  input_tokens: number
  output_tokens: number
  cost: number
  timestamp: number
  metadata?: Record<string, unknown>
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
    return fetchApi<TokenMetadata[]>('/api/tokens')
  }

  async function getToken(id: number): Promise<TokenMetadata> {
    return fetchApi<TokenMetadata>(`/api/tokens/${id}`)
  }

  async function createToken(data: CreateTokenRequest): Promise<CreateTokenResponse> {
    return fetchApi<CreateTokenResponse>('/api/tokens', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async function rotateToken(id: number): Promise<CreateTokenResponse> {
    return fetchApi<CreateTokenResponse>(`/api/tokens/${id}/rotate`, {
      method: 'POST',
      body: JSON.stringify({}),
    })
  }

  async function revokeToken(id: number): Promise<void> {
    await fetchApi<void>(`/api/tokens/${id}`, {
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
    }>('/api/usage/aggregate')

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
    return fetchApi<UsageRecord[]>(`/api/usage/token/${tokenId}`)
  }

  // Limits API methods
  async function getLimits(): Promise<LimitRecord[]> {
    return fetchApi<LimitRecord[]>('/api/limits')
  }

  async function getLimit(id: number): Promise<LimitRecord> {
    return fetchApi<LimitRecord>(`/api/limits/${id}`)
  }

  async function createLimit(data: CreateLimitRequest): Promise<LimitRecord> {
    return fetchApi<LimitRecord>('/api/limits', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async function updateLimit(id: number, data: UpdateLimitRequest): Promise<LimitRecord> {
    return fetchApi<LimitRecord>(`/api/limits/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async function deleteLimit(id: number): Promise<void> {
    await fetchApi<void>(`/api/limits/${id}`, {
      method: 'DELETE',
    })
  }

  // Traces API methods
  async function getTraces(): Promise<TraceRecord[]> {
    return fetchApi<TraceRecord[]>('/api/traces')
  }

  async function getTrace(id: number): Promise<TraceRecord> {
    return fetchApi<TraceRecord>(`/api/traces/${id}`)
  }

  // Provider Key API methods
  async function getProviderKeys(): Promise<ProviderKey[]> {
    return fetchApi<ProviderKey[]>('/api/providers')
  }

  async function getProviderKey(id: number): Promise<ProviderKey> {
    return fetchApi<ProviderKey>(`/api/providers/${id}`)
  }

  async function createProviderKey(data: CreateProviderKeyRequest): Promise<ProviderKey> {
    return fetchApi<ProviderKey>('/api/providers', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async function updateProviderKey(id: number, data: UpdateProviderKeyRequest): Promise<ProviderKey> {
    return fetchApi<ProviderKey>(`/api/providers/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async function deleteProviderKey(id: number): Promise<void> {
    await fetchApi<void>(`/api/providers/${id}`, {
      method: 'DELETE',
    })
  }

  async function assignProjectProvider(projectId: string, keyId: number): Promise<void> {
    await fetchApi<void>(`/api/projects/${projectId}/provider`, {
      method: 'POST',
      body: JSON.stringify({ provider_key_id: keyId }),
    })
  }

  async function unassignProjectProvider(projectId: string): Promise<void> {
    await fetchApi<void>(`/api/projects/${projectId}/provider`, {
      method: 'DELETE',
    })
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
    getTraces,
    getTrace,
    getProviderKeys,
    getProviderKey,
    createProviderKey,
    updateProviderKey,
    deleteProviderKey,
    assignProjectProvider,
    unassignProjectProvider,
  }
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
  TraceRecord,
  ProviderType,
  ProviderKey,
  CreateProviderKeyRequest,
  UpdateProviderKeyRequest,
  AssignProviderRequest,
}
