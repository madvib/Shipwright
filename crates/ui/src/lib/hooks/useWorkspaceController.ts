import { useState } from 'react';
import {
  AdrEntry,
  EventRecord,
  FeatureDocument,
  FeatureInfo as FeatureEntry,
  IssueEntry,
  ModeConfig,
  ProjectDiscovery as Project,
  ProjectConfig,
  ReleaseDocument,
  ReleaseInfo as ReleaseEntry,
  SpecDocument,
  SpecInfo as SpecEntry,
} from '@/bindings';
import { Config, DEFAULT_STATUSES } from '@/lib/workspace-ui';
import {
  getActiveProject,
  ingestEventChanges,
  listEventEntries,
  setActiveModeCmd,
} from '../platform/tauri/commands';
import { isTauriRuntime } from '../platform/tauri/runtime';
import { applyTheme, SIDEBAR_COLLAPSED_STORAGE_KEY, projectFromInfo } from './workspace/constants';
import { useWorkspaceLifecycle } from './workspace/useWorkspaceLifecycle';
import { useProjectActions } from './workspace/useProjectActions';
import { useIssueActions } from './workspace/useIssueActions';
import { useAdrActions } from './workspace/useAdrActions';
import { useSpecActions } from './workspace/useSpecActions';
import { useSettingsActions } from './workspace/useSettingsActions';
import { useReleaseActions } from './workspace/useReleaseActions';
import { useFeatureActions } from './workspace/useFeatureActions';

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
  const [activeProject, setActiveProject] = useState<Project | null>(null);
  const [detectedProject, setDetectedProject] = useState<Project | null>(null);
  const [detectingProject, setDetectingProject] = useState(false);
  const [creatingProject, setCreatingProject] = useState(false);
  const [recentProjects, setRecentProjects] = useState<Project[]>([]);
  const [issues, setIssues] = useState<IssueEntry[]>([]);
  const [adrs, setAdrs] = useState<AdrEntry[]>([]);
  const [specs, setSpecs] = useState<SpecEntry[]>([]);
  const [releases, setReleases] = useState<ReleaseEntry[]>([]);
  const [features, setFeatures] = useState<FeatureEntry[]>([]);
  const [eventEntries, setEventEntries] = useState<EventRecord[]>([]);
  const [config, setConfig] = useState<Config>({});
  const [projectConfig, setProjectConfig] = useState<ProjectConfig | null>(null);
  const [globalAgentConfig, setGlobalAgentConfig] = useState<ProjectConfig | null>(null);
  const [selectedIssue, setSelectedIssue] = useState<IssueEntry | null>(null);
  const [selectedAdr, setSelectedAdr] = useState<AdrEntry | null>(null);
  const [selectedSpec, setSelectedSpec] = useState<SpecDocument | null>(null);
  const [selectedRelease, setSelectedRelease] = useState<ReleaseDocument | null>(null);
  const [selectedFeature, setSelectedFeature] = useState<FeatureDocument | null>(null);
  const [showNewIssue, setShowNewIssue] = useState(false);
  const [showNewAdr, setShowNewAdr] = useState(false);
  const [switchingMode, setSwitchingMode] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState<boolean>(() => {
    if (typeof window === 'undefined') return false;
    return window.localStorage.getItem(SIDEBAR_COLLAPSED_STORAGE_KEY) === '1';
  });
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

  const { loadProjectData, loadProjectConfig, refreshDetectedProject } = useWorkspaceLifecycle({
    activeProject,
    sidebarCollapsed,
    setIssues,
    setAdrs,
    setSpecs,
    setReleases,
    setFeatures,
    setEventEntries,
    setProjectConfig,
    setGlobalAgentConfig,
    setDetectedProject,
    setDetectingProject,
    setRecentProjects,
    setConfig,
    setError,
    setLoading,
  });

  const refreshEvents = async () => {
    if (!isTauriRuntime()) return;
    const entries = await listEventEntries(0, 200).catch(() => []);
    setEventEntries(entries);
  };

  const refreshActivity = async () => {
    if (!isTauriRuntime()) return;
    await refreshEvents();
  };

  const ingestEvents = async () => {
    if (!isTauriRuntime()) return 0;
    const count = await ingestEventChanges().catch(() => 0);
    await loadProjectData();
    await refreshEvents();
    return count;
  };

  const refreshProjectInfo = async () => {
    if (!isTauriRuntime()) return;
    const info = await getActiveProject().catch(() => null);
    if (info) setActiveProject(projectFromInfo(info));
  };

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
    setSelectedIssue,
    setSelectedAdr,
    setSelectedSpec,
    setSelectedRelease,
    setSelectedFeature,
    setCreatingProject,
    loadProjectData,
    loadProjectConfig,
  });

  const {
    handleCreateIssue,
    handleStatusChange,
    handleSaveIssue,
    handleDeleteIssue,
  } = useIssueActions({
    issues,
    setIssues,
    setSelectedIssue,
    setShowNewIssue,
    setError,
    refreshActivity,
    refreshProjectInfo,
  });

  const {
    handleCreateAdr,
    handleSelectAdr,
    handleSaveAdr,
    handleDeleteAdr,
  } = useAdrActions({
    setAdrs,
    setSelectedAdr,
    setShowNewAdr,
    setError,
    refreshActivity,
  });

  const {
    handleSelectSpec,
    handleCreateSpec,
    handleSaveSpec,
    handleDeleteSpec,
  } = useSpecActions({
    setSpecs,
    setSelectedSpec,
    setError,
    refreshActivity,
  });

  const {
    handleSelectRelease,
    handleCreateRelease,
    handleSaveRelease,
  } = useReleaseActions({
    setReleases,
    setSelectedRelease,
    setError,
    refreshActivity,
  });

  const {
    handleSelectFeature,
    handleCreateFeature,
    handleSaveFeature,
  } = useFeatureActions({
    setFeatures,
    setSelectedFeature,
    setError,
    refreshActivity,
  });

  const { handleSaveSettings, handleSaveProjectSettings, handleSaveGlobalAgentSettings } = useSettingsActions({
    setConfig,
    setProjectConfig,
    setGlobalAgentConfig,
    setError,
  });

  const handleSetActiveMode = async (modeId: string | null) => {
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
  };

  const noProject = !activeProject;
  const tagSuggestions = Array.from(
    new Set(
      issues
        .flatMap((entry) => entry.issue?.tags ?? [])
        .filter((tag): tag is string => typeof tag === 'string' && tag.trim().length > 0)
    )
  ).sort((a, b) => a.localeCompare(b));
  const mcpEnabled = config.mcp_enabled !== false;

  return {
    activeProject,
    detectedProject,
    detectingProject,
    creatingProject,
    recentProjects,
    issues,
    adrs,
    specs,
    releases,
    features,
    eventEntries,
    config,
    projectConfig,
    globalAgentConfig,
    selectedIssue,
    selectedAdr,
    selectedSpec,
    selectedRelease,
    selectedFeature,
    showNewIssue,
    showNewAdr,
    sidebarCollapsed,
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
    tagSuggestions,
    mcpEnabled,
    setSelectedIssue,
    setSelectedAdr,
    setSelectedSpec,
    setSelectedRelease,
    setSelectedFeature,
    setShowNewIssue,
    setShowNewAdr,
    setSidebarCollapsed,
    setError,
    refreshDetectedProject,
    handleOpenProject,
    handleNewProject,
    handlePickProjectDirectory,
    handleCreateProjectFromForm,
    handleSelectProject,
    handleCreateIssue,
    handleStatusChange,
    handleSaveIssue,
    handleDeleteIssue,
    handleCreateAdr,
    handleSelectAdr,
    handleSaveAdr,
    handleDeleteAdr,
    handleSelectSpec,
    handleCreateSpec,
    handleSaveSpec,
    handleDeleteSpec,
    handleSelectRelease,
    handleCreateRelease,
    handleSaveRelease,
    handleSelectFeature,
    handleCreateFeature,
    handleSaveFeature,
    refreshActivity,
    refreshEvents,
    ingestEvents,
    handleSaveSettings,
    handleSaveProjectSettings,
    handleSaveGlobalAgentSettings,
    handleSetActiveMode,
    applyTheme,
  };
}
