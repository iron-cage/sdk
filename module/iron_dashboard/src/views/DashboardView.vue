<script setup lang="ts">
import { computed } from 'vue'
import { formatTimestamp, formatCostUsd, formatNumber } from '@/lib/formatters'
import { useRouter } from 'vue-router'
import { useQuery } from '@tanstack/vue-query'
import { useApi } from '../composables/useApi'
import { useAuthStore } from '../stores/auth'
import StatCard from '@/components/cards/StatCard.vue'
import IconCoin from '@/components/icons/IconCoin.vue'
import IconBarChart from '@/components/icons/IconBarChart.vue'
import IconCheckCircle from '@/components/icons/IconCheckCircle.vue'
import IconCog from '@/components/icons/IconCog.vue'
import IconServer from '@/components/icons/IconServer.vue'
import IconUsers from '@/components/icons/IconUsers.vue'
import IconChevronRight from '@/components/icons/IconChevronRight.vue'
import PageLayout from '@/components/PageLayout.vue'

const router = useRouter()
const authStore = useAuthStore()
const api = useApi()

const { data: spending } = useQuery({
  queryKey: ['analytics-spending'],
  queryFn: () => api.getAnalyticsSpendingTotal({ period: 'all-time' }),
})
const { data: requestUsage } = useQuery({
  queryKey: ['analytics-requests'],
  queryFn: () => api.getAnalyticsUsageRequests({ period: 'all-time' }),
})
const { data: agents } = useQuery({
  queryKey: ['agents'],
  queryFn: () => api.getAgents(),
})
const { data: agentSpending } = useQuery({
  queryKey: ['analytics-spending-agent-dashboard'],
  queryFn: () => api.getAnalyticsSpendingByAgent({ period: 'all-time' }, { per_page: 5 }),
})

const { data: health, isLoading: healthLoading, isError: healthError } = useQuery({
  queryKey: ['health'],
  queryFn: () => api.getHealth(),
  refetchInterval: 30_000,
  retry: 0,
})

const totalSpend     = computed(() => spending.value?.total_spend ?? 0)
const totalRequests  = computed(() => requestUsage.value?.total_requests ?? 0)
const successRate    = computed(() => requestUsage.value?.success_rate ?? 0)
const successfulReqs = computed(() => requestUsage.value?.successful_requests ?? 0)
const agentCount     = computed(() => agents.value?.length ?? 0)
const topSpenders    = computed(() => agentSpending.value?.data ?? [])
</script>

