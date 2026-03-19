import { describe, it, expect } from 'vitest'
import seedRepos from '#/lib/seed-repos.json'

const VALID_CATEGORIES = ['ai-tools', 'web-frameworks', 'devops', 'languages', 'editors']

describe('seed-repos.json', () => {
  it('is a non-empty array', () => {
    expect(Array.isArray(seedRepos)).toBe(true)
    expect(seedRepos.length).toBeGreaterThanOrEqual(20)
  })

  it('every entry has url and category', () => {
    for (const entry of seedRepos) {
      expect(entry).toHaveProperty('url')
      expect(entry).toHaveProperty('category')
      expect(typeof entry.url).toBe('string')
      expect(typeof entry.category).toBe('string')
    }
  })

  it('every url is a valid GitHub URL', () => {
    for (const entry of seedRepos) {
      const url = new URL(entry.url)
      expect(url.hostname).toBe('github.com')
      const parts = url.pathname.replace(/^\//, '').split('/')
      expect(parts.length).toBeGreaterThanOrEqual(2)
      expect(parts[0].length).toBeGreaterThan(0)
      expect(parts[1].length).toBeGreaterThan(0)
    }
  })

  it('every category is valid', () => {
    for (const entry of seedRepos) {
      expect(VALID_CATEGORIES).toContain(entry.category)
    }
  })

  it('has no duplicate URLs', () => {
    const urls = seedRepos.map((e) => e.url)
    expect(new Set(urls).size).toBe(urls.length)
  })

  it('covers all categories', () => {
    const categories = new Set(seedRepos.map((e) => e.category))
    for (const cat of VALID_CATEGORIES) {
      expect(categories.has(cat)).toBe(true)
    }
  })
})
