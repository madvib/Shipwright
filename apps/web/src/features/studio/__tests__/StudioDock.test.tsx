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
  it('renders 3 nav items + output toggle', () => {
    render(<StudioDock />)
    expect(screen.getByRole('navigation')).toBeTruthy()
    const buttons = screen.getAllByRole('button')
    expect(buttons).toHaveLength(4) // Agents, Skills, Settings, Output
  })

  it('Agents is active on /studio/agents', () => {
    mockPathname = '/studio/agents'
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

  it('Settings is active on /studio/settings', () => {
    mockPathname = '/studio/settings'
    render(<StudioDock />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[2]?.className).toContain('bg-primary')
  })

  it('output toggle calls onTogglePreview', () => {
    const onToggle = vi.fn()
    render(<StudioDock onTogglePreview={onToggle} />)
    const buttons = screen.getAllByRole('button')
    buttons[3]?.click()
    expect(onToggle).toHaveBeenCalledOnce()
  })

  it('nav has accessible label', () => {
    render(<StudioDock />)
    const nav = screen.getByRole('navigation')
    expect(nav.getAttribute('aria-label')).toBe('Studio navigation')
  })
})
