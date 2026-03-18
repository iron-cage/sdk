<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useApi, type AnalyticsPeriod, type AnalyticsEvent, type AgentSpending } from '../composables/useApi'
import IconChip from '@/components/icons/IconChip.vue'
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
import IconChevronLeft from '@/components/icons/IconChevronLeft.vue'
import IconChevronRight from '@/components/icons/IconChevronRight.vue'
import IconDownload from '@/components/icons/IconDownload.vue'
import IconBarChart from '@/components/icons/IconBarChart.vue'
import IconCheckCircle from '@/components/icons/IconCheckCircle.vue'
import IconArrowDownToLine from '@/components/icons/IconArrowDownToLine.vue'
import IconArrowUpFromLine from '@/components/icons/IconArrowUpFromLine.vue'
import IconCoin from '@/components/icons/IconCoin.vue'
import IconServer from '@/components/icons/IconServer.vue'
import IconGrid from '@/components/icons/IconGrid.vue'
import IconClipboard from '@/components/icons/IconClipboard.vue'
import IconUsers from '@/components/icons/IconUsers.vue'
import IconExternalLink from '@/components/icons/IconExternalLink.vue'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {cn} from "@/lib/utils"
import TrendBadge from '@/components/TrendBadge.vue'
import { getProviderLabel } from '@/lib/providers'

const api = useApi()

const selectedAgentId = ref<string>('all')
const selectedProviderId = ref<string>('all')
const selectedPeriod = ref<AnalyticsPeriod>('last7-days')

const logsPage = ref(1)
const logsPerPage = 10
const accumulatedLogs = ref<AnalyticsEvent[]>([])
const totalEvents = ref(0)
const totalPages = ref(1)

const ANALYTICS_PER_PAGE = 10
const agentSpendingPage = ref(1)
const modelsPage = ref(1)
const tokensPage = ref(1)

const periodOptions: { value: AnalyticsPeriod; label: string }[] = [
  { value: 'today',       label: 'Today'        },
  { value: 'yesterday',   label: 'Yesterday'    },
  { value: 'last7-days',  label: 'Last 7 Days'  },
  { value: 'last30-days', label: 'Last 30 Days' },
  { value: 'this-month',  label: 'This Month'   },
  { value: 'last-month',  label: 'Last Month'   },
  { value: 'all-time',    label: 'All Time'     },
]

const { data: agents } = useQuery({
  queryKey: ['agents'],
  queryFn: () => api.getAgents(),
})

// Provider list for the dropdown — uses the providers endpoint so all
// configured providers appear regardless of whether they have usage in the current period.
const { data: providerList } = useQuery({
  queryKey: ['providers'],
  queryFn: () => api.getProviderKeys(),
})

const activeFilters = computed(() => ({
  period: selectedPeriod.value,
  agent_id: selectedAgentId.value !== 'all' ? Number(selectedAgentId.value) : undefined,
  provider_key_id: selectedProviderId.value !== 'all' ? Number(selectedProviderId.value) : undefined,
}))

const { data: requestStats, isLoading: requestsLoading, error: requestsError } = useQuery({
  queryKey: ['analytics-requests', selectedPeriod, selectedAgentId, selectedProviderId],
  queryFn: () => api.getAnalyticsUsageRequests({ ...activeFilters.value, compare: true }),
})

const { data: spendingByProvider, isLoading: providerLoading, error: providerError } = useQuery({
  queryKey: ['analytics-spending-provider', selectedPeriod, selectedAgentId, selectedProviderId],
  queryFn: () => api.getAnalyticsSpendingByProvider(activeFilters.value),
})

const { data: modelUsage, isLoading: modelLoading, error: modelError } = useQuery({
  queryKey: ['analytics-models', selectedPeriod, selectedAgentId, selectedProviderId, modelsPage],
  queryFn: () => api.getAnalyticsUsageModels(activeFilters.value, { page: modelsPage.value, per_page: ANALYTICS_PER_PAGE }),
})

const { data: spendingTotal, isLoading: spendingTotalLoading } = useQuery({
  queryKey: ['analytics-spending-total', selectedPeriod, selectedAgentId, selectedProviderId],
  queryFn: () => api.getAnalyticsSpendingTotal({ ...activeFilters.value, compare: true }),
})

const { data: eventsList, isLoading: eventsLoading, isFetching: eventsFetching, error: eventsError } = useQuery({
  queryKey: ['analytics-events', selectedPeriod, selectedAgentId, selectedProviderId, logsPage],
  queryFn: () => api.getAnalyticsEventsList(activeFilters.value, { page: logsPage.value, per_page: logsPerPage }),
})

