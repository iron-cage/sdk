<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useApi, type AnalyticsPeriod, type AnalyticsEvent } from '../composables/useApi'
import { Button } from '@/components/ui/button'
import PageLayout from '@/components/PageLayout.vue'
import StatCard from '@/components/cards/StatCard.vue'
import PercentBar from '@/components/PercentBar.vue'
import DataTable from '@/components/DataTable.vue'

const api = useApi()

const selectedAgentId = ref<number | null>(null)

const { data: agents } = useQuery({
  queryKey: ['agents'],
  queryFn: () => api.getAgents(),
})

const selectedPeriod = ref<AnalyticsPeriod>('last7-days')

const logsPage = ref(1)
const logsPerPage = 10
const accumulatedLogs = ref<AnalyticsEvent[]>([])
const totalEvents = ref(0)
const totalPages = ref(1)

const periodOptions: { value: AnalyticsPeriod; label: string }[] = [
  { value: 'today',       label: 'Today'       },
  { value: 'yesterday',   label: 'Yesterday'   },
  { value: 'last7-days',  label: 'Last 7 Days' },
  { value: 'last30-days', label: 'Last 30 Days'},
  { value: 'this-month',  label: 'This Month'  },
  { value: 'all-time',    label: 'All Time'    },
]

const { data: requestStats, isLoading: requestsLoading, error: requestsError } = useQuery({
  queryKey: ['analytics-requests', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsUsageRequests({ period: selectedPeriod.value, agent_id: selectedAgentId.value ?? undefined }),
})

const { data: spendingByProvider, isLoading: providerLoading, error: providerError } = useQuery({
  queryKey: ['analytics-spending-provider', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsSpendingByProvider({ period: selectedPeriod.value, agent_id: selectedAgentId.value ?? undefined }),
})

const { data: modelUsage, isLoading: modelLoading, error: modelError } = useQuery({
  queryKey: ['analytics-models', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsUsageModels({ period: selectedPeriod.value, agent_id: selectedAgentId.value ?? undefined }),
})

const { data: spendingTotal, isLoading: spendingTotalLoading } = useQuery({
  queryKey: ['analytics-spending-total', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsSpendingTotal({ period: selectedPeriod.value, agent_id: selectedAgentId.value ?? undefined }),
})

const { data: eventsList, isLoading: eventsLoading, isFetching: eventsFetching } = useQuery({
  queryKey: ['analytics-events', selectedPeriod, selectedAgentId, logsPage],
  queryFn: () => api.getAnalyticsEventsList(
    { period: selectedPeriod.value, agent_id: selectedAgentId.value ?? undefined },
    { page: logsPage.value, per_page: logsPerPage },
  ),
})

watch(eventsList, (newData) => {
  if (newData) {
    accumulatedLogs.value = logsPage.value === 1
      ? newData.data
      : [...accumulatedLogs.value, ...newData.data]
    totalEvents.value = newData.pagination.total
    totalPages.value = newData.pagination.total_pages
  }
}, { immediate: true })

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

const totalRequests = computed(() => requestStats.value?.total_requests || 0)
const successRate = computed(() => requestStats.value?.success_rate || 0)
const totalSpend = computed(() => spendingTotal.value?.total_spend || 0)
const totalInputTokens = computed(() => modelUsage.value?.data.reduce((sum, m) => sum + m.input_tokens, 0) || 0)
const totalOutputTokens = computed(() => modelUsage.value?.data.reduce((sum, m) => sum + m.output_tokens, 0) || 0)

const providerBreakdown = computed(() => {
  const data = spendingByProvider.value?.data ?? []
  if (!data.length) return []
  const maxCost = Math.max(...data.map(p => p.spending), 0.001)
  return data.map(p => ({ ...p, percentage: (p.spending / maxCost) * 100 }))
            .sort((a, b) => b.percentage - a.percentage)
})

const modelBreakdown = computed(() => {
  const data = modelUsage.value?.data ?? []
  if (!data.length) return []
  const maxRequests = Math.max(...data.map(m => m.request_count), 1)
  return data.map(m => ({ ...m, percentage: (m.request_count / maxRequests) * 100 }))
            .sort((a, b) => b.percentage - a.percentage)
})

const BREAKDOWN_LIMIT = 10
const showAllProviders = ref(false)
const showAllModels = ref(false)

const visibleProviders = computed(() =>
  showAllProviders.value ? providerBreakdown.value : providerBreakdown.value.slice(0, BREAKDOWN_LIMIT)
)
const visibleModels = computed(() =>
  showAllModels.value ? modelBreakdown.value : modelBreakdown.value.slice(0, BREAKDOWN_LIMIT)
)

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
  <PageLayout title="Analytics" content-class="p-4 lg:p-6">
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
        <StatCard title="Total Requests">
          <template #icon>
            <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
          </template>
          <div class="text-2xl font-bold text-foreground">{{ formatNumber(totalRequests) }}</div>
        </StatCard>

        <StatCard title="Success Rate">
          <template #icon>
            <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </template>
          <div class="text-2xl font-bold text-foreground">{{ successRate.toFixed(1) }}%</div>
        </StatCard>

        <StatCard title="Input Tokens">
          <template #icon>
            <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
          </template>
          <div class="text-2xl font-bold text-foreground">{{ formatNumber(totalInputTokens) }}</div>
        </StatCard>

        <StatCard title="Output Tokens">
          <template #icon>
            <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
          </template>
          <div class="text-2xl font-bold text-foreground">{{ formatNumber(totalOutputTokens) }}</div>
        </StatCard>

          <StatCard title="Total Cost">
          <template #icon>
            <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </template>
          <div class="text-2xl font-bold text-foreground">{{ formatCost(totalSpend) }}</div>
        </StatCard>
      </div>

      <!-- Provider and Model breakdown -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <StatCard title="Usage by Provider" :showSeparator="true">
          <template #icon>
            <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
            </svg>
          </template>
          <div v-if="providerBreakdown.length === 0" class="text-center text-muted-foreground">
            No provider data available
          </div>
          <div v-else class="space-y-4">
            <div v-for="provider in visibleProviders" :key="provider.provider">
              <div class="flex justify-between items-center mb-2">
                <span class="text-base font-medium text-foreground">{{ provider.provider }}</span>
                <div class="text-right">
                  <span class="text-base font-semibold text-foreground">{{ formatCost(provider.spending) }}</span>
                  <span class="text-xs text-muted-foreground ml-2">{{ formatNumber(provider.request_count) }} requests</span>
                </div>
              </div>
              <PercentBar :percentage="provider.percentage" />
            </div>
            <Button
              v-if="providerBreakdown.length > BREAKDOWN_LIMIT"
              variant="ghost"
              size="sm"
              class="w-full"
              @click="showAllProviders = !showAllProviders"
            >
              {{ showAllProviders ? 'Show less' : `Show all ${providerBreakdown.length}` }}
            </Button>
          </div>
        </StatCard>

        <StatCard title="Usage by Model" :showSeparator="true">
          <template #icon>
            <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3H5a2 2 0 00-2 2v4m6-6h10a2 2 0 012 2v4M9 3v18m0 0h10a2 2 0 002-2V9M9 21H5a2 2 0 01-2-2V9m0 0h18" />
            </svg>
          </template>
          <div v-if="modelBreakdown.length === 0" class="text-center text-muted-foreground">
            No model data available
          </div>
          <div v-else class="space-y-4">
            <div v-for="model in visibleModels" :key="model.model">
              <div class="flex justify-between items-center mb-2">
                <span class="text-base font-medium text-foreground">{{ model.model }}</span>
                <div class="text-right">
                  <span class="text-base font-semibold text-foreground">{{ formatNumber(model.request_count) }} requests</span>
                  <span class="text-xs text-muted-foreground ml-2">{{ formatCost(model.spending) }}</span>
                </div>
              </div>
              <PercentBar :percentage="model.percentage" />
            </div>
            <Button
              v-if="modelBreakdown.length > BREAKDOWN_LIMIT"
              variant="ghost"
              size="sm"
              class="w-full"
              @click="showAllModels = !showAllModels"
            >
              {{ showAllModels ? 'Show less' : `Show all ${modelBreakdown.length}` }}
            </Button>
          </div>
        </StatCard>
      </div>

      <!-- Recent Logs -->
      <StatCard title="Recent Logs" :showSeparator="true">
        <template #icon>
          <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" />
          </svg>
        </template>
        <template #action>
          <span v-if="totalEvents > 0" class="text-base text-muted-foreground">
            Showing {{ accumulatedLogs.length }} of {{ totalEvents }} events
          </span>
        </template>

        <DataTable
          :columns="[
            { label: 'Time' },
            { label: 'Agent' },
            { label: 'Model' },
            { label: 'Status' },
            { label: 'Tokens' },
            { label: 'Cost' },
          ]"
          :isLoading="eventsLoading && accumulatedLogs.length === 0"
          :isEmpty="accumulatedLogs.length === 0"
          loadingText="Loading logs..."
        >
          <template #empty>
            <p class="text-muted-foreground">No logs available</p>
          </template>
          <tr v-for="event in accumulatedLogs" :key="event.event_id">
            <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-muted-foreground">{{ formatTimestamp(event.timestamp_ms) }}</td>
            <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">{{ event.agent_name }}</td>
            <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">{{ event.model }}</td>
            <td class="px-3 sm:px-6 py-4 whitespace-nowrap">
              <span
                class="px-2 py-1 text-xs font-medium rounded-full"
                :class="event.event_type === 'llm_request_completed' ? 'bg-success/10 text-success' : 'bg-destructive/10 text-destructive'"
              >
                {{ event.event_type === 'llm_request_completed' ? 'Success' : 'Failed' }}
              </span>
            </td>
            <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-muted-foreground">{{ formatNumber(event.input_tokens + event.output_tokens) }}</td>
            <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">{{ formatMicrodollars(event.cost_micros) }}</td>
          </tr>
          <template #footer>
            <div v-if="logsPage < totalPages" class="p-4 text-center">
              <Button variant="outline" @click="loadMoreLogs" :disabled="eventsFetching">
                {{ eventsFetching ? 'Loading...' : 'Load More Logs' }}
              </Button>
            </div>
          </template>
        </DataTable>
      </StatCard>
    </div>
  </PageLayout>
</template>
