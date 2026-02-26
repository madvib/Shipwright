import { Dispatch, SetStateAction } from 'react';
import { SpecDocument, SpecInfo as SpecEntry } from '@/bindings';
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
      setSelectedSpec({
        ...entry,
        content: '',
      });
      return;
    }

    try {
      const latest = await getSpecCmd(entry.file_name);
      setSelectedSpec(latest);
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
      const created = await createSpecCmd(title, content);
      setSpecs((prev) => [
        ...prev,
        { file_name: created.file_name, title: created.title, path: created.path },
      ]);
      setSelectedSpec(created);
      await refreshActivity();
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
      const updated = await updateSpecCmd(fileName, content);
      setSpecs((prev) =>
        prev.map((entry) =>
          entry.file_name === updated.file_name
            ? { file_name: updated.file_name, title: updated.title, path: updated.path }
            : entry
        )
      );
      setSelectedSpec(updated);
      await refreshActivity();
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
      await deleteSpecCmd(fileName);
      setSpecs((prev) => prev.filter((entry) => entry.file_name !== fileName));
      setSelectedSpec(null);
      await refreshActivity();
    } catch (error) {
      setError(String(error));
    }
  };

  return {
    handleSelectSpec,
    handleCreateSpec,
    handleSaveSpec,
    handleDeleteSpec,
  };
}
