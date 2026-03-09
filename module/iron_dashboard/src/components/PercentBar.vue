<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  percentage: number
  class?: string
}>()

const colorClass = computed(() => {
  if (props.percentage >= 90) return 'bg-destructive'
  if (props.percentage >= 75) return 'bg-warning'
  return 'bg-success'
})
</script>

<template>
  <div :class="cn('w-full bg-muted rounded-full h-2', props.class)">
    <div
      class="h-1.5 rounded-full transition-all"
      :class="colorClass"
      :style="{ width: `${percentage}%` }"
    />
  </div>
</template>
