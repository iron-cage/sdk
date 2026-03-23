import { describe, it, expect } from 'vitest'
import { detectProviderFromKey, generateProviderAlias } from './providers'

describe('detectProviderFromKey', () => {
  it('returns null for an empty string', () => {
    expect(detectProviderFromKey('')).toBeNull()
  })

  it('returns null for whitespace-only input', () => {
    expect(detectProviderFromKey('   ')).toBeNull()
  })

  it('detects Anthropic from sk-ant- prefix', () => {
    expect(detectProviderFromKey('sk-ant-abc123')).toBe('anthropic')
  })

  it('detects OpenAI from sk- prefix', () => {
    expect(detectProviderFromKey('sk-proj-abc123')).toBe('openai')
  })

  it('detects xAI from xai- prefix', () => {
    expect(detectProviderFromKey('xai-abc123')).toBe('xai')
  })

  it('detects Gemini from AIzaSy prefix', () => {
    expect(detectProviderFromKey('AIzaSyABC123')).toBe('gemini')
  })

  it('returns anthropic (not openai) for sk-ant- — prefix ordering guard', () => {
    const result = detectProviderFromKey('sk-ant-xxx')
    expect(result).toBe('anthropic')
    expect(result).not.toBe('openai')
  })

  it('returns null for an unrecognized prefix', () => {
    expect(detectProviderFromKey('gsk_abc123')).toBeNull()
  })

  it('trims surrounding whitespace before matching', () => {
    expect(detectProviderFromKey('  sk-ant-abc  ')).toBe('anthropic')
  })
})

describe('generateProviderAlias', () => {
  it('returns "OpenAI 1" for an empty list', () => {
    expect(generateProviderAlias('openai', [])).toBe('OpenAI 1')
  })

  it('returns "Anthropic 1" when no existing Anthropic keys', () => {
    expect(
      generateProviderAlias('anthropic', [
        { provider: 'openai', alias: 'OpenAI 1' },
      ]),
    ).toBe('Anthropic 1')
  })

  it('increments past the highest existing numbered alias', () => {
    expect(
      generateProviderAlias('openai', [
        { provider: 'openai', alias: 'OpenAI 1' },
        { provider: 'openai', alias: 'OpenAI 3' },
      ]),
    ).toBe('OpenAI 4')
  })

  it('ignores aliases from other providers', () => {
    expect(
      generateProviderAlias('openai', [
        { provider: 'anthropic', alias: 'Anthropic 5' },
      ]),
    ).toBe('OpenAI 1')
  })

  it('ignores keys without an alias', () => {
    expect(
      generateProviderAlias('gemini', [
        { provider: 'gemini' },
        { provider: 'gemini', alias: 'Gemini 2' },
      ]),
    ).toBe('Gemini 3')
  })

  it('ignores non-numbered aliases', () => {
    expect(
      generateProviderAlias('xai', [
        { provider: 'xai', alias: 'My custom key' },
      ]),
    ).toBe('xAI 1')
  })
})
