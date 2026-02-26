import { Dispatch, SetStateAction } from 'react';
import { FeatureDocument, FeatureInfo as FeatureEntry } from '@/bindings';
import {
  createFeatureCmd,
  getFeatureCmd,
  updateFeatureCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseFeatureActionsParams {
  setFeatures: Dispatch<SetStateAction<FeatureEntry[]>>;
  setSelectedFeature: Dispatch<SetStateAction<FeatureDocument | null>>;
  setError: Dispatch<SetStateAction<string | null>>;
  refreshActivity: () => Promise<void>;
}

export function useFeatureActions({
  setFeatures,
  setSelectedFeature,
  setError,
  refreshActivity,
}: UseFeatureActionsParams) {
  const handleSelectFeature = async (entry: FeatureEntry) => {
    if (!isTauriRuntime()) {
      setSelectedFeature({ ...entry, content: '' });
      return;
    }

    try {
      const latest = await getFeatureCmd(entry.file_name);
      setSelectedFeature(latest);
    } catch (error) {
      setError(String(error));
    }
  };

  const handleCreateFeature = async (
    title: string,
    content: string,
    release?: string | null,
    spec?: string | null
  ) => {
    if (!isTauriRuntime()) {
      setError('Feature creation is only available in Tauri runtime.');
      return;
    }

    try {
      const created = await createFeatureCmd(title, content, release, spec);
      setFeatures((prev) => [
        ...prev,
        {
          file_name: created.file_name,
          title: created.title,
          status: created.status,
          release: created.release,
          path: created.path,
          updated: created.updated,
        },
      ]);
      setSelectedFeature(created);
      await refreshActivity();
    } catch (error) {
      setError(String(error));
      throw error;
    }
  };

  const handleSaveFeature = async (fileName: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Saving features is only available in Tauri runtime.');
      return;
    }

    try {
      const updated = await updateFeatureCmd(fileName, content);
      setFeatures((prev) =>
        prev.map((entry) =>
          entry.file_name === updated.file_name
            ? {
                file_name: updated.file_name,
                title: updated.title,
                status: updated.status,
                release: updated.release,
                path: updated.path,
                updated: updated.updated,
              }
            : entry
        )
      );
      setSelectedFeature(updated);
      await refreshActivity();
    } catch (error) {
      setError(String(error));
    }
  };

  return {
    handleSelectFeature,
    handleCreateFeature,
    handleSaveFeature,
  };
}
