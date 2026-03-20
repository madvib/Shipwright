const SETTINGS_KEY = 'ship-settings-v1'

export interface SettingsData {
  autoImport: boolean
  createPr: boolean
  defaultProvider: string
  defaultModel: string
  defaultMode: string
  extendedThinking: boolean
  autoMemory: boolean
  permissionPreset: string
  hooks: Array<{ trigger: string; command: string }>
  envVars: Array<{ key: string; value: string }>
}

export const DEFAULT_SETTINGS: SettingsData = {
  autoImport: true,
  createPr: true,
  defaultProvider: 'claude',
  defaultModel: 'claude-sonnet-4-6',
  defaultMode: 'default',
  extendedThinking: true,
  autoMemory: false,
  permissionPreset: 'ship-guarded',
  hooks: [{ trigger: 'Stop', command: 'ship mcp sync-permissions' }],
  envVars: [
    { key: 'SHIP_DIR', value: '.ship' },
    { key: 'NODE_ENV', value: 'development' },
  ],
}

export function loadSettings(): SettingsData {
  try {
    const raw =
      typeof window !== 'undefined'
        ? window.localStorage.getItem(SETTINGS_KEY)
        : null
    if (!raw) return DEFAULT_SETTINGS
    return { ...DEFAULT_SETTINGS, ...JSON.parse(raw) }
  } catch {
    return DEFAULT_SETTINGS
  }
}

export function saveSettings(data: SettingsData) {
  try {
    window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(data))
  } catch {
    /* ignore */
  }
}
