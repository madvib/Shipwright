import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
import { FeatureInfo, FeatureDocument } from '@/bindings';
import {
  createFeatureCmd,
  getFeatureCmd,
  updateFeatureCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseFeatureActionsParams {
  setFeatures: Dispatch<SetStateAction<FeatureInfo[]>>;
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
  const handleSelectFeature = useCallback(async (entry: FeatureInfo) => {
    if (!isTauriRuntime()) {
      // In non-tauri, we can't fetch the document, so we just set the info
      // but the types will complain. This is a fallback case.
      setSelectedFeature(entry as unknown as FeatureDocument);
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
  }, [setSelectedFeature, setError]);

  const handleCreateFeature = useCallback(async (
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
        setFeatures((prev) => [...prev, created]);
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
  }, [setFeatures, setSelectedFeature, setError, refreshActivity]);

  const handleSaveFeature = useCallback(async (fileName: string, content: string) => {
    if (!isTauriRuntime()) {
      setError('Saving features is only available in Tauri runtime.');
      return;
    }

    try {
      const result = await updateFeatureCmd(fileName, content);
      if (result.status === 'ok') {
        const updated = result.data;
        setFeatures((prev) =>
          prev.map((entry: FeatureInfo) => (entry.file_name === updated.file_name ? updated : entry))
        );
        setSelectedFeature(updated);
        await refreshActivity();
      } else {
        setError(String(result.error));
      }
    } catch (error) {
      setError(String(error));
    }
  }, [setFeatures, setSelectedFeature, setError, refreshActivity]);

  return useMemo(() => ({
    handleSelectFeature,
    handleCreateFeature,
    handleSaveFeature,
  }), [handleSelectFeature, handleCreateFeature, handleSaveFeature]);
}
