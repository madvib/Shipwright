import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import Dock from './Dock'

// Mock TanStack Router's useLocation
vi.mock('@tanstack/react-router', () => ({
  useLocation: () => ({ pathname: '/studio' }),
}))

describe('Dock', () => {
  it('renders three navigation buttons and a compile button', () => {
    render(<Dock />)
    expect(screen.getByRole('navigation', { name: 'Studio navigation' })).toBeDefined()
    expect(screen.getAllByRole('button')).toHaveLength(4) // 3 nav + 1 compile
  })

  it('marks the active section with aria-current', () => {
    render(<Dock activeSection="agents" />)
    const buttons = screen.getAllByRole('button')
    const agentsButton = buttons[0]
    expect(agentsButton.getAttribute('aria-current')).toBe('page')
    // Skills and Registry should not be active
    expect(buttons[1].getAttribute('aria-current')).toBeNull()
    expect(buttons[2].getAttribute('aria-current')).toBeNull()
  })

  it('calls onNavigate with the section label when a nav item is clicked', () => {
    const onNavigate = vi.fn()
    render(<Dock onNavigate={onNavigate} />)
    const buttons = screen.getAllByRole('button')
    fireEvent.click(buttons[1]) // Skills button
    expect(onNavigate).toHaveBeenCalledWith('skills')
  })

  it('calls onCompile when the compile button is clicked', () => {
    const onCompile = vi.fn()
    render(<Dock onCompile={onCompile} />)
    const compileButton = screen.getAllByRole('button').at(-1)!
    fireEvent.click(compileButton)
    expect(onCompile).toHaveBeenCalledOnce()
  })

  it('disables the compile button when isCompiling is true', () => {
    render(<Dock isCompiling />)
    const compileButton = screen.getAllByRole('button').at(-1)!
    expect(compileButton).toHaveProperty('disabled', true)
  })

  it('shows tooltip on hover', async () => {
    render(<Dock />)
    const firstNavButton = screen.getAllByRole('button')[0]
    const wrapper = firstNavButton.parentElement!
    fireEvent.mouseEnter(wrapper)
    expect(screen.getByText('Agents')).toBeDefined()
    fireEvent.mouseLeave(wrapper)
  })
})
