import { useCallback } from 'react'
import type { PresetTemplate } from './FirstRunBanner'
import type { Profile } from './useProfiles'

interface PresetInitOps {
  addProfile: () => string
  updateProfile: (id: string, patch: Partial<Profile>) => void
  setActiveId: (id: string) => void
}

/**
 * Hook to initialize a profile from a preset template.
 * Accepts profile operations from the parent's useProfiles() to ensure
 * shared state. Pattern: operation hook + error handling + loading state.
 */
export function usePresetInit(ops: PresetInitOps) {
  const { addProfile, updateProfile, setActiveId } = ops

  const initFromPreset = useCallback((preset: PresetTemplate): string => {
    const id = addProfile()
    const name = preset.id === 'blank' ? 'New Profile' : preset.label.split('(')[0].trim()
    updateProfile(id, {
      name,
      persona: preset.persona,
      selectedProviders: preset.providers,
      rules: preset.rules,
    })
    setActiveId(id)
    return id
  }, [addProfile, updateProfile, setActiveId])

  return { initFromPreset }
}
