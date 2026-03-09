<script setup lang="ts">
import type { ProviderType } from '@/composables/useApi'
import { Badge } from '@/components/ui/badge'
import { getProviderLabel, getProviderBadgeClass } from '@/lib/providers'

defineProps<{ provider: ProviderType }>()
</script>

<template>
  <Badge variant="outline" :class="getProviderBadgeClass(provider)">
    {{ getProviderLabel(provider) }}
  </Badge>
</template>
