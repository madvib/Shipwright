import { Dispatch, SetStateAction } from 'react';
import { ReleaseDocument, ReleaseInfo as ReleaseEntry } from '@/bindings';
import {
  createReleaseCmd,
  getReleaseCmd,
  updateReleaseCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseReleaseActionsParams {
  setReleases: Dispatch<SetStateAction<ReleaseEntry[]>>;
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
  const handleSelectRelease = async (entry: ReleaseEntry) => {
    if (!isTauriRuntime()) {
      setSelectedRelease({ ...entry, content: '' });
      return;
    }

    try {
      const latest = await getReleaseCmd(entry.file_name);
      setSelectedRelease(latest);
    } catch (error) {
      setError(String(error));
    }
  };

  const handleCreateRelease = async (version: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Release creation is only available in Tauri runtime.');
      return;
    }

    try {
      const created = await createReleaseCmd(version, content);
      setReleases((prev) => [
        ...prev,
        {
          file_name: created.file_name,
          version: created.version,
          status: created.status,
          path: created.path,
          updated: created.updated,
        },
      ]);
      setSelectedRelease(created);
      await refreshActivity();
    } catch (error) {
      setError(String(error));
      throw error;
    }
  };

  const handleSaveRelease = async (fileName: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Saving releases is only available in Tauri runtime.');
      return;
    }

    try {
      const updated = await updateReleaseCmd(fileName, content);
      setReleases((prev) =>
        prev.map((entry) =>
          entry.file_name === updated.file_name
            ? {
                file_name: updated.file_name,
                version: updated.version,
                status: updated.status,
                path: updated.path,
                updated: updated.updated,
              }
            : entry
        )
      );
      setSelectedRelease(updated);
      await refreshActivity();
    } catch (error) {
      setError(String(error));
    }
  };

  return {
    handleSelectRelease,
    handleCreateRelease,
    handleSaveRelease,
  };
}
