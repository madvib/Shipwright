import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
import { ReleaseInfo, ReleaseDocument } from '@/bindings';
import {
  createReleaseCmd,
  getReleaseCmd,
  ReleaseMetadataUpdate,
  updateReleaseCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseReleaseActionsParams {
  setReleases: Dispatch<SetStateAction<ReleaseInfo[]>>;
  setSelectedRelease: Dispatch<SetStateAction<ReleaseDocument | null>>;
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
      setSelectedRelease(entry as unknown as ReleaseDocument);
      return;
    }

    try {
      const result = await getReleaseCmd(entry.file_name);
      if (result.status === 'ok') {
        setSelectedRelease(result.data);
      } else {
        setError(String(result.error));
      }
    } catch (error) {
      setError(String(error));
    }
  }, [setSelectedRelease, setError]);

  const handleCreateRelease = useCallback(async (
    version: string,
    content: string,
    metadata?: ReleaseMetadataUpdate,
  ) => {
    if (!isTauriRuntime()) {
      setError('Release creation is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await createReleaseCmd(version, content, metadata);
      if (result.status === 'ok') {
        const created = result.data;
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

  const handleSaveRelease = useCallback(async (
    fileName: string,
    content: string,
    metadata?: ReleaseMetadataUpdate,
  ) => {
    if (!isTauriRuntime()) {
      setError('Saving releases is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await updateReleaseCmd(fileName, content, metadata);
      if (result.status === 'ok') {
        const updated = result.data;
        setReleases((prev) =>
          prev.map((entry) => (entry.id === updated.id ? updated : entry))
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
