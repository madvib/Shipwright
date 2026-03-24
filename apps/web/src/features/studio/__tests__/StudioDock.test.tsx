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

vi.mock('#/features/studio/CliStatusPopover', () => ({
  CliStatusPopover: () => <button>CLI</button>,
}))

const noopAddSkill = vi.fn()
const dockProps = { onAddSkill: noopAddSkill }

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

describe('StudioDock', () => {
  // 3 nav items + CLI mock + Preview = 5
  it('renders nav items, CLI status, and preview toggle', () => {
    render(<StudioDock {...dockProps} />)
    expect(screen.getByRole('navigation')).toBeTruthy()
    const buttons = screen.getAllByRole('button')
    expect(buttons).toHaveLength(5)
  })

  it('Agents is active on /studio/agents', () => {
    mockPathname = '/studio/agents'
    render(<StudioDock {...dockProps} />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[0]?.className).toContain('bg-primary')
  })

  it('Skills is active on /studio/skills', () => {
    mockPathname = '/studio/skills'
    render(<StudioDock {...dockProps} />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[1]?.className).toContain('bg-primary')
  })

  it('Settings is active on /studio/settings', () => {
    mockPathname = '/studio/settings'
    render(<StudioDock {...dockProps} />)
    const buttons = screen.getAllByRole('button')
    expect(buttons[2]?.className).toContain('bg-primary')
  })

  it('output toggle calls onTogglePreview', () => {
    const onToggle = vi.fn()
    render(<StudioDock {...dockProps} onTogglePreview={onToggle} />)
    const buttons = screen.getAllByRole('button')
    // CLI is [3], Preview is [4]
    buttons[4]?.click()
    expect(onToggle).toHaveBeenCalledOnce()
  })

  it('nav has accessible label', () => {
    render(<StudioDock {...dockProps} />)
    const nav = screen.getByRole('navigation')
    expect(nav.getAttribute('aria-label')).toBe('Studio navigation')
  })
})
