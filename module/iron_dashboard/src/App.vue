<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import MainLayout from './components/MainLayout.vue'

const route = useRoute()
const requiresAuth = computed(() => route.meta.requiresAuth !== false)
</script>

<template>
  <MainLayout v-if="requiresAuth">
    <router-view />
  </MainLayout>
  <router-view v-else />
</template>
