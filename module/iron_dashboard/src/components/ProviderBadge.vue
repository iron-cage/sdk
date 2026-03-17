<script setup lang="ts">
import { Badge } from '@/components/ui/badge'
import { getProviderLabel, getProviderBadgeClass } from '@/lib/providers'

defineProps<{ provider: string }>()
</script>

<template>
  <Badge variant="outline" :class="getProviderBadgeClass(provider)">
    {{ getProviderLabel(provider) }}
  </Badge>
</template>
