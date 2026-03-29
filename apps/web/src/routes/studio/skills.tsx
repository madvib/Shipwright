import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback, useRef } from 'react'
import type { FrontmatterEntry } from '@ship/primitives'
import { composeFrontmatterDocument, splitFrontmatterDocument } from '@ship/primitives'
import { Zap, WifiOff } from 'lucide-react'
import { useSkillsIDE } from '#/features/studio/skills-ide/useSkillsIDE'
import { SkillsFileExplorer } from '#/features/studio/skills-ide/SkillsFileExplorer'
import { SkillsEditor } from '#/features/studio/skills-ide/SkillsEditor'
import { SkillsPreviewPanel } from '#/features/studio/skills-ide/SkillsPreviewPanel'
import { CreateSkillDialog } from '#/features/studio/skills-ide/CreateSkillDialog'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'

import { SkillsIdeSkeleton } from '#/features/studio/StudioSkeleton'

interface SkillFeedbackEntry {
  skillName: string
  tabId: string
  selectedText: string
  comment: string
  timestamp: string
}


export const Route = createFileRoute('/studio/skills')({
  component: SkillsIDEPage,
  pendingComponent: SkillsIdeSkeleton,
})

function SkillsIDEPage() {
  const ide = useSkillsIDE()
  const [createOpen, setCreateOpen] = useState(false)
  const mcp = useLocalMcpContext()
  const feedbackRef = useRef<SkillFeedbackEntry[]>([])

  // Frontmatter state — shared between editor and preview panel
  const [fmEntries, setFmEntries] = useState<FrontmatterEntry[]>([])
  const [fmRaw, setFmRaw] = useState<string | null>(null)

  const handleFrontmatterParsed = useCallback((entries: FrontmatterEntry[], raw: string | null) => {
    setFmEntries(entries)
    setFmRaw(raw)
  }, [])

  const handleFrontmatterUpdate = useCallback((newRaw: string) => {
    // Recompose the document with updated frontmatter
    if (!ide.state.activeTabId) return
    const currentContent = ide.activeContent
    const doc = splitFrontmatterDocument(currentContent)
    const updated = composeFrontmatterDocument(newRaw, doc.body, doc.delimiter ?? '---')
    ide.updateContent(ide.state.activeTabId, updated)
  }, [ide])

  const handleComment = useCallback(
    (selectedText: string, comment: string, skillName: string, tabId: string) => {
      const entry: SkillFeedbackEntry = {
        skillName,
        tabId,
        selectedText: selectedText.slice(0, 500),
        comment,
        timestamp: new Date().toISOString(),
      }
      feedbackRef.current = [...feedbackRef.current, entry]

      if (mcp?.status === 'connected') {
        mcp
          .callTool('write_session_file', {
            path: 'skill-feedback.json',
            content: JSON.stringify(feedbackRef.current, null, 2),
          })
          .catch(() => {
            // MCP write failed; feedback is still retained in memory
          })
      }
    },
    [mcp],
  )

  return (
    <>
      {/* Mobile fallback */}
      <div className="flex md:hidden flex-col items-center justify-center gap-4 px-8 py-20 text-center min-h-[60vh]">
        <div className="flex size-12 items-center justify-center rounded-xl border border-border bg-muted/40">
          <Zap className="size-5 text-muted-foreground" />
        </div>
        <div>
          <p className="font-display text-base font-semibold">Best on desktop</p>
          <p className="mt-1 text-sm text-muted-foreground max-w-xs">
            The Skills IDE is a three-panel editor. Open it on a wider screen for the full experience.
          </p>
        </div>
      </div>

      {/* Full IDE layout */}
      <div className="hidden md:flex flex-1 flex-col h-full min-h-0 overflow-hidden">
        {!ide.isConnected && (
          <div className="flex items-center gap-2 px-4 py-1.5 border-b border-amber-500/30 bg-amber-500/10 text-[11px] text-amber-600 dark:text-amber-400 shrink-0">
            <WifiOff className="size-3 shrink-0" />
            CLI disconnected — edits saved locally, connect to sync
          </div>
        )}

        <div className="flex flex-1 min-h-0">
          <SkillsFileExplorer
            filteredSkills={ide.filteredSkills}
            activeTabId={ide.state.activeTabId}
            expandedFolders={ide.state.expandedFolders}
            searchQuery={ide.state.searchQuery}
            isConnected={ide.isConnected}
            isLoading={ide.isLoading}
            getLibrarySkill={ide.getLibrarySkill}
            onSearchChange={ide.setSearchQuery}
            onToggleFolder={ide.toggleFolder}
            onCollapseAll={ide.collapseAll}
            onOpenFile={ide.openFile}
            onAddFile={ide.addFile}
            onDeleteFile={ide.deleteFile}
            onCreateSkill={() => setCreateOpen(true)}
          />

          <SkillsEditor
            skills={ide.skills}
            openTabIds={ide.state.openTabIds}
            activeTabId={ide.state.activeTabId}
            unsavedIds={ide.state.unsavedIds}
            content={ide.activeContent}
            isConnected={ide.isConnected}
            isLoading={ide.isLoading}
            previewOpen={ide.state.previewOpen}
            onTabSelect={ide.setActiveTabId}
            onTabClose={ide.closeTab}
            onContentChange={ide.updateContent}
            onSave={ide.saveSkill}
            onTogglePreview={() => ide.setPreviewOpen(!ide.state.previewOpen)}
            onCreateSkill={() => setCreateOpen(true)}
            onComment={handleComment}
            onFrontmatterParsed={handleFrontmatterParsed}
          />

          {ide.state.previewOpen && (
            <SkillsPreviewPanel
              skill={ide.activeSkill}
              activeTab={ide.state.previewTab}
              onTabChange={ide.setPreviewTab}
              onClose={() => ide.setPreviewOpen(false)}
              onAddFile={ide.addFile}
              frontmatterEntries={fmEntries}
              frontmatterRaw={fmRaw}
              onFrontmatterUpdate={handleFrontmatterUpdate}
            />
          )}
        </div>
      </div>

      <CreateSkillDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        onCreateSkill={ide.createSkill}
        existingIds={ide.skills.map((s) => s.id)}
      />
    </>
  )
}
