import { toast } from 'vue-sonner'

export function useClipboard() {
  async function copyText(text: string, successLabel = 'Copied') {
    if (!navigator.clipboard) {
      toast.error('Copy not supported')
      return
    }
    try {
      await navigator.clipboard.writeText(text)
      toast.success(successLabel)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Copy failed')
    }
  }

  return { copyText }
}
