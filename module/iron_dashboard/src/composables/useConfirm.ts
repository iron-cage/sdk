import { ref } from 'vue'

export function useConfirm() {
  const showConfirmModal = ref(false)
  const confirmTitle = ref('')
  const confirmDescription = ref('')
  const confirmLabel = ref('Confirm')
  const confirmVariant = ref<'default' | 'destructive'>('default')
  const confirmCallback = ref<(() => void | Promise<void>) | null>(null)

  function openConfirm(
    title: string,
    description: string,
    label: string,
    action: () => void | Promise<void>,
    variant: 'default' | 'destructive' = 'default'
  ) {
    confirmTitle.value = title
    confirmDescription.value = description
    confirmLabel.value = label
    confirmVariant.value = variant
    confirmCallback.value = async () => {
      confirmCallback.value = null
      try {
        await action()
      } catch (err) {
        console.error('Confirm action failed:', err)
      }
    }
    showConfirmModal.value = true
  }

  return {
    showConfirmModal,
    confirmTitle,
    confirmDescription,
    confirmLabel,
    confirmVariant,
    confirmCallback,
    openConfirm,
  }
}
