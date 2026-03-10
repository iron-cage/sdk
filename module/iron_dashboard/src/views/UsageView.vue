<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useApi, type AnalyticsPeriod, type AnalyticsEvent } from '../composables/useApi'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { formatCostUsd, formatMicrodollars, formatNumber, formatTimestamp } from '@/lib/formatters'
import PageLayout from '@/components/PageLayout.vue'
import StatCard from '@/components/cards/StatCard.vue'
import PercentBar from '@/components/PercentBar.vue'
import DataTable from '@/components/DataTable.vue'
import IconChevronUp from '@/components/icons/IconChevronUp.vue'
import IconChevronDown from '@/components/icons/IconChevronDown.vue'
import IconDownload from '@/components/icons/IconDownload.vue'
import IconBarChart from '@/components/icons/IconBarChart.vue'
import IconCheckCircle from '@/components/icons/IconCheckCircle.vue'
import IconArrowDownToLine from '@/components/icons/IconArrowDownToLine.vue'
import IconArrowUpFromLine from '@/components/icons/IconArrowUpFromLine.vue'
import IconCoin from '@/components/icons/IconCoin.vue'
import IconServer from '@/components/icons/IconServer.vue'
import IconGrid from '@/components/icons/IconGrid.vue'
import IconClipboard from '@/components/icons/IconClipboard.vue'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

const api = useApi()

const selectedAgentId = ref<string>('all')

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
  { value: 'today',       label: 'Today'        },
  { value: 'yesterday',   label: 'Yesterday'    },
  { value: 'last7-days',  label: 'Last 7 Days'  },
  { value: 'last30-days', label: 'Last 30 Days' },
  { value: 'this-month',  label: 'This Month'   },
  { value: 'last-month',  label: 'Last Month'   },
  { value: 'all-time',    label: 'All Time'     },
]

