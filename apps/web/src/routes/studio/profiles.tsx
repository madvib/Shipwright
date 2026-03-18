import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback } from 'react'
import { useProfiles } from '#/features/studio/useProfiles'
import { ProfileList } from '#/features/studio/ProfileList'
import { ProfileEditor } from '#/features/studio/ProfileEditor'
import { FirstRunBanner, useFirstRunBanner } from '#/features/studio/FirstRunBanner'
import { usePresetInit } from '#/features/studio/usePresetInit'
import type { ProjectLibrary } from '#/features/compiler/types'

export const Route = createFileRoute('/studio/profiles')({ component: ProfilesPage })

function ProfilesPage() {
  const { profiles, activeId, setActiveId, addProfile, updateProfile } = useProfiles()
  const [editing, setEditing] = useState<string | null>(null)
  const banner = useFirstRunBanner()
  const { initFromPreset } = usePresetInit({ addProfile, updateProfile, setActiveId })

  const editingProfile = editing ? profiles.find((p) => p.id === editing) ?? null : null

  const handleNew = () => {
    const id = addProfile()
    setEditing(id)
    setActiveId(id)
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
    // Create a profile from imported data
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

  return (
    <div className="h-full flex flex-col">
      {editingProfile ? (
        <ProfileEditor
          profile={editingProfile}
          onChange={(patch) => updateProfile(editingProfile.id, patch)}
          onBack={() => setEditing(null)}
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
          />
        </>
      )}
    </div>
  )
}
