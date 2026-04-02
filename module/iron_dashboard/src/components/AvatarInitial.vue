<script setup lang="ts">
const props = defineProps<{ name: string }>()

const AVATAR_COLORS = ['#5E6AD2', '#26B5CE', '#4CB782', '#F2994A']

function avatarColor(name: string): string {
  let hash = 0
  for (let i = 0; i < name.length; i++) hash = (hash * 31 + name.charCodeAt(i)) >>> 0
  return AVATAR_COLORS[hash % AVATAR_COLORS.length]!
}
</script>

<template>
  <span
    class="w-6 h-6 rounded-[6px] flex items-center justify-center text-xs font-semibold text-white flex-shrink-0"
    :style="{ backgroundColor: avatarColor(props.name) }"
  >
    {{ props.name[0]?.toUpperCase() ?? '?' }}
  </span>
</template>