const { data: requestStats, isLoading: requestsLoading, error: requestsError } = useQuery({
  queryKey: ['analytics-requests', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsUsageRequests({ period: selectedPeriod.value, agent_id: selectedAgentId.value !== 'all' ? Number(selectedAgentId.value) : undefined }),
})

const { data: spendingByProvider, isLoading: providerLoading, error: providerError } = useQuery({
  queryKey: ['analytics-spending-provider', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsSpendingByProvider({ period: selectedPeriod.value, agent_id: selectedAgentId.value !== 'all' ? Number(selectedAgentId.value) : undefined }),
})

const { data: modelUsage, isLoading: modelLoading, error: modelError } = useQuery({
  queryKey: ['analytics-models', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsUsageModels({ period: selectedPeriod.value, agent_id: selectedAgentId.value !== 'all' ? Number(selectedAgentId.value) : undefined }),
})

const { data: spendingTotal, isLoading: spendingTotalLoading } = useQuery({
  queryKey: ['analytics-spending-total', selectedPeriod, selectedAgentId],
  queryFn: () => api.getAnalyticsSpendingTotal({ period: selectedPeriod.value, agent_id: selectedAgentId.value !== 'all' ? Number(selectedAgentId.value) : undefined }),
})

const { data: eventsList, isLoading: eventsLoading, isFetching: eventsFetching } = useQuery({
  queryKey: ['analytics-events', selectedPeriod, selectedAgentId, logsPage],
  queryFn: () => api.getAnalyticsEventsList(
    { period: selectedPeriod.value, agent_id: selectedAgentId.value !== 'all' ? Number(selectedAgentId.value) : undefined },
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
  return formatCostUsd(cost, 4)
}

function loadMoreLogs() {
  logsPage.value++
}

function openLogModal(key: AnalyticsEvent) {
  
}


</script>

<template>
  <PageLayout title="Analytics" content-class="p-4 lg:p-6">
    <template #actions>
      <div class="w-full md:w-40">
        <Select v-model="selectedAgentId">
          <SelectTrigger>
            <SelectValue placeholder="All Agents" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Agents</SelectItem>
            <SelectItem v-for="agent in agents" :key="agent.id" :value="String(agent.id)">
              {{ agent.name }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div class="w-full md:w-40">
        <Select v-model="selectedPeriod">
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="option in periodOptions" :key="option.value" :value="option.value">
              {{ option.label }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
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
            <IconBarChart class="h-4 w-4 text-muted-foreground" />
          </template>
          <div class="text-2xl font-bold text-foreground">{{ formatNumber(totalRequests) }}</div>
        </StatCard>

        <StatCard title="Success Rate">
          <template #icon>
            <IconCheckCircle class="h-4 w-4 text-muted-foreground" />
          </template>
          <div class="text-2xl font-bold text-foreground">{{ successRate.toFixed(1) }}%</div>
        </StatCard>

        <StatCard title="Input Tokens">
          <template #icon>
            <IconArrowDownToLine class="h-4 w-4 text-muted-foreground" />
          </template>
          <div class="text-2xl font-bold text-foreground">{{ formatNumber(totalInputTokens) }}</div>
        </StatCard>

        <StatCard title="Output Tokens">
          <template #icon>
            <IconArrowUpFromLine class="h-4 w-4 text-muted-foreground" />
          </template>
          <div class="text-2xl font-bold text-foreground">{{ formatNumber(totalOutputTokens) }}</div>
        </StatCard>

          <StatCard title="Total Cost">
          <template #icon>
            <IconCoin class="h-4 w-4 text-muted-foreground" />
          </template>
          <div class="text-2xl font-bold text-foreground">{{ formatCost(totalSpend) }}</div>
        </StatCard>
      </div>

      <!-- Provider and Model breakdown -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <StatCard title="Usage by Provider" :showSeparator="true">
          <template #icon>
            <IconServer class="h-4 w-4 text-muted-foreground" />
          </template>
          <div v-if="providerBreakdown.length === 0" class="text-center text-muted-foreground mt-6">
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
              <IconChevronUp v-if="showAllProviders" />
              <IconChevronDown v-else />
              {{ showAllProviders ? 'Show less' : `Show all ${providerBreakdown.length}` }}
            </Button>
          </div>
        </StatCard>

        <StatCard title="Usage by Model" :showSeparator="true">
          <template #icon>
            <IconGrid class="h-4 w-4 text-muted-foreground" />
          </template>
          <div v-if="modelBreakdown.length === 0" class="text-center text-muted-foreground mt-6">
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
              <IconChevronUp v-if="showAllModels" />
              <IconChevronDown v-else />
              {{ showAllModels ? 'Show less' : `Show all ${modelBreakdown.length}` }}
            </Button>
          </div>
        </StatCard>
      </div>

      <!-- Recent Logs -->
      <StatCard title="Recent Logs" :showSeparator="true">
        <template #icon>
          <IconClipboard class="h-4 w-4 text-muted-foreground" />
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
            { label: 'Actions', align: 'right' },
          ]"
          :isLoading="eventsLoading && accumulatedLogs.length === 0"
          :isEmpty="accumulatedLogs.length === 0"
          loadingText="Loading logs..."
        >
          <template #empty>
            <p class="text-muted-foreground mt-4">No logs available</p>
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
            <td class="px-3 sm:px-6 py-4 text-right whitespace-nowrap">
              <DropdownMenu>
                <DropdownMenuTrigger as-child>
                  <Button variant="ghost" size="sm">
                    <span class="sr-only">Open menu</span>
                    <IconDotsHorizontal />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem @click="openLogModal(event)">
                    <IconEdit />
                    Edit
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </td>
          </tr>
          <template #footer>
            <div v-if="logsPage < totalPages" class="p-4 text-center">
              <Button variant="outline" @click="loadMoreLogs" :disabled="eventsFetching">
                <IconDownload />
                {{ eventsFetching ? 'Loading...' : 'Load More Logs' }}
              </Button>
            </div>
          </template>
        </DataTable>
      </StatCard>
    </div>
  </PageLayout>
</template>
