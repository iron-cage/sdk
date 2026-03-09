<script setup lang="ts">
defineOptions({ inheritAttrs: false })
</script>

<template>
  <svg
    v-bind="$attrs"
    class="h-4 w-4"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
    aria-hidden="true"
  >
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
  </svg>
</template>
