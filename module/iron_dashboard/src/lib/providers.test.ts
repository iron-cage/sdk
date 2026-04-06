import { describe, it, expect } from 'vitest'
import { detectProviderFromKey, generateProviderAlias, getProviderKeyPlaceholder } from './providers'

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

  it('returns null for AIza prefix that is not AIzaSy', () => {
    expect(detectProviderFromKey('AIzaXXXXXX')).toBeNull()
  })

  it('returns null for an unrecognized prefix', () => {
    expect(detectProviderFromKey('gsk_abc123')).toBeNull()
  })

  it('trims surrounding whitespace before matching', () => {
    expect(detectProviderFromKey('  sk-ant-abc  ')).toBe('anthropic')
  })

  // Case-sensitivity is intentional — prefixes are defined by the providers
  // and real keys never vary in casing.
  it('returns null for upper-case SK-ANT- (case-sensitive match)', () => {
    expect(detectProviderFromKey('SK-ANT-abc123')).toBeNull()
  })

  it('returns null for lower-case aizasy (case-sensitive match)', () => {
    expect(detectProviderFromKey('aizasyABC123')).toBeNull()
  })

  it('returns null for upper-case XAI- (case-sensitive match)', () => {
    expect(detectProviderFromKey('XAI-abc123')).toBeNull()
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

describe('getProviderKeyPlaceholder', () => {
  it('returns sk-proj-... for openai', () => {
    expect(getProviderKeyPlaceholder('openai')).toBe('sk-proj-...')
  })
  it('returns sk-ant-... for anthropic', () => {
    expect(getProviderKeyPlaceholder('anthropic')).toBe('sk-ant-...')
  })
  it('returns AIzaSy... for gemini', () => {
    expect(getProviderKeyPlaceholder('gemini')).toBe('AIzaSy...')
  })
  it('returns xai-... for xai', () => {
    expect(getProviderKeyPlaceholder('xai')).toBe('xai-...')
  })
  it('returns generic fallback for unknown provider', () => {
    expect(getProviderKeyPlaceholder('unknown')).toBe('API key...')
  })
})
