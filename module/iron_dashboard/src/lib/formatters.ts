export function formatDate(ms: number): string {
  return new Date(ms).toLocaleString()
}

export function formatTimestamp(ts?: number | null): string {
  if (!ts) return '-'
  const millis = ts > 1_000_000_000_000 ? ts : ts * 1000
  return new Date(millis).toLocaleString()
}

export function formatMicrodollars(micros: number): string {
  return `$${(micros / 1_000_000).toFixed(4)}`
}

export function formatCostUsd(usd: number, decimals = 2): string {
  return `$${usd.toFixed(decimals)}`
}

export function formatCostFromCents(cents: number): string {
  return `$${(cents / 100).toFixed(2)}`
}

export function formatNumber(n: number): string {
  return n.toLocaleString()
}
