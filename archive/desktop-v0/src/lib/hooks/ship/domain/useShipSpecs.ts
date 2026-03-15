import { Dispatch, SetStateAction, useState, useMemo } from 'react';
import { SpecInfo as SpecEntry } from '@/lib/types/spec';
import { useSpecActions } from '../../workspace/useSpecActions';

interface UseShipSpecsParams {
    setError: Dispatch<SetStateAction<string | null>>;
    refreshActivity: () => Promise<void>;
}

export function useShipSpecs({
    setError,
    refreshActivity,
}: UseShipSpecsParams) {
    const [specs, setSpecs] = useState<SpecEntry[]>([]);
    const [selectedSpec, setSelectedSpec] = useState<SpecEntry | null>(null);

    const actions = useSpecActions({
        setSpecs,
        setSelectedSpec,
        setError,
        refreshActivity,
    });

    const specSuggestions = useMemo(() => {
        return specs
            .map((entry) => ({
                id: entry.file_name,
                title: entry.spec.metadata.title
            }))
            .filter((s) => s.id.trim().length > 0)
            .sort((a, b) => a.title.localeCompare(b.title));
    }, [specs]);

    return useMemo(() => ({
        specs,
        setSpecs,
        selectedSpec,
        setSelectedSpec,
        specSuggestions,
        ...actions,
    }), [specs, selectedSpec, specSuggestions, actions]);
}
