<script setup lang="ts">
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

defineProps<{
  title: string
  showSeparator?: boolean
  grow?: boolean
}>()
</script>

<template>
  <Card :class="grow ? 'flex flex-col' : ''">
    <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
      <CardTitle class="text-base font-medium text-foreground flex gap-2 items-center">
        <slot name="icon" />
        {{ title }}
      </CardTitle>
      <slot name="action" />
    </CardHeader>
    <div v-if="showSeparator" class="border-t border-border mb-2" />
    <CardContent :class="grow ? 'flex-1 pb-2' : ''">
      <slot />
    </CardContent>
  </Card>
</template>
