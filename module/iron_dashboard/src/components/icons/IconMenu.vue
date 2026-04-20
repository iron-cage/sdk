<script setup lang="ts">
defineOptions({ inheritAttrs: false })
</script>

<template>
  <svg
    aria-hidden="true"
    v-bind="$attrs"
    class="h-5 w-5"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M4 6h16M4 12h16M4 18h16"
    />
  </svg>
</template>
