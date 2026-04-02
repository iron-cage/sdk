export function formatTimestamp(ts?: number | null): string {
  if (ts == null) return '-'
  const millis = ts > 1_000_000_000_000 ? ts : ts * 1000
  return new Date(millis).toLocaleString()
}

export function formatMicrodollars(micros: number): string {
  return `$${(micros / 1_000_000).toFixed(4)}`
}

export function formatCostUsd(usd: number | null | undefined, decimals = 2): string {
  if (usd == null || isNaN(usd)) return '-'
  return `$${usd.toFixed(decimals)}`
}

export function formatNumber(n: number): string {
  return n.toLocaleString()
}
