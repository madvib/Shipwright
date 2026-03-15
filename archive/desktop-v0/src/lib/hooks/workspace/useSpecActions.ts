import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
import {
  SpecInfo,
} from '@/lib/types/spec';
import {
  createSpecCmd,
  deleteSpecCmd,
  getSpecCmd,
  updateSpecCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseSpecActionsParams {
  setSpecs: Dispatch<SetStateAction<SpecInfo[]>>;
  setSelectedSpec: Dispatch<SetStateAction<SpecInfo | null>>;
  setError: Dispatch<SetStateAction<string | null>>;
  refreshActivity: () => Promise<void>;
}

export function useSpecActions({
  setSpecs,
  setSelectedSpec,
  setError,
  refreshActivity,
}: UseSpecActionsParams) {
  const handleSelectSpec = useCallback(async (entry: SpecInfo) => {
    if (!isTauriRuntime()) {
      setSelectedSpec(entry);
      return;
    }

    try {
      const result = await getSpecCmd(entry.file_name);
      if (result.status === 'ok') {
        setSelectedSpec(result.data);
      } else {
        setError(String(result.error));
      }
    } catch (error) {
      setError(String(error));
    }
  }, [setSelectedSpec, setError]);

  const handleCreateSpec = useCallback(async (title: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Spec creation is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await createSpecCmd(title, content);
      if (result.status === 'ok') {
        const created = result.data;
        setSpecs((prev) => [...prev, created]);
        setSelectedSpec(created);
        await refreshActivity();
      } else {
        setError(String(result.error));
        throw new Error(String(result.error));
      }
    } catch (error) {
      setError(String(error));
      throw error;
    }
  }, [setSpecs, setSelectedSpec, setError, refreshActivity]);

  const handleSaveSpec = useCallback(async (fileName: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Saving specs is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await updateSpecCmd(fileName, content);
      if (result.status === 'ok') {
        const updated = result.data;
        setSpecs((prev) =>
          prev.map((entry) => (entry.file_name === updated.file_name ? updated : entry))
        );
        setSelectedSpec(updated);
        await refreshActivity();
      } else {
        setError(String(result.error));
      }
    } catch (error) {
      setError(String(error));
    }
  }, [setSpecs, setSelectedSpec, setError, refreshActivity]);

  const handleDeleteSpec = useCallback(async (fileName: string) => {
    if (!isTauriRuntime()) {
      setError('Deleting specs is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await deleteSpecCmd(fileName);
      if (result.status === 'ok') {
        setSpecs((prev) => prev.filter((entry) => entry.file_name !== fileName));
        setSelectedSpec(null);
        await refreshActivity();
      } else {
        setError(String(result.error));
        throw new Error(String(result.error));
      }
    } catch (error) {
      setError(String(error));
      throw error;
    }
  }, [setSpecs, setSelectedSpec, setError, refreshActivity]);

  return useMemo(() => ({
    handleSelectSpec,
    handleCreateSpec,
    handleSaveSpec,
    handleDeleteSpec,
  }), [handleSelectSpec, handleCreateSpec, handleSaveSpec, handleDeleteSpec]);
}
