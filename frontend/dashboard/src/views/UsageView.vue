<script setup lang="ts">
import { computed } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useApi } from '../composables/useApi'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

const api = useApi()

// Fetch usage data
const { data: usageRecords, isLoading: recordsLoading, error: recordsError } = useQuery({
  queryKey: ['usage'],
  queryFn: () => api.getUsage(),
})

const { data: stats, isLoading: statsLoading, error: statsError } = useQuery({
  queryKey: ['usage-stats'],
  queryFn: () => api.getUsageStats(),
})

const isLoading = computed(() => recordsLoading.value || statsLoading.value)
const error = computed(() => recordsError.value || statsError.value)

// Recent usage (last 10)
const recentUsage = computed(() => {
  if( !usageRecords.value ) return []
  return [ ...usageRecords.value ]
    .sort( ( a, b ) => b.timestamp - a.timestamp )
    .slice( 0, 10 )
})

// Provider breakdown with visual bars
const providerBreakdown = computed(() => {
  if( !stats.value?.by_provider ) return []
  const maxCost = Math.max( ...stats.value.by_provider.map( p => p.cost ) )
  return stats.value.by_provider.map( p => ({
    ...p,
    percentage: maxCost > 0 ? ( p.cost / maxCost ) * 100 : 0,
  }))
})

// Model breakdown with visual bars
const modelBreakdown = computed(() => {
  if( !stats.value?.by_model ) return []
  const maxRequests = Math.max( ...stats.value.by_model.map( m => m.requests ) )
  return stats.value.by_model.map( m => ({
    ...m,
    percentage: maxRequests > 0 ? ( m.requests / maxRequests ) * 100 : 0,
  }))
})

function formatDate( timestamp: number ): string {
  return new Date( timestamp ).toLocaleString()
}

function formatCost( cost: number ): string {
  return `$${cost.toFixed( 4 )}`
}

function formatNumber( num: number ): string {
  return num.toLocaleString()
}
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold text-gray-900 mb-6">Usage Analytics</h1>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading usage analytics...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="bg-white rounded-lg shadow p-6">
      <p class="text-red-600">Error loading usage analytics: {{ error.message }}</p>
    </div>

    <!-- Analytics content -->
    <div v-else>
      <!-- Summary statistics -->
      <div class="grid grid-cols-1 md:grid-cols-4 gap-6 mb-6">
        <!-- Total requests -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Total Requests</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-gray-900">
              {{ formatNumber( stats?.total_requests || 0 ) }}
            </div>
          </CardContent>
        </Card>

        <!-- Total input tokens -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Input Tokens</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-blue-600">
              {{ formatNumber( stats?.total_input_tokens || 0 ) }}
            </div>
          </CardContent>
        </Card>

        <!-- Total output tokens -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Output Tokens</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-green-600">
              {{ formatNumber( stats?.total_output_tokens || 0 ) }}
            </div>
          </CardContent>
        </Card>

        <!-- Total cost -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Total Cost</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-purple-600">
              {{ formatCost( stats?.total_cost || 0 ) }}
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- Provider and Model breakdown -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <!-- Provider breakdown -->
        <Card>
          <CardHeader>
            <CardTitle>Usage by Provider</CardTitle>
          </CardHeader>
          <CardContent>
            <div v-if="providerBreakdown.length === 0" class="text-center text-gray-600">
              No provider data available
            </div>
            <div v-else class="space-y-4">
              <div v-for="provider in providerBreakdown" :key="provider.provider">
                <div class="flex justify-between items-center mb-2">
                  <span class="text-sm font-medium text-gray-900">{{ provider.provider }}</span>
                  <div class="text-right">
                    <span class="text-sm font-semibold text-gray-900">
                      {{ formatCost( provider.cost ) }}
                    </span>
                    <span class="text-xs text-gray-500 ml-2">
                      {{ formatNumber( provider.requests ) }} requests
                    </span>
                  </div>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-blue-600 h-2 rounded-full transition-all"
                    :style="{ width: `${provider.percentage}%` }"
                  />
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <!-- Model breakdown -->
        <Card>
          <CardHeader>
            <CardTitle>Usage by Model</CardTitle>
          </CardHeader>
          <CardContent>
            <div v-if="modelBreakdown.length === 0" class="text-center text-gray-600">
              No model data available
            </div>
            <div v-else class="space-y-4">
              <div v-for="model in modelBreakdown" :key="model.model">
                <div class="flex justify-between items-center mb-2">
                  <span class="text-sm font-medium text-gray-900">{{ model.model }}</span>
                  <div class="text-right">
                    <span class="text-sm font-semibold text-gray-900">
                      {{ formatNumber( model.requests ) }} requests
                    </span>
                    <span class="text-xs text-gray-500 ml-2">
                      {{ formatCost( model.cost ) }}
                    </span>
                  </div>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-green-600 h-2 rounded-full transition-all"
                    :style="{ width: `${model.percentage}%` }"
                  />
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- Recent usage records -->
      <Card>
        <CardHeader>
          <CardTitle>Recent Usage</CardTitle>
        </CardHeader>
        <CardContent>
          <div v-if="recentUsage.length === 0" class="text-center text-gray-600">
            No usage records found
          </div>
          <table v-else class="min-w-full divide-y divide-gray-200">
            <thead class="bg-gray-50">
              <tr>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Timestamp
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Token ID
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Provider
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Model
                </th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Input Tokens
                </th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Output Tokens
                </th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Cost
                </th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
              <tr v-for="record in recentUsage" :key="record.id">
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {{ formatDate( record.timestamp ) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {{ record.token_id }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {{ record.provider }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {{ record.model }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                  {{ formatNumber( record.input_tokens ) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">
                  {{ formatNumber( record.output_tokens ) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-right font-medium text-gray-900">
                  {{ formatCost( record.cost ) }}
                </td>
              </tr>
            </tbody>
          </table>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
