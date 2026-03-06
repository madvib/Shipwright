import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
import { isTauriRuntime } from '../../platform/tauri/runtime';
import { listAdrs, listFeatures, listIssues, listNotes, listReleases, listSpecs } from '../../platform/tauri/commands';
import { useShipIssues } from './domain/useShipIssues';
import { useShipAdrs } from './domain/useShipAdrs';
import { useShipSpecs } from './domain/useShipSpecs';
import { useShipReleases } from './domain/useShipReleases';
import { useShipFeatures } from './domain/useShipFeatures';
import { useShipNotes } from './domain/useShipNotes';

interface UseShipEntitiesParams {
    refreshActivity: () => Promise<void>;
    refreshProjectInfo: () => Promise<void>;
    setError: Dispatch<SetStateAction<string | null>>;
}

export function useShipEntities({
    refreshActivity,
    refreshProjectInfo,
    setError,
}: UseShipEntitiesParams) {
    const issues = useShipIssues({ setError, refreshActivity, refreshProjectInfo });
    const adrs = useShipAdrs({ setError, refreshActivity });
    const specs = useShipSpecs({ setError, refreshActivity });
    const releases = useShipReleases({ setError, refreshActivity });
    const features = useShipFeatures({ setError, refreshActivity });
    const notes = useShipNotes({ setError, refreshActivity });

    const loadShipData = useCallback(async () => {
        if (!isTauriRuntime()) {
            issues.setIssues([]);
            adrs.setAdrs([]);
            specs.setSpecs([]);
            releases.setReleases([]);
            features.setFeatures([]);
            notes.setNotes([]);
            return;
        }

        try {
            const [issueList, adrList, specList, releaseList, featureList, noteList] = await Promise.all([
                listIssues().catch(() => []),
                listAdrs().catch(() => []),
                listSpecs().catch(() => []),
                listReleases().catch(() => []),
                listFeatures().catch(() => []),
                listNotes().catch(() => []),
            ]);
            issues.setIssues(issueList);
            adrs.setAdrs(adrList);
            specs.setSpecs(specList);
            releases.setReleases(releaseList);
            features.setFeatures(featureList);
            notes.setNotes(noteList);
        } catch (error) {
            console.error('Failed to load ship data', error);
        }
    }, [issues, adrs, specs, releases, features, notes]);

    const tagSuggestions = useMemo(() => {
        return Array.from(
            new Set([
                ...issues.tagSuggestions,
                ...adrs.tagSuggestions,
            ])
        ).sort((a, b) => a.localeCompare(b));
    }, [issues.tagSuggestions, adrs.tagSuggestions]);

    return useMemo(() => ({
        ...issues,
        ...adrs,
        ...specs,
        ...releases,
        ...features,
        ...notes,
        tagSuggestions,
        loadShipData,
    }), [issues, adrs, specs, releases, features, notes, tagSuggestions, loadShipData]);
}
