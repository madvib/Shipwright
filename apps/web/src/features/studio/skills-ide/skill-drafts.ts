// Pure data model for skill editor drafts.
// All dirty/local-file derivation is computed from a single drafts record.
// Also exports tab-ID utilities used by components and the IDE hook.

export const SKILL_MD = 'SKILL.md'

/** Composite tab ID: `skillId::filePath` */
export function makeTabId(skillId: string, filePath: string): string {
  return `${skillId}::${filePath}`
}

export function parseTabId(tabId: string): { skillId: string; filePath: string } {
  const idx = tabId.indexOf('::')
  if (idx === -1) return { skillId: tabId, filePath: SKILL_MD }
  return { skillId: tabId.slice(0, idx), filePath: tabId.slice(idx + 2) }
}

export interface SkillDraft {
  /** Current buffer content */
  content: string
  /** Content when the file was opened or last saved (for dirty detection) */
  originalContent: string
  /** True if this file was created locally (not from MCP pull data) */
  isNew: boolean
}

/** A draft is dirty if it is new (not yet persisted) or content has changed. */
export function isDraftDirty(draft: SkillDraft): boolean {
  if (draft.isNew) return true
  return draft.content !== draft.originalContent
}

/** Derive the set of unsaved tab IDs from the drafts map. */
export function deriveUnsavedIds(drafts: Record<string, SkillDraft>): Set<string> {
  const ids = new Set<string>()
  for (const [tabId, draft] of Object.entries(drafts)) {
    if (isDraftDirty(draft)) ids.add(tabId)
  }
  return ids
}

/** Derive the map of locally-added files (isNew) grouped by skillId. */
export function deriveLocalFiles(drafts: Record<string, SkillDraft>): Record<string, string[]> {
  const result: Record<string, string[]> = {}
  for (const [tabId, draft] of Object.entries(drafts)) {
    if (!draft.isNew) continue
    const { skillId, filePath } = parseTabId(tabId)
    if (!result[skillId]) result[skillId] = []
    result[skillId].push(filePath)
  }
  return result
}
