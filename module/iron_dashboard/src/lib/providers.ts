import type { ProviderType } from '../composables/useApi'

const PROVIDER_LABELS: Record<ProviderType, string> = {
  openai: 'OpenAI',
  anthropic: 'Anthropic',
}

const PROVIDER_BADGE_CLASSES: Record<ProviderType, string> = {
  openai: 'bg-success/80',
  anthropic: 'bg-accent/80',
}

export function getProviderLabel(provider: string): string {
  return PROVIDER_LABELS[provider as ProviderType] ?? provider
}

export function getProviderBadgeClass(provider: string): string {
  const bg = PROVIDER_BADGE_CLASSES[provider as ProviderType] ?? 'bg-muted/80'
  return `${bg} text-white text-xs font-medium px-2 py-0.5 rounded-full`
}
