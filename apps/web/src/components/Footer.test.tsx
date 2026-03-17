import { describe, it, expect, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import Footer from './Footer'

afterEach(() => cleanup())

describe('Footer', () => {
  it('renders Ship Studio branding', () => {
    render(<Footer />)
    expect(screen.getAllByText('Ship Studio')[0]).toBeTruthy()
  })

  it('renders current year in copyright notice', () => {
    render(<Footer />)
    const year = new Date().getFullYear().toString()
    const matches = screen.getAllByText((content) => content.includes(year))
    expect(matches.length).toBeGreaterThan(0)
  })

  it('renders GitHub link', () => {
    render(<Footer />)
    const githubLinks = screen.getAllByLabelText('GitHub')
    expect(githubLinks[0].getAttribute('href')).toContain('github.com')
  })

  it('renders X (Twitter) link', () => {
    render(<Footer />)
    const xLinks = screen.getAllByLabelText('X (Twitter)')
    expect(xLinks[0].getAttribute('href')).toContain('x.com')
  })

  it('external links open in new tab with rel noopener', () => {
    render(<Footer />)
    const githubLinks = screen.getAllByLabelText('GitHub')
    expect(githubLinks[0].getAttribute('target')).toBe('_blank')
    expect(githubLinks[0].getAttribute('rel')).toContain('noopener')
  })
})
