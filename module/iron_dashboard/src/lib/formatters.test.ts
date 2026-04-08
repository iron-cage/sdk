import { describe, it, expect } from 'vitest'
import { formatTimestamp, formatMicrodollars, formatCostUsd, formatNumber } from './formatters'

describe('formatTimestamp', () => {
  it('returns "-" for null', () => {
    expect(formatTimestamp(null)).toBe('-')
  })

  it('returns "-" for undefined', () => {
    expect(formatTimestamp(undefined)).toBe('-')
  })

  it('treats values > 1_000_000_000_000 as milliseconds', () => {
    const ms = 1_710_000_000_000
    expect(formatTimestamp(ms)).toBe(new Date(ms).toLocaleString())
  })

  it('treats values <= 1_000_000_000_000 as seconds (multiplies by 1000)', () => {
    const seconds = 1_710_000_000
    expect(formatTimestamp(seconds)).toBe(new Date(seconds * 1000).toLocaleString())
  })
})

describe('formatMicrodollars', () => {
  it('formats zero correctly', () => {
    expect(formatMicrodollars(0)).toBe('$0.0000')
  })

  it('converts microdollars to dollars with 4 decimal places', () => {
    expect(formatMicrodollars(1_000_000)).toBe('$1.0000')
    expect(formatMicrodollars(500_000)).toBe('$0.5000')
  })
})

describe('formatCostUsd', () => {
  it('returns "-" for null', () => {
    expect(formatCostUsd(null)).toBe('-')
  })

  it('returns "-" for undefined', () => {
    expect(formatCostUsd(undefined)).toBe('-')
  })

  it('returns "-" for NaN', () => {
    expect(formatCostUsd(NaN)).toBe('-')
  })

  it('formats with default 2 decimal places', () => {
    expect(formatCostUsd(1.5)).toBe('$1.50')
  })

  it('formats with specified decimal places', () => {
    expect(formatCostUsd(1.23456, 4)).toBe('$1.2346')
  })
})

describe('formatNumber', () => {
  it('formats a number with locale separators', () => {
    expect(formatNumber(1000)).toBe((1000).toLocaleString())
  })
})
