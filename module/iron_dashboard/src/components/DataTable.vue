<script setup lang="ts">
import { ref, computed, watch, onUnmounted, useId } from 'vue'
import { Button } from '@/components/ui/button'

defineProps<{
  columns: { key?: string; label: string; align?: 'left' | 'right' }[]
  isLoading?: boolean
  error?: Error | null
  isEmpty?: boolean
  loadingText?: string
  onRetry?: () => void
}>()

const wrapperRef = ref<HTMLElement | null>(null)
const scrollRatio = ref(1)
const thumbPos = ref(0)

// Unique ID for the scroll wrapper so the ARIA scrollbar can reference it
const wrapperId = useId()

// Clamp scrollRatio to [0,1] to prevent thumb geometry breaking at certain
// zoom levels or before layout settles (clientWidth can briefly exceed scrollWidth)
const clampedRatio = computed(() => Math.min(1, Math.max(0, scrollRatio.value)))
const showScrollbar = computed(() => clampedRatio.value < 0.9999)
const thumbWidthPct = computed(() => clampedRatio.value * 100)
const thumbLeftPct  = computed(() => thumbPos.value * (100 - thumbWidthPct.value))

function updateScroll() {
  const el = wrapperRef.value
  if (!el) return
  scrollRatio.value = el.clientWidth / el.scrollWidth
  const maxScroll = el.scrollWidth - el.clientWidth
  thumbPos.value = maxScroll > 0 ? el.scrollLeft / maxScroll : 0
}

// Drag — listeners attached to document so fast pointer movement on Firefox/Safari
// does not outrun the element-bound handler before pointer capture takes effect
let isDragging = false
let dragStartX = 0
let dragStartThumbPos = 0

function onThumbPointerDown(e: PointerEvent) {
  e.preventDefault()
  isDragging = true
  dragStartX = e.clientX
  dragStartThumbPos = thumbPos.value
  ;(e.target as HTMLElement).setPointerCapture(e.pointerId)
  document.addEventListener('pointermove', onThumbPointerMove)
  document.addEventListener('pointerup', onThumbPointerUp)
}

function onThumbPointerMove(e: PointerEvent) {
  if (!isDragging || !wrapperRef.value) return
  const el = wrapperRef.value
  const availableTrack = el.clientWidth * (1 - clampedRatio.value)
  if (availableTrack === 0) return
  const newPos = Math.max(0, Math.min(1, dragStartThumbPos + (e.clientX - dragStartX) / availableTrack))
  el.scrollLeft = newPos * (el.scrollWidth - el.clientWidth)
}

function onThumbPointerUp() {
  isDragging = false
  document.removeEventListener('pointermove', onThumbPointerMove)
  document.removeEventListener('pointerup', onThumbPointerUp)
}

function onTrackClick(e: MouseEvent) {
  const thumb = (e.currentTarget as HTMLElement).querySelector('[data-scrollbar-thumb]')
  if (thumb?.contains(e.target as Node)) return
  if (!wrapperRef.value) return
  const el = wrapperRef.value
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  const thumbW = rect.width * clampedRatio.value
  const available = rect.width - thumbW
  if (available < 1) return
  el.scrollLeft = Math.max(0, Math.min(1, (e.clientX - rect.left - thumbW / 2) / available)) * (el.scrollWidth - el.clientWidth)
}

function scrollByKey(dir: -1 | 1) {
  const el = wrapperRef.value
  if (!el) return
  el.scrollLeft += dir * 40
}

function scrollToEnd(pos: 0 | 1) {
  const el = wrapperRef.value
  if (!el) return
  el.scrollLeft = pos === 0 ? 0 : el.scrollWidth - el.clientWidth
}

function scrollByPage(dir: -1 | 1) {
  const el = wrapperRef.value
  if (!el) return
  el.scrollLeft += dir * el.clientWidth * 0.8
}

let ro: ResizeObserver | null = null

watch(wrapperRef, (el, oldEl) => {
  if (oldEl) {
    oldEl.removeEventListener('scroll', updateScroll)
  }
  ro?.disconnect()

  if (!el) return
  el.addEventListener('scroll', updateScroll, { passive: true })
  updateScroll()
  ro = new ResizeObserver(updateScroll)
  ro.observe(el)
  const tableEl = el.querySelector('table')
  if (tableEl) ro.observe(tableEl)
}, { flush: 'post' })

onUnmounted(() => {
  wrapperRef.value?.removeEventListener('scroll', updateScroll)
  document.removeEventListener('pointermove', onThumbPointerMove)
  document.removeEventListener('pointerup', onThumbPointerUp)
  ro?.disconnect()
})
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
      <Button v-if="onRetry" variant="outline" class="mt-4" @click="onRetry">
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
      <div class="relative" :class="{ 'pb-6': showScrollbar }">
        <!-- Scroll wrapper: overflow-x-auto with native scrollbar hidden -->
        <div
          ref="wrapperRef"
          :id="wrapperId"
          class="overflow-x-auto touch-pan-x [&::-webkit-scrollbar]:hidden [scrollbar-width:none]"
        >
          <table class="min-w-[700px] w-full divide-y divide-border">
            <thead>
              <tr class="text-foreground/70">
                <th
                  v-for="(col, index) in columns"
                  :key="col.key ?? `${col.label}-${index}`"
                  :class="[
                    'px-3 sm:px-6 py-3 text-xs font-medium uppercase tracking-wider text-nowrap',
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

        <!-- Custom scrollbar — h-6 invisible hit zone, h-1.5 visible track -->
        <div
          v-if="showScrollbar"
          role="scrollbar"
          aria-label="Scroll table horizontally"
          aria-orientation="horizontal"
          aria-valuemin="0"
          aria-valuemax="100"
          :aria-valuenow="Math.round(thumbPos * 100)"
          :aria-controls="wrapperId"
          tabindex="0"
          class="absolute bottom-0 left-0 right-0 h-6 flex items-end cursor-pointer focus:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          @click="onTrackClick"
          @keydown.left.prevent="scrollByKey(-1)"
          @keydown.right.prevent="scrollByKey(1)"
          @keydown.home.prevent="scrollToEnd(0)"
          @keydown.end.prevent="scrollToEnd(1)"
          @keydown.page-up.prevent="scrollByPage(-1)"
          @keydown.page-down.prevent="scrollByPage(1)"
        >
          <div class="relative w-full h-1.5">
            <div
              data-scrollbar-thumb
              class="absolute h-full rounded-full bg-border hover:bg-foreground/30 cursor-grab active:cursor-grabbing transition-colors"
              :style="{ width: thumbWidthPct + '%', left: thumbLeftPct + '%' }"
              @pointerdown="onThumbPointerDown"
              @pointercancel="onThumbPointerUp"
            />
          </div>
        </div>
      </div>
      <slot name="footer" />
    </template>
  </div>
</template>
