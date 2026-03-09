<script setup lang="ts">
import { Badge } from '@/components/ui/badge'

withDefaults(defineProps<{
  active: boolean
  activeLabel?: string
  inactiveLabel?: string
}>(), {
  activeLabel: 'Active',
  inactiveLabel: 'Inactive',
})
</script>

<template>
  <Badge variant="outline" :class="active ? 'text-success border-none' : 'text-muted-foreground border-none'">
    <svg width="7" height="7" viewBox="0 0 12 12" fill="none" class="mr-1">
      <circle cx="6" cy="6" r="6" fill="currentColor" />
    </svg>
    <span class="text-foreground">{{ active ? activeLabel : inactiveLabel }}</span>
  </Badge>
</template>
