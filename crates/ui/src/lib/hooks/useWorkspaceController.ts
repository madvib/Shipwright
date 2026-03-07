import { useState, useCallback, useMemo } from 'react';
import {
  EventRecord,
  ModeConfig,
  ProjectDiscovery as Project,
  ProjectConfig,
} from '@/bindings';
import { Config, DEFAULT_STATUSES } from '@/lib/workspace-ui';
import {
  ingestEventChanges,
  listEventEntries,
  setActiveModeCmd,
} from '../platform/tauri/commands';
import { isTauriRuntime } from '../platform/tauri/runtime';
import { SIDEBAR_COLLAPSED_STORAGE_KEY } from './workspace/constants';
import { useWorkspaceLifecycle } from './workspace/useWorkspaceLifecycle';
import { useProjectActions } from './workspace/useProjectActions';
import { useSettingsActions } from './workspace/useSettingsActions';
import { useShipEntities } from './ship/useShipEntities';

function mergeModes(base: ModeConfig[] = [], overlay: ModeConfig[] = []): ModeConfig[] {
  const merged = [...base];
  for (const mode of overlay) {
    const index = merged.findIndex((entry) => entry.id === mode.id);
    if (index >= 0) {
      merged[index] = mode;
    } else {
      merged.push(mode);
    }
  }
  return merged;
}

