<script setup lang="ts">
import { PopoverContent, PopoverPortal } from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: string
  side?: 'top' | 'right' | 'bottom' | 'left'
  align?: 'start' | 'center' | 'end'
  sideOffset?: number
}>()
</script>

<template>
  <PopoverPortal>
    <PopoverContent
      :side="props.side ?? 'bottom'"
      :align="props.align ?? 'center'"
      :side-offset="props.sideOffset ?? 4"
      :class="cn(
        'z-50 rounded-md border border-border bg-popover p-3 text-popover-foreground shadow-md outline-none',
        'data-[state=open]:animate-in data-[state=closed]:animate-out',
        'data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0',
        'data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95',
        'data-[side=bottom]:slide-in-from-top-2 data-[side=top]:slide-in-from-bottom-2',
        props.class,
      )"
    >
      <slot />
    </PopoverContent>
  </PopoverPortal>
</template>
