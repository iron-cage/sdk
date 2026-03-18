<script setup lang="ts">
defineOptions({ inheritAttrs: false })
</script>

<template>
  <svg
    aria-hidden="true"
    v-bind="$attrs"
    class="h-4 w-4"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
  </svg>
</template>
