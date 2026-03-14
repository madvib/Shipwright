import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
import { ProjectConfig } from '@/bindings';
import { Config } from '@/lib/workspace-ui';
import { saveAppSettingsCmd, saveProjectConfigCmd } from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';
import { SETTINGS_STORAGE_KEY, THEME_STORAGE_KEY, applyTheme } from './constants';

interface UseSettingsActionsParams {
  setConfig: Dispatch<SetStateAction<Config>>;
  setProjectConfig: Dispatch<SetStateAction<ProjectConfig | null>>;
  setGlobalAgentConfig: Dispatch<SetStateAction<ProjectConfig | null>>;
  setError: Dispatch<SetStateAction<string | null>>;
}

export function useSettingsActions({
  setConfig,
  setProjectConfig,
  setGlobalAgentConfig,
  setError,
}: UseSettingsActionsParams) {
  const handleSaveSettings = useCallback(async (newConfig: Config) => {
    try {
      setConfig(newConfig);
      localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(newConfig));
      if (newConfig.theme) {
        localStorage.setItem(THEME_STORAGE_KEY, newConfig.theme);
      }
      applyTheme(newConfig.theme);
    } catch (error) {
      setError(String(error));
    }
  }, [setConfig, setError]);

  const handleSaveProjectSettings = useCallback(async (newConfig: ProjectConfig) => {
    if (!isTauriRuntime()) {
      setError('Project settings can only be saved in Tauri runtime.');
      return;
    }

    try {
      await saveProjectConfigCmd(newConfig);
      setProjectConfig(newConfig);
    } catch (error) {
      setError(String(error));
    }
  }, [setProjectConfig, setError]);

  const handleSaveGlobalAgentSettings = useCallback(async (newConfig: ProjectConfig) => {
    if (!isTauriRuntime()) {
      setError('Global settings can only be saved in Tauri runtime.');
      return;
    }

    try {
      await saveAppSettingsCmd(newConfig);
      setGlobalAgentConfig(newConfig);
    } catch (error) {
      setError(String(error));
    }
  }, [setGlobalAgentConfig, setError]);

  return useMemo(() => ({
    handleSaveSettings,
    handleSaveProjectSettings,
    handleSaveGlobalAgentSettings,
  }), [handleSaveSettings, handleSaveProjectSettings, handleSaveGlobalAgentSettings]);
}
