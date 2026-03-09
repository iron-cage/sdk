import type { ProviderType } from '../composables/useApi'

export function getProviderLabel(provider: ProviderType): string {
  return provider === 'openai' ? 'OpenAI' : 'Anthropic'
}

export function getProviderBadgeClass(provider: ProviderType): string {
  const bg = provider === 'openai' ? 'bg-success/80' : 'bg-accent/80'
  return `${bg} text-white text-xs font-medium px-2 py-0.5 rounded-full`
}
