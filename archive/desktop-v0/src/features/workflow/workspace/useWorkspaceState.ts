import { useState, useEffect, useMemo } from 'react';
import {
  getCurrentBranchCmd,
  listWorkspacesCmd,
  listWorkspaceEditorsCmd,
  listProvidersCmd,
  listGitBranchesCmd,
  getActiveWorkspaceSessionCmd,
  listWorkspaceSessionsCmd,
  getWorkspaceProviderMatrixCmd,
  listWorkspaceChangesCmd,
  getWorkspaceGitStatusCmd,
  getBranchDetailCmd,
  getBranchFileDiffCmd,
  type WorkspaceSessionInfo,
  type WorkspaceProviderMatrix,
  type WorkspaceEditorInfo,
  type WorkspaceFileChange,
  type WorkspaceGitStatusSummary,
  type GitBranchInfo,
  type BranchDetailSummary,
} from '@/lib/platform/tauri/commands';
import { ProviderInfo, Workspace } from '@/bindings';
import { RuntimeWorkspace } from '@/lib/types/workspace';
import { WorkspaceRow, WorkspaceGroupBy } from './types';
import { WorkspaceGraphStatus, WorkspaceGraphRow } from '../components/WorkspaceLifecycleGraph';

const SESSION_SCAN_LIMIT = 200;

