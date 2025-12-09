import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

interface LoginCredentials {
  username: string
  password: string
}

interface AuthTokens {
  access_token: string
  refresh_token: string
  token_type: string
  expires_in: number
  role: string
}

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000'

export const useAuthStore = defineStore('auth', () => {
  const accessToken = ref<string | null>(null)
  const refreshToken = ref<string | null>(null)
  const username = ref<string | null>(null)
  const role = ref<string | null>(null)
  const isAuthenticated = computed(() => !!accessToken.value)
  const isAdmin = computed(() => role.value === 'admin')

  // Load tokens from localStorage on init
  function loadTokens() {
    const storedAccessToken = localStorage.getItem('access_token')
    const storedRefreshToken = localStorage.getItem('refresh_token')
    const storedUsername = localStorage.getItem('username')
    const storedRole = localStorage.getItem('role')

    if (storedAccessToken && storedRefreshToken) {
      accessToken.value = storedAccessToken
      refreshToken.value = storedRefreshToken
      username.value = storedUsername
      role.value = storedRole
    }
  }

  // Save tokens to localStorage
  function saveTokens(tokens: AuthTokens, user: string) {
    accessToken.value = tokens.access_token
    refreshToken.value = tokens.refresh_token
    username.value = user
    role.value = tokens.role

    localStorage.setItem('access_token', tokens.access_token)
    localStorage.setItem('refresh_token', tokens.refresh_token)
    localStorage.setItem('username', user)
    localStorage.setItem('role', tokens.role)
  }

  // Clear tokens from localStorage
  function clearTokens() {
    accessToken.value = null
    refreshToken.value = null
    username.value = null
    role.value = null

    localStorage.removeItem('access_token')
    localStorage.removeItem('refresh_token')
    localStorage.removeItem('username')
    localStorage.removeItem('role')
  }

  // Login
  async function login(credentials: LoginCredentials) {
    const response = await fetch(`${API_BASE_URL}/api/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(credentials),
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Login failed' }))
      throw new Error(error.error || 'Login failed')
    }

    const tokens: AuthTokens = await response.json()
    saveTokens(tokens, credentials.username)
  }

  // Refresh access token
  async function refresh() {
    if (!refreshToken.value) {
      throw new Error('No refresh token available')
    }

    const response = await fetch(`${API_BASE_URL}/api/auth/refresh`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ refresh_token: refreshToken.value }),
    })

    if (!response.ok) {
      clearTokens()
      throw new Error('Token refresh failed')
    }

    const tokens: AuthTokens = await response.json()
    saveTokens(tokens, username.value || 'unknown')
  }

  // Logout
  async function logout() {
    if (refreshToken.value) {
      try {
        await fetch(`${API_BASE_URL}/api/auth/logout`, {
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
    role,
    isAuthenticated,
    isAdmin,
    login,
    refresh,
    logout,
    getAuthHeader,
  }
})
