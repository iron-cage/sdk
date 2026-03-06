<script setup lang="ts">
import { useSlots } from 'vue'
import { ScrollArea } from '@/components/ui/scroll-area'

defineProps<{
  title: string
}>()

const slots = useSlots()
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Title bar -->
    <div class="flex items-center px-4 h-10 shrink-0">
      <h1 class="text-sm font-semibold text-foreground">{{ title }}</h1>
    </div>

    <div class="border-t border-border shrink-0" />

    <!-- Actions bar -->
    <template v-if="slots.actions">
      <div class="flex items-center justify-end gap-2 px-4 py-2 shrink-0 min-h-[40px]">
        <slot name="actions" />
      </div>
      <div class="border-t border-border shrink-0" />
    </template>

    <!-- Scrollable content -->
    <ScrollArea class="flex-1">
      <div class="p-4">
        <slot />
      </div>
    </ScrollArea>
  </div>
</template>
