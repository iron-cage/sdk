<script setup lang="ts">
import { Button } from '@/components/ui/button'

defineProps<{
  columns: { label: string; align?: 'left' | 'right' }[]
  isLoading?: boolean
  error?: Error | null
  isEmpty?: boolean
  loadingText?: string
  onRetry?: () => void
}>()
</script>

<template>
  <div class="overflow-hidden">
    <!-- Loading -->
    <div v-if="isLoading" class="p-4">
      <p class="text-muted-foreground">{{ loadingText ?? 'Loading...' }}</p>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="p-4">
      <p class="text-destructive">{{ error.message }}</p>
      <Button v-if="onRetry" @click="onRetry" variant="outline" class="mt-4">
        Retry
      </Button>
    </div>

    <!-- Empty -->
    <div v-else-if="isEmpty" class="p-4 text-center">
      <slot name="empty">
        <p class="text-muted-foreground">No data</p>
      </slot>
    </div>

    <!-- Table -->
    <template v-else>
      <div class="overflow-x-auto touch-pan-x">
        <table class="min-w-[700px] w-full divide-y divide-border">
          <thead>
            <tr class="text-foreground/70">
              <th
                v-for="col in columns"
                :key="col.label"
                :class="[
                  'px-3 sm:px-6 py-3 text-xs font-medium uppercase tracking-wider',
                  col.align === 'right' ? 'text-right' : 'text-left',
                ]"
              >
                {{ col.label }}
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-border">
            <slot />
          </tbody>
        </table>
      </div>
      <slot name="footer" />
    </template>
  </div>
</template>
