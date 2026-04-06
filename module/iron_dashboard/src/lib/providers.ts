import { cn } from '@/lib/utils'

const PROVIDER_LABELS: Record<string, string> = {
  openai: 'OpenAI',
  anthropic: 'Anthropic',
  gemini: 'Gemini',
  xai: 'xAI',
}

const PROVIDER_BADGE_COLORS: Record<string, string> = {
  openai: 'bg-success/80',
  anthropic: 'bg-accent/80',
  gemini: 'bg-chart-1/80',
  xai: 'bg-chart-4/80',
}

const PROVIDER_KEY_PLACEHOLDERS: Record<string, string> = {
  openai: 'sk-proj-...',
  anthropic: 'sk-ant-...',
  gemini: 'AIzaSy...',
  xai: 'xai-...',
}

export function getProviderLabel(provider: string): string {
  return PROVIDER_LABELS[provider] ?? provider
}

export function getProviderBadgeClass(provider: string): string {
  return cn(
    'text-xs font-medium px-2 py-0.5 rounded-full',
    PROVIDER_BADGE_COLORS[provider] ?? 'bg-muted/80',
    PROVIDER_BADGE_COLORS[provider] ? 'text-primary-foreground' : 'text-foreground',
  )
}

export function getProviderKeyPlaceholder(provider: string): string {
  return PROVIDER_KEY_PLACEHOLDERS[provider] ?? 'API key...'
}

export type ProviderType = 'openai' | 'anthropic' | 'gemini' | 'xai'

export function detectProviderFromKey(key: string): ProviderType | null {
  const trimmed = key.trim()
  if (trimmed.startsWith('sk-ant-')) return 'anthropic'
  if (trimmed.startsWith('xai-')) return 'xai'
  if (trimmed.startsWith('AIzaSy')) return 'gemini'
  if (trimmed.startsWith('sk-')) return 'openai'
  return null
}

export function generateProviderAlias(
  provider: ProviderType,
  existingKeys: Array<{ provider: string; alias?: string }>,
): string {
  const label = getProviderLabel(provider)
  const escaped = label.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const re = new RegExp(`^${escaped} (\\d+)$`)
  let max = 0
  for (const k of existingKeys) {
    if (k.provider !== provider) continue
    const m = k.alias?.match(re)
    if (m) max = Math.max(max, parseInt(m[1]!, 10))
  }
  return `${label} ${max + 1}`
}
