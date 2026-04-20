import { describe, it, expect, vi } from 'vitest'
import { useConfirm } from './useConfirm'

describe('useConfirm', () => {
  it('sets modal state when openConfirm is called', () => {
    const {
      showConfirmModal,
      confirmTitle,
      confirmDescription,
      confirmLabel,
      confirmVariant,
      openConfirm,
    } = useConfirm()

    openConfirm('Delete', 'Are you sure?', 'Yes', () => {}, 'destructive')

    expect(showConfirmModal.value).toBe(true)
    expect(confirmTitle.value).toBe('Delete')
    expect(confirmDescription.value).toBe('Are you sure?')
    expect(confirmLabel.value).toBe('Yes')
    expect(confirmVariant.value).toBe('destructive')
  })

  it('defaults variant to "default" when not specified', () => {
    const { confirmVariant, openConfirm } = useConfirm()

    openConfirm('Test', 'Test', 'OK', () => {})

    expect(confirmVariant.value).toBe('default')
  })

  it('sets callback to null before invoking action (prevents re-fire via reactive check)', async () => {
    const { confirmCallback, openConfirm } = useConfirm()

    const action = vi.fn()
    openConfirm('Test', 'Test', 'OK', action)

    expect(confirmCallback.value).not.toBeNull()
    await confirmCallback.value?.()
    expect(confirmCallback.value).toBeNull()
    expect(action).toHaveBeenCalledTimes(1)
  })

  it('awaits async actions', async () => {
    const { confirmCallback, openConfirm } = useConfirm()

    let resolved = false
    openConfirm('Test', 'Test', 'OK', async () => {
      await new Promise((r) => setTimeout(r, 1))
      resolved = true
    })

    await confirmCallback.value?.()
    expect(resolved).toBe(true)
  })

  it('propagates errors thrown by action so callers can handle them', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const { confirmCallback, openConfirm } = useConfirm()

    openConfirm('Test', 'Test', 'OK', async () => {
      throw new Error('Action failed')
    })

    await expect(confirmCallback.value?.()).rejects.toThrow('Action failed')
    consoleSpy.mockRestore()
  })

  it('logs errors thrown by action', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const { confirmCallback, openConfirm } = useConfirm()

    openConfirm('Test', 'Test', 'OK', async () => {
      throw new Error('oops')
    })

    try {
      await confirmCallback.value?.()
    } catch {
      // swallow — this test verifies the console.error side effect, not the re-throw
    }
    expect(consoleSpy).toHaveBeenCalledWith('Confirm action failed:', expect.any(Error))
    consoleSpy.mockRestore()
  })
})
