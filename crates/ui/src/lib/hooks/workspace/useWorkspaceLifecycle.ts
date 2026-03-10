import { Dispatch, SetStateAction, useCallback, useEffect, useMemo } from 'react';
import {
  EventRecord,
  ProjectDiscovery as Project,
  ProjectConfig,
} from '@/bindings';
import { Config } from '@/lib/workspace-ui';
import { subscribeProjectEvents } from '../../platform/tauri/events';
import {
  detectCurrentProject,
  getAppSettingsCmd,
  getProjectConfigCmd,
  listEventEntries,
  listProjects,
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
  setEventEntries: Dispatch<SetStateAction<EventRecord[]>>;
  setProjectConfig: Dispatch<SetStateAction<ProjectConfig | null>>;
  setGlobalAgentConfig: Dispatch<SetStateAction<ProjectConfig | null>>;
  setDetectedProject: Dispatch<SetStateAction<Project | null>>;
  setDetectingProject: Dispatch<SetStateAction<boolean>>;
  setRecentProjects: Dispatch<SetStateAction<Project[]>>;
  setConfig: Dispatch<SetStateAction<Config>>;
  setError: Dispatch<SetStateAction<string | null>>;
  setLoading: Dispatch<SetStateAction<boolean>>;
  onProjectDataChange?: () => Promise<void>;
}

export function useWorkspaceLifecycle({
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
  onProjectDataChange,
}: UseWorkspaceLifecycleParams) {
  const loadProjectData = useCallback(async () => {
    if (!isTauriRuntime()) {
      return;
    }

    try {
      const eventList = await listEventEntries(0, 200).catch(() => []);
      setEventEntries(eventList);
      if (onProjectDataChange) {
        await onProjectDataChange();
      }
    } catch (error) {
      console.error('Failed to load project data', error);
    }
  }, [setEventEntries, onProjectDataChange]);

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

    let projectDataTimer: number | undefined;
    let configTimer: number | undefined;
    let eventTimer: number | undefined;
    let disposed = false;
    let cleanup: (() => void) | null = null;
    let projectDataInFlight = false;
    let projectDataQueued = false;

    const debounce = (timer: number | undefined, action: () => void): number => {
      if (timer) window.clearTimeout(timer);
      return window.setTimeout(action, 200);
    };

    const runProjectDataReload = async () => {
      if (projectDataInFlight) {
        projectDataQueued = true;
        return;
      }
      projectDataInFlight = true;
      try {
        await loadProjectData();
      } finally {
        projectDataInFlight = false;
        if (projectDataQueued && !disposed) {
          projectDataQueued = false;
          void runProjectDataReload();
        }
      }
    };

    const scheduleProjectDataReload = () => {
      if (typeof document !== 'undefined' && document.visibilityState === 'hidden') {
        projectDataQueued = true;
        return;
      }
      projectDataTimer = debounce(projectDataTimer, () => {
        void runProjectDataReload();
      });
    };

    const onVisibilityChange = () => {
      if (document.visibilityState === 'visible' && projectDataQueued) {
        projectDataQueued = false;
        void runProjectDataReload();
      }
    };
    document.addEventListener('visibilitychange', onVisibilityChange);

    void subscribeProjectEvents({
      onSpecsChanged: () => {
        scheduleProjectDataReload();
      },
      onAdrsChanged: () => {
        scheduleProjectDataReload();
      },
      onFeaturesChanged: () => {
        scheduleProjectDataReload();
      },
      onReleasesChanged: () => {
        scheduleProjectDataReload();
      },
      onNotesChanged: () => {
        scheduleProjectDataReload();
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
      if (projectDataTimer) window.clearTimeout(projectDataTimer);
      if (configTimer) window.clearTimeout(configTimer);
      if (eventTimer) window.clearTimeout(eventTimer);
      document.removeEventListener('visibilitychange', onVisibilityChange);
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

  return useMemo(() => ({
    loadProjectData,
    loadProjectConfig,
    refreshDetectedProject,
  }), [loadProjectData, loadProjectConfig, refreshDetectedProject]);
}
