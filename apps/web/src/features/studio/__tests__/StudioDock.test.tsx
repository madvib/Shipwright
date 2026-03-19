import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import { StudioDock } from '../StudioDock'

const mockNavigate = vi.fn()
let mockPathname = '/studio'

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => mockNavigate,
  useRouterState: ({ select }: { select: (s: { location: { pathname: string } }) => string }) =>
    select({ location: { pathname: mockPathname } }),
}))

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

describe('StudioDock', () => {
  it('renders all 6 nav items', () => {
    render(<StudioDock />)
    expect(screen.getByRole('navigation')).toBeTruthy()
    const buttons = screen.getAllByRole('button')
    expect(buttons).toHaveLength(6)
  })

  it('renders Overview nav item', () => {
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    // Overview is the first button
    expect(buttons[0]).toBeTruthy()
  })

  it('active item has bg-primary/12 class when on /studio', () => {
    mockPathname = '/studio'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    // Overview is exact match for /studio
    expect(buttons[0]?.className).toContain('bg-primary')
  })

  it('non-active items do not have bg-primary class', () => {
    mockPathname = '/studio'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    // Profiles button should not be active on /studio
    expect(buttons[1]?.className).not.toContain('bg-primary')
  })

  it('Profiles item is active when pathname starts with /studio/profiles', () => {
    mockPathname = '/studio/profiles'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[1]?.className).toContain('bg-primary')
  })

  it('Skills item is active when pathname starts with /studio/skills', () => {
    mockPathname = '/studio/skills'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[2]?.className).toContain('bg-primary')
  })

  it('MCP item is active when pathname starts with /studio/mcp', () => {
    mockPathname = '/studio/mcp'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[3]?.className).toContain('bg-primary')
  })

  it('Export item is active when pathname starts with /studio/export', () => {
    mockPathname = '/studio/export'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[4]?.className).toContain('bg-primary')
  })

  it('Registry item is active when pathname starts with /studio/registry', () => {
    mockPathname = '/studio/registry'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[5]?.className).toContain('bg-primary')
  })

  it('active item shows bottom indicator bar', () => {
    mockPathname = '/studio'
    render(<StudioDock />)
    // The active indicator is a <span> with bg-primary class inside the active button
    const indicators = document.querySelectorAll('span.bg-primary')
    expect(indicators.length).toBeGreaterThan(0)
  })

  it('nav element has accessible label', () => {
    render(<StudioDock />)
    const nav = screen.getByRole('navigation')
    expect(nav.getAttribute('aria-label')).toBe('Studio navigation')
  })
})
