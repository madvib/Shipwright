import { describe, it, expect } from 'vitest'
import { parseSemver, compareSemver, isNewerVersion } from './semver'

describe('parseSemver', () => {
  it('parses standard semver', () => {
    expect(parseSemver('1.2.3')).toEqual({
      major: 1, minor: 2, patch: 3, prerelease: null,
    })
  })

  it('parses semver with v prefix', () => {
    expect(parseSemver('v2.0.1')).toEqual({
      major: 2, minor: 0, patch: 1, prerelease: null,
    })
  })

  it('parses semver with pre-release', () => {
    expect(parseSemver('1.0.0-alpha.1')).toEqual({
      major: 1, minor: 0, patch: 0, prerelease: 'alpha.1',
    })
  })

  it('returns null for invalid input', () => {
    expect(parseSemver('')).toBeNull()
    expect(parseSemver('not-a-version')).toBeNull()
    expect(parseSemver('1.2')).toBeNull()
    expect(parseSemver('1')).toBeNull()
  })
})

describe('compareSemver', () => {
  it('returns 1 when a > b (major)', () => {
    expect(compareSemver('2.0.0', '1.0.0')).toBe(1)
  })

  it('returns -1 when a < b (minor)', () => {
    expect(compareSemver('1.0.0', '1.1.0')).toBe(-1)
  })

  it('returns 0 for equal versions', () => {
    expect(compareSemver('1.2.3', '1.2.3')).toBe(0)
  })

  it('returns 0 for invalid semver strings', () => {
    expect(compareSemver('bad', '1.0.0')).toBe(0)
    expect(compareSemver('1.0.0', 'bad')).toBe(0)
  })

  it('handles patch comparison', () => {
    expect(compareSemver('1.0.2', '1.0.1')).toBe(1)
  })

  it('pre-release is lower than release', () => {
    expect(compareSemver('1.0.0-alpha', '1.0.0')).toBe(-1)
    expect(compareSemver('1.0.0', '1.0.0-beta')).toBe(1)
  })

  it('compares pre-release strings lexically', () => {
    expect(compareSemver('1.0.0-alpha', '1.0.0-beta')).toBe(-1)
    expect(compareSemver('1.0.0-beta', '1.0.0-alpha')).toBe(1)
  })

  it('handles v prefix correctly', () => {
    expect(compareSemver('v1.0.0', '1.0.0')).toBe(0)
    expect(compareSemver('v2.0.0', 'v1.0.0')).toBe(1)
  })
})

describe('isNewerVersion', () => {
  it('returns true when candidate is newer', () => {
    expect(isNewerVersion('2.0.0', '1.0.0')).toBe(true)
    expect(isNewerVersion('1.1.0', '1.0.0')).toBe(true)
    expect(isNewerVersion('1.0.1', '1.0.0')).toBe(true)
  })

  it('returns false when candidate is older or equal', () => {
    expect(isNewerVersion('1.0.0', '2.0.0')).toBe(false)
    expect(isNewerVersion('1.0.0', '1.0.0')).toBe(false)
  })

  it('returns false for invalid input', () => {
    expect(isNewerVersion('bad', '1.0.0')).toBe(false)
  })
})
