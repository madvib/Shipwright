import { Dispatch, SetStateAction, useState, useMemo } from 'react';
import { ReleaseInfo, ReleaseDocument } from '@/bindings';
import { useReleaseActions } from '../../workspace/useReleaseActions';

interface UseShipReleasesParams {
    setError: Dispatch<SetStateAction<string | null>>;
    refreshActivity: () => Promise<void>;
}

export function useShipReleases({
    setError,
    refreshActivity,
}: UseShipReleasesParams) {
    const [releases, setReleases] = useState<ReleaseInfo[]>([]);
    const [selectedRelease, setSelectedRelease] = useState<ReleaseDocument | null>(null);

    const actions = useReleaseActions({
        setReleases,
        setSelectedRelease,
        setError,
        refreshActivity,
    });

    return useMemo(() => ({
        releases,
        setReleases,
        selectedRelease,
        setSelectedRelease,
        ...actions,
    }), [releases, selectedRelease, actions]);
}
