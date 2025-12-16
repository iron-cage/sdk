<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { useApi } from '../composables/useApi'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

const api = useApi()

// Fetch spending analytics
const { data: spending, isLoading: spendingLoading } = useQuery({
  queryKey: ['analytics-spending'],
  queryFn: () => api.getAnalyticsSpendingTotal({ period: 'all-time' }),
})

// Fetch request usage analytics
const { data: requestUsage, isLoading: requestsLoading } = useQuery({
  queryKey: ['analytics-requests'],
  queryFn: () => api.getAnalyticsUsageRequests({ period: 'all-time' }),
})

// Fetch agents count
const { data: agents, isLoading: agentsLoading } = useQuery({
  queryKey: ['agents'],
  queryFn: () => api.getAgents(),
})

const isLoading = spendingLoading || requestsLoading || agentsLoading

function formatCurrency(usd: number): string {
  return `$${usd.toFixed(3)}`
}
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold text-gray-900 mb-6">Dashboard</h1>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading dashboard...</p>
    </div>

    <!-- Dashboard content -->
    <div v-else>
      <!-- Stats cards -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <!-- Total Spending -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Total Spending</CardTitle>
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
                  d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-gray-900">
              {{ spending ? formatCurrency(spending.total_spend) : '$0.00' }}
            </div>
            <p class="text-xs text-gray-500 mt-1">All time</p>
          </CardContent>
        </Card>

        <!-- Success Rate -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Success Rate</CardTitle>
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
            <div class="text-3xl font-bold" :class="requestUsage && requestUsage.success_rate >= 95 ? 'text-green-600' : requestUsage && requestUsage.success_rate >= 80 ? 'text-yellow-600' : 'text-red-600'">
              {{ requestUsage ? requestUsage.success_rate.toFixed(1) : '0' }}%
            </div>
            <p class="text-xs text-gray-500 mt-1">
              {{ requestUsage ? `${requestUsage.successful_requests} / ${requestUsage.total_requests} requests` : 'No requests' }}
            </p>
          </CardContent>
        </Card>

        <!-- Total Agents -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Active Agents</CardTitle>
            <div class="h-12 w-12 bg-purple-100 rounded-full flex items-center justify-center">
              <svg
                class="h-6 w-6 text-purple-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
            </div>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-purple-600">
              {{ agents ? agents.length : 0 }}
            </div>
            <p class="text-xs text-gray-500 mt-1">Registered agents</p>
          </CardContent>
        </Card>
      </div>
    </div>
  </div>
</template>
