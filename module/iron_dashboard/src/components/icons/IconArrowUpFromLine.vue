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
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
    />
  </svg>
</template>
