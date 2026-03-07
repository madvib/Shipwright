import { useState, useEffect, useMemo } from 'react';
import {
    getCurrentBranchCmd,
    listWorkspacesCmd,
    listWorkspaceEditorsCmd,
    listModesCmd,
    listProvidersCmd,
    getActiveWorkspaceSessionCmd,
    listWorkspaceSessionsCmd,
    getWorkspaceProviderMatrixCmd,
    type WorkspaceSessionInfo,
    type WorkspaceProviderMatrix,
    type WorkspaceEditorInfo
} from '@/lib/platform/tauri/commands';
import { Workspace } from '@/bindings';
import { ModeConfig, ProviderInfo } from '@/bindings';
import { RuntimeWorkspace } from '@/lib/types/workspace';
import { WorkspaceRow, WorkspaceGroupBy } from './types';
import { WorkspaceGraphStatus, WorkspaceGraphRow } from '../components/WorkspaceLifecycleGraph';

const NO_LINK_VALUE = '__none__';

export function useWorkspaceState(workspaceUi: any, _ship: any) {
    const [branch, setBranch] = useState<string | null>(null);
    const [runtimeWorkspaces, setRuntimeWorkspaces] = useState<Workspace[]>([]);
    const [modeOptions, setModeOptions] = useState<ModeConfig[]>([]);
    const [availableEditors, setAvailableEditors] = useState<WorkspaceEditorInfo[]>([]);
    const [providerInfos, setProviderInfos] = useState<ProviderInfo[]>([]);
    const [selectedBranch, setSelectedBranch] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [searchQuery, setSearchQuery] = useState('');
    const [groupBy, setGroupBy] = useState<WorkspaceGroupBy>('type');
    const [activeSession, setActiveSession] = useState<WorkspaceSessionInfo | null>(null);
    const [recentSessions, setRecentSessions] = useState<WorkspaceSessionInfo[]>([]);
    const [providerMatrix, setProviderMatrix] = useState<WorkspaceProviderMatrix | null>(null);
    const [startingSession, setStartingSession] = useState(false);
    const [endingSession, setEndingSession] = useState(false);
    const [sessionGoalInput, setSessionGoalInput] = useState('');
    const [sessionSummaryInput, setSessionSummaryInput] = useState('');
    const [syncing, setSyncing] = useState(false);
    const [activating, setActivating] = useState(false);
    const [updatingLinks, setUpdatingLinks] = useState(false);
    const [linkFeatureId, setLinkFeatureId] = useState<string>(NO_LINK_VALUE);
    const [linkSpecId, setLinkSpecId] = useState<string>(NO_LINK_VALUE);

    const load = async () => {
        setLoading(true);
        setError(null);
        try {
            const currentBranch = await getCurrentBranchCmd();
            setBranch(currentBranch);
            const result = await listWorkspacesCmd();
            if (result.status === 'ok') {
                setRuntimeWorkspaces(result.data);
                setSelectedBranch((previous) => {
                    if (previous && result.data.some((item) => item.branch === previous)) {
                        return previous;
                    }
                    if (
                        currentBranch &&
                        result.data.some((item) => item.branch === currentBranch)
                    ) {
                        return currentBranch;
                    }
                    return result.data[0]?.branch ?? null;
                });
            }
            else setError(result.error || 'Failed to load workspaces.');
        } catch (err) { setError(String(err)); }
        finally { setLoading(false); }
    };

    const loadEditors = async () => {
        const result = await listWorkspaceEditorsCmd();
        if (result.status === 'ok') setAvailableEditors(result.data);
    };

    const loadModes = async () => {
        try {
            const modes = await listModesCmd();
            setModeOptions(modes);
        } catch {
            setModeOptions([]);
        }
    };

    const loadProviders = async () => {
        const result = await listProvidersCmd();
        if (result.status === 'ok') {
            setProviderInfos(result.data);
        }
    };

    const fetchSessionSnapshot = async (targetBranch: string) => {
        const [activeRes, listRes] = await Promise.all([
            getActiveWorkspaceSessionCmd(targetBranch),
            listWorkspaceSessionsCmd(targetBranch, 8),
        ]);
        return {
            active: activeRes.status === 'ok' ? activeRes.data : null,
            sessions: listRes.status === 'ok' ? listRes.data : [],
        };
    };

    const normalizeWorkspaceType = (type: string | null | undefined): WorkspaceGraphRow['workspaceType'] => {
        if (['feature', 'refactor', 'experiment', 'hotfix'].includes(type as any)) return type as any;
        return 'feature';
    };

    const normalizeWorkspaceStatus = (status: string | null | undefined): WorkspaceGraphStatus => {
        if (['planned', 'active', 'idle', 'review', 'merged', 'archived'].includes(status as any)) return status as any;
        return 'idle';
    };

    const rows = useMemo<WorkspaceRow[]>(() => {
        const statusRank: Record<WorkspaceGraphStatus, number> = { active: 0, review: 1, idle: 2, planned: 3, merged: 4, archived: 5 };
        return runtimeWorkspaces
            .map((w) => {
                const rw = w as RuntimeWorkspace;
                const s = normalizeWorkspaceStatus(rw.status ?? (w.branch === branch ? 'active' : 'idle'));
                return {
                    id: w.branch, branch: w.branch, workspaceType: normalizeWorkspaceType(rw.workspace_type),
                    featureId: w.feature_id ?? null, specId: w.spec_id ?? null, releaseId: rw.release_id ?? null,
                    activeMode: w.active_mode ?? null, providers: w.providers ?? [], resolvedAt: w.resolved_at,
                    isWorktree: w.is_worktree, worktreePath: w.worktree_path ?? null, lastActivatedAt: rw.last_activated_at ?? null,
                    contextHash: rw.context_hash ?? null,
                    configGeneration: rw.config_generation ?? 0,
                    compiledAt: rw.compiled_at ?? null,
                    compileError: rw.compile_error ?? null,
                    status: s,
                };
            })
            .sort((a, b) => {
                if (a.status !== b.status) return statusRank[a.status] - statusRank[b.status];
                return b.resolvedAt.localeCompare(a.resolvedAt);
            });
    }, [runtimeWorkspaces, branch]);

    const filteredRows = useMemo(() => {
        const query = searchQuery.trim().toLowerCase();
        if (!query) return rows;
        return rows.filter((r) => r.branch.toLowerCase().includes(query) || (r.featureId?.toLowerCase().includes(query)));
    }, [rows, searchQuery]);

    const detail = useMemo(() => filteredRows.find((r) => r.branch === selectedBranch) ?? null, [filteredRows, selectedBranch]);

    useEffect(() => { load(); loadEditors(); loadModes(); loadProviders(); }, []);

    useEffect(() => {
        if (!detail) {
            setLinkFeatureId(NO_LINK_VALUE);
            setLinkSpecId(NO_LINK_VALUE);
            return;
        }
        setLinkFeatureId(detail.featureId ?? NO_LINK_VALUE);
        setLinkSpecId(detail.specId ?? NO_LINK_VALUE);
    }, [detail?.branch, detail?.featureId, detail?.specId]);

    useEffect(() => {
        if (!detail) {
            setActiveSession(null); setRecentSessions([]); setProviderMatrix(null);
            return;
        }
        let cancelled = false;
        const loadContext = async () => {
            const snapshot = await fetchSessionSnapshot(detail.branch);
            if (cancelled) return;
            setActiveSession(snapshot.active); setRecentSessions(snapshot.sessions);

            const modeForMatrix = detail.activeMode ?? workspaceUi.activeModeId ?? null;
            const matrixRes = await getWorkspaceProviderMatrixCmd(detail.branch, modeForMatrix);
            if (!cancelled && matrixRes.status === 'ok') setProviderMatrix(matrixRes.data);
        };
        void loadContext();
        return () => { cancelled = true; };
    }, [detail?.branch, detail?.activeMode, detail?.configGeneration, workspaceUi.activeModeId]);

    return {
        branch, runtimeWorkspaces, modeOptions, availableEditors, selectedBranch, setSelectedBranch,
        loading, error, setError, searchQuery, setSearchQuery, groupBy, setGroupBy,
        activeSession, recentSessions, providerMatrix, startingSession, setStartingSession,
        endingSession, setEndingSession, sessionGoalInput, setSessionGoalInput,
        sessionSummaryInput, setSessionSummaryInput, syncing, setSyncing, activating, setActivating,
        updatingLinks, setUpdatingLinks, linkFeatureId, setLinkFeatureId, linkSpecId, setLinkSpecId,
        rows, filteredRows, detail, providerInfos, load, fetchSessionSnapshot
    };
}
