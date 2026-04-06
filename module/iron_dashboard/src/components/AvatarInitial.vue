<script setup lang="ts">
import { cn } from '@/lib/utils'

const props = defineProps<{ name: string }>()

const AVATAR_VARIANTS = [
  'bg-accent text-accent-foreground',
  'bg-chart-2 text-primary-foreground',
  'bg-success text-primary-foreground',
  'bg-chart-5 text-primary-foreground',
]

function avatarVariant(name: string): string {
  let hash = 0
  for (let i = 0; i < name.length; i++) hash = (hash * 31 + name.charCodeAt(i)) >>> 0
  return AVATAR_VARIANTS[hash % AVATAR_VARIANTS.length]!
}
</script>

<template>
  <span
    :class="cn('w-6 h-6 rounded-[6px] flex items-center justify-center text-xs font-semibold flex-shrink-0', avatarVariant(props.name))"
  >
    {{ props.name[0]?.toUpperCase() ?? '?' }}
  </span>
</template>
