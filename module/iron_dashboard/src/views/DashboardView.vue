<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useQuery } from '@tanstack/vue-query'
import { useApi, type TokenMetadata } from '../composables/useApi'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

const router = useRouter()
const api = useApi()

// Fetch tokens for dashboard stats
const { data: tokens, isLoading, error } = useQuery({
  queryKey: ['tokens'],
  queryFn: () => api.getTokens(),
})

// Compute statistics
const stats = computed(() => {
  if( !tokens.value ) return { total: 0, active: 0, revoked: 0 }

  const active = tokens.value.filter( ( t: TokenMetadata ) => t.is_active ).length
  const revoked = tokens.value.filter( ( t: TokenMetadata ) => !t.is_active ).length

  return {
    total: tokens.value.length,
    active,
    revoked,
  }
})

// Recent tokens (last 5)
const recentTokens = computed(() => {
  if( !tokens.value ) return []
  return [ ...tokens.value ]
    .sort( ( a, b ) => b.created_at - a.created_at )
    .slice( 0, 5 )
})

function formatDate( timestamp: number ): string {
  return new Date( timestamp ).toLocaleString()
}

function navigateToTokens() {
  router.push( '/tokens' )
}

function navigateToUsage() {
  router.push( '/usage' )
}

function navigateToLimits() {
  router.push( '/limits' )
}
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold text-gray-900 mb-6">Dashboard</h1>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading dashboard...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="bg-white rounded-lg shadow p-6">
      <p class="text-red-600">Error loading dashboard: {{ error.message }}</p>
    </div>

    <!-- Dashboard content -->
    <div v-else>
      <!-- Stats cards -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        <!-- Total tokens -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Total Tokens</CardTitle>
            <div class="h-12 w-12 bg-blue-100 rounded-full flex items-center justify-center">
              <svg
                class="h-6 w-6 text-blue-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"
                />
              </svg>
            </div>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-gray-900">{{ stats.total }}</div>
          </CardContent>
        </Card>

        <!-- Active tokens -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Active Tokens</CardTitle>
            <div class="h-12 w-12 bg-green-100 rounded-full flex items-center justify-center">
              <svg
                class="h-6 w-6 text-green-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-green-600">{{ stats.active }}</div>
          </CardContent>
        </Card>

        <!-- Revoked tokens -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Revoked Tokens</CardTitle>
            <div class="h-12 w-12 bg-red-100 rounded-full flex items-center justify-center">
              <svg
                class="h-6 w-6 text-red-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-red-600">{{ stats.revoked }}</div>
          </CardContent>
        </Card>
      </div>

      <!-- Quick actions -->
      <Card class="mb-6">
        <CardHeader>
          <CardTitle>Quick Actions</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="flex flex-wrap gap-3">
            <Button @click="navigateToTokens">
              Manage Tokens
            </Button>
            <Button @click="navigateToUsage" variant="secondary">
              View Usage Analytics
            </Button>
            <Button @click="navigateToLimits" variant="secondary">
              Configure Limits
            </Button>
          </div>
        </CardContent>
      </Card>

      <!-- Recent tokens -->
      <Card>
        <CardHeader>
          <CardTitle>Recent Tokens</CardTitle>
        </CardHeader>
        <CardContent>
          <div v-if="recentTokens.length === 0" class="text-center text-gray-600 py-4">
            No tokens found. Generate your first token to get started.
          </div>
          <div v-else class="overflow-hidden">
            <table class="min-w-full divide-y divide-gray-200">
              <thead class="bg-gray-50">
                <tr>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    ID
                  </th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Provider
                  </th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Description
                  </th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Created
                  </th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                </tr>
              </thead>
              <tbody class="bg-white divide-y divide-gray-200">
                <tr v-for="token in recentTokens" :key="token.id">
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ token.id }}
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ token.provider || '-' }}
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {{ token.name || '-' }}
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {{ formatDate( token.created_at ) }}
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap">
                    <Badge :variant="token.is_active ? 'default' : 'destructive'">
                      {{ token.is_active ? 'Active' : 'Revoked' }}
                    </Badge>
                  </td>
                </tr>
              </tbody>
            </table>
            <div class="mt-4 pt-4 border-t border-gray-200">
              <Button @click="navigateToTokens" variant="ghost" size="sm">
                View all tokens →
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