export function useWorkspaceState(workspaceUi: any, _ship: any) {
  const [branch, setBranch] = useState<string | null>(null);
  const [runtimeWorkspaces, setRuntimeWorkspaces] = useState<Workspace[]>([]);
  const [availableEditors, setAvailableEditors] = useState<WorkspaceEditorInfo[]>([]);
  const [providerInfos, setProviderInfos] = useState<ProviderInfo[]>([]);
  const [gitBranches, setGitBranches] = useState<GitBranchInfo[]>([]);
  const [activeSessionBranches, setActiveSessionBranches] = useState<string[]>([]);
  const [selectedBranch, setSelectedBranch] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [groupBy, setGroupBy] = useState<WorkspaceGroupBy>('type');
  const [activeSession, setActiveSession] = useState<WorkspaceSessionInfo | null>(null);
  const [recentSessions, setRecentSessions] = useState<WorkspaceSessionInfo[]>([]);
  const [providerMatrix, setProviderMatrix] = useState<WorkspaceProviderMatrix | null>(null);
  const [workspaceChanges, setWorkspaceChanges] = useState<WorkspaceFileChange[]>([]);
  const [workspaceGitSummary, setWorkspaceGitSummary] = useState<WorkspaceGitStatusSummary | null>(null);
  const [branchDetail, setBranchDetail] = useState<BranchDetailSummary | null>(null);
  const [branchDiffPath, setBranchDiffPath] = useState<string | null>(null);
  const [branchFileDiff, setBranchFileDiff] = useState('');
  const [loadingBranchDiff, setLoadingBranchDiff] = useState(false);
  const [startingSession, setStartingSession] = useState(false);
  const [endingSession, setEndingSession] = useState(false);
  const [sessionGoalInput, setSessionGoalInput] = useState('');
  const [sessionSummaryInput, setSessionSummaryInput] = useState('');
  const [syncing, setSyncing] = useState(false);
  const [updatingLinks, setUpdatingLinks] = useState(false);
  const [linkFeatureId, setLinkFeatureId] = useState<string | null>(null);
  const [linkReleaseId, setLinkReleaseId] = useState<string | null>(null);

  const load = async () => {
    setLoading(true);
    setError(null);

    try {
      const [
        currentBranch,
        workspacesResult,
        gitBranchesResult,
        sessionsResult,
      ] = await Promise.all([
        getCurrentBranchCmd(),
        listWorkspacesCmd(),
        listGitBranchesCmd(),
        listWorkspaceSessionsCmd(null, SESSION_SCAN_LIMIT),
      ]);

      setBranch(currentBranch);

      if (workspacesResult.status === 'ok') {
        setRuntimeWorkspaces(workspacesResult.data);
        setSelectedBranch((previous) => {
          if (previous) return previous;
          if (currentBranch) return currentBranch;
          return workspacesResult.data[0]?.branch ?? null;
        });
      } else {
        setRuntimeWorkspaces([]);
        setSelectedBranch(null);
        setError(workspacesResult.error || 'Failed to load workspaces.');
      }

      if (gitBranchesResult.status === 'ok') {
        setGitBranches(gitBranchesResult.data);
      } else {
        setGitBranches([]);
      }

      if (sessionsResult.status === 'ok') {
        const activeBranches = Array.from(
          new Set(
            sessionsResult.data
              .filter((session) => session.status === 'active')
              .map((session) => session.workspace_branch)
              .filter(Boolean),
          ),
        );
        setActiveSessionBranches(activeBranches);
      } else {
        setActiveSessionBranches([]);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const loadEditors = async () => {
    const result = await listWorkspaceEditorsCmd();
    if (result.status === 'ok') {
      setAvailableEditors(result.data);
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
    if (!type) return 'feature';
    const normalized = type.trim().toLowerCase();
    if (['feature', 'patch', 'service'].includes(normalized)) {
      return normalized as WorkspaceGraphRow['workspaceType'];
    }
    return 'feature';
  };

  const normalizeWorkspaceStatus = (status: string | null | undefined): WorkspaceGraphStatus => {
    if (status === 'active' || status === 'archived') return status;
    return 'archived';
  };

  const rows = useMemo<WorkspaceRow[]>(() => {
    const statusRank: Record<WorkspaceGraphStatus, number> = { active: 0, archived: 1 };
    return runtimeWorkspaces
      .map((workspace) => {
        const runtimeWorkspace = workspace as RuntimeWorkspace;
        const normalizedStatus = normalizeWorkspaceStatus(
          runtimeWorkspace.status ?? (workspace.branch === branch ? 'active' : 'archived'),
        );

        return {
          id: workspace.branch,
          branch: workspace.branch,
          workspaceType: normalizeWorkspaceType(runtimeWorkspace.workspace_type),
          featureId: workspace.feature_id ?? null,
          releaseId: runtimeWorkspace.release_id ?? null,
          environmentId: runtimeWorkspace.environment_id ?? null,
          activeMode: workspace.active_mode ?? null,
          providers: workspace.providers ?? [],
          mcpServers: runtimeWorkspace.mcp_servers ?? [],
          skills: runtimeWorkspace.skills ?? [],
          resolvedAt: workspace.resolved_at,
          isWorktree: workspace.is_worktree,
          worktreePath: workspace.worktree_path ?? null,
          lastActivatedAt: runtimeWorkspace.last_activated_at ?? null,
          contextHash: runtimeWorkspace.context_hash ?? null,
          configGeneration: runtimeWorkspace.config_generation ?? 0,
          compiledAt: runtimeWorkspace.compiled_at ?? null,
          compileError: runtimeWorkspace.compile_error ?? null,
          status: normalizedStatus,
        };
      })
      .sort((left, right) => {
        if (left.status !== right.status) return statusRank[left.status] - statusRank[right.status];
        return right.resolvedAt.localeCompare(left.resolvedAt);
      });
  }, [runtimeWorkspaces, branch]);

  const filteredRows = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    if (!query) return rows;
    return rows.filter((row) => {
      if (row.branch.toLowerCase().includes(query)) return true;
      if (row.featureId?.toLowerCase().includes(query)) return true;
      if (row.releaseId?.toLowerCase().includes(query)) return true;
      return false;
    });
  }, [rows, searchQuery]);

  const detail = useMemo(
    () => rows.find((row) => row.branch === selectedBranch) ?? null,
    [rows, selectedBranch],
  );

  useEffect(() => {
    void load();
    void loadEditors();
    void loadProviders();
  }, []);

  useEffect(() => {
    if (!detail) {
      setLinkFeatureId(null);
      setLinkReleaseId(null);
      return;
    }

    setLinkFeatureId(detail.featureId ?? null);
    setLinkReleaseId(detail.releaseId ?? null);
  }, [detail?.branch, detail?.featureId, detail?.releaseId]);

  useEffect(() => {
    let cancelled = false;

    if (!detail) {
      setActiveSession(null);
      setRecentSessions([]);
      setProviderMatrix(null);
      setWorkspaceChanges([]);
      setWorkspaceGitSummary(null);

      if (!selectedBranch) {
        setBranchDetail(null);
        setBranchDiffPath(null);
        setBranchFileDiff('');
        setLoadingBranchDiff(false);
        return;
      }

      const loadBranchContext = async () => {
        const branchRes = await getBranchDetailCmd(selectedBranch);
        if (cancelled) return;

        if (branchRes.status === 'ok') {
          setBranchDetail(branchRes.data);
          setBranchDiffPath((current) => {
            if (current && branchRes.data.changes.some((change) => change.path === current)) {
              return current;
            }
            return branchRes.data.changes[0]?.path ?? null;
          });
        } else {
          setBranchDetail(null);
          setBranchDiffPath(null);
          setBranchFileDiff('');
        }
      };

      void loadBranchContext();
      return () => {
        cancelled = true;
      };
    }


    const loadContext = async () => {
      const modeForMatrix = detail.activeMode ?? workspaceUi.activeModeId ?? null;
      const [snapshot, matrixRes, changesRes, gitSummaryRes, branchRes] = await Promise.all([
        fetchSessionSnapshot(detail.branch),
        getWorkspaceProviderMatrixCmd(detail.branch, modeForMatrix),
        listWorkspaceChangesCmd(detail.branch),
        getWorkspaceGitStatusCmd(detail.branch),
        getBranchDetailCmd(detail.branch),
      ]);

      if (cancelled) return;

      setActiveSession(snapshot.active);
      setRecentSessions(snapshot.sessions);
      setProviderMatrix(matrixRes.status === 'ok' ? matrixRes.data : null);
      setWorkspaceChanges(changesRes.status === 'ok' ? changesRes.data : []);
      setWorkspaceGitSummary(gitSummaryRes.status === 'ok' ? gitSummaryRes.data : null);

      if (branchRes.status === 'ok') {
        setBranchDetail(branchRes.data);
        setBranchDiffPath((current) => {
          if (current && branchRes.data.changes.some((change) => change.path === current)) {
            return current;
          }
          return branchRes.data.changes[0]?.path ?? null;
        });
      } else {
        setBranchDetail(null);
        setBranchDiffPath(null);
        setBranchFileDiff('');
      }
    };

    void loadContext();

    return () => {
      cancelled = true;
    };
  }, [detail?.branch, detail?.activeMode, detail?.configGeneration, selectedBranch, workspaceUi.activeModeId]);

  useEffect(() => {
    if (!branchDetail || !branchDiffPath) {
      setBranchFileDiff('');
      setLoadingBranchDiff(false);
      return;
    }

    let cancelled = false;
    setLoadingBranchDiff(true);

    const loadDiff = async () => {
      const diffRes = await getBranchFileDiffCmd(branchDetail.branch, branchDiffPath);
      if (cancelled) return;
      if (diffRes.status === 'ok') {
        setBranchFileDiff(diffRes.data);
      } else {
        setBranchFileDiff(diffRes.error || 'Failed to load branch diff.');
      }
      setLoadingBranchDiff(false);
    };

    void loadDiff();

    return () => {
      cancelled = true;
    };
  }, [branchDetail?.branch, branchDiffPath]);

  return {
    branch,
    runtimeWorkspaces,
    availableEditors,
    providerInfos,
    gitBranches,
    activeSessionBranches,
    selectedBranch,
    setSelectedBranch,
    loading,
    error,
    setError,
    searchQuery,
    setSearchQuery,
    groupBy,
    setGroupBy,
    activeSession,
    recentSessions,
    providerMatrix,
    startingSession,
    setStartingSession,
    endingSession,
    setEndingSession,
    sessionGoalInput,
    setSessionGoalInput,
    sessionSummaryInput,
    setSessionSummaryInput,
    syncing,
    setSyncing,
    updatingLinks,
    setUpdatingLinks,
    linkFeatureId,
    setLinkFeatureId,
    linkReleaseId,
    setLinkReleaseId,
    workspaceChanges,
    workspaceGitSummary,
    branchDetail,
    branchDiffPath,
    setBranchDiffPath,
    branchFileDiff,
    loadingBranchDiff,
    rows,
    filteredRows,
    detail,
    load,
    fetchSessionSnapshot,
  };
}
