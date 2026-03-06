import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
import { isTauriRuntime } from '../../platform/tauri/runtime';
import { listAdrs, listFeatures, listNotes, listReleases, listSpecs } from '../../platform/tauri/commands';
import { useShipAdrs } from './domain/useShipAdrs';
import { useShipSpecs } from './domain/useShipSpecs';
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
    const specs = useShipSpecs({ setError, refreshActivity });
    const releases = useShipReleases({ setError, refreshActivity });
    const features = useShipFeatures({ setError, refreshActivity });
    const notes = useShipNotes({ setError, refreshActivity });

    const loadShipData = useCallback(async () => {
        if (!isTauriRuntime()) {
            adrs.setAdrs([]);
            specs.setSpecs([]);
            releases.setReleases([]);
            features.setFeatures([]);
            notes.setNotes([]);
            return;
        }

        try {
            const [adrList, specList, releaseList, featureList, noteList] = await Promise.all([
                listAdrs().catch(() => []),
                listSpecs().catch(() => []),
                listReleases().catch(() => []),
                listFeatures().catch(() => []),
                listNotes().catch(() => []),
            ]);
            adrs.setAdrs(adrList);
            specs.setSpecs(specList);
            releases.setReleases(releaseList);
            features.setFeatures(featureList);
            notes.setNotes(noteList);
        } catch (error) {
            console.error('Failed to load ship data', error);
        }
    }, [adrs, specs, releases, features, notes]);

    const tagSuggestions = useMemo(() => {
        return Array.from(
            new Set([
                ...adrs.tagSuggestions,
            ])
        ).sort((a, b) => a.localeCompare(b));
    }, [adrs.tagSuggestions]);

    return useMemo(() => ({
        ...adrs,
        ...specs,
        ...releases,
        ...features,
        ...notes,
        tagSuggestions,
        loadShipData,
    }), [adrs, specs, releases, features, notes, tagSuggestions, loadShipData]);
}
