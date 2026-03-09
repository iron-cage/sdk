<script setup lang="ts">
import StatCard from '@/components/cards/StatCard.vue'

defineProps<{
  spending?: { total_spend: number } | null
  requestUsage?: { success_rate: number; successful_requests: number; total_requests: number } | null
  agentCount?: number | null
}>()

function formatCurrency(usd: number): string {
  return `$${usd.toFixed(3)}`
}
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    <StatCard title="Total Spending">
      <template #icon>
        <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </template>
      <div class="text-2xl font-semibold text-foreground">
        {{ spending != null ? formatCurrency(spending.total_spend) : '$0.00' }}
      </div>
      <p class="text-xs text-muted-foreground mt-1">All time</p>
    </StatCard>

    <StatCard title="Success Rate">
      <template #icon>
        <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </template>
      <div class="text-2xl font-semibold text-foreground">
        {{ requestUsage != null ? requestUsage.success_rate.toFixed(1) : '0' }}%
      </div>
      <p class="text-xs text-muted-foreground mt-1">
        {{ requestUsage != null ? `${requestUsage.successful_requests} / ${requestUsage.total_requests} requests` : 'No requests' }}
      </p>
    </StatCard>

    <StatCard title="Active Agents">
      <template #icon>
        <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </template>
      <div class="text-2xl font-semibold text-foreground">
        {{ agentCount ?? 0 }}
      </div>
      <p class="text-xs text-muted-foreground mt-1">Registered agents</p>
    </StatCard>
  </div>
</template>
