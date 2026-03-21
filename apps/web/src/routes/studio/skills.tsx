import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { Zap } from 'lucide-react'
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
        <div className="flex size-12 items-center justify-center rounded-xl border border-border/60 bg-muted/40">
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
      <div className="hidden md:flex flex-1 h-full min-h-0 overflow-hidden">
        <SkillsFileExplorer
          filteredSkills={ide.filteredSkills}
          activeTabId={ide.state.activeTabId}
          expandedFolders={ide.state.expandedFolders}
          searchQuery={ide.state.searchQuery}
          onSearchChange={ide.setSearchQuery}
          onToggleFolder={ide.toggleFolder}
          onOpenSkill={ide.openSkill}
          onCreateSkill={() => setCreateOpen(true)}
        />

        <SkillsEditor
          skills={ide.skills}
          openTabIds={ide.state.openTabIds}
          activeTabId={ide.state.activeTabId}
          unsavedIds={ide.state.unsavedIds}
          content={ide.activeContent}
          onTabSelect={ide.setActiveTabId}
          onTabClose={ide.closeTab}
          onContentChange={ide.updateContent}
          onSave={ide.saveSkill}
        />

        {ide.state.previewOpen && (
          <SkillsPreviewPanel
            skill={ide.activeSkill}
            content={ide.activeContent}
            activeTab={ide.state.previewTab}
            onTabChange={ide.setPreviewTab}
            onClose={() => ide.setPreviewOpen(false)}
          />
        )}
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
