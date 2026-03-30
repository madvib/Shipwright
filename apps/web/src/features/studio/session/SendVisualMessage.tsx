// FAB + compose panel for sending staged annotations as a visual message to the agent.
// Floats bottom-right of the content area. Cmd+Enter / Ctrl+Enter sends.

import { useState, useCallback, useEffect, useRef } from 'react'
import { Send, X, MessageSquare } from 'lucide-react'

interface SendVisualMessageProps {
  stagedCount: number
  onSend: (summary: string) => Promise<void>
}

export function SendVisualMessage({ stagedCount, onSend }: SendVisualMessageProps) {
  const [open, setOpen] = useState(false)
  const [summary, setSummary] = useState('')
  const [sending, setSending] = useState(false)
  const [sent, setSent] = useState(false)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    if (open) textareaRef.current?.focus()
  }, [open])

  const handleSend = useCallback(async () => {
    if (sending) return
    setSending(true)
    try {
      await onSend(summary.trim())
      setSummary('')
      setOpen(false)
      setSent(true)
      setTimeout(() => setSent(false), 2000)
    } finally {
      setSending(false)
    }
  }, [sending, summary, onSend])

  // Cmd+Enter / Ctrl+Enter to send
  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
        e.preventDefault()
        void handleSend()
      }
      if (e.key === 'Escape') setOpen(false)
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [open, handleSend])

  if (stagedCount === 0 && !sent) return null

  return (
    <div className="absolute bottom-4 right-4 z-20 flex flex-col items-end gap-2">
      {open && (
        <div className="w-72 rounded-xl border border-border bg-popover shadow-xl flex flex-col overflow-hidden">
          <div className="flex items-center justify-between px-3 py-2 border-b border-border">
            <span className="text-xs font-semibold text-foreground">Send to agent</span>
            <button onClick={() => setOpen(false)} className="rounded p-0.5 hover:bg-muted/50 transition">
              <X className="size-3.5 text-muted-foreground" />
            </button>
          </div>

          <div className="px-3 py-2 text-[11px] text-muted-foreground border-b border-border/50">
            {stagedCount} annotation{stagedCount !== 1 ? 's' : ''} staged
          </div>

          <textarea
            ref={textareaRef}
            value={summary}
            onChange={(e) => setSummary(e.target.value)}
            placeholder="Optional summary… (Cmd+Enter to send)"
            rows={3}
            className="resize-none w-full bg-transparent px-3 py-2 text-xs text-foreground placeholder:text-muted-foreground/50 focus:outline-none"
          />

          <div className="flex items-center justify-end px-3 py-2 border-t border-border/50">
            <button
              onClick={() => void handleSend()}
              disabled={sending}
              className="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition"
            >
              <Send className="size-3" />
              {sending ? 'Sending…' : 'Send'}
            </button>
          </div>
        </div>
      )}

      <button
        onClick={() => setOpen((v) => !v)}
        className={`relative flex items-center gap-2 rounded-full px-4 py-2.5 text-xs font-semibold shadow-lg transition-all ${
          sent
            ? 'bg-emerald-500 text-white'
            : 'bg-primary text-primary-foreground hover:bg-primary/90'
        }`}
      >
        <MessageSquare className="size-3.5" />
        {sent ? 'Sent!' : 'Send to agent'}
        {stagedCount > 0 && !sent && (
          <span className="absolute -top-1.5 -right-1.5 flex size-4 items-center justify-center rounded-full bg-destructive text-[9px] font-bold text-white">
            {stagedCount > 9 ? '9+' : stagedCount}
          </span>
        )}
      </button>
    </div>
  )
}
