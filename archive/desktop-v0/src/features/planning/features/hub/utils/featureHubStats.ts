import { FeatureInfo } from '@/bindings';
import { featureStatusFallbackReadiness } from '@/features/planning/common/hub/utils/featureMetrics';

export interface FeatureHubSummaryMetrics {
    total: number;
    implemented: number;
    blocking: number;
    unlinked: number;
    avgReadiness: number;
}

export function featureHubStats(features: FeatureInfo[]): FeatureHubSummaryMetrics {
    const total = features.length;
    const implemented = features.filter(f => f.status === 'implemented').length;
    const blocking = 0; // Fallback: we don't have individual checklist content in the list view
    const unlinked = features.filter(f => !f.release_id).length;
    const avgReadiness = total > 0
        ? Math.round(features.reduce((acc, f) => acc + featureStatusFallbackReadiness(f.status || 'draft'), 0) / total)
        : 0;

    return {
        total,
        implemented,
        blocking,
        unlinked,
        avgReadiness
    };
}
