// Floating comment tooltip that appears when text is selected in the milkdown editor.
// Uses native DOM Selection API — no ProseMirror plugin needed.
// Shows a "Comment" button, expands to a textarea, calls onComment(selectedText, comment).

import { useState, useEffect, useRef, useCallback } from 'react'
import { createPortal } from 'react-dom'

interface Props {
  /** Ref to the editor container — tooltip only appears for selections within this element */
  containerRef: React.RefObject<HTMLElement | null>
  /** Called when the user submits a comment */
  onComment: (selectedText: string, comment: string) => void
}

interface TooltipState {
  x: number
  y: number
  selectedText: string
}

export function SelectionCommentTooltip({ containerRef, onComment }: Props) {
  const [tooltip, setTooltip] = useState<TooltipState | null>(null)
  const [expanded, setExpanded] = useState(false)
  const [comment, setComment] = useState('')
  const inputRef = useRef<HTMLTextAreaElement>(null)
  const tooltipRef = useRef<HTMLDivElement>(null)

  const checkSelection = useCallback(() => {
    const sel = window.getSelection()
    if (!sel || sel.isCollapsed || !sel.rangeCount) {
      // Don't dismiss if the input is focused (user is typing a comment)
      if (expanded && inputRef.current === document.activeElement) return
      setTooltip(null)
      setExpanded(false)
      setComment('')
      return
    }

    const container = containerRef.current
    if (!container) return

    // Check selection is within our editor
    const anchorNode = sel.anchorNode
    if (!anchorNode || !container.contains(anchorNode)) {
      if (expanded && inputRef.current === document.activeElement) return
      setTooltip(null)
      setExpanded(false)
      setComment('')
      return
    }

    const range = sel.getRangeAt(0)
    const rect = range.getBoundingClientRect()
    const text = sel.toString().trim()
    if (!text) return

    setTooltip({
      x: rect.left + rect.width / 2,
      y: rect.top - 8,
      selectedText: text,
    })
  }, [containerRef, expanded])

  useEffect(() => {
    document.addEventListener('selectionchange', checkSelection)
    return () => document.removeEventListener('selectionchange', checkSelection)
  }, [checkSelection])

  // Dismiss on Escape
  useEffect(() => {
    if (!tooltip) return
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setTooltip(null)
        setExpanded(false)
        setComment('')
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [tooltip])

  // Focus input when expanded
  useEffect(() => {
    if (expanded) inputRef.current?.focus()
  }, [expanded])

  const handleSubmit = useCallback(() => {
    if (!tooltip || !comment.trim()) return
    onComment(tooltip.selectedText, comment.trim())
    setTooltip(null)
    setExpanded(false)
    setComment('')
    // Collapse the selection
    window.getSelection()?.removeAllRanges()
  }, [tooltip, comment, onComment])

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault()
      handleSubmit()
    }
  }, [handleSubmit])

  if (!tooltip) return null

  return createPortal(
    <div
      ref={tooltipRef}
      className="ship-comment-tooltip"
      style={{
        position: 'fixed',
        left: tooltip.x,
        top: tooltip.y,
        transform: 'translate(-50%, -100%)',
        zIndex: 100,
      }}
      onMouseDown={(e) => e.stopPropagation()}
    >
      {!expanded ? (
        <button
          onClick={() => setExpanded(true)}
          className="ship-comment-tooltip-btn"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z" />
          </svg>
          Comment
        </button>
      ) : (
        <div className="ship-comment-tooltip-expanded">
          <div className="ship-comment-tooltip-selected">
            &ldquo;{tooltip.selectedText.slice(0, 80)}{tooltip.selectedText.length > 80 ? '...' : ''}&rdquo;
          </div>
          <textarea
            ref={inputRef}
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Add your comment..."
            className="ship-comment-tooltip-input"
            rows={2}
          />
          <div className="ship-comment-tooltip-actions">
            <button
              onClick={() => { setExpanded(false); setComment('') }}
              className="ship-comment-tooltip-cancel"
            >
              Cancel
            </button>
            <button
              onClick={handleSubmit}
              disabled={!comment.trim()}
              className="ship-comment-tooltip-submit"
            >
              Add comment
              <kbd>⌘↵</kbd>
            </button>
          </div>
        </div>
      )}
    </div>,
    document.body,
  )
}
