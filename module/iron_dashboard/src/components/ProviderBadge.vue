<script setup lang="ts">
import { Badge } from '@/components/ui/badge'
import { getProviderLabel, getProviderBadgeClass } from '@/lib/providers'
import { cn } from '@/lib/utils'

const props = defineProps<{ provider: string; class?: string }>()
</script>

<template>
  <Badge
    variant="outline"
    :class="cn(getProviderBadgeClass(provider), 'max-w-[120px]', props.class)"
    :title="getProviderLabel(provider)"
  >
    <span class="block truncate">{{ getProviderLabel(provider) }}</span>
  </Badge>
</template>
