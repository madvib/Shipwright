import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { Zap, WifiOff } from 'lucide-react'
import { useSkillsIDE } from '#/features/studio/skills-ide/useSkillsIDE'
import { SkillsFileExplorer } from '#/features/studio/skills-ide/SkillsFileExplorer'
import { SkillsEditor } from '#/features/studio/skills-ide/SkillsEditor'
import { SkillsPreviewPanel } from '#/features/studio/skills-ide/SkillsPreviewPanel'
import { CreateSkillDialog } from '#/features/studio/skills-ide/CreateSkillDialog'

import { SkillsIdeSkeleton } from '#/features/studio/StudioSkeleton'

export const Route = createFileRoute('/studio/skills')({
  component: SkillsIDEPage,
  pendingComponent: SkillsIdeSkeleton,
})

function SkillsIDEPage() {
  const ide = useSkillsIDE()
  const [createOpen, setCreateOpen] = useState(false)

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
          />

          {ide.state.previewOpen && (
            <SkillsPreviewPanel
              skill={ide.activeSkill}
              content={ide.activeContent}
              activeTab={ide.state.previewTab}
              onTabChange={ide.setPreviewTab}
              onClose={() => ide.setPreviewOpen(false)}
              onAddFile={ide.addFile}
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
