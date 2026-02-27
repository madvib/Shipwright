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
  const handleCreateAdr = async (
    title: string,
    decision: string,
    options?: {
      status?: string;
      date?: string;
      spec?: string | null;
      tags?: string[];
    }
  ) => {
    if (!isTauriRuntime()) {
      setError('ADR creation is only available in Tauri runtime.');
      return;
    }

    try {
      let entry = await createNewAdrCmd(title, decision);
      const nextStatus = options?.status?.trim() || entry.adr.metadata.status;
      const nextDate = options?.date?.trim() || entry.adr.metadata.date;
      const nextSpec = options?.spec?.trim() ? options.spec.trim() : null;
      const nextTags = Array.from(new Set((options?.tags ?? []).map((tag) => tag.trim()).filter(Boolean)));
      const currentSpec = entry.adr.metadata.spec ?? null;
      const currentTags = entry.adr.metadata.tags ?? [];
      const metadataChanged =
        nextStatus !== entry.adr.metadata.status ||
        nextDate !== entry.adr.metadata.date ||
        nextSpec !== currentSpec ||
        nextTags.join('\n') !== currentTags.join('\n');

      if (metadataChanged) {
        entry = await updateAdrCmd(entry.file_name, {
          ...entry.adr,
          metadata: {
            ...entry.adr.metadata,
            status: nextStatus,
            date: nextDate,
            spec: nextSpec,
            tags: nextTags,
          },
        });
      }

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
