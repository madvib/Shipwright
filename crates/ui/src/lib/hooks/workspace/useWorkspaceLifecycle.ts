import { Dispatch, SetStateAction, useCallback, useEffect } from 'react';
import {
  AdrEntry,
  EventRecord,
  FeatureInfo as FeatureEntry,
  IssueEntry,
  ProjectDiscovery as Project,
  ProjectConfig,
  ReleaseInfo as ReleaseEntry,
  SpecInfo as SpecEntry,
} from '@/bindings';
import { Config } from '@/lib/workspace-ui';
import { subscribeProjectEvents } from '../../platform/tauri/events';
import {
  detectCurrentProject,
  getAppSettingsCmd,
  getProjectConfigCmd,
  listAdrs,
  listEventEntries,
  listFeatures,
  listIssues,
  listProjects,
  listReleases,
  listSpecs,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';
import {
  SIDEBAR_COLLAPSED_STORAGE_KEY,
  SETTINGS_STORAGE_KEY,
  THEME_STORAGE_KEY,
  applyTheme,
  dedupeProjects,
  projectFromInfo,
} from './constants';

interface UseWorkspaceLifecycleParams {
  activeProject: Project | null;
  sidebarCollapsed: boolean;
  setIssues: Dispatch<SetStateAction<IssueEntry[]>>;
  setAdrs: Dispatch<SetStateAction<AdrEntry[]>>;
  setSpecs: Dispatch<SetStateAction<SpecEntry[]>>;
  setReleases: Dispatch<SetStateAction<ReleaseEntry[]>>;
  setFeatures: Dispatch<SetStateAction<FeatureEntry[]>>;
  setEventEntries: Dispatch<SetStateAction<EventRecord[]>>;
  setProjectConfig: Dispatch<SetStateAction<ProjectConfig | null>>;
  setGlobalAgentConfig: Dispatch<SetStateAction<ProjectConfig | null>>;
  setDetectedProject: Dispatch<SetStateAction<Project | null>>;
  setDetectingProject: Dispatch<SetStateAction<boolean>>;
  setRecentProjects: Dispatch<SetStateAction<Project[]>>;
  setConfig: Dispatch<SetStateAction<Config>>;
  setError: Dispatch<SetStateAction<string | null>>;
  setLoading: Dispatch<SetStateAction<boolean>>;
}

export function useWorkspaceLifecycle({
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
}: UseWorkspaceLifecycleParams) {
  const loadProjectData = useCallback(async () => {
    if (!isTauriRuntime()) {
      setIssues([]);
      setAdrs([]);
      setSpecs([]);
      setReleases([]);
      setFeatures([]);
      return;
    }

    try {
      const [issueList, adrList, specList, releaseList, featureList, eventList] = await Promise.all([
        listIssues().catch(() => []),
        listAdrs().catch(() => []),
        listSpecs().catch(() => []),
        listReleases().catch(() => []),
        listFeatures().catch(() => []),
        listEventEntries(0, 200).catch(() => []),
      ]);
      setIssues(issueList);
      setAdrs(adrList);
      setSpecs(specList);
      setReleases(releaseList);
      setFeatures(featureList);
      setEventEntries(eventList);
    } catch (error) {
      console.error('Failed to load project data', error);
    }
  }, [setAdrs, setEventEntries, setFeatures, setIssues, setReleases, setSpecs]);

  const loadProjectConfig = useCallback(async () => {
    if (!isTauriRuntime()) {
      setProjectConfig(null);
      return;
    }

    const cfg = await getProjectConfigCmd().catch((error) => {
      console.error('Failed to load project config:', error);
      return null;
    });
    setProjectConfig(cfg);
  }, [setProjectConfig]);

  const refreshDetectedProject = useCallback(async (): Promise<Project | null> => {
    if (!isTauriRuntime()) {
      setDetectedProject(null);
      return null;
    }

    setDetectingProject(true);
    try {
      const detected = await detectCurrentProject().catch((error) => {
        console.error('Failed to detect current project:', error);
        return null;
      });
      const mapped = detected ? projectFromInfo(detected) : null;
      setDetectedProject(mapped);
      return mapped;
    } finally {
      setDetectingProject(false);
    }
  }, [setDetectedProject, setDetectingProject]);

  useEffect(() => {
    if (typeof window === 'undefined') return;
    window.localStorage.setItem(SIDEBAR_COLLAPSED_STORAGE_KEY, sidebarCollapsed ? '1' : '0');
  }, [sidebarCollapsed]);

  useEffect(() => {
    const handleFocus = () => {
      if (activeProject) {
        void loadProjectData();
      }
    };

    window.addEventListener('focus', handleFocus);
    return () => window.removeEventListener('focus', handleFocus);
  }, [activeProject, loadProjectData]);

  useEffect(() => {
    if (!activeProject || !isTauriRuntime()) return;

    let issueTimer: number | undefined;
    let configTimer: number | undefined;
    let eventTimer: number | undefined;
    let disposed = false;
    let cleanup: (() => void) | null = null;

    const debounce = (timer: number | undefined, action: () => void): number => {
      if (timer) window.clearTimeout(timer);
      return window.setTimeout(action, 200);
    };

    void subscribeProjectEvents({
      onIssuesChanged: () => {
        issueTimer = debounce(issueTimer, () => {
          void loadProjectData();
        });
      },
      onLogChanged: () => {
        eventTimer = debounce(eventTimer, () => {
          void listEventEntries(0, 200)
            .then(setEventEntries)
            .catch(() => []);
        });
      },
      onConfigChanged: () => {
        configTimer = debounce(configTimer, () => {
          void loadProjectConfig();
        });
      },
      onEventsChanged: () => {
        eventTimer = debounce(eventTimer, () => {
          void listEventEntries(0, 200)
            .then(setEventEntries)
            .catch(() => []);
          void loadProjectData();
        });
      },
    })
      .then((unlisten) => {
        if (disposed) {
          unlisten();
          return;
        }
        cleanup = unlisten;
      })
      .catch(() => {
        cleanup = null;
      });

    return () => {
      disposed = true;
      if (issueTimer) window.clearTimeout(issueTimer);
      if (configTimer) window.clearTimeout(configTimer);
      if (eventTimer) window.clearTimeout(eventTimer);
      cleanup?.();
    };
  }, [activeProject, loadProjectConfig, loadProjectData, setEventEntries]);

  useEffect(() => {
    (async () => {
      try {
        const savedTheme = localStorage.getItem(THEME_STORAGE_KEY) ?? 'dark';
        const storedSettings = localStorage.getItem(SETTINGS_STORAGE_KEY);

        let localSettings: Config = {};
        if (storedSettings) {
          try {
            localSettings = JSON.parse(storedSettings) as Config;
          } catch {
            localSettings = {};
          }
        }

        if (!isTauriRuntime()) {
          const localConfig: Config = { ...localSettings, theme: localSettings.theme ?? savedTheme };
          setConfig(localConfig);
          setGlobalAgentConfig(null);
          applyTheme(localConfig.theme);
          setLoading(false);
          return;
        }

        const projects = await listProjects().catch((error) => {
          console.error('Failed to load projects:', error);
          return [];
        });
        setRecentProjects(dedupeProjects(projects));

        await refreshDetectedProject();

        const globalSettings = await getAppSettingsCmd().catch((error) => {
          console.error('Failed to load global settings:', error);
          return null;
        });
        setGlobalAgentConfig(globalSettings);

        const merged = { ...localSettings, theme: localSettings.theme ?? savedTheme ?? 'dark' };
        setConfig(merged);
        applyTheme(merged.theme);
      } catch (error) {
        console.error('Init error', error);
        setError(String(error));
      } finally {
        setLoading(false);
      }
    })();
  }, [
    refreshDetectedProject,
    setConfig,
    setError,
    setGlobalAgentConfig,
    setLoading,
    setRecentProjects,
  ]);

  return {
    loadProjectData,
    loadProjectConfig,
    refreshDetectedProject,
  };
}
