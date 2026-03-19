import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup, fireEvent } from '@testing-library/react'
import { DefaultCatchBoundary } from '../error/DefaultCatchBoundary'

const mockInvalidate = vi.fn()

vi.mock('@tanstack/react-router', () => ({
  useRouter: () => ({ invalidate: mockInvalidate }),
  useMatch: () => false,
  rootRouteId: '__root__',
  Link: ({ to, children, className }: { to: string; children: React.ReactNode; className?: string }) => (
    <a href={to} className={className}>{children}</a>
  ),
}))

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

describe('DefaultCatchBoundary', () => {
  it('renders the error message from the error object', () => {
    render(<DefaultCatchBoundary error={new Error('Something broke badly')} reset={() => {}} />)
    expect(screen.getByText('Something broke badly')).toBeTruthy()
  })

  it('renders fallback message when error has no message', () => {
    const emptyError = Object.assign(new Error(''), { message: '' })
    render(<DefaultCatchBoundary error={emptyError} reset={() => {}} />)
    expect(screen.getByText('An unexpected error occurred.')).toBeTruthy()
  })

  it('renders a heading indicating something went wrong', () => {
    render(<DefaultCatchBoundary error={new Error('Test error')} reset={() => {}} />)
    expect(screen.getByRole('heading')).toBeTruthy()
    expect(screen.getByRole('heading').textContent).toContain('Something went wrong')
  })

  it('renders a Try Again button', () => {
    render(<DefaultCatchBoundary error={new Error('Test error')} reset={() => {}} />)
    expect(screen.getByRole('button', { name: /try again/i })).toBeTruthy()
  })

  it('Try Again button calls router.invalidate when clicked', () => {
    render(<DefaultCatchBoundary error={new Error('Test error')} reset={() => {}} />)
    fireEvent.click(screen.getByRole('button', { name: /try again/i }))
    expect(mockInvalidate).toHaveBeenCalledOnce()
  })

  it('renders a home link', () => {
    render(<DefaultCatchBoundary error={new Error('Test error')} reset={() => {}} />)
    const links = screen.getAllByRole('link')
    expect(links.length).toBeGreaterThan(0)
  })

  it('home link points to root path', () => {
    render(<DefaultCatchBoundary error={new Error('Test error')} reset={() => {}} />)
    const links = screen.getAllByRole('link')
    const homeLink = links.find((l) => l.getAttribute('href') === '/')
    expect(homeLink).toBeTruthy()
  })
})
