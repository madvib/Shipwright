import { useEffect, useState } from 'react';
import { FeatureInfo as FeatureEntry } from '@/bindings';
import { getFeatureCmd } from '@/lib/platform/tauri/commands';
import { isTauriRuntime } from '@/lib/platform/tauri/runtime';
import {
  deriveFeatureChecklistMetrics,
  FeatureChecklistMetrics,
} from '@/features/planning/common/hub/utils/featureMetrics';

interface UseFeatureChecklistMetricsResult {
  metricsByFile: Record<string, FeatureChecklistMetrics>;
  loading: boolean;
}

export function useFeatureChecklistMetrics(
  features: FeatureEntry[]
): UseFeatureChecklistMetricsResult {
  const [metricsByFile, setMetricsByFile] = useState<Record<string, FeatureChecklistMetrics>>({});
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;

    if (!isTauriRuntime() || features.length === 0) {
      setMetricsByFile({});
      setLoading(false);
      return () => {
        cancelled = true;
      };
    }

    const loadFeatureMetrics = async () => {
      setLoading(true);
      const pairs = await Promise.all(
        features.map(async (feature) => {
          try {
            const result = await getFeatureCmd(feature.file_name);
            if (result.status !== 'ok') {
              return [feature.file_name, null] as const;
            }
            return [
              feature.file_name,
              deriveFeatureChecklistMetrics(result.data.content, feature.status),
            ] as const;
          } catch {
            return [feature.file_name, null] as const;
          }
        })
      );

      if (cancelled) return;
      const next: Record<string, FeatureChecklistMetrics> = {};
      for (const [fileName, metrics] of pairs) {
        if (metrics) {
          next[fileName] = metrics;
        }
      }
      setMetricsByFile(next);
      setLoading(false);
    };

    void loadFeatureMetrics();

    return () => {
      cancelled = true;
    };
  }, [features]);

  return {
    metricsByFile,
    loading,
  };
}
