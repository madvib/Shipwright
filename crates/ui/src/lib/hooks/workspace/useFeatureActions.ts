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
      const result = await getFeatureCmd(entry.file_name);
      if (result.status === 'ok') {
        setSelectedFeature(result.data);
      } else {
        setError(String(result.error));
      }
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
      const result = await createFeatureCmd(title, content, release, spec);
      if (result.status === 'ok') {
        const created = result.data;
        setFeatures((prev) => [
          ...prev,
          {
            file_name: created.file_name,
            title: created.title,
            status: created.status,
            release_id: created.release_id,
            spec_id: created.spec_id,
            branch: created.branch,
            description: created.description,
            path: created.path,
            updated: created.updated,
          },
        ]);
        setSelectedFeature(created);
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

  const handleSaveFeature = async (fileName: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Saving features is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await updateFeatureCmd(fileName, content);
      if (result.status === 'ok') {
        const updated = result.data;
        setFeatures((prev) =>
          prev.map((entry) =>
            entry.file_name === updated.file_name
              ? {
                file_name: updated.file_name,
                title: updated.title,
                status: updated.status,
                release_id: updated.release_id,
                spec_id: updated.spec_id,
                branch: updated.branch,
                description: updated.description,
                path: updated.path,
                updated: updated.updated,
              }
              : entry
          )
        );
        setSelectedFeature(updated);
        await refreshActivity();
      } else {
        setError(String(result.error));
      }
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
