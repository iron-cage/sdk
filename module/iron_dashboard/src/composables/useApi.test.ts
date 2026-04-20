import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { createRouter, createMemoryHistory, type Router } from 'vue-router'
import { defineComponent, h } from 'vue'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'

import { useApi } from './useApi'
import { useAuthStore } from '../stores/auth'

const mockFetch = vi.fn()
vi.stubGlobal('fetch', mockFetch)

function makeJwt(payload: object): string {
  const seg = btoa(JSON.stringify(payload))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '')
  return `header.${seg}.sig`
}

function createTestRouter(): Router {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', redirect: '/dashboard' },
      { path: '/login', name: 'login', component: { template: '<div />' } },
      { path: '/dashboard', name: 'dashboard', component: { template: '<div />' } },
    ],
  })
}

function mountApi(router: Router): { api: ReturnType<typeof useApi>; wrapper: VueWrapper } {
  let api: ReturnType<typeof useApi> | undefined
  const Harness = defineComponent({
    setup() {
      api = useApi()
      return () => h('div')
    },
  })
  const wrapper = mount(Harness, { global: { plugins: [router] } })
  return { api: api!, wrapper }
}

function okJson<T>(body: T) {
  const text = JSON.stringify(body)
  return {
    ok: true,
    status: 200,
    json: async () => body,
    text: async () => text,
  }
}

function unauthorized() {
  return {
    ok: false,
    status: 401,
    json: async () => ({ error: 'Unauthorized' }),
    text: async () => '',
  }
}

function noContent() {
  return {
    ok: true,
    status: 204,
    json: async () => ({}),
    text: async () => '',
  }
}

describe('useApi — 401 handling and logout deduplication', () => {
  let router: Router
  let wrapper: VueWrapper | undefined

  beforeEach(async () => {
    setActivePinia(createPinia())
    localStorage.clear()
    mockFetch.mockReset()
    router = createTestRouter()
    await router.push('/dashboard')
    await router.isReady()
  })

  afterEach(() => {
    wrapper?.unmount()
    wrapper = undefined
  })

  function setupAuthenticatedSession() {
    const jwt = makeJwt({ role: 'admin' })
    localStorage.setItem('access_token', jwt)
    localStorage.setItem('refresh_token', 'rt_valid')
    localStorage.setItem('username', 'testuser')
    localStorage.setItem('user_id', 'u1')
  }

  it('concurrent 401s after failed refresh clear the session and redirect to /login once', async () => {
    setupAuthenticatedSession()
    const mounted = mountApi(router)
    wrapper = mounted.wrapper
    const api = mounted.api
    const authStore = useAuthStore()

    mockFetch.mockImplementation(async (url: string) => {
      if (url.includes('/api/v1/auth/refresh')) return unauthorized()
      if (url.includes('/api/v1/auth/logout')) return noContent()
      return unauthorized()
    })

    const results = await Promise.allSettled([api.getAgents(), api.getProviderKeys()])
    await flushPromises()

    expect(results[0].status).toBe('rejected')
    expect(results[1].status).toBe('rejected')
    if (results[0].status === 'rejected') {
      expect((results[0].reason as Error).message).toBe('Session expired')
    }
    if (results[1].status === 'rejected') {
      expect((results[1].reason as Error).message).toBe('Session expired')
    }

    // The refresh endpoint is hit exactly once despite two concurrent 401s —
    // this verifies _refreshPromise deduplication.
    const refreshCalls = mockFetch.mock.calls.filter((c) =>
      String(c[0]).includes('/api/v1/auth/refresh')
    )
    expect(refreshCalls.length).toBe(1)

    expect(authStore.isAuthenticated).toBe(false)
    expect(authStore.accessToken).toBeNull()
    expect(authStore.refreshToken).toBeNull()
    expect(router.currentRoute.value.path).toBe('/login')
  })

  it('successful refresh retries the original request and clears the session on second 401', async () => {
    setupAuthenticatedSession()
    const mounted = mountApi(router)
    wrapper = mounted.wrapper
    const api = mounted.api
    const authStore = useAuthStore()
    const newJwt = makeJwt({ role: 'admin' })

    let businessHits = 0
    mockFetch.mockImplementation(async (url: string) => {
      if (url.includes('/api/v1/auth/refresh')) {
        return okJson({
          user_token: newJwt,
          refresh_token: 'rt_valid_2',
          token_type: 'Bearer',
          expires_in: 3600,
          user: { id: 'u1', email: 'a@b.c', role: 'admin', name: 'testuser' },
        })
      }
      if (url.includes('/api/v1/auth/logout')) return noContent()
      businessHits++
      return unauthorized()
    })

    const err = await api.getAgents().catch((e: Error) => e)
    await flushPromises()

    expect(err).toBeInstanceOf(Error)
    expect((err as Error).message).toBe('Session expired')

    // Original call + one retry after refresh
    expect(businessHits).toBe(2)

    expect(authStore.isAuthenticated).toBe(false)
    expect(router.currentRoute.value.path).toBe('/login')
  })

  it('401 without a refresh token clears the session and redirects to /login', async () => {
    // Authenticate without a refresh token
    const jwt = makeJwt({ role: 'admin' })
    localStorage.setItem('access_token', jwt)
    localStorage.setItem('username', 'testuser')
    localStorage.setItem('user_id', 'u1')

    const mounted = mountApi(router)
    wrapper = mounted.wrapper
    const api = mounted.api
    const authStore = useAuthStore()

    mockFetch.mockImplementation(async () => unauthorized())

    const results = await Promise.allSettled([api.getAgents(), api.getProviderKeys()])
    await flushPromises()

    expect(results[0].status).toBe('rejected')
    expect(results[1].status).toBe('rejected')

    // Without a refresh token, no refresh attempt is made
    const refreshCalls = mockFetch.mock.calls.filter((c) =>
      String(c[0]).includes('/api/v1/auth/refresh')
    )
    expect(refreshCalls.length).toBe(0)

    expect(authStore.isAuthenticated).toBe(false)
    expect(authStore.accessToken).toBeNull()
    expect(router.currentRoute.value.path).toBe('/login')
  })
})
