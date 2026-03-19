import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

interface LoginCredentials {
  email: string
  password: string
}

interface AuthTokens {
  user_token: string
  refresh_token?: string  // backend uses Option<String> with skip_serializing_if
  token_type: string
  expires_in: number
  user: {
    id: string  // User ID like 'user_admin' - used for FK relations
    email: string
    role: string
    name: string
  }
}

/** Decode role claim from JWT payload without verifying signature. */
function decodeJwtRole(token: string): string | null {
  try {
    const seg = token.split('.')[1].replace(/-/g, '+').replace(/_/g, '/')
    const padded = seg.padEnd(Math.ceil(seg.length / 4) * 4, '=')
    const payload = JSON.parse(atob(padded))
    return typeof payload.role === 'string' ? payload.role : null
  } catch {
    return null
  }
}

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3001'

export const useAuthStore = defineStore('auth', () => {
  const accessToken = ref<string | null>(null)
  const refreshToken = ref<string | null>(null)
  const username = ref<string | null>(null)  // display name
  const userId = ref<string | null>(null)    // FK / ID
  const role = ref<string | null>(null)
  const isAuthenticated = computed(() => !!accessToken.value)
  const isAdmin = computed(() => role.value === 'admin')

  // Load tokens from localStorage on init
  function loadTokens() {
    const storedAccessToken = localStorage.getItem('access_token')
    const storedRefreshToken = localStorage.getItem('refresh_token')
    const storedUsername = localStorage.getItem('username')
    const storedUserId = localStorage.getItem('user_id')

    // Treat the literal string "undefined" written by old code as absent
    const validRefreshToken =
      storedRefreshToken && storedRefreshToken !== 'undefined'
        ? storedRefreshToken
        : null

    if (storedAccessToken) {
      accessToken.value = storedAccessToken
      refreshToken.value = validRefreshToken
      username.value = storedUsername
      userId.value = storedUserId
      // Derive role from the JWT payload — do not trust the localStorage value
      role.value = decodeJwtRole(storedAccessToken)
    }
  }

  // Save tokens to localStorage
  function saveTokens(tokens: AuthTokens) {
    accessToken.value = tokens.user_token
    refreshToken.value = tokens.refresh_token ?? null
    username.value = tokens.user.name || tokens.user.email
    userId.value = tokens.user.id
    // Prefer JWT-decoded role; fall back to the login response body
    role.value = decodeJwtRole(tokens.user_token) ?? tokens.user.role

    // SECURITY NOTE: localStorage is XSS-accessible. Access and refresh tokens are
    // stored here for SPA session persistence. The accepted risk is that an XSS
    // vulnerability would expose these tokens. Mitigations: strict CSP, short token
    // TTLs, and refresh token rotation are required to limit blast radius.
    localStorage.setItem('access_token', tokens.user_token)
    if (tokens.refresh_token) {
      localStorage.setItem('refresh_token', tokens.refresh_token)
    } else {
      localStorage.removeItem('refresh_token')
    }
    localStorage.setItem('username', username.value)
    localStorage.setItem('user_id', tokens.user.id)
  }

  // Clear tokens from localStorage
  function clearTokens() {
    accessToken.value = null
    refreshToken.value = null
    username.value = null
    userId.value = null
    role.value = null

    localStorage.removeItem('access_token')
    localStorage.removeItem('refresh_token')
    localStorage.removeItem('username')
    localStorage.removeItem('user_id')
    localStorage.removeItem('role')  // clean up legacy key
  }

  // Login
  async function login(credentials: LoginCredentials) {
    const response = await fetch(`${API_BASE_URL}/api/v1/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(credentials),
    })

    if (!response.ok) {
      let msg: string;
      try {
        const body = await response.json();
        msg = body?.error?.message ?? body?.error ?? body?.message ?? 'Login failed'
      } catch {
        msg = 'Login failed'
      }
      throw new Error(msg)
    }

    const tokens: AuthTokens = await response.json()
    saveTokens(tokens)
  }

  // Refresh access token
  async function refresh() {
    if (!refreshToken.value) {
      throw new Error('No refresh token available')
    }

    const response = await fetch(`${API_BASE_URL}/api/v1/auth/refresh`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${refreshToken.value}`,
      },
    })

    if (!response.ok) {
      clearTokens()
      throw new Error('Token refresh failed')
    }

    const tokens: AuthTokens = await response.json()
    saveTokens(tokens)
  }

  // Logout
  async function logout() {
    if (refreshToken.value) {
      try {
        await fetch(`${API_BASE_URL}/api/v1/auth/logout`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ refresh_token: refreshToken.value }),
        })
      } catch (error) {
        console.error('Logout API call failed:', error)
      }
    }

    clearTokens()
  }

  // Get authorization header
  function getAuthHeader() {
    return accessToken.value ? `Bearer ${accessToken.value}` : null
  }

  // Initialize store
  loadTokens()

  return {
    accessToken,
    refreshToken,
    username,
    userId,
    role,
    isAuthenticated,
    isAdmin,
    login,
    refresh,
    logout,
    getAuthHeader,
  }
})
