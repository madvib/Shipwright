// Right-side drawer: Chat + Events + Terminal tabs.
// Opens/closes with a toggle button at the right edge. State persisted to localStorage.
// Width: 320px (w-80). Canvas reflows in a flex layout — not fixed positioned.

import { useState, useCallback } from 'react'
import { MessageSquare, Activity, PanelRightClose, PanelRightOpen, TerminalIcon } from 'lucide-react'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@ship/primitives'
import { ChatTab } from './ChatTab'
import { EventStreamPanel } from '../events/EventStreamPanel'
import { TerminalTab } from './TerminalTab'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import type { StagedAnnotation } from './types'

const LS_KEY = 'ship-studio:right-drawer-open'

function readInitialOpen(): boolean {
  try {
    const stored = localStorage.getItem(LS_KEY)
    return stored === null ? true : stored === 'true'
  } catch {
    return true
  }
}

interface Props {
  stagedAnnotations: StagedAnnotation[]
  onSend: (text: string) => Promise<void>
  onRemoveAnnotation: (id: string) => void
  onUploadFiles: (files: FileList) => void
  disabled?: boolean
}

export function RightDrawer({
  stagedAnnotations,
  onSend,
  onRemoveAnnotation,
  onUploadFiles,
  disabled = false,
}: Props) {
  const [open, setOpen] = useState<boolean>(readInitialOpen)
  const [activeTab, setActiveTab] = useState<string>('chat')
  const { workspaces } = useDaemon()

  const activeWorkspaceId = workspaces.find((w) => w.status === 'active')?.id ?? null

  const toggle = useCallback(() => {
    setOpen((prev) => {
      const next = !prev
      try { localStorage.setItem(LS_KEY, String(next)) } catch { /* ignore */ }
      return next
    })
  }, [])

  return (
    <div className="flex shrink-0 relative">
      {/* Toggle button — sits at the left edge of this container (right edge of canvas) */}
      <button
        onClick={toggle}
        className="absolute left-0 top-1/2 -translate-y-1/2 -translate-x-full z-10 flex items-center justify-center w-5 h-10 rounded-l-md border border-r-0 border-border bg-card/80 text-muted-foreground hover:text-foreground hover:bg-card transition-colors"
        title={open ? 'Close panel' : 'Open Chat & Events'}
        aria-label={open ? 'Close right panel' : 'Open right panel'}
      >
        {open
          ? <PanelRightClose className="size-3" />
          : <PanelRightOpen className="size-3" />
        }
      </button>

      {open && (
        <div className="w-80 shrink-0 flex flex-col border-l border-border bg-card/30 min-h-0 overflow-hidden">
          <Tabs value={activeTab} onValueChange={setActiveTab} className="flex flex-col flex-1 min-h-0 gap-0">
            <TabsList className="h-9 w-full rounded-none border-b border-border bg-transparent p-0 justify-start gap-0 shrink-0">
              <TabsTrigger
                value="chat"
                className="flex items-center gap-1.5 flex-1 h-full rounded-none border-b-2 text-[11px] font-medium data-[active]:border-primary data-[active]:text-primary border-transparent text-muted-foreground"
              >
                <MessageSquare className="size-3" />
                Chat
                {stagedAnnotations.length > 0 && (
                  <span className="ml-0.5 flex size-4 items-center justify-center rounded-full bg-primary text-[9px] font-bold text-primary-foreground">
                    {stagedAnnotations.length > 9 ? '9+' : stagedAnnotations.length}
                  </span>
                )}
              </TabsTrigger>
              <TabsTrigger
                value="events"
                className="flex items-center gap-1.5 flex-1 h-full rounded-none border-b-2 text-[11px] font-medium data-[active]:border-primary data-[active]:text-primary border-transparent text-muted-foreground"
              >
                <Activity className="size-3" />
                Events
              </TabsTrigger>
              <TabsTrigger
                value="terminal"
                className="flex items-center gap-1.5 flex-1 h-full rounded-none border-b-2 text-[11px] font-medium data-[active]:border-primary data-[active]:text-primary border-transparent text-muted-foreground"
              >
                <TerminalIcon className="size-3" />
                Terminal
              </TabsTrigger>
            </TabsList>

            <TabsContent value="chat" className="flex flex-col flex-1 min-h-0 mt-0">
              <ChatTab
                stagedAnnotations={stagedAnnotations}
                onSend={onSend}
                onRemoveAnnotation={onRemoveAnnotation}
                onUploadFiles={onUploadFiles}
                disabled={disabled}
              />
            </TabsContent>

            <TabsContent value="events" className="flex flex-col flex-1 min-h-0 mt-0">
              <EventStreamPanel />
            </TabsContent>

            <TabsContent value="terminal" className="flex flex-col flex-1 min-h-0 mt-0">
              <TerminalTab
                workspaceId={activeWorkspaceId}
                visible={activeTab === 'terminal'}
              />
            </TabsContent>
          </Tabs>
        </div>
      )}
    </div>
  )
}