<template>
  <PageLayout title="Dashboard" content-class="p-4 lg:p-6 space-y-6">
    <!-- System health bar -->
    <div class="flex items-center justify-between px-4 py-3 bg-card border border-border rounded-lg">
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2">
          <span
            class="h-2 w-2 rounded-full"
            :class="healthLoading ? 'bg-muted-foreground' : healthError ? 'bg-destructive' : 'bg-success animate-pulse'"
          />
          <span class="text-sm font-medium text-foreground">
            {{ healthLoading ? 'Checking...' : healthError ? 'Service Unavailable' : 'All Systems Operational' }}
          </span>
        </div>
        <template v-if="!healthLoading && health">
          <span class="text-muted-foreground max-sm:hidden">·</span>
          <span class="text-xs text-muted-foreground max-sm:hidden">Last checked {{ formatTimestamp(health.timestamp) }}</span>
        </template>
      </div>
      <RouterLink
        :to="{ name: 'usage' }"
        class="text-xs font-medium text-primary hover:text-primary/80 transition-colors"
      >
        View Analytics
      </RouterLink>
    </div>

    <!-- Stat cards -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-6">
      <!-- Total Spending -->
      <div class="bg-card border border-border rounded-lg p-5">
        <div class="flex items-start justify-between mb-3">
          <span class="text-xs text-foreground">Total Spending</span>
          <IconCoin class="h-4 w-4 text-muted-foreground" />
        </div>
        <p class="text-2xl font-semibold text-foreground tracking-tight">{{ formatCostUsd(totalSpend, 3) }}</p>
        <p class="text-xs text-muted-foreground mt-1">All time</p>
      </div>

      <!-- Total Requests -->
      <div class="bg-card border border-border rounded-lg p-5">
        <div class="flex items-start justify-between mb-3">
          <span class="text-xs text-foreground">Total Requests</span>
          <IconBarChart class="h-4 w-4 text-muted-foreground" />
        </div>
        <p class="text-2xl font-semibold text-foreground tracking-tight">{{ formatNumber(totalRequests) }}</p>
        <p class="text-xs text-muted-foreground mt-1">API calls made</p>
      </div>

      <!-- Success Rate -->
      <div class="bg-card border border-border rounded-lg p-5">
        <div class="flex items-start justify-between mb-3">
          <span class="text-xs text-foreground">Success Rate</span>
          <IconCheckCircle class="h-4 w-4 text-muted-foreground" />
        </div>
        <p class="text-2xl font-semibold text-foreground tracking-tight">{{ successRate.toFixed(1) }}%</p>
        <p class="text-xs text-muted-foreground mt-1">{{ formatNumber(successfulReqs) }}/{{ formatNumber(totalRequests) }} successful</p>
      </div>

      <!-- Active Agents -->
      <div class="bg-card border border-border rounded-lg p-5">
        <div class="flex items-start justify-between mb-3">
          <span class="text-xs text-foreground">Active Agents</span>
          <IconCog class="h-4 w-4 text-muted-foreground" />
        </div>
        <p class="text-2xl font-semibold text-foreground tracking-tight">{{ agentCount }}</p>
        <p class="text-xs text-muted-foreground mt-1">Registered agents</p>
      </div>
    </div>

    <!-- Quick Actions + Top Spenders row -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">

      <!-- Quick Actions -->
      <StatCard title="Quick Actions" :show-separator="true" :grow="true">
        <template #icon>
          <IconCog class="h-4 w-4 text-muted-foreground" />
        </template>
        <div class="space-y-2 h-full flex flex-col justify-center">
          <button
            v-if="authStore.isAdmin"
            class="w-full flex items-center justify-between p-3 border border-border rounded-md hover:bg-muted/50 hover:border-primary/30 transition-all group"
            @click="router.push({ name: 'providers' })"
          >
            <div class="flex items-center gap-3">
              <div class="h-8 w-8 rounded-md bg-primary/10 flex items-center justify-center transition-colors">
                <IconServer class="h-4 w-4 text-primary transition-colors" />
              </div>
              <span class="text-sm font-medium text-foreground">Add Provider</span>
            </div>
            <IconChevronRight class="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors" />
          </button>

          <button
            class="w-full flex items-center justify-between p-3 border border-border rounded-md hover:bg-muted/50 hover:border-primary/30 transition-all group"
            @click="router.push({ name: 'agents' })"
          >
            <div class="flex items-center gap-3">
              <div class="h-8 w-8 rounded-md bg-primary/10 flex items-center justify-center transition-colors">
                <IconUsers class="h-4 w-4 text-primary transition-colors" />
              </div>
              <span class="text-sm font-medium text-foreground">Create Agent</span>
            </div>
            <IconChevronRight class="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors" />
          </button>

          <button
            class="w-full flex items-center justify-between p-3 border border-border rounded-md hover:bg-muted/50 hover:border-primary/30 transition-all group"
            @click="router.push({ name: 'limits' })"
          >
            <div class="flex items-center gap-3">
              <div class="h-8 w-8 rounded-md bg-primary/10 flex items-center justify-center transition-colors">
                <IconCoin class="h-4 w-4 text-primary transition-colors" />
              </div>
              <span class="text-sm font-medium text-foreground">Set Budget</span>
            </div>
            <IconChevronRight class="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors" />
          </button>
        </div>
      </StatCard>

      <!-- Top Spenders -->
      <StatCard title="Top Spenders" :show-separator="true" :grow="true">
        <template #icon>
          <IconUsers class="h-4 w-4 text-muted-foreground" />
        </template>
        <template #action>
          <RouterLink :to="{ name: 'usage' }" class="text-xs font-medium text-primary hover:text-primary/80 transition-colors">
            View all
          </RouterLink>
        </template>
        <div class="space-y-3">
          <p v-if="topSpenders.length === 0" class="text-sm text-muted-foreground text-center py-2">No data yet</p>
          <div
            v-for="(agent, i) in topSpenders"
            :key="agent.agent_id"
            class="flex items-center justify-between"
          >
            <div class="flex items-center gap-3 min-w-0">
              <span class="text-xs text-muted-foreground w-4 shrink-0">{{ i + 1 }}</span>
              <div class="min-w-0">
                <p class="text-sm font-medium text-foreground truncate" :title="agent.agent_name">{{ agent.agent_name }}</p>
                <p class="text-xs text-muted-foreground">{{ formatNumber(agent.request_count) }} req</p>
              </div>
            </div>
            <span class="text-sm font-semibold text-foreground shrink-0 ml-4">
              {{ formatCostUsd(agent.spending, 2) }}
            </span>
          </div>
        </div>
      </StatCard>
    </div>
  </PageLayout>
</template>