export function useWorkspaceController() {
  const [notesScope, setNotesScope] = useState<'project' | 'global'>('project');
  const [activeProject, setActiveProject] = useState<Project | null>(null);
  const [detectedProject, setDetectedProject] = useState<Project | null>(null);
  const [detectingProject, setDetectingProject] = useState(false);
  const [creatingProject, setCreatingProject] = useState(false);
  const [recentProjects, setRecentProjects] = useState<Project[]>([]);
  const [eventEntries, setEventEntries] = useState<EventRecord[]>([]);
  const [config, setConfig] = useState<Config>({});
  const [projectConfig, setProjectConfig] = useState<ProjectConfig | null>(null);
  const [globalAgentConfig, setGlobalAgentConfig] = useState<ProjectConfig | null>(null);
  const [switchingMode, setSwitchingMode] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState<boolean>(() => {
    if (typeof window === 'undefined') return true;
    const stored = window.localStorage.getItem(SIDEBAR_COLLAPSED_STORAGE_KEY);
    if (stored === null) return true;
    return stored === '1';
  });
  const [isWorkspaceFocusMode, setIsWorkspaceFocusMode] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const statuses = projectConfig?.statuses?.length ? projectConfig.statuses : DEFAULT_STATUSES;
  const modes = mergeModes(globalAgentConfig?.modes ?? [], projectConfig?.modes ?? []);
  const activeModeId = projectConfig?.active_mode ?? globalAgentConfig?.active_mode ?? null;
  const activeMode = modes.find((mode) => mode.id === activeModeId) ?? null;
  const aiProvider =
    projectConfig?.ai?.provider?.trim() ||
    globalAgentConfig?.ai?.provider?.trim() ||
    null;
  const aiModel =
    projectConfig?.ai?.model?.trim() ||
    globalAgentConfig?.ai?.model?.trim() ||
    null;

  const refreshEvents = useCallback(async () => {
    if (!isTauriRuntime()) return;
    const entries = await listEventEntries(0, 200).catch(() => []);
    setEventEntries(entries);
  }, []);

  const refreshActivity = useCallback(async () => {
    if (!isTauriRuntime()) return;
    await refreshEvents();
  }, [refreshEvents]);

  const ship = useShipEntities({
    refreshActivity,
    setError,
  });
  const { loadProjectData, loadProjectConfig, refreshDetectedProject } = useWorkspaceLifecycle({
    activeProject,
    sidebarCollapsed,
    setEventEntries,
    setProjectConfig,
    setGlobalAgentConfig,
    setDetectedProject,
    setDetectingProject,
    setRecentProjects,
    setConfig,
    setError,
    setLoading,
    onProjectDataChange: ship.loadShipData,
  });

  const ingestEvents = useCallback(async () => {
    if (!isTauriRuntime()) return 0;
    const count = await ingestEventChanges().catch(() => 0);
    await loadProjectData();
    await refreshEvents();
    return count;
  }, [loadProjectData, refreshEvents]);

  const {
    handleOpenProject,
    handleNewProject,
    handlePickProjectDirectory,
    handleCreateProjectFromForm,
    handleSelectProject,
  } = useProjectActions({
    setError,
    setActiveProject,
    setDetectedProject,
    setSelectedAdr: ship.setSelectedAdr,
    setSelectedSpec: ship.setSelectedSpec,
    setSelectedRelease: ship.setSelectedRelease,
    setSelectedFeature: ship.setSelectedFeature,
    setCreatingProject,
    loadProjectData,
    loadProjectConfig,
  });

  const { handleSaveSettings, handleSaveProjectSettings, handleSaveGlobalAgentSettings } = useSettingsActions({
    setConfig,
    setProjectConfig,
    setGlobalAgentConfig,
    setError,
  });

  const handleSetActiveMode = useCallback(async (modeId: string | null) => {
    if (!projectConfig) return;

    if (!isTauriRuntime()) {
      setProjectConfig((current) => (current ? { ...current, active_mode: modeId } : current));
      return;
    }

    setSwitchingMode(true);
    setError(null);
    try {
      await setActiveModeCmd(modeId);
      await loadProjectConfig();
      await refreshActivity();
      await refreshEvents();
    } catch (error) {
      setError(`Failed to switch mode: ${String(error)}`);
    } finally {
      setSwitchingMode(false);
    }
  }, [projectConfig, loadProjectConfig, refreshActivity, refreshEvents]);

  const noProject = !activeProject;
  const mcpEnabled = config.mcp_enabled !== false;

  const applyTheme = useCallback((theme?: 'light' | 'dark') => {
    if (theme) {
      handleSaveSettings({ ...config, theme });
    }
  }, [config, handleSaveSettings]);

  return useMemo(() => ({
    activeProject,
    detectedProject,
    detectingProject,
    creatingProject,
    recentProjects,
    eventEntries,
    config,
    projectConfig,
    globalAgentConfig,
    sidebarCollapsed,
    isWorkspaceFocusMode,
    error,
    loading,
    statuses,
    modes,
    activeMode,
    activeModeId,
    aiProvider,
    aiModel,
    switchingMode,
    noProject,
    mcpEnabled,
    notesScope,
    setNotesScope,
    setSidebarCollapsed,
    setIsWorkspaceFocusMode,
    setError,
    refreshDetectedProject,
    handleOpenProject,
    handleNewProject,
    handlePickProjectDirectory,
    handleCreateProjectFromForm,
    handleSelectProject,
    refreshActivity,
    refreshEvents,
    ingestEvents,
    handleSaveSettings,
    handleSaveProjectSettings,
    handleSaveGlobalAgentSettings,
    handleSetActiveMode,
    applyTheme,
    ship,
  }), [
    activeProject,
    detectedProject,
    detectingProject,
    creatingProject,
    recentProjects,
    eventEntries,
    config,
    projectConfig,
    globalAgentConfig,
    sidebarCollapsed,
    isWorkspaceFocusMode,
    error,
    loading,
    statuses,
    modes,
    activeMode,
    activeModeId,
    aiProvider,
    aiModel,
    switchingMode,
    noProject,
    mcpEnabled,
    notesScope,
    setNotesScope,
    setSidebarCollapsed,
    setIsWorkspaceFocusMode,
    setError,
    refreshDetectedProject,
    handleOpenProject,
    handleNewProject,
    handlePickProjectDirectory,
    handleCreateProjectFromForm,
    handleSelectProject,
    refreshActivity,
    refreshEvents,
    ingestEvents,
    handleSaveSettings,
    handleSaveProjectSettings,
    handleSaveGlobalAgentSettings,
    handleSetActiveMode,
    applyTheme,
    ship,
  ]);
}
