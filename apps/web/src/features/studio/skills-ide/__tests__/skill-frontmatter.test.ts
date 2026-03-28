import { describe, it, expect } from 'vitest'
import {
  parseFrontmatter,
  validateFrontmatter,
  extractFrontmatterBlock,
} from '../skill-frontmatter'

const FULL_FRONTMATTER = `---
name: my-skill
stable-id: my-skill
description: Use when testing
tags: [testing, ci]
authors: [alice, bob]
version: 1.0.0
license: MIT
compatibility: 0.2.0+
allowed-tools: [Bash, Read, Grep]
attribution: Original by Alice
---

# My Skill
`

describe('parseFrontmatter extracts all known fields', () => {
  it('parses every known field from a complete frontmatter block', () => {
    const fm = parseFrontmatter(FULL_FRONTMATTER)
    expect(fm.name).toBe('my-skill')
    expect(fm['stable-id']).toBe('my-skill')
    expect(fm.description).toBe('Use when testing')
    expect(fm.tags).toEqual(['testing', 'ci'])
    expect(fm.authors).toEqual(['alice', 'bob'])
    expect(fm.version).toBe('1.0.0')
    expect(fm.license).toBe('MIT')
    expect(fm.compatibility).toBe('0.2.0+')
    expect(fm['allowed-tools']).toEqual(['Bash', 'Read', 'Grep'])
    expect(fm.attribution).toBe('Original by Alice')
  })

  it('returns empty object when no frontmatter fences present', () => {
    const fm = parseFrontmatter('# Just a heading\nSome content.')
    expect(fm).toEqual({})
  })

  it('ignores comment lines in frontmatter', () => {
    const content = `---
name: test
# This is a comment
description: valid
---
`
    const fm = parseFrontmatter(content)
    expect(fm.name).toBe('test')
    expect(fm.description).toBe('valid')
  })
})

describe('parseFrontmatter handles inline arrays', () => {
  it('parses [a, b, c] as string array', () => {
    const content = `---
tags: [alpha, beta, gamma]
---
`
    const fm = parseFrontmatter(content)
    expect(fm.tags).toEqual(['alpha', 'beta', 'gamma'])
  })

  it('handles single-element inline array', () => {
    const content = `---
tags: [solo]
---
`
    const fm = parseFrontmatter(content)
    expect(fm.tags).toEqual(['solo'])
  })

  it('handles empty inline array', () => {
    const content = `---
tags: []
---
`
    const fm = parseFrontmatter(content)
    expect(fm.tags).toEqual([])
  })

  it('trims whitespace in array elements', () => {
    const content = `---
authors: [  alice  ,  bob  ,  carol  ]
---
`
    const fm = parseFrontmatter(content)
    expect(fm.authors).toEqual(['alice', 'bob', 'carol'])
  })
})

describe('validateFrontmatter flags missing name as error', () => {
  it('reports error when name is missing', () => {
    const content = `---
stable-id: valid-id
description: has description
---
`
    const warnings = validateFrontmatter(content)
    const nameError = warnings.find((w) => w.field === 'name')
    expect(nameError).toBeDefined()
    expect(nameError!.severity).toBe('error')
    expect(nameError!.message).toContain('required')
  })

  it('does not flag name when present', () => {
    const content = `---
name: valid-name
stable-id: valid-id
description: ok
---
`
    const warnings = validateFrontmatter(content)
    expect(warnings.find((w) => w.field === 'name')).toBeUndefined()
  })
})

describe('validateFrontmatter flags invalid stable-id format', () => {
  it('flags uppercase characters in stable-id', () => {
    const content = `---
name: test
stable-id: Invalid-Id
---
`
    const warnings = validateFrontmatter(content)
    const stableIdErr = warnings.find(
      (w) => w.field === 'stable-id' && w.severity === 'error',
    )
    expect(stableIdErr).toBeDefined()
    expect(stableIdErr!.message).toContain('a-z0-9')
  })

  it('flags stable-id starting with a hyphen', () => {
    const content = `---
name: test
stable-id: -starts-bad
---
`
    const warnings = validateFrontmatter(content)
    const stableIdErr = warnings.find(
      (w) => w.field === 'stable-id' && w.severity === 'error',
    )
    expect(stableIdErr).toBeDefined()
  })

  it('accepts valid stable-id with lowercase and hyphens', () => {
    const content = `---
name: test
stable-id: my-valid-id-123
description: ok
---
`
    const warnings = validateFrontmatter(content)
    expect(warnings.find((w) => w.field === 'stable-id' && w.severity === 'error')).toBeUndefined()
  })
})

describe('validateFrontmatter flags unknown fields as warnings', () => {
  it('warns on a single unknown field', () => {
    const content = `---
name: test
stable-id: test
custom-field: value
---
`
    const warnings = validateFrontmatter(content)
    const unknown = warnings.find((w) => w.field === 'custom-field')
    expect(unknown).toBeDefined()
    expect(unknown!.severity).toBe('warning')
    expect(unknown!.message).toContain('Unknown field')
  })

  it('warns on multiple unknown fields', () => {
    const content = `---
name: test
stable-id: test
foo: bar
baz: qux
---
`
    const warnings = validateFrontmatter(content)
    const unknowns = warnings.filter((w) => w.message.startsWith('Unknown field'))
    expect(unknowns).toHaveLength(2)
    expect(unknowns.map((w) => w.field).sort()).toEqual(['baz', 'foo'])
  })

  it('does not warn on known fields', () => {
    const content = `---
name: test
stable-id: test
description: ok
tags: [a]
authors: [b]
version: 1.0
license: MIT
compatibility: 0.2+
allowed-tools: [Bash]
attribution: test
---
`
    const warnings = validateFrontmatter(content)
    const unknowns = warnings.filter((w) => w.message.startsWith('Unknown field'))
    expect(unknowns).toHaveLength(0)
  })
})

describe('extractFrontmatterBlock', () => {
  it('returns null when no fences present', () => {
    expect(extractFrontmatterBlock('# No frontmatter here')).toBeNull()
  })

  it('extracts content between --- fences', () => {
    const block = extractFrontmatterBlock('---\nname: test\n---\n# Body')
    expect(block).toBe('name: test')
  })
})
