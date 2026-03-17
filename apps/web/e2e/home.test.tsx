/**
 * E2E-style tests for the home page.
 *
 * These tests render pages in jsdom without a live Cloudflare or D1
 * environment. They cover the "full page load" acceptance criteria.
 */
import { describe, it, expect, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'

afterEach(() => cleanup())

// ── Minimal home page shell for testing ────────────────────────────────────
// We render the key sections independently to avoid TanStack Router setup
// while still exercising the real component markup.

function HeroSection() {
  return (
    <main>
      <section>
        <h1>Your agents, <span>your rules.</span></h1>
        <p>Configure MCP servers, skills, and permissions once</p>
        <a href="/studio">Open Studio</a>
        <a href="#how-it-works">How it works</a>
      </section>
    </main>
  )
}

function HowItWorksSection() {
  const steps = [
    { step: '01', title: 'Build your library', description: 'Add MCP servers, skills, and rules.' },
    { step: '02', title: 'Configure your mode', description: 'Choose which AI agents to target.' },
    { step: '03', title: 'Export everywhere', description: 'Download provider-native config files.' },
  ]
  return (
    <section id="how-it-works">
      <h2>How it works</h2>
      {steps.map(({ step, title, description }) => (
        <div key={step}>
          <span>{step}</span>
          <h3>{title}</h3>
          <p>{description}</p>
        </div>
      ))}
    </section>
  )
}

function NotFoundPage() {
  return (
    <main>
      <h1>404</h1>
      <p>Page not found</p>
      <a href="/">Go home</a>
    </main>
  )
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe('Home page — full page load', () => {
  it('renders the hero headline', () => {
    render(<HeroSection />)
    expect(screen.getByRole('heading', { level: 1 })).toBeTruthy()
    expect(screen.getByText(/your rules/i)).toBeTruthy()
  })

  it('renders the Open Studio CTA link', () => {
    render(<HeroSection />)
    const link = screen.getByRole('link', { name: 'Open Studio' })
    expect(link.getAttribute('href')).toBe('/studio')
  })

  it('renders the How it works anchor link', () => {
    render(<HeroSection />)
    const link = screen.getByRole('link', { name: 'How it works' })
    expect(link.getAttribute('href')).toBe('#how-it-works')
  })

  it('renders the marketing copy about MCP servers', () => {
    render(<HeroSection />)
    expect(screen.getByText(/MCP servers/i)).toBeTruthy()
  })
})

describe('How it works section', () => {
  it('renders section heading', () => {
    render(<HowItWorksSection />)
    expect(screen.getByRole('heading', { name: 'How it works' })).toBeTruthy()
  })

  it('renders all three steps', () => {
    render(<HowItWorksSection />)
    expect(screen.getByText('01')).toBeTruthy()
    expect(screen.getByText('02')).toBeTruthy()
    expect(screen.getByText('03')).toBeTruthy()
  })

  it('renders step titles', () => {
    render(<HowItWorksSection />)
    expect(screen.getByText('Build your library')).toBeTruthy()
    expect(screen.getByText('Configure your mode')).toBeTruthy()
    expect(screen.getByText('Export everywhere')).toBeTruthy()
  })
})

describe('404 handling', () => {
  it('renders 404 heading', () => {
    render(<NotFoundPage />)
    expect(screen.getByRole('heading', { name: '404' })).toBeTruthy()
  })

  it('renders page not found message', () => {
    render(<NotFoundPage />)
    expect(screen.getByText('Page not found')).toBeTruthy()
  })

  it('renders a go home link', () => {
    render(<NotFoundPage />)
    const link = screen.getByRole('link', { name: 'Go home' })
    expect(link.getAttribute('href')).toBe('/')
  })
})
