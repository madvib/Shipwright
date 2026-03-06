import { Dispatch, SetStateAction, useState, useMemo } from 'react';
import { IssueEntry } from '@/bindings';
import { useIssueActions } from '../../workspace/useIssueActions';

interface UseShipIssuesParams {
    setError: Dispatch<SetStateAction<string | null>>;
    refreshActivity: () => Promise<void>;
    refreshProjectInfo: () => Promise<void>;
}

export function useShipIssues({
    setError,
    refreshActivity,
    refreshProjectInfo,
}: UseShipIssuesParams) {
    const [issues, setIssues] = useState<IssueEntry[]>([]);
    const [selectedIssue, setSelectedIssue] = useState<IssueEntry | null>(null);
    const [showNewIssue, setShowNewIssue] = useState(false);

    const actions = useIssueActions({
        issues,
        setIssues,
        setSelectedIssue,
        setShowNewIssue,
        setError,
        refreshActivity,
        refreshProjectInfo,
    });

    const tagSuggestions = useMemo(() => {
        return Array.from(
            new Set(
                issues
                    .flatMap((entry) => entry.issue?.tags ?? [])
                    .filter((tag): tag is string => typeof tag === 'string' && tag.trim().length > 0)
            )
        ).sort((a, b) => a.localeCompare(b));
    }, [issues]);

    const issueFileSuggestions = useMemo(() => {
        return issues
            .map((entry) => entry.file_name)
            .filter((value) => value.trim().length > 0)
            .sort((a, b) => a.localeCompare(b));
    }, [issues]);

    return useMemo(() => ({
        issues,
        setIssues,
        selectedIssue,
        setSelectedIssue,
        showNewIssue,
        setShowNewIssue,
        tagSuggestions,
        issueFileSuggestions,
        ...actions,
    }), [issues, selectedIssue, showNewIssue, tagSuggestions, issueFileSuggestions, actions]);
}
