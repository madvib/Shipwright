import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
import { ReleaseInfo } from '@/bindings';
import {
  createReleaseCmd,
  getReleaseCmd,
  updateReleaseCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseReleaseActionsParams {
  setReleases: Dispatch<SetStateAction<ReleaseInfo[]>>;
  setSelectedRelease: Dispatch<SetStateAction<ReleaseInfo | null>>;
  setError: Dispatch<SetStateAction<string | null>>;
  refreshActivity: () => Promise<void>;
}

export function useReleaseActions({
  setReleases,
  setSelectedRelease,
  setError,
  refreshActivity,
}: UseReleaseActionsParams) {
  const handleSelectRelease = useCallback(async (entry: ReleaseInfo) => {
    if (!isTauriRuntime()) {
      setSelectedRelease(entry);
      return;
    }

    try {
      const result = await getReleaseCmd(entry.file_name);
      if (result.status === 'ok') {
        setSelectedRelease(result.data as unknown as ReleaseInfo);
      } else {
        setError(String(result.error));
      }
    } catch (error) {
      setError(String(error));
    }
  }, [setSelectedRelease, setError]);

  const handleCreateRelease = useCallback(async (version: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Release creation is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await createReleaseCmd(version, content);
      if (result.status === 'ok') {
        const created = result.data as unknown as ReleaseInfo;
        setReleases((prev) => [...prev, created]);
        setSelectedRelease(created);
        await refreshActivity();
      } else {
        setError(String(result.error));
        throw new Error(String(result.error));
      }
    } catch (error) {
      setError(String(error));
      throw error;
    }
  }, [setReleases, setSelectedRelease, setError, refreshActivity]);

  const handleSaveRelease = useCallback(async (fileName: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Saving releases is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await updateReleaseCmd(fileName, content);
      if (result.status === 'ok') {
        const updated = result.data as unknown as ReleaseInfo;
        setReleases((prev) =>
          prev.map((entry) => (entry.file_name === updated.file_name ? updated : entry))
        );
        setSelectedRelease(updated);
        await refreshActivity();
      } else {
        setError(String(result.error));
      }
    } catch (error) {
      setError(String(error));
    }
  }, [setReleases, setSelectedRelease, setError, refreshActivity]);

  return useMemo(() => ({
    handleSelectRelease,
    handleCreateRelease,
    handleSaveRelease,
  }), [handleSelectRelease, handleCreateRelease, handleSaveRelease]);
}
