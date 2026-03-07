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

const METRICS_CACHE = new Map<string, FeatureChecklistMetrics>();
const MAX_CONCURRENT_LOADS = 6;

function cacheKey(feature: FeatureEntry): string {
  return `${feature.file_name}::${feature.updated}::${feature.status ?? ''}`;
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
      const pairs: Array<readonly [string, FeatureChecklistMetrics | null]> = [];
      const pending: FeatureEntry[] = [];

      for (const feature of features) {
        const cached = METRICS_CACHE.get(cacheKey(feature));
        if (cached) {
          pairs.push([feature.file_name, cached] as const);
        } else {
          pending.push(feature);
        }
      }

      setLoading(pending.length > 0);

      for (let i = 0; i < pending.length; i += MAX_CONCURRENT_LOADS) {
        const chunk = pending.slice(i, i + MAX_CONCURRENT_LOADS);
        const chunkPairs = await Promise.all(
          chunk.map(async (feature) => {
            try {
              const result = await getFeatureCmd(feature.file_name);
              if (result.status !== 'ok') {
                return [feature.file_name, null] as const;
              }
              const metrics = deriveFeatureChecklistMetrics(result.data.content, feature.status);
              METRICS_CACHE.set(cacheKey(feature), metrics);
              return [feature.file_name, metrics] as const;
            } catch {
              return [feature.file_name, null] as const;
            }
          })
        );
        pairs.push(...chunkPairs);
      }

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
