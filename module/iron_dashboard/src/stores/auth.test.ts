import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useAuthStore } from './auth'

const mockFetch = vi.fn()
vi.stubGlobal('fetch', mockFetch)

function makeJwt(payload: object): string {
  return `header.${btoa(JSON.stringify(payload))}.sig`
}

describe('useAuthStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
    mockFetch.mockReset()
  })

  describe('loadTokens (called on store init)', () => {
    it('restores session when access_token is present', () => {
      localStorage.setItem('access_token', makeJwt({ role: 'admin' }))
      localStorage.setItem('username', 'Alice')
      localStorage.setItem('user_id', 'u1')

      const store = useAuthStore()
      expect(store.isAuthenticated).toBe(true)
      expect(store.username).toBe('Alice')
      expect(store.role).toBe('admin')
      expect(store.isAdmin).toBe(true)
    })

    it('derives role from JWT payload, ignoring localStorage role key', () => {
      localStorage.setItem('access_token', makeJwt({ role: 'developer' }))
      localStorage.setItem('role', 'admin')  // legacy key should be ignored

      const store = useAuthStore()
      expect(store.role).toBe('developer')
      expect(store.isAdmin).toBe(false)
    })

    it('treats literal "undefined" string refresh_token as null', () => {
      localStorage.setItem('access_token', makeJwt({ role: 'user' }))
      localStorage.setItem('refresh_token', 'undefined')

      const store = useAuthStore()
      expect(store.isAuthenticated).toBe(true)
      expect(store.refreshToken).toBeNull()
    })

    it('restores valid refresh token when present', () => {
      localStorage.setItem('access_token', makeJwt({ role: 'user' }))
      localStorage.setItem('refresh_token', 'rt_valid_token')

      const store = useAuthStore()
      expect(store.refreshToken).toBe('rt_valid_token')
    })

    it('does not restore session when access_token is absent', () => {
      const store = useAuthStore()
      expect(store.isAuthenticated).toBe(false)
      expect(store.role).toBeNull()
    })
  })

  describe('login', () => {
    it('sets auth state and persists tokens to localStorage', async () => {
      const jwt = makeJwt({ role: 'admin' })
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          user_token: jwt,
          refresh_token: 'rt_abc',
          token_type: 'Bearer',
          expires_in: 3600,
          user: { id: 'u1', email: 'admin@test.com', role: 'admin', name: 'Admin' },
        }),
      })

      const store = useAuthStore()
      await store.login({ email: 'admin@test.com', password: 'pass' })

      expect(store.isAuthenticated).toBe(true)
      expect(store.role).toBe('admin')
      expect(store.isAdmin).toBe(true)
      expect(localStorage.getItem('access_token')).toBe(jwt)
      expect(localStorage.getItem('refresh_token')).toBe('rt_abc')
    })

    it('does not write refresh_token to localStorage when absent', async () => {
      const jwt = makeJwt({ role: 'user' })
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          user_token: jwt,
          token_type: 'Bearer',
          expires_in: 3600,
          user: { id: 'u2', email: 'user@test.com', role: 'user', name: 'User' },
        }),
      })

      const store = useAuthStore()
      await store.login({ email: 'user@test.com', password: 'pass' })

      expect(store.refreshToken).toBeNull()
      expect(localStorage.getItem('refresh_token')).toBeNull()
    })

    it('throws on non-ok response with server error message', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
        json: async () => ({ error: 'Invalid credentials' }),
      })

      const store = useAuthStore()
      await expect(store.login({ email: 'bad@test.com', password: 'wrong' }))
        .rejects.toThrow('Invalid credentials')
      expect(store.isAuthenticated).toBe(false)
    })

    it('throws generic message when response body is not parseable', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => { throw new Error('not json') },
      })

      const store = useAuthStore()
      await expect(store.login({ email: 'x@test.com', password: 'y' }))
        .rejects.toThrow('Login failed')
    })
  })

  describe('logout', () => {
    it('clears all auth state and localStorage', async () => {
      localStorage.setItem('access_token', makeJwt({ role: 'user' }))
      localStorage.setItem('refresh_token', 'rt_xyz')
      localStorage.setItem('username', 'Bob')
      localStorage.setItem('user_id', 'u3')

      // Mock the logout API call
      mockFetch.mockResolvedValueOnce({ ok: true, json: async () => ({}) })

      const store = useAuthStore()
      expect(store.isAuthenticated).toBe(true)

      await store.logout()

      expect(store.isAuthenticated).toBe(false)
      expect(store.username).toBeNull()
      expect(store.refreshToken).toBeNull()
      expect(localStorage.getItem('access_token')).toBeNull()
      expect(localStorage.getItem('refresh_token')).toBeNull()
    })

    it('clears state even when logout API call fails', async () => {
      localStorage.setItem('access_token', makeJwt({ role: 'user' }))
      localStorage.setItem('refresh_token', 'rt_xyz')

      mockFetch.mockRejectedValueOnce(new Error('Network error'))

      const store = useAuthStore()
      await store.logout()

      expect(store.isAuthenticated).toBe(false)
    })
  })
})
