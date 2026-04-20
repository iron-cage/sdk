<script setup lang="ts">
import type { PopoverContentEmits, PopoverContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { PopoverContent, PopoverPortal, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/utils'

const props = withDefaults(
  defineProps<PopoverContentProps & { class?: HTMLAttributes['class'] }>(),
  { side: 'bottom', align: 'center', sideOffset: 4, avoidCollisions: true }
)

const emits = defineEmits<PopoverContentEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <PopoverPortal>
    <PopoverContent
      v-bind="forwarded"
      :class="
        cn(
          'z-50 rounded-md border border-border bg-popover p-3 text-popover-foreground shadow-md outline-none',
          'data-[state=open]:animate-in data-[state=closed]:animate-out',
          'data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0',
          'data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95',
          'data-[side=bottom]:slide-in-from-top-2 data-[side=top]:slide-in-from-bottom-2',
          'data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2',
          props.class
        )
      "
    >
      <slot />
    </PopoverContent>
  </PopoverPortal>
</template>
