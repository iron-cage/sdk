<script setup lang="ts">
import StatCard from '@/components/cards/StatCard.vue'
import IconCoin from '@/components/icons/IconCoin.vue'
import IconCheckCircle from '@/components/icons/IconCheckCircle.vue'
import IconCog from '@/components/icons/IconCog.vue'
import { formatCostUsd } from '@/lib/formatters'

defineProps<{
  spending?: { total_spend: number } | null
  requestUsage?: { success_rate: number; successful_requests: number; total_requests: number } | null
  agentCount?: number | null
}>()
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    <StatCard title="Total Spending">
      <template #icon>
        <IconCoin class="h-4 w-4 text-muted-foreground" />
      </template>
      <div class="text-2xl font-semibold text-foreground">
        {{ spending != null ? formatCostUsd(spending.total_spend, 3) : '$0.00' }}
      </div>
      <p class="text-xs text-muted-foreground mt-1">All time</p>
    </StatCard>

    <StatCard title="Success Rate">
      <template #icon>
        <IconCheckCircle class="h-4 w-4 text-muted-foreground" />
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
        <IconCog class="h-4 w-4 text-muted-foreground" />
      </template>
      <div class="text-2xl font-semibold text-foreground">
        {{ agentCount ?? 0 }}
      </div>
      <p class="text-xs text-muted-foreground mt-1">Registered agents</p>
    </StatCard>
  </div>
</template>
