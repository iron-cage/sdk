<script setup lang="ts">
import { Sparkles } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'

defineProps<{
  active?: boolean
  title?: string
}>()

defineEmits<{
  (e: 'click'): void
}>()
</script>

<template>
  <Button
    type="button"
    variant="ghost"
    size="icon-sm"
    :class="[
      'rounded-full',
      active
        ? 'bg-primary/10 text-primary hover:bg-primary/15'
        : 'text-muted-foreground hover:text-foreground',
    ]"
    :title="title ?? 'Switch to FreeForm Mode'"
    :aria-pressed="active ? 'true' : 'false'"
    aria-label="Toggle FreeForm Mode"
    @click="$emit('click')"
  >
    <Sparkles />
  </Button>
</template>
