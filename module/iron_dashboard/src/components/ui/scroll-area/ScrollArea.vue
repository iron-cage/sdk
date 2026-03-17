<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import {
  ScrollAreaCorner,
  ScrollAreaRoot,
  ScrollAreaScrollbar,
  ScrollAreaThumb,
  ScrollAreaViewport,
} from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: HTMLAttributes['class']
  // Pass :horizontal="false" to suppress the horizontal scrollbar. Defaults to true.
  horizontal?: boolean
}>()
</script>

<template>
  <ScrollAreaRoot :class="cn('relative overflow-hidden', props.class)">
    <ScrollAreaViewport class="h-full w-full rounded-[inherit]">
      <slot />
    </ScrollAreaViewport>
    <ScrollAreaScrollbar
      orientation="vertical"
      class="flex touch-none select-none transition-colors w-2 border-l border-l-transparent p-px"
    >
      <ScrollAreaThumb class="relative flex-1 rounded-full bg-border" />
    </ScrollAreaScrollbar>
    <ScrollAreaScrollbar
      v-if="props.horizontal !== false"
      orientation="horizontal"
      class="flex touch-none select-none transition-colors h-2 border-t border-t-transparent p-px"
    >
      <ScrollAreaThumb class="relative rounded-full bg-border" />
    </ScrollAreaScrollbar>
    <ScrollAreaCorner />
  </ScrollAreaRoot>
</template>
