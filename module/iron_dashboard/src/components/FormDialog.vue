<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import IconX from '@/components/icons/IconX.vue'

const props = defineProps<{
  open: boolean
  title: string
  description?: string
  confirmLabel?: string
  cancelLabel?: string
  confirmDisabled?: boolean
  maxWidth?: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  confirm: []
}>()
</script>

<template>
  <Dialog :open="props.open" @update:open="emit('update:open', $event)">
    <DialogContent :class="props.maxWidth ?? 'sm:max-w-md'">
      <DialogHeader>
        <DialogTitle>{{ props.title }}</DialogTitle>
        <DialogDescription v-if="props.description">{{ props.description }}</DialogDescription>
      </DialogHeader>

      <slot />

      <DialogFooter>
        <Button
          variant="outline"
          @click="emit('update:open', false)"
        >
          <IconX />
          {{ props.cancelLabel ?? 'Cancel' }}
        </Button>
        <Button
          :disabled="props.confirmDisabled"
          @click="emit('confirm')"
        >
          <slot name="confirm-icon" />
          {{ props.confirmLabel ?? 'Confirm' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
