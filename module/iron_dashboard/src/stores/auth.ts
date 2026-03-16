import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

interface LoginCredentials {
  email: string
  password: string
}

interface AuthTokens {
  user_token: string
  refresh_token: string
  token_type: string
  expires_in: number
  user: {
    id: string  // User ID like 'user_admin' - used for FK relations
    email: string
    role: string
    name: string
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
    const storedRole = localStorage.getItem('role')

    if (storedAccessToken && storedRefreshToken) {
      accessToken.value = storedAccessToken
      refreshToken.value = storedRefreshToken
      username.value = storedUsername
      userId.value = storedUserId
      role.value = storedRole
    }
  }

  // Save tokens to localStorage
  function saveTokens(tokens: AuthTokens) {
    accessToken.value = tokens.user_token
    refreshToken.value = tokens.refresh_token
    username.value = tokens.user.name || tokens.user.email
    userId.value = tokens.user.id
    role.value = tokens.user.role

    localStorage.setItem('access_token', tokens.user_token)
    localStorage.setItem('refresh_token', tokens.refresh_token)
    localStorage.setItem('username', username.value)
    localStorage.setItem('user_id', tokens.user.id)
    localStorage.setItem('role', tokens.user.role)
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
    localStorage.removeItem('role')
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
        const error = await response.json();
        msg = error.error.message
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
