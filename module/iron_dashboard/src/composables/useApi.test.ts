import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

// Mock vue-router before any import that uses it
const mockReplace = vi.fn()
vi.mock('vue-router', () => ({
  useRouter: () => ({ replace: mockReplace }),
}))

const mockFetch = vi.fn()
vi.stubGlobal('fetch', mockFetch)

function makeJwt(payload: object): string {
  const seg = btoa(JSON.stringify(payload))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '')
  return `header.${seg}.sig`
}

describe('useApi — _logoutPromise deduplication', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
    mockFetch.mockReset()
    mockReplace.mockReset()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  /**
   * Helper: set up an authenticated store with a refresh token so the
   * 401-handling code paths are exercised.
   */
  function setupAuthenticatedStore() {
    const jwt = makeJwt({ role: 'admin' })
    localStorage.setItem('access_token', jwt)
    localStorage.setItem('refresh_token', 'rt_valid')
    localStorage.setItem('username', 'testuser')
    localStorage.setItem('user_id', 'u1')
  }

  it('concurrent 401s after failed refresh trigger authStore.logout exactly once', async () => {
    setupAuthenticatedStore()

    // We need to dynamically import so the store is created after pinia is set up
    const { useAuthStore } = await import('../stores/auth')
    const authStore = useAuthStore()

    const logoutSpy = vi.spyOn(authStore, 'logout')

    // Mock the refresh call to fail
    const refreshMock = vi
      .spyOn(authStore, 'refresh')
      .mockRejectedValue(new Error('Token refresh failed'))

    const { useApi } = await import('./useApi')
    const api = useApi()

    // Both initial requests return 401
    mockFetch.mockResolvedValue({
      ok: false,
      status: 401,
      json: async () => ({ error: 'Unauthorized' }),
    })

    // Fire two concurrent requests that will both get 401
    const results = await Promise.allSettled([api.getAgents(), api.getProviderKeys()])

    // Both should reject with 'Session expired'
    expect(results[0].status).toBe('rejected')
    expect(results[1].status).toBe('rejected')
    if (results[0].status === 'rejected') {
      expect(results[0].reason.message).toBe('Session expired')
    }
    if (results[1].status === 'rejected') {
      expect(results[1].reason.message).toBe('Session expired')
    }

    // Logout should be called exactly once despite two concurrent 401s
    expect(logoutSpy).toHaveBeenCalledTimes(1)
    expect(mockReplace).toHaveBeenCalledWith('/login')

    refreshMock.mockRestore()
    logoutSpy.mockRestore()
  })

  it('401 after successful refresh retries and triggers logout on second 401', async () => {
    setupAuthenticatedStore()

    const { useAuthStore } = await import('../stores/auth')
    const authStore = useAuthStore()

    const logoutSpy = vi.spyOn(authStore, 'logout')

    // Refresh succeeds
    const newJwt = makeJwt({ role: 'admin' })
    const refreshMock = vi.spyOn(authStore, 'refresh').mockImplementation(async () => {
      authStore.$patch({ accessToken: newJwt })
    })

    const { useApi } = await import('./useApi')
    const api = useApi()

    // First fetch returns 401, retry after refresh also returns 401
    mockFetch.mockResolvedValue({
      ok: false,
      status: 401,
      json: async () => ({ error: 'Unauthorized' }),
    })

    const result = await api.getAgents().catch((e: Error) => e)

    expect(result).toBeInstanceOf(Error)
    expect((result as Error).message).toBe('Session expired')
    expect(logoutSpy).toHaveBeenCalledTimes(1)

    refreshMock.mockRestore()
    logoutSpy.mockRestore()
  })

  it('401 without refresh token triggers logout (catch path 3)', async () => {
    // Set up store WITHOUT refresh token
    const jwt = makeJwt({ role: 'admin' })
    localStorage.setItem('access_token', jwt)
    localStorage.setItem('username', 'testuser')
    localStorage.setItem('user_id', 'u1')

    const { useAuthStore } = await import('../stores/auth')
    const authStore = useAuthStore()

    const logoutSpy = vi.spyOn(authStore, 'logout')

    const { useApi } = await import('./useApi')
    const api = useApi()

    // Request returns 401, no refresh token available
    mockFetch.mockResolvedValue({
      ok: false,
      status: 401,
      json: async () => ({ error: 'Unauthorized' }),
    })

    // Fire two concurrent requests
    const results = await Promise.allSettled([api.getAgents(), api.getProviderKeys()])

    expect(results[0].status).toBe('rejected')
    expect(results[1].status).toBe('rejected')

    // Logout should be called exactly once
    expect(logoutSpy).toHaveBeenCalledTimes(1)

    logoutSpy.mockRestore()
  })
})
