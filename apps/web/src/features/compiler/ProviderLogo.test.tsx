import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ProviderLogo } from './ProviderLogo'

describe('ProviderLogo', () => {
  it('renders cursor logo images (light and dark)', () => {
    const { container } = render(<ProviderLogo provider="cursor" />)
    const imgs = container.querySelectorAll('img')
    expect(imgs).toHaveLength(2)
    const alts = Array.from(imgs).map((img) => img.getAttribute('alt'))
    expect(alts).toEqual(['Cursor', 'Cursor'])
  })

  it('renders codex as a single img with OpenAI src', () => {
    const { container } = render(<ProviderLogo provider="codex" />)
    const img = container.querySelector('img')
    expect(img).not.toBeNull()
    expect(img!.getAttribute('alt')).toBe('Codex')
    expect(img!.getAttribute('src')).toContain('OpenAI')
  })

  it('renders claude as a single img', () => {
    const { container } = render(<ProviderLogo provider="claude" />)
    const img = container.querySelector('img')
    expect(img).not.toBeNull()
    expect(img!.getAttribute('alt')).toBe('Claude')
  })

  it('renders gemini as a single img', () => {
    const { container } = render(<ProviderLogo provider="gemini" />)
    const img = container.querySelector('img')
    expect(img).not.toBeNull()
    expect(img!.getAttribute('alt')).toBe('Gemini')
  })

  it('renders fallback badge for unknown provider', () => {
    render(<ProviderLogo provider="unknown-provider" />)
    // fallback renders a span with first 2 chars uppercased
    const span = screen.getByText('UN')
    expect(span).toBeTruthy()
  })

  it('applies size class md correctly', () => {
    const { container } = render(<ProviderLogo provider="claude" size="md" />)
    const img = container.querySelector('img')
    expect(img!.className).toContain('size-4')
  })

  it('applies size class lg correctly', () => {
    const { container } = render(<ProviderLogo provider="claude" size="lg" />)
    const img = container.querySelector('img')
    expect(img!.className).toContain('size-5')
  })

  it('applies extra className prop', () => {
    const { container } = render(<ProviderLogo provider="claude" className="my-custom" />)
    const img = container.querySelector('img')
    expect(img!.className).toContain('my-custom')
  })
})
