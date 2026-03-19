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
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
  </svg>
</template>
