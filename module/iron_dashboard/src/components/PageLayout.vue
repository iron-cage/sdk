<script setup lang="ts">
import { useSlots } from 'vue'
import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'
const props = defineProps<{
  title: string
  class?: string
  contentClass?: string
}>()

const slots = useSlots()
</script>

<template>
  <div :class="cn('flex flex-col h-full', props.class)">
    <!-- Title bar -->
    <div class="flex items-center px-4 lg:px-6 h-10 shrink-0">
      <h1 class="text-base text-foreground">{{ title }}</h1>
    </div>

    <div class="border-t border-border shrink-0" />

    <!-- Actions bar -->
    <template v-if="slots.actions">
      <div class="flex flex-wrap items-center justify-end gap-2 px-4 lg:px-6 py-2 shrink-0 min-h-[40px]">
        <slot name="actions" />
      </div>
      <div class="border-t border-border shrink-0" />
    </template>

    <!-- Scrollable content -->
    <ScrollArea class="flex-1">
      <div class="" :class="props.contentClass">
        <slot />
      </div>
    </ScrollArea>
  </div>
</template>
