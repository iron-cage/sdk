import { ref } from 'vue'

export function useConfirm() {
  const showConfirmModal = ref(false)
  const confirmTitle = ref('')
  const confirmDescription = ref('')
  const confirmLabel = ref('Confirm')
  const confirmVariant = ref<'default' | 'destructive'>('destructive')
  const confirmCallback = ref<(() => void) | null>(null)

  function openConfirm(
    title: string,
    description: string,
    label: string,
    action: () => void | Promise<void>,
    variant: 'default' | 'destructive' = 'destructive',
  ) {
    confirmTitle.value = title
    confirmDescription.value = description
    confirmLabel.value = label
    confirmVariant.value = variant
    confirmCallback.value = async () => {
      confirmCallback.value = null
      await action()
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