const { data: spendingByAgent, isLoading: agentSpendingLoading } = useQuery({
  queryKey: ['analytics-spending-agent', selectedPeriod, selectedAgentId, selectedProviderId, agentSpendingPage],
  queryFn: () => api.getAnalyticsSpendingByAgent(activeFilters.value, { page: agentSpendingPage.value, per_page: ANALYTICS_PER_PAGE }),
})

const { data: avgCostData } = useQuery({
  queryKey: ['analytics-avg-cost', selectedPeriod, selectedAgentId, selectedProviderId],
  queryFn: () => api.getAnalyticsSpendingAvgPerRequest(activeFilters.value),
})

const { data: tokensByAgent, isLoading: tokensByAgentLoading } = useQuery({
  queryKey: ['analytics-tokens-by-agent', selectedPeriod, selectedAgentId, selectedProviderId, tokensPage],
  queryFn: () => api.getAnalyticsUsageTokensByAgent(activeFilters.value, { page: tokensPage.value, per_page: ANALYTICS_PER_PAGE }),
})

watch(eventsList, (newData) => {
  if (newData) {
    if (logsPage.value === 1) {
      accumulatedLogs.value = newData.data
    } else {
      const existingIds = new Set(accumulatedLogs.value.map(e => e.event_id))
      const newEvents = newData.data.filter(e => !existingIds.has(e.event_id))
      accumulatedLogs.value = [...accumulatedLogs.value, ...newEvents]
    }
    totalEvents.value = newData.pagination.total
    totalPages.value = newData.pagination.total_pages
  }
}, { immediate: true })

watch([selectedPeriod, selectedAgentId, selectedProviderId], () => {
  logsPage.value = 1
  accumulatedLogs.value = []
  agentSpendingPage.value = 1
  modelsPage.value = 1
  tokensPage.value = 1
})

const agentBreakdown = computed<AgentSpending[]>(() => {
  return spendingByAgent.value?.data ?? []
})

const isLoading = computed(() =>
  requestsLoading.value || providerLoading.value || modelLoading.value || spendingTotalLoading.value
)
const error = computed(() =>
  requestsError.value || providerError.value || modelError.value || eventsError.value
)

const totalRequests = computed(() => requestStats.value?.total_requests || 0)
const successRate = computed(() => requestStats.value?.success_rate || 0)
const totalSpend = computed(() => spendingTotal.value?.total_spend || 0)
const totalInputTokens = computed(() => tokensByAgent.value?.summary?.total_input_tokens ?? 0)
const totalOutputTokens = computed(() => tokensByAgent.value?.summary?.total_output_tokens ?? 0)

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

const visibleProviders = computed(() =>
  showAllProviders.value ? providerBreakdown.value : providerBreakdown.value.slice(0, BREAKDOWN_LIMIT)
)

function formatCost(cost: number): string {
  return formatCostUsd(cost, 4)
}

function loadMoreLogs() {
  logsPage.value++
}

const selectedLog = ref<AnalyticsEvent | null>(null)
const showLogModal = ref(false)

