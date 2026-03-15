import { Dispatch, SetStateAction, useState, useMemo } from 'react';
import { FeatureInfo, FeatureDocument } from '@/bindings';
import { useFeatureActions } from '../../workspace/useFeatureActions';

interface UseShipFeaturesParams {
    setError: Dispatch<SetStateAction<string | null>>;
    refreshActivity: () => Promise<void>;
}

export function useShipFeatures({
    setError,
    refreshActivity,
}: UseShipFeaturesParams) {
    const [features, setFeatures] = useState<FeatureInfo[]>([]);
    const [selectedFeature, setSelectedFeature] = useState<FeatureDocument | null>(null);

    const actions = useFeatureActions({
        setFeatures,
        setSelectedFeature,
        setError,
        refreshActivity,
    });

    return useMemo(() => ({
        features,
        setFeatures,
        selectedFeature,
        setSelectedFeature,
        ...actions,
    }), [features, selectedFeature, actions]);
}
