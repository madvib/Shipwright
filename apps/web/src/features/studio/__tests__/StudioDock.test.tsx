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
  it('renders 3 nav items + compile button', () => {
    render(<StudioDock />)
    expect(screen.getByRole('navigation')).toBeTruthy()
    const buttons = screen.getAllByRole('button')
    expect(buttons).toHaveLength(4) // Agents, Skills, Registry, Compile
  })

  it('Agents is active on /studio', () => {
    mockPathname = '/studio'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[0]?.className).toContain('bg-primary')
  })

  it('Skills is active on /studio/skills', () => {
    mockPathname = '/studio/skills'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[1]?.className).toContain('bg-primary')
  })

  it('Registry is active on /registry', () => {
    mockPathname = '/registry'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[2]?.className).toContain('bg-primary')
  })

  it('non-active items do not have bg-primary class', () => {
    mockPathname = '/studio'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[1]?.className).not.toContain('bg-primary')
  })

  it('compile button calls onCompile', () => {
    const onCompile = vi.fn()
    render(<StudioDock onCompile={onCompile} />)
    const buttons = screen.getAllByRole('button')
    buttons[3]?.click()
    expect(onCompile).toHaveBeenCalledOnce()
  })

  it('active item shows bottom indicator bar', () => {
    mockPathname = '/studio'
    render(<StudioDock />)
    const indicators = document.querySelectorAll('span.bg-primary')
    expect(indicators.length).toBeGreaterThan(0)
  })

  it('nav has accessible label', () => {
    render(<StudioDock />)
    const nav = screen.getByRole('navigation')
    expect(nav.getAttribute('aria-label')).toBe('Studio navigation')
  })
})
