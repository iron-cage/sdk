<script setup lang="ts">
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

withDefaults(
  defineProps<{
    open: boolean
    title: string
    description: string
    confirmLabel?: string
    variant?: 'default' | 'destructive'
  }>(),
  {
    confirmLabel: 'Confirm',
    variant: 'default',
  }
)

const emit = defineEmits<{
  'update:open': [value: boolean]
  confirm: []
}>()

function handleCancel() {
  emit('update:open', false)
}

function handleConfirm() {
  emit('confirm')
  emit('update:open', false)
}
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>{{ title }}</DialogTitle>
        <DialogDescription>{{ description }}</DialogDescription>
      </DialogHeader>
      <DialogFooter>
        <Button variant="outline" @click="handleCancel">Cancel</Button>
        <Button :variant="variant" @click="handleConfirm">
          {{ confirmLabel }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