function openLogModal(event: AnalyticsEvent) {
  selectedLog.value = event
  showLogModal.value = true
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
        <Select v-model="selectedProviderId">
          <SelectTrigger>
            <SelectValue placeholder="All Providers" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Providers</SelectItem>
            <SelectItem
              v-for="p in providerList"
              :key="p.id"
              :value="String(p.id)"
            >
              {{ p.alias || getProviderLabel(p.provider) }}
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
      <p class="text-destructive">Error loading usage analytics: {{ error instanceof Error ? error.message : String(error) }}</p>
    </div>

    <!-- Analytics content -->
    <div v-else>
      <!-- Summary statistics -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6 mb-6">
        <StatCard title="Total Requests">
          <template #icon>
            <IconBarChart class="h-4 w-4 text-muted-foreground" />
          </template>
          <div class="text-2xl font-bold text-foreground">{{ formatNumber(totalRequests) }}</div>
          <TrendBadge :change-percent="requestStats?.previous_period?.change_percent" class="mt-1" />
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

        <StatCard title="Success Rate">
          <template #icon>
            <IconCheckCircle class="h-4 w-4 text-muted-foreground" />
          </template>
          <div class="text-2xl font-bold text-foreground">{{ successRate.toFixed(1) }}%</div>

          <div class="flex gap-2 items-center">
            <TrendBadge
            v-if="requestStats?.previous_period"
            :change-percent="(requestStats.success_rate - requestStats.previous_period.success_rate)"
            class="mt-1"
          />

          <span v-if="requestStats?.failed_requests && requestStats.previous_period" class="size-1 inline-block bg-muted-foreground rounded-full"></span>

          <div v-if="requestStats?.failed_requests" class="text-xs text-destructive mt-1">
            {{ formatNumber(requestStats.failed_requests) }} failed
          </div>

          </div>
        </StatCard>

        <StatCard title="Total Cost">
          <template #icon>
            <IconCoin class="h-4 w-4 text-muted-foreground" />
          </template>
          <div class="text-2xl font-bold text-foreground">{{ formatCost(totalSpend) }}</div>
          <TrendBadge :change-percent="spendingTotal?.previous_period?.change_percent" class="mt-1" />
        </StatCard>

        <StatCard title="Avg Cost / Request">
          <template #icon>
            <IconCoin class="h-4 w-4 text-muted-foreground" />
          </template>
          <div :class="cn('text-2xl font-bold', avgCostData ? 'text-foreground' : 'text-muted-foreground')">
            {{ avgCostData ? formatCost(avgCostData.average_cost_per_request) : "No data" }}
          </div>
          <div v-if="avgCostData?.median_cost_per_request != null" class="text-xs text-muted-foreground mt-1">
            median {{ formatCost(avgCostData.median_cost_per_request) }}
          </div>
        </StatCard>
      </div>

      <!-- Provider and Model breakdown -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <StatCard title="Usage by Provider" :show-separator="true">
          <template #icon>
            <IconServer class="h-4 w-4 text-muted-foreground" />
          </template>
          <div v-if="providerBreakdown.length === 0" class="text-center text-muted-foreground mt-6">
            No provider data available
          </div>
          <div v-else class="space-y-4">
            <div v-for="provider in visibleProviders" :key="provider.provider">
              <div class="flex justify-between items-center mb-2">
                <span class="text-base font-medium text-foreground">{{ provider.alias || getProviderLabel(provider.provider) }}</span>
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

        <StatCard title="Usage by Model" :show-separator="true">
          <template #icon>
            <IconGrid class="h-4 w-4 text-muted-foreground" />
          </template>
          <div v-if="modelBreakdown.length === 0" class="text-center text-muted-foreground mt-6">
            No model data available
          </div>
          <div v-else class="space-y-4">
            <div v-for="model in modelBreakdown" :key="model.model">
              <div class="flex justify-between items-center mb-2">
                <div>
                  <span class="text-base font-medium text-foreground">{{ model.model }}</span>
                  <span class="text-xs text-muted-foreground block">{{ getProviderLabel(model.provider) }}</span>
                </div>
                <div class="text-right">
                  <span class="text-base font-semibold text-foreground">{{ formatNumber(model.request_count) }} requests</span>
                  <span class="text-xs text-muted-foreground ml-2">{{ formatCost(model.spending) }}</span>
                  <span class="text-xs text-muted-foreground block">
                    {{ formatCost((model.spending / ((model.input_tokens + model.output_tokens) || 1)) * 1000) }}/1k tokens
                  </span>
                </div>
              </div>
              <PercentBar :percentage="model.percentage" />
            </div>
            <div v-if="modelUsage?.pagination && modelUsage.pagination.total_pages > 1" class="flex items-center justify-between pt-2 border-t border-border">
              <p class="text-xs text-muted-foreground">
                Page <span class="font-medium">{{ modelsPage }}</span> of <span class="font-medium">{{ modelUsage.pagination.total_pages }}</span>
                · <span class="font-medium">{{ modelUsage.pagination.total }}</span> models
              </p>
              <div class="flex gap-2">
                <Button variant="outline" size="sm" :disabled="modelsPage === 1" @click="modelsPage--">
                  <IconChevronLeft />Previous
                </Button>
                <Button variant="outline" size="sm" :disabled="modelsPage >= modelUsage.pagination.total_pages" @click="modelsPage++">
                  Next<IconChevronRight />
                </Button>
              </div>
            </div>
          </div>
        </StatCard>
      </div>

      <!-- Spending by Agent -->
      <StatCard title="Spending by Agent" :show-separator="true" class="mb-6">
        <template #icon>
          <IconUsers class="h-4 w-4 text-muted-foreground" />
        </template>
        <template v-if="spendingByAgent?.summary" #action>
          <span class="text-xs text-muted-foreground">
            Total {{ formatCost(spendingByAgent.summary.total_spend) }}
            · avg {{ spendingByAgent.summary.total_budget > 0 ? ((spendingByAgent.summary.total_spend / spendingByAgent.summary.total_budget) * 100).toFixed(1) : '0.0' }}% budget used
          </span>
        </template>
        <DataTable
          :columns="[
            { label: 'Agent' },
            { label: 'Spending' },
            { label: 'Budget' },
            { label: 'Used' },
            { label: 'Requests' },
          ]"
          :is-loading="agentSpendingLoading"
          :is-empty="agentBreakdown.length === 0"
          loading-text="Loading agent spending..."
        >
          <template #empty>
            <p class="text-muted-foreground">No agent spending data available</p>
          </template>
          <tr v-for="agent in agentBreakdown" :key="agent.agent_id">
            <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base font-medium text-foreground">{{ agent.agent_name }}</td>
            <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">{{ formatCost(agent.spending) }}</td>
            <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">{{ formatCost(agent.budget) }}</td>
            <td class="px-3 sm:px-6 py-2 text-base text-foreground">
              <div class="flex items-center gap-2 min-w-[100px]">
                <PercentBar :percentage="agent.percent_used" class="max-w-[100px]" />
                <span class="shrink-0 text-muted-foreground text-xs">{{ agent.percent_used.toFixed(1) }}%</span>
              </div>
            </td>
            <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">{{ formatNumber(agent.request_count) }}</td>
          </tr>
          <template #footer>
            <div v-if="spendingByAgent?.pagination && spendingByAgent.pagination.total_pages > 1" class="px-4 py-3 flex items-center justify-between border-t border-border sm:px-6">
              <p class="text-xs text-muted-foreground">
                Showing <span class="font-medium">{{ (agentSpendingPage - 1) * ANALYTICS_PER_PAGE + 1 }}</span>
                – <span class="font-medium">{{ Math.min(agentSpendingPage * ANALYTICS_PER_PAGE, spendingByAgent.pagination.total) }}</span>
                of <span class="font-medium">{{ spendingByAgent.pagination.total }}</span>
              </p>
              <div class="flex gap-2">
                <Button variant="outline" size="sm" :disabled="agentSpendingPage === 1" @click="agentSpendingPage--">
                  <IconChevronLeft />Previous
                </Button>
                <Button variant="outline" size="sm" :disabled="agentSpendingPage >= spendingByAgent.pagination.total_pages" @click="agentSpendingPage++">
                  Next<IconChevronRight />
                </Button>
              </div>
            </div>
          </template>
        </DataTable>
      </StatCard>

      <!-- Token Usage by Agent -->
      <StatCard title="Token Usage by Agent" :show-separator="true" class="mb-6">
        <template #icon>
          <IconChip class="h-4 w-4 text-muted-foreground" />
        </template>
        <template v-if="tokensByAgent?.summary" #action>
          <span class="text-xs text-muted-foreground">
            {{ formatNumber(tokensByAgent.summary.total_tokens) }} total tokens
            · {{ formatNumber(tokensByAgent.summary.total_input_tokens) }} in
            · {{ formatNumber(tokensByAgent.summary.total_output_tokens) }} out
          </span>
        </template>
        <DataTable
          :columns="[
            { label: 'Agent' },
            { label: 'Input Tokens' },
            { label: 'Output Tokens' },
            { label: 'Total Tokens' },
            { label: 'Requests' },
            { label: 'Avg / Request' },
          ]"
          :is-loading="tokensByAgentLoading"
          :is-empty="!tokensByAgent?.data?.length"
          loading-text="Loading token usage..."
        >
          <template #empty>
            <p class="text-muted-foreground">No token usage data available</p>
          </template>
          <tr v-for="row in tokensByAgent?.data" :key="row.agent_id">
            <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base font-medium text-foreground">{{ row.agent_name }}</td>
            <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">{{ formatNumber(row.input_tokens) }}</td>
            <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-foreground">{{ formatNumber(row.output_tokens) }}</td>
            <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base font-medium text-foreground">{{ formatNumber(row.total_tokens) }}</td>
            <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">{{ formatNumber(row.request_count) }}</td>
            <td class="px-3 sm:px-6 py-2 whitespace-nowrap text-base text-muted-foreground">{{ formatNumber(Math.round(row.avg_tokens_per_request)) }}</td>
          </tr>
          <template #footer>
            <div v-if="tokensByAgent?.pagination && tokensByAgent.pagination.total_pages > 1" class="px-4 py-3 flex items-center justify-between border-t border-border sm:px-6">
              <p class="text-xs text-muted-foreground">
                Showing <span class="font-medium">{{ (tokensPage - 1) * ANALYTICS_PER_PAGE + 1 }}</span>
                – <span class="font-medium">{{ Math.min(tokensPage * ANALYTICS_PER_PAGE, tokensByAgent.pagination.total) }}</span>
                of <span class="font-medium">{{ tokensByAgent.pagination.total }}</span>
              </p>
              <div class="flex gap-2">
                <Button variant="outline" size="sm" :disabled="tokensPage === 1" @click="tokensPage--">
                  <IconChevronLeft />Previous
                </Button>
                <Button variant="outline" size="sm" :disabled="tokensPage >= tokensByAgent.pagination.total_pages" @click="tokensPage++">
                  Next<IconChevronRight />
                </Button>
              </div>
            </div>
          </template>
        </DataTable>
      </StatCard>

      <!-- Recent Logs -->
      <StatCard title="Recent Logs" :show-separator="true">
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
          :is-loading="eventsLoading && accumulatedLogs.length === 0"
          :is-empty="accumulatedLogs.length === 0"
          loading-text="Loading logs..."
        >
          <template #empty>
            <p class="text-muted-foreground mt-4">No logs available</p>
          </template>
          <tr v-for="event in accumulatedLogs" :key="event.event_id">
            <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-muted-foreground">{{ formatTimestamp(event.timestamp_ms) }}</td>
            <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">{{ event.agent_name }}</td>
            <td class="px-3 sm:px-6 py-4 whitespace-nowrap text-base text-foreground">{{ event.model }}</td>
            <td class="px-3 sm:px-6 py-4">
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
              <Button variant="ghost" size="sm" @click="openLogModal(event)">
                <IconExternalLink class="h-4 w-4" />
              </Button>
            </td>
          </tr>
          <template #footer>
            <div v-if="logsPage < totalPages" class="p-4 text-center">
              <Button variant="outline" :disabled="eventsFetching" @click="loadMoreLogs">
                <IconDownload />
                {{ eventsFetching ? 'Loading...' : 'Load More Logs' }}
              </Button>
            </div>
          </template>
        </DataTable>
      </StatCard>
    </div>

    <!-- Log detail modal -->
    <Dialog v-model:open="showLogModal">
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle class="flex items-center gap-2">
            <span
              class="px-2 py-0.5 text-xs font-medium rounded-full"
              :class="selectedLog?.event_type === 'llm_request_completed' ? 'bg-success/10 text-success' : 'bg-destructive/10 text-destructive'"
            >
              {{ selectedLog?.event_type === 'llm_request_completed' ? 'Success' : 'Failed' }}
            </span>
            Event Details
          </DialogTitle>
        </DialogHeader>

        <div v-if="selectedLog" class="space-y-4 text-sm">
          <!-- Error block (only for failed) -->
          <div v-if="selectedLog.event_type !== 'llm_request_completed'" class="rounded-md border border-destructive/30 bg-destructive/5 p-3 space-y-1">
            <p class="font-medium text-destructive">{{ selectedLog.error_code ?? 'Unknown error' }}</p>
            <p v-if="selectedLog.error_message" class="text-destructive/80">{{ selectedLog.error_message }}</p>
          </div>

          <!-- Fields grid -->
          <div class="grid grid-cols-2 gap-x-6 gap-y-3">
            <div>
              <p class="text-xs text-muted-foreground mb-0.5">Event ID</p>
              <p class="font-mono text-xs text-foreground truncate">{{ selectedLog.event_id }}</p>
            </div>
            <div>
              <p class="text-xs text-muted-foreground mb-0.5">Time</p>
              <p class="text-foreground">{{ formatTimestamp(selectedLog.timestamp_ms) }}</p>
            </div>
            <div>
              <p class="text-xs text-muted-foreground mb-0.5">Agent</p>
              <p class="text-foreground">{{ selectedLog.agent_name }}</p>
            </div>
            <div>
              <p class="text-xs text-muted-foreground mb-0.5">Provider</p>
              <p class="text-foreground">{{ getProviderLabel(selectedLog.provider) }}</p>
            </div>
            <div>
              <p class="text-xs text-muted-foreground mb-0.5">Model</p>
              <p class="text-foreground">{{ selectedLog.model }}</p>
            </div>
            <div>
              <p class="text-xs text-muted-foreground mb-0.5">Cost</p>
              <p class="text-foreground">{{ formatMicrodollars(selectedLog.cost_micros) }}</p>
            </div>
            <div>
              <p class="text-xs text-muted-foreground mb-0.5">Input Tokens</p>
              <p class="text-foreground">{{ formatNumber(selectedLog.input_tokens) }}</p>
            </div>
            <div>
              <p class="text-xs text-muted-foreground mb-0.5">Output Tokens</p>
              <p class="text-foreground">{{ formatNumber(selectedLog.output_tokens) }}</p>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  </PageLayout>
</template>
