<script setup lang="ts">
defineProps<{
  changePercent: number | null | undefined
}>()
</script>

<template>
  <span
    v-if="changePercent != null && Number.isFinite(changePercent) && changePercent !== 0"
    class="inline-flex items-center gap-0.5 text-xs font-medium"
    :class="changePercent > 0 ? 'text-success' : 'text-destructive'"
  >
    <svg
      v-if="changePercent > 0"
      class="h-3 w-3"
      viewBox="0 0 12 12"
      fill="currentColor"
    >
      <path d="M6 2l4 5H2l4-5z" />
    </svg>
    <svg
      v-else
      class="h-3 w-3"
      viewBox="0 0 12 12"
      fill="currentColor"
    >
      <path d="M6 10L2 5h8l-4 5z" />
    </svg>
    {{ changePercent > 0 ? '+' : '' }}{{ changePercent.toFixed(1) }}% vs prev
  </span>
</template>
