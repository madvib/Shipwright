import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act } from '@testing-library/react'
import { ChatDraftArea } from '../ChatDraftArea'
import type { StagedAnnotation } from '../types'

afterEach(cleanup)

const makeClickAnnotation = (id: string): StagedAnnotation => ({
  filePath: 'canvas.html',
  id,
  ann: {
    type: 'click',
    id,
    selector: '#btn',
    text: 'Submit',
    note: 'Click note',
    timestamp: new Date().toISOString(),
    x: 10,
    y: 20,
  },
})

describe('ChatDraftArea', () => {
  it('renders textarea and send button', () => {
    render(
      <ChatDraftArea
        stagedAnnotations={[]}
        onSend={vi.fn()}
        onRemoveAnnotation={vi.fn()}
        onUploadFiles={vi.fn()}
      />,
    )
    expect(screen.getByPlaceholderText(/Message agent/)).toBeTruthy()
    expect(screen.getByLabelText('Send message')).toBeTruthy()
  })

  it('send button is disabled when text is empty and no annotations', () => {
    render(
      <ChatDraftArea
        stagedAnnotations={[]}
        onSend={vi.fn()}
        onRemoveAnnotation={vi.fn()}
        onUploadFiles={vi.fn()}
      />,
    )
    const btn = screen.getByLabelText('Send message') as HTMLButtonElement
    expect(btn.disabled).toBe(true)
  })

  it('send button is enabled when text is entered', () => {
    render(
      <ChatDraftArea
        stagedAnnotations={[]}
        onSend={vi.fn()}
        onRemoveAnnotation={vi.fn()}
        onUploadFiles={vi.fn()}
      />,
    )
    const textarea = screen.getByPlaceholderText(/Message agent/) as HTMLTextAreaElement
    fireEvent.change(textarea, { target: { value: 'Hello agent' } })
    const btn = screen.getByLabelText('Send message') as HTMLButtonElement
    expect(btn.disabled).toBe(false)
  })

  it('send button is enabled with staged annotations even without text', () => {
    render(
      <ChatDraftArea
        stagedAnnotations={[makeClickAnnotation('ann-1')]}
        onSend={vi.fn()}
        onRemoveAnnotation={vi.fn()}
        onUploadFiles={vi.fn()}
      />,
    )
    const btn = screen.getByLabelText('Send message') as HTMLButtonElement
    expect(btn.disabled).toBe(false)
  })

  it('calls onSend with trimmed text and clears input', async () => {
    const onSend = vi.fn().mockResolvedValue(undefined)
    render(
      <ChatDraftArea
        stagedAnnotations={[]}
        onSend={onSend}
        onRemoveAnnotation={vi.fn()}
        onUploadFiles={vi.fn()}
      />,
    )
    const textarea = screen.getByPlaceholderText(/Message agent/) as HTMLTextAreaElement
    fireEvent.change(textarea, { target: { value: '  hello  ' } })
    await act(async () => {
      fireEvent.click(screen.getByLabelText('Send message'))
    })
    expect(onSend).toHaveBeenCalledWith('hello')
    expect(textarea.value).toBe('')
  })

  it('renders annotation cards for staged annotations', () => {
    render(
      <ChatDraftArea
        stagedAnnotations={[makeClickAnnotation('ann-1'), makeClickAnnotation('ann-2')]}
        onSend={vi.fn()}
        onRemoveAnnotation={vi.fn()}
        onUploadFiles={vi.fn()}
      />,
    )
    const removeButtons = screen.getAllByLabelText('Remove annotation')
    expect(removeButtons).toHaveLength(2)
  })

  it('calls onRemoveAnnotation when X is clicked on a card', () => {
    const onRemove = vi.fn()
    render(
      <ChatDraftArea
        stagedAnnotations={[makeClickAnnotation('ann-42')]}
        onSend={vi.fn()}
        onRemoveAnnotation={onRemove}
        onUploadFiles={vi.fn()}
      />,
    )
    fireEvent.click(screen.getByLabelText('Remove annotation'))
    expect(onRemove).toHaveBeenCalledWith('ann-42')
  })

  it('shows disabled placeholder when disabled prop is set', () => {
    render(
      <ChatDraftArea
        stagedAnnotations={[]}
        onSend={vi.fn()}
        onRemoveAnnotation={vi.fn()}
        onUploadFiles={vi.fn()}
        disabled
      />,
    )
    expect(screen.getByPlaceholderText('Connect CLI to send messages')).toBeTruthy()
  })

  it('send button is disabled when disabled prop is set', () => {
    render(
      <ChatDraftArea
        stagedAnnotations={[makeClickAnnotation('ann-1')]}
        onSend={vi.fn()}
        onRemoveAnnotation={vi.fn()}
        onUploadFiles={vi.fn()}
        disabled
      />,
    )
    const btn = screen.getByLabelText('Send message') as HTMLButtonElement
    expect(btn.disabled).toBe(true)
  })
})
