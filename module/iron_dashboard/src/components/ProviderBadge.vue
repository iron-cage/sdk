<script setup lang="ts">
import { getProviderLabel, getProviderBadgeClass } from '@/lib/providers'

import { Badge } from '@/components/ui/badge'

defineProps<{ provider: string }>()
</script>

<template>
  <Badge variant="outline" :class="getProviderBadgeClass(provider)">
    {{ getProviderLabel(provider) }}
  </Badge>
</template>
