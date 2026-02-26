import { Dispatch, SetStateAction } from 'react';
import { AdrEntry } from '@/bindings';
import {
  createNewAdrCmd,
  deleteAdrCmd,
  getAdrCmd,
  updateAdrCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseAdrActionsParams {
  setAdrs: Dispatch<SetStateAction<AdrEntry[]>>;
  setSelectedAdr: Dispatch<SetStateAction<AdrEntry | null>>;
  setShowNewAdr: Dispatch<SetStateAction<boolean>>;
  setError: Dispatch<SetStateAction<string | null>>;
  refreshActivity: () => Promise<void>;
}

export function useAdrActions({
  setAdrs,
  setSelectedAdr,
  setShowNewAdr,
  setError,
  refreshActivity,
}: UseAdrActionsParams) {
  const handleCreateAdr = async (title: string, decision: string) => {
    if (!isTauriRuntime()) {
      setError('ADR creation is only available in Tauri runtime.');
      return;
    }

    try {
      const entry = await createNewAdrCmd(title, decision);
      setAdrs((prev) => [...prev, entry]);
      setShowNewAdr(false);
      await refreshActivity();
    } catch (error) {
      setError(String(error));
    }
  };

  const handleSelectAdr = async (entry: AdrEntry) => {
    if (!isTauriRuntime()) {
      setSelectedAdr(entry);
      return;
    }

    try {
      const latest = await getAdrCmd(entry.file_name);
      setSelectedAdr(latest);
    } catch {
      setSelectedAdr(entry);
    }
  };

  const handleSaveAdr = async (fileName: string, adr: AdrEntry['adr']) => {
    if (!isTauriRuntime()) {
      setError('Saving ADRs is only available in Tauri runtime.');
      return;
    }

    try {
      const updated = await updateAdrCmd(fileName, adr);
      setAdrs((prev) => prev.map((item) => (item.file_name === fileName ? updated : item)));
      setSelectedAdr(updated);
      await refreshActivity();
    } catch (error) {
      setError(String(error));
    }
  };

  const handleDeleteAdr = async (fileName: string) => {
    if (!isTauriRuntime()) {
      setError('Deleting ADRs is only available in Tauri runtime.');
      return;
    }

    try {
      await deleteAdrCmd(fileName);
      setAdrs((prev) => prev.filter((item) => item.file_name !== fileName));
      setSelectedAdr(null);
      await refreshActivity();
    } catch (error) {
      setError(String(error));
    }
  };

  return {
    handleCreateAdr,
    handleSelectAdr,
    handleSaveAdr,
    handleDeleteAdr,
  };
}
