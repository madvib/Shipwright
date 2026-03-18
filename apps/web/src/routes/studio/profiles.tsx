import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { useProfiles } from '#/features/studio/useProfiles'
import { ProfileList } from '#/features/studio/ProfileList'
import { ProfileEditor } from '#/features/studio/ProfileEditor'

export const Route = createFileRoute('/studio/profiles')({ component: ProfilesPage })

function ProfilesPage() {
  const { profiles, activeId, setActiveId, addProfile, updateProfile } = useProfiles()
  const [editing, setEditing] = useState<string | null>(null)

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

  return (
    <div className="h-full flex flex-col">
      {editingProfile ? (
        <ProfileEditor
          profile={editingProfile}
          onChange={(patch) => updateProfile(editingProfile.id, patch)}
          onBack={() => setEditing(null)}
        />
      ) : (
        <ProfileList
          profiles={profiles}
          activeId={activeId}
          onSelect={handleSelect}
          onNew={handleNew}
        />
      )}
    </div>
  )
}
