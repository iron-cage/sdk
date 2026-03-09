<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import MainLayout from './components/MainLayout.vue'
import { Toaster } from '@/components/ui/sonner'

const route = useRoute()
const requiresAuth = computed(() => route.meta.requiresAuth !== false)
</script>

<template>
  <MainLayout v-if="requiresAuth">
    <router-view />
  </MainLayout>
  <router-view v-else />
  <Toaster rich-colors position="bottom-right" />
</template>
