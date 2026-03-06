<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useApi, type AnalyticsPeriod, type AnalyticsEvent } from '../composables/useApi'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import PageLayout from '@/components/PageLayout.vue'

const api = useApi()

// Agent selector
const selectedAgentId = ref<number | null>(null)

// Fetch agents for dropdown
const { data: agents } = useQuery({
  queryKey: ['agents'],
  queryFn: () => api.getAgents(),
})

// Period selector
const selectedPeriod = ref<AnalyticsPeriod>('last7-days')

// Logs pagination - accumulate logs across pages
const logsPage = ref(1)
const logsPerPage = 10
const accumulatedLogs = ref<AnalyticsEvent[]>([])
const totalEvents = ref(0)
const totalPages = ref(1)

const periodOptions: { value: AnalyticsPeriod; label: string }[] = [
  { value: 'today', label: 'Today' },
  { value: 'yesterday', label: 'Yesterday' },
  { value: 'last7-days', label: 'Last 7 Days' },
  { value: 'last30-days', label: 'Last 30 Days' },
  { value: 'this-month', label: 'This Month' },
  { value: 'all-time', label: 'All Time' },
]

// Fetch from Protocol 012 endpoints with agent filter
const { data: requestStats, isLoading: requestsLoading, error: requestsError } = useQuery({
  queryKey: ['analytics-requests', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsUsageRequests({
    period: selectedPeriod.value,
    agent_id: selectedAgentId.value ?? undefined,
  }),
})

const { data: spendingByProvider, isLoading: providerLoading, error: providerError } = useQuery({
  queryKey: ['analytics-spending-provider', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsSpendingByProvider({
    period: selectedPeriod.value,
    agent_id: selectedAgentId.value ?? undefined,
  }),
})

const { data: modelUsage, isLoading: modelLoading, error: modelError } = useQuery({
  queryKey: ['analytics-models', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsUsageModels({
    period: selectedPeriod.value,
    agent_id: selectedAgentId.value ?? undefined,
  }),
})

const { data: spendingTotal, isLoading: spendingTotalLoading } = useQuery({
  queryKey: ['analytics-spending-total', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsSpendingTotal({
    period: selectedPeriod.value,
    agent_id: selectedAgentId.value ?? undefined,
  }),
})

// Fetch recent events/logs
const { data: eventsList, isLoading: eventsLoading, isFetching: eventsFetching } = useQuery({
  queryKey: ['analytics-events', selectedPeriod, selectedAgentId, logsPage],
  queryFn: () => api.getAnalyticsEventsList(
    {
      period: selectedPeriod.value,
      agent_id: selectedAgentId.value ?? undefined,
    },
    {
      page: logsPage.value,
      per_page: logsPerPage,
    }
  ),
})

// Accumulate logs when new data arrives
watch(eventsList, (newData) => {
  if (newData) {
    if (logsPage.value === 1) {
      // First page - replace all
      accumulatedLogs.value = newData.data
    } else {
      // Append new logs
      accumulatedLogs.value = [...accumulatedLogs.value, ...newData.data]
    }
    totalEvents.value = newData.pagination.total
    totalPages.value = newData.pagination.total_pages
  }
}, { immediate: true })

// Reset when filters change
watch([selectedPeriod, selectedAgentId], () => {
  logsPage.value = 1
  accumulatedLogs.value = []
})

const isLoading = computed(() =>
  requestsLoading.value || providerLoading.value || modelLoading.value || spendingTotalLoading.value
)
const error = computed(() =>
  requestsError.value || providerError.value || modelError.value
)

// Computed values from Protocol 012 responses
const totalRequests = computed(() => requestStats.value?.total_requests || 0)
const successRate = computed(() => requestStats.value?.success_rate || 0)
const totalSpend = computed(() => spendingTotal.value?.total_spend || 0)
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

function formatTimestamp(ms: number): string {
  return new Date(ms).toLocaleString()
}

function formatMicrodollars(micros: number): string {
  return `$${(micros / 1_000_000).toFixed(4)}`
}

function loadMoreLogs() {
  logsPage.value++
}
</script>

<template>
  <PageLayout title="Analytics">
    <template #actions>
      <select
        v-model="selectedAgentId"
        class="px-3 py-1.5 text-base border border-border rounded-md bg-background text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
      >
        <option :value="null">All Agents</option>
        <option v-for="agent in agents" :key="agent.id" :value="agent.id">
          {{ agent.name }}
        </option>
      </select>
      <select
        v-model="selectedPeriod"
        class="px-3 py-1.5 text-base border border-border rounded-md bg-background text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
      >
        <option v-for="option in periodOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </template>

    <!-- Loading state -->
    <div v-if="isLoading" class="border border-border rounded-lg p-4">
      <p class="text-muted-foreground">Loading usage analytics...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="border border-border rounded-lg p-4">
      <p class="text-destructive">Error loading usage analytics: {{ (error as Error).message }}</p>
    </div>

    <!-- Analytics content -->
    <div v-else>
      <!-- Summary statistics -->
      <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4 sm:gap-6 mb-6">
        <!-- Total requests -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-base font-medium text-muted-foreground">Total Requests</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-foreground">
              {{ formatNumber(totalRequests) }}
            </div>
          </CardContent>
        </Card>

        <!-- Success rate -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-base font-medium text-muted-foreground">Success Rate</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold" :class="successRate >= 95 ? 'text-foreground' : 'text-foreground'">
              {{ successRate.toFixed(1) }}%
            </div>
          </CardContent>
        </Card>

        <!-- Total input tokens -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-base font-medium text-muted-foreground">Input Tokens</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-foreground">
              {{ formatNumber(totalInputTokens) }}
            </div>
          </CardContent>
        </Card>

        <!-- Total output tokens -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-base font-medium text-muted-foreground">Output Tokens</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-foreground">
              {{ formatNumber(totalOutputTokens) }}
            </div>
          </CardContent>
        </Card>

        <!-- Total cost -->
        <Card>
          <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle class="text-base font-medium text-muted-foreground">Total Cost</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold text-foreground">
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
            <div v-if="providerBreakdown.length === 0" class="text-center text-muted-foreground">
              No provider data available
            </div>
            <div v-else class="space-y-4">
              <div v-for="provider in providerBreakdown" :key="provider.provider">
                <div class="flex justify-between items-center mb-2">
                  <span class="text-base font-medium text-foreground">{{ provider.provider }}</span>
                  <div class="text-right">
                    <span class="text-base font-semibold text-foreground">
                      {{ formatCost(provider.spending) }}
                    </span>
                    <span class="text-xs text-muted-foreground ml-2">
                      {{ formatNumber(provider.request_count) }} requests
                    </span>
                  </div>
                </div>
                <div class="w-full bg-muted rounded-full h-2">
                  <div
                    class="bg-foreground h-2 rounded-full transition-all"
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
            <div v-if="modelBreakdown.length === 0" class="text-center text-muted-foreground">
              No model data available
            </div>
            <div v-else class="space-y-4">
              <div v-for="model in modelBreakdown" :key="model.model">
                <div class="flex justify-between items-center mb-2">
                  <span class="text-base font-medium text-foreground">{{ model.model }}</span>
                  <div class="text-right">
                    <span class="text-base font-semibold text-foreground">
                      {{ formatNumber(model.request_count) }} requests
                    </span>
                    <span class="text-xs text-muted-foreground ml-2">
                      {{ formatCost(model.spending) }}
                    </span>
                  </div>
                </div>
                <div class="w-full bg-muted rounded-full h-2">
                  <div
                    class="bg-foreground h-2 rounded-full transition-all"
                    :style="{ width: `${model.percentage}%` }"
                  />
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- Recent Logs -->
      <Card>
        <CardHeader class="flex flex-row items-center justify-between">
          <CardTitle>Recent Logs</CardTitle>
          <span v-if="totalEvents > 0" class="text-base text-muted-foreground">
            Showing {{ accumulatedLogs.length }} of {{ totalEvents }} events
          </span>
        </CardHeader>
        <CardContent>
          <div v-if="eventsLoading && accumulatedLogs.length === 0" class="text-center text-muted-foreground py-4">
            Loading logs...
          </div>
          <div v-else-if="accumulatedLogs.length === 0" class="text-center text-muted-foreground py-4">
            No logs available
          </div>
          <div v-else>
            <div class="overflow-x-auto touch-pan-x">
              <table class="min-w-[600px] w-full divide-y divide-border">
                <thead>
                  <tr>
                    <th class="px-3 sm:px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Time</th>
                    <th class="px-3 sm:px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Agent</th>
                    <th class="px-3 sm:px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Model</th>
                    <th class="px-3 sm:px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Status</th>
                    <th class="px-3 sm:px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Tokens</th>
                    <th class="px-3 sm:px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Cost</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-border">
                  <tr v-for="event in accumulatedLogs" :key="event.event_id">
                    <td class="px-3 sm:px-4 py-3 whitespace-nowrap text-base text-muted-foreground">
                      {{ formatTimestamp(event.timestamp_ms) }}
                    </td>
                    <td class="px-3 sm:px-4 py-3 whitespace-nowrap text-base text-foreground">
                      {{ event.agent_name }}
                    </td>
                    <td class="px-3 sm:px-4 py-3 whitespace-nowrap text-base text-muted-foreground">
                      {{ event.model }}
                    </td>
                    <td class="px-3 sm:px-4 py-3 whitespace-nowrap">
                      <span
                        class="px-2 py-1 text-xs font-medium rounded-full"
                        :class="event.event_type === 'llm_request_completed'
                          ? 'bg-muted text-foreground'
                          : 'bg-destructive/10 text-destructive'"
                      >
                        {{ event.event_type === 'llm_request_completed' ? 'Success' : 'Failed' }}
                      </span>
                    </td>
                    <td class="px-3 sm:px-4 py-3 whitespace-nowrap text-base text-muted-foreground">
                      {{ formatNumber(event.input_tokens + event.output_tokens) }}
                    </td>
                    <td class="px-3 sm:px-4 py-3 whitespace-nowrap text-base text-muted-foreground">
                      {{ formatMicrodollars(event.cost_micros) }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>

            <!-- Load More Button -->
            <div v-if="logsPage < totalPages" class="mt-4 text-center">
              <Button variant="outline" @click="loadMoreLogs" :disabled="eventsFetching">
                {{ eventsFetching ? 'Loading...' : 'Load More Logs' }}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  </PageLayout>
</template>
