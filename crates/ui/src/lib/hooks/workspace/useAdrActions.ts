import { Dispatch, SetStateAction } from 'react';
import { ADR, AdrEntry, AdrStatus } from '@/bindings';
import {
  createNewAdrCmd,
  deleteAdrCmd,
  getAdrCmd,
  moveAdrCmd,
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

function toAdrStatus(value?: string): AdrStatus | null {
  const normalized = (value ?? '').trim().toLowerCase();
  if (
    normalized === 'proposed' ||
    normalized === 'accepted' ||
    normalized === 'rejected' ||
    normalized === 'superseded' ||
    normalized === 'deprecated'
  ) {
    return normalized;
  }
  return null;
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
    context: string,
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
      let entry = await createNewAdrCmd(title, context, decision);
      const nextDate = options?.date?.trim() || entry.adr.metadata.date;
      const nextSpec = options?.spec?.trim() ? options.spec.trim() : null;
      const nextTags = Array.from(
        new Set((options?.tags ?? []).map((tag) => tag.trim()).filter(Boolean))
      );
      const currentSpec = entry.adr.metadata.spec_id ?? null;
      const currentTags = entry.adr.metadata.tags ?? [];
      const metadataChanged =
        nextDate !== entry.adr.metadata.date ||
        nextSpec !== currentSpec ||
        nextTags.join('\n') !== currentTags.join('\n');

      if (metadataChanged) {
        entry = await updateAdrCmd(entry.id, {
          ...entry.adr,
          metadata: {
            ...entry.adr.metadata,
            date: nextDate,
            spec_id: nextSpec,
            tags: nextTags,
          },
        });
      }

      const requestedStatus = toAdrStatus(options?.status);
      if (requestedStatus && requestedStatus !== entry.status) {
        entry = await moveAdrCmd(entry.id, requestedStatus);
      }

      setAdrs((prev) => [...prev, entry]);
      setShowNewAdr(false);
      await refreshActivity();
      return entry;
    } catch (error) {
      setError(String(error));
      return;
    }
  };

  const handleSelectAdr = async (entry: AdrEntry) => {
    if (!isTauriRuntime()) {
      setSelectedAdr(entry);
      return;
    }

    try {
      const latest = await getAdrCmd(entry.id);
      setSelectedAdr(latest);
    } catch {
      setSelectedAdr(entry);
    }
  };

  const handleSaveAdr = async (id: string, adr: ADR) => {
    if (!isTauriRuntime()) {
      setError('Saving ADRs is only available in Tauri runtime.');
      return;
    }

    try {
      const updated = await updateAdrCmd(id, adr);
      setAdrs((prev) => prev.map((item) => (item.id === id ? updated : item)));
      setSelectedAdr(updated);
      await refreshActivity();
    } catch (error) {
      setError(String(error));
    }
  };

  const handleMoveAdr = async (id: string, newStatus: AdrStatus) => {
    if (!isTauriRuntime()) {
      setError('Moving ADRs is only available in Tauri runtime.');
      return;
    }

    try {
      const updated = await moveAdrCmd(id, newStatus);
      setAdrs((prev) => prev.map((item) => (item.id === id ? updated : item)));
      setSelectedAdr((current) => (current && current.id === id ? updated : current));
      await refreshActivity();
    } catch (error) {
      setError(String(error));
    }
  };

  const handleDeleteAdr = async (id: string) => {
    if (!isTauriRuntime()) {
      setError('Deleting ADRs is only available in Tauri runtime.');
      return;
    }

    try {
      await deleteAdrCmd(id);
      setAdrs((prev) => prev.filter((item) => item.id !== id));
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
    handleMoveAdr,
    handleDeleteAdr,
  };
}
