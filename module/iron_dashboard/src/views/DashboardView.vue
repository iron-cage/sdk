<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { useApi } from '../composables/useApi'
import PageLayout from '@/components/PageLayout.vue'
import DashboardCards from '@/components/DashboardCards.vue'

const api = useApi()

const { data: spending, isLoading: spendingLoading } = useQuery({
  queryKey: ['analytics-spending'],
  queryFn: () => api.getAnalyticsSpendingTotal({ period: 'all-time' }),
})

const { data: requestUsage, isLoading: requestsLoading } = useQuery({
  queryKey: ['analytics-requests'],
  queryFn: () => api.getAnalyticsUsageRequests({ period: 'all-time' }),
})

const { data: agents, isLoading: agentsLoading } = useQuery({
  queryKey: ['agents'],
  queryFn: () => api.getAgents(),
})

const isLoading = spendingLoading || requestsLoading || agentsLoading
</script>

<template>
  <PageLayout title="Dashboard" content-class="p-4 lg:p-6">

    <div v-if="isLoading" class="p-4">
      <p class="text-muted-foreground text-base">Loading dashboard...</p>
    </div>

    <DashboardCards
      v-else
      :spending="spending"
      :requestUsage="requestUsage"
      :agentCount="agents?.length"
    />

  </PageLayout>
</template>
