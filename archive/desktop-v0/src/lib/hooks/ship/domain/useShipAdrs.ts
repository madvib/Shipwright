import { Dispatch, SetStateAction, useState, useMemo } from 'react';
import { AdrEntry } from '@/bindings';
import { useAdrActions } from '../../workspace/useAdrActions';

interface UseShipAdrsParams {
    setError: Dispatch<SetStateAction<string | null>>;
    refreshActivity: () => Promise<void>;
}

export function useShipAdrs({
    setError,
    refreshActivity,
}: UseShipAdrsParams) {
    const [adrs, setAdrs] = useState<AdrEntry[]>([]);
    const [selectedAdr, setSelectedAdr] = useState<AdrEntry | null>(null);
    const [showNewAdr, setShowNewAdr] = useState(false);

    const actions = useAdrActions({
        setAdrs,
        setSelectedAdr,
        setShowNewAdr,
        setError,
        refreshActivity,
    });

    const tagSuggestions = useMemo(() => {
        return Array.from(
            new Set(
                adrs
                    .flatMap((entry) => entry.adr.metadata.tags ?? [])
                    .filter((tag): tag is string => typeof tag === 'string' && tag.trim().length > 0)
            )
        ).sort((a, b) => a.localeCompare(b));
    }, [adrs]);

    const adrSuggestions = useMemo(() => {
        return adrs
            .map((entry) => ({
                id: entry.adr.metadata.id,
                title: entry.adr.metadata.title
            }))
            .filter((s) => s.id.trim().length > 0)
            .sort((a, b) => a.title.localeCompare(b.title));
    }, [adrs]);

    return useMemo(() => ({
        adrs,
        setAdrs,
        selectedAdr,
        setSelectedAdr,
        showNewAdr,
        setShowNewAdr,
        tagSuggestions,
        adrSuggestions,
        ...actions,
    }), [adrs, selectedAdr, showNewAdr, tagSuggestions, adrSuggestions, actions]);
}
