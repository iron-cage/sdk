<script setup lang="ts">
import { ref, computed } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useApi, type AnalyticsPeriod } from '../composables/useApi'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

const api = useApi()

// Period selector
const selectedPeriod = ref<AnalyticsPeriod>('last7-days')

const periodOptions: { value: AnalyticsPeriod; label: string }[] = [
  { value: 'today', label: 'Today' },
  { value: 'yesterday', label: 'Yesterday' },
  { value: 'last7-days', label: 'Last 7 Days' },
  { value: 'last30-days', label: 'Last 30 Days' },
  { value: 'this-month', label: 'This Month' },
  { value: 'all-time', label: 'All Time' },
]

// Fetch from Protocol 012 endpoints
const { data: requestStats, isLoading: requestsLoading, error: requestsError } = useQuery({
  queryKey: ['analytics-requests', selectedPeriod],
  queryFn: () => api.getAnalyticsUsageRequests({ period: selectedPeriod.value }),
})

const { data: spendingByProvider, isLoading: providerLoading, error: providerError } = useQuery({
  queryKey: ['analytics-spending-provider', selectedPeriod],
  queryFn: () => api.getAnalyticsSpendingByProvider({ period: selectedPeriod.value }),
})

const { data: modelUsage, isLoading: modelLoading, error: modelError } = useQuery({
  queryKey: ['analytics-models', selectedPeriod],
  queryFn: () => api.getAnalyticsUsageModels({ period: selectedPeriod.value }),
})

const isLoading = computed(() =>
  requestsLoading.value || providerLoading.value || modelLoading.value
)
const error = computed(() =>
  requestsError.value || providerError.value || modelError.value
)

// Computed values from Protocol 012 responses
const totalRequests = computed(() => requestStats.value?.total_requests || 0)
const successRate = computed(() => requestStats.value?.success_rate || 0)
const totalSpend = computed(() => spendingByProvider.value?.summary.total_spend || 0)
const totalInputTokens = computed(() =>
  modelUsage.value?.data.reduce((sum, m) => sum + m.input_tokens, 0) || 0
)
const totalOutputTokens = computed(() =>
  modelUsage.value?.data.reduce((sum, m) => sum + m.output_tokens, 0) || 0
)

// Provider breakdown with visual bars
const providerBreakdown = computed(() => {
  if (!spendingByProvider.value?.data) return []
  const data = spendingByProvider.value.data
  const maxCost = Math.max(...data.map(p => p.spending), 0.001)
  return data.map(p => ({
    ...p,
    percentage: maxCost > 0 ? (p.spending / maxCost) * 100 : 0,
  }))
})

// Model breakdown with visual bars
const modelBreakdown = computed(() => {
  if (!modelUsage.value?.data) return []
  const data = modelUsage.value.data
  const maxRequests = Math.max(...data.map(m => m.request_count), 1)
  return data.map(m => ({
    ...m,
    percentage: maxRequests > 0 ? (m.request_count / maxRequests) * 100 : 0,
  }))
})

function formatCost(cost: number): string {
  return `$${cost.toFixed(4)}`
}

function formatNumber(num: number): string {
  return num.toLocaleString()
}
</script>

<template>
  <div>
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold text-gray-900">Usage Analytics</h1>

      <!-- Period selector -->
      <select
        v-model="selectedPeriod"
        class="px-4 py-2 border border-gray-300 rounded-lg bg-white text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option v-for="option in periodOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </div>

    <!-- Loading state -->
    <div v-if="isLoading" class="bg-white rounded-lg shadow p-6">
      <p class="text-gray-600">Loading usage analytics...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="bg-white rounded-lg shadow p-6">
      <p class="text-red-600">Error loading usage analytics: {{ (error as Error).message }}</p>
    </div>

    <!-- Analytics content -->
    <div v-else>
      <!-- Summary statistics -->
      <div class="grid grid-cols-1 md:grid-cols-5 gap-6 mb-6">
        <!-- Total requests -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Total Requests</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-gray-900">
              {{ formatNumber(totalRequests) }}
            </div>
          </CardContent>
        </Card>

        <!-- Success rate -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-sm font-medium text-gray-600">Success Rate</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold" :class="successRate >= 95 ? 'text-green-600' : 'text-yellow-600'">
              {{ successRate.toFixed(1) }}%
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
              {{ formatNumber(totalInputTokens) }}
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
              {{ formatNumber(totalOutputTokens) }}
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
              {{ formatCost(totalSpend) }}
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
                      {{ formatCost(provider.spending) }}
                    </span>
                    <span class="text-xs text-gray-500 ml-2">
                      {{ formatNumber(provider.request_count) }} requests
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
                      {{ formatNumber(model.request_count) }} requests
                    </span>
                    <span class="text-xs text-gray-500 ml-2">
                      {{ formatCost(model.spending) }}
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
    </div>
  </div>
</template>
