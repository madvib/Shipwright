import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback } from 'react'
import { toast } from 'sonner'
import { useProfiles } from '#/features/studio/useProfiles'
import { ProfileList } from '#/features/studio/ProfileList'
import { ProfileEditor } from '#/features/studio/ProfileEditor'
import { ConfirmDialog } from '#/components/ConfirmDialog'
import { FirstRunBanner, useFirstRunBanner } from '#/features/studio/FirstRunBanner'
import { usePresetInit } from '#/features/studio/usePresetInit'
import type { ProjectLibrary } from '#/features/compiler/types'

export const Route = createFileRoute('/studio/profiles')({ component: ProfilesPage })

function ProfilesPage() {
  const { profiles, activeId, setActiveId, addProfile, updateProfile, removeProfile } = useProfiles()
  const [editing, setEditing] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null)
  const banner = useFirstRunBanner()
  const { initFromPreset } = usePresetInit({ addProfile, updateProfile, setActiveId })

  const editingProfile = editing ? profiles.find((p) => p.id === editing) ?? null : null
  const deleteTargetProfile = deleteTarget ? profiles.find((p) => p.id === deleteTarget) : null

  const handleNew = () => {
    const id = addProfile()
    setEditing(id)
    setActiveId(id)
    toast.success('Profile created')
  }

  const handleSelect = (id: string) => {
    setActiveId(id)
    setEditing(id)
  }

  const handlePresetInit = useCallback((preset: Parameters<typeof initFromPreset>[0]) => {
    const id = initFromPreset(preset)
    setEditing(id)
    banner.dismiss()
  }, [initFromPreset, banner])

  const handleImportUrl = useCallback(async (url: string) => {
    const res = await fetch('/api/github/import', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url }),
    })
    if (!res.ok) {
      const data = await res.json().catch(() => ({ error: 'Request failed' })) as { error?: string }
      throw new Error(data.error ?? `HTTP ${res.status}`)
    }
    const library = await res.json() as ProjectLibrary
    const id = addProfile()
    const repoName = url.split('/').pop() ?? 'imported'
    updateProfile(id, {
      name: repoName,
      skills: library.skills ?? [],
      mcpServers: library.mcp_servers ?? [],
      rules: (library.rules ?? []).map((r) => r.content).filter(Boolean),
    })
    setActiveId(id)
    setEditing(id)
    banner.dismiss()
  }, [addProfile, updateProfile, setActiveId, banner])

  const handleDelete = (id: string) => {
    setDeleteTarget(id)
  }

  const confirmDelete = () => {
    if (!deleteTarget) return
    const name = deleteTargetProfile?.name || 'profile'
    removeProfile(deleteTarget)
    if (editing === deleteTarget) setEditing(null)
    setDeleteTarget(null)
    toast.success(`Deleted "${name}"`)
  }

  return (
    <div className="h-full flex flex-col">
      {editingProfile ? (
        <ProfileEditor
          profile={editingProfile}
          onChange={(patch) => updateProfile(editingProfile.id, patch)}
          onBack={() => setEditing(null)}
          onDelete={() => handleDelete(editingProfile.id)}
        />
      ) : (
        <>
          {banner.show && profiles.length <= 1 && (
            <div className="px-5 pt-5">
              <FirstRunBanner
                onDismiss={banner.dismiss}
                onPresetInit={handlePresetInit}
                onImportUrl={handleImportUrl}
              />
            </div>
          )}
          <ProfileList
            profiles={profiles}
            activeId={activeId}
            onSelect={handleSelect}
            onNew={handleNew}
            onDelete={handleDelete}
          />
        </>
      )}
      <ConfirmDialog
        open={deleteTarget !== null}
        onClose={() => setDeleteTarget(null)}
        onConfirm={confirmDelete}
        title="Delete profile"
        message={`Are you sure you want to delete "${deleteTargetProfile?.name || 'this profile'}"? This action cannot be undone.`}
        confirmLabel="Delete"
        variant="danger"
      />
    </div>
  )
}
