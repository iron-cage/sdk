<script setup lang="ts">
import { ref, watch } from 'vue'
import { Sparkles } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Alert, AlertDescription } from '@/components/ui/alert'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

const props = withDefaults(
  defineProps<{
    open: boolean
    title?: string
    description?: string
    placeholder?: string
    initialText?: string
    parseLabel?: string
    cancelLabel?: string
    parsing?: boolean
    error?: string | null
  }>(),
  {
    title: 'FreeForm Mode',
    description: 'Paste anything - a sentence, a list, an email, or just describe yourself.',
    placeholder: 'Paste anything here...',
    initialText: '',
    parseLabel: 'Parse',
    cancelLabel: 'Cancel',
    parsing: false,
    error: null,
  }
)

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void
  (e: 'parse', text: string): void
  (e: 'cancel'): void
}>()

const text = ref(props.initialText)

watch(
  () => props.open,
  (isOpen: boolean) => {
    if (isOpen) {
      text.value = props.initialText
    }
  }
)

watch(
  () => props.initialText,
  (value: string) => {
    if (props.open) {
      text.value = value
    }
  }
)

function onOpenUpdate(value: boolean) {
  emit('update:open', value)
  if (!value) {
    emit('cancel')
  }
}

function onCancel() {
  emit('update:open', false)
  emit('cancel')
}

function onParse() {
  const trimmed = text.value.trim()
  if (!trimmed || props.parsing) return
  emit('parse', trimmed)
}
</script>

<template>
  <Dialog
    :open="open"
    @update:open="onOpenUpdate"
  >
    <DialogContent class="sm:max-w-xl">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <Sparkles class="size-4 text-primary" />
          {{ title }}
        </DialogTitle>
        <DialogDescription>{{ description }}</DialogDescription>
      </DialogHeader>

      <Textarea
        v-model="text"
        :placeholder="placeholder"
        class="min-h-[180px] font-mono text-sm"
        :disabled="parsing"
        autofocus
      />

      <Alert
        v-if="error"
        variant="destructive"
      >
        <AlertDescription>{{ error }}</AlertDescription>
      </Alert>

      <DialogFooter>
        <Button
          type="button"
          variant="ghost"
          :disabled="parsing"
          @click="onCancel"
        >
          {{ cancelLabel }}
        </Button>
        <Button
          type="button"
          :disabled="parsing || !text.trim()"
          @click="onParse"
        >
          {{ parsing ? 'Parsing...' : parseLabel }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
