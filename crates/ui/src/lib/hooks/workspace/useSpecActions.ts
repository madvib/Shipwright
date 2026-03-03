import { Dispatch, SetStateAction } from 'react';
import {
  SpecDocument,
  SpecInfo as SpecEntry,
  stubSpecDocument,
} from '@/lib/types/spec';
import {
  createSpecCmd,
  deleteSpecCmd,
  getSpecCmd,
  updateSpecCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseSpecActionsParams {
  setSpecs: Dispatch<SetStateAction<SpecEntry[]>>;
  setSelectedSpec: Dispatch<SetStateAction<SpecDocument | null>>;
  setError: Dispatch<SetStateAction<string | null>>;
  refreshActivity: () => Promise<void>;
}

export function useSpecActions({
  setSpecs,
  setSelectedSpec,
  setError,
  refreshActivity,
}: UseSpecActionsParams) {
  const handleSelectSpec = async (entry: SpecEntry) => {
    if (!isTauriRuntime()) {
      setSelectedSpec(stubSpecDocument(entry, ''));
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
  };

  const handleCreateSpec = async (title: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Spec creation is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await createSpecCmd(title, content);
      if (result.status === 'ok') {
        const created = result.data;
        setSpecs((prev) => [
          ...prev,
          {
            id: created.id,
            file_name: created.file_name,
            title: created.title,
            path: created.path,
            status: created.status,
          },
        ]);
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
  };

  const handleSaveSpec = async (fileName: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Saving specs is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await updateSpecCmd(fileName, content);
      if (result.status === 'ok') {
        const updated = result.data;
        setSpecs((prev) =>
          prev.map((entry) =>
            entry.file_name === updated.file_name
              ? {
                id: updated.id,
                file_name: updated.file_name,
                title: updated.title,
                path: updated.path,
                status: updated.status,
              }
              : entry
          )
        );
        setSelectedSpec(updated);
        await refreshActivity();
      } else {
        setError(String(result.error));
      }
    } catch (error) {
      setError(String(error));
    }
  };

  const handleDeleteSpec = async (fileName: string) => {
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
  };

  return {
    handleSelectSpec,
    handleCreateSpec,
    handleSaveSpec,
    handleDeleteSpec,
  };
}
