import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup, fireEvent } from '@testing-library/react'
import { EmptyState } from '../EmptyState'

afterEach(() => cleanup())

describe('EmptyState', () => {
  it('renders the title', () => {
    render(
      <EmptyState
        icon={<span>icon</span>}
        title="No packages found"
        description="Try searching for something else"
      />
    )
    expect(screen.getByText('No packages found')).toBeTruthy()
  })

  it('renders the description', () => {
    render(
      <EmptyState
        icon={<span>icon</span>}
        title="Empty"
        description="Nothing here yet"
      />
    )
    expect(screen.getByText('Nothing here yet')).toBeTruthy()
  })

  it('renders the icon when provided', () => {
    render(
      <EmptyState
        icon={<span data-testid="test-icon">icon-content</span>}
        title="Empty"
        description="Nothing here"
      />
    )
    expect(screen.getByTestId('test-icon')).toBeTruthy()
  })

  it('renders action element when action prop is provided', () => {
    render(
      <EmptyState
        icon={<span>icon</span>}
        title="Empty"
        description="Nothing here"
        action={<button>Add Item</button>}
      />
    )
    expect(screen.getByRole('button', { name: 'Add Item' })).toBeTruthy()
  })

  it('action button onClick fires when clicked', () => {
    const handleClick = vi.fn()
    render(
      <EmptyState
        icon={<span>icon</span>}
        title="Empty"
        description="Nothing here"
        action={<button onClick={handleClick}>Add Item</button>}
      />
    )
    fireEvent.click(screen.getByRole('button', { name: 'Add Item' }))
    expect(handleClick).toHaveBeenCalledOnce()
  })

  it('does not render action section when action prop is omitted', () => {
    render(
      <EmptyState
        icon={<span>icon</span>}
        title="Empty"
        description="Nothing here"
      />
    )
    expect(screen.queryByRole('button')).toBeNull()
  })

  it('renders title and description in correct order', () => {
    render(
      <EmptyState
        icon={<span>icon</span>}
        title="My Title"
        description="My Description"
      />
    )
    const title = screen.getByText('My Title')
    const description = screen.getByText('My Description')
    // Title should come before description in DOM order
    expect(title.compareDocumentPosition(description) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
  })
})
