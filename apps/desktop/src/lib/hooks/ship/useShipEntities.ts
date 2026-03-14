import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
import { isTauriRuntime } from '../../platform/tauri/runtime';
import { listAdrs, listFeatures, listNotes, listReleases } from '../../platform/tauri/commands';
import { useShipAdrs } from './domain/useShipAdrs';
import { useShipReleases } from './domain/useShipReleases';
import { useShipFeatures } from './domain/useShipFeatures';
import { useShipNotes } from './domain/useShipNotes';

interface UseShipEntitiesParams {
    refreshActivity: () => Promise<void>;
    setError: Dispatch<SetStateAction<string | null>>;
}

export function useShipEntities({
    refreshActivity,
    setError,
}: UseShipEntitiesParams) {
    const adrs = useShipAdrs({ setError, refreshActivity });
    const releases = useShipReleases({ setError, refreshActivity });
    const features = useShipFeatures({ setError, refreshActivity });
    const notes = useShipNotes({ setError, refreshActivity });

    const loadShipData = useCallback(async () => {
        if (!isTauriRuntime()) {
            adrs.setAdrs([]);
            releases.setReleases([]);
            features.setFeatures([]);
            notes.setNotes([]);
            return;
        }

        try {
            const [adrList, releaseList, featureList, noteList] = await Promise.all([
                listAdrs().catch(() => []),
                listReleases().catch(() => []),
                listFeatures().catch(() => []),
                listNotes().catch(() => []),
            ]);
            adrs.setAdrs(adrList);
            releases.setReleases(releaseList);
            features.setFeatures(featureList);
            notes.setNotes(noteList);
        } catch (error) {
            console.error('Failed to load ship data', error);
        }
    }, [adrs, releases, features, notes]);

    const tagSuggestions = useMemo(() => {
        return Array.from(
            new Set([
                ...adrs.tagSuggestions,
            ])
        ).sort((a, b) => a.localeCompare(b));
    }, [adrs.tagSuggestions]);

    return useMemo(() => ({
        ...adrs,
        ...releases,
        ...features,
        ...notes,
        tagSuggestions,
        loadShipData,
    }), [adrs, releases, features, notes, tagSuggestions, loadShipData]);
}
