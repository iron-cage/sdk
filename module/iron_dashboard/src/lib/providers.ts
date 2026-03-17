const PROVIDER_LABELS: Record<string, string> = {
  openai: 'OpenAI',
  anthropic: 'Anthropic',
  gemini: 'Gemini',
  xai: 'xAI',
}

const PROVIDER_BADGE_COLORS: Record<string, string> = {
  openai: 'bg-success/80',
  anthropic: 'bg-accent/80',
  gemini: 'bg-blue-500/80',
  xai: 'bg-purple-500/80',
}

const PROVIDER_KEY_PLACEHOLDERS: Record<string, string> = {
  openai: 'sk-proj-...',
  anthropic: 'sk-ant-...',
  gemini: 'AIza...',
  xai: 'xai-...',
}

export function getProviderLabel(provider: string): string {
  return PROVIDER_LABELS[provider] ?? provider
}

export function getProviderBadgeClass(provider: string): string {
  const bg = PROVIDER_BADGE_COLORS[provider] ?? 'bg-muted/80'
  const text = PROVIDER_BADGE_COLORS[provider] ? 'text-white' : 'text-foreground'
  return `${bg} ${text} text-xs font-medium px-2 py-0.5 rounded-full`
}

export function getProviderKeyPlaceholder(provider: string): string {
  return PROVIDER_KEY_PLACEHOLDERS[provider] ?? 'API key...'
}
