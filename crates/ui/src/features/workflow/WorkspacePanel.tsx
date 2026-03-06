import { useEffect, useMemo, useState, type ChangeEvent, type KeyboardEvent } from 'react';
import { useNavigate } from '@tanstack/react-router';
import {
  ExternalLink,
  Clock3,
  GitBranch,
  GitFork,
  Loader2,
  NotebookPen,
  PanelRightClose,
  PanelRightOpen,
  Plus,
  Search,
  RefreshCw,
  Settings2,
  TerminalSquare,
  Trash2,
  X,
  Zap,
} from 'lucide-react';
import {
  activateWorkspaceCmd,
  createWorkspaceCmd,
  deleteWorkspaceCmd,
  getActiveWorkspaceSessionCmd,
  getCurrentBranchCmd,
  listWorkspaceChangesCmd,
  listWorkspaceEditorsCmd,
  listWorkspaceSessionsCmd,
  listWorkspacesCmd,
  openWorkspaceEditorCmd,
  syncWorkspaceCmd,
  type WorkspaceEditorInfo,
  type WorkspaceFileChange,
  type WorkspaceSessionInfo,
} from '@/lib/platform/tauri/commands';
import { Workspace } from '@/bindings';
import { RuntimeWorkspace } from '@/lib/types/workspace';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Alert, AlertDescription, AlertTitle } from '@ship/ui';
import { Input } from '@ship/ui';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/ui';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import {
  AGENTS_PROVIDERS_ROUTE,
  FEATURES_ROUTE,
} from '@/lib/constants/routes';
import {
  WorkspaceLifecycleGraph,
  type WorkspaceGroupBy,
  type WorkspaceGraphRow,
  type WorkspaceGraphStatus,
} from './components/WorkspaceLifecycleGraph';

const GROUP_BY_OPTIONS: Array<{ key: WorkspaceGroupBy; label: string }> = [
  { key: 'type', label: 'Type' },
  { key: 'release', label: 'Release' },
  { key: 'status', label: 'Status' },
];

const WORKSPACE_TYPE_OPTIONS: Array<{
  value: WorkspaceGraphRow['workspaceType'];
  label: string;
}> = [
  { value: 'feature', label: 'Feature' },
  { value: 'refactor', label: 'Refactor' },
  { value: 'hotfix', label: 'Hotfix' },
  { value: 'experiment', label: 'Experiment' },
];

const NO_LINK_VALUE = '__none__';

interface WorkspaceRow extends WorkspaceGraphRow {
  id: string;
  branch: string;
  resolvedAt: string;
  worktreePath: string | null;
  lastActivatedAt: string | null;
  contextHash: string | null;
}

function normalizeWorkspaceType(type: string | null | undefined): WorkspaceGraphRow['workspaceType'] {
  switch (type) {
    case 'feature':
    case 'refactor':
    case 'experiment':
    case 'hotfix':
      return type;
    default:
      return 'feature';
  }
}

function normalizeWorkspaceStatus(
  status: string | null | undefined,
): WorkspaceGraphStatus {
  switch (status) {
    case 'planned':
    case 'active':
    case 'idle':
    case 'review':
    case 'merged':
    case 'archived':
      return status;
    default:
      return 'idle';
  }
}

function statusVariant(status: WorkspaceGraphStatus): 'default' | 'secondary' | 'outline' {
  if (status === 'active') return 'default';
  if (status === 'review') return 'secondary';
  return 'outline';
}

function shortToken(value: string, size = 10): string {
  return value.length <= size ? value : `${value.slice(0, size)}…`;
}

function readInputValue(event: ChangeEvent<HTMLInputElement>): string {
  return event.target.value;
}

function isEnterKey(event: KeyboardEvent<HTMLInputElement>): boolean {
  return event.key === 'Enter';
}

export default function WorkspacePanel() {
  const navigate = useNavigate();
  const workspaceUi = useWorkspace();
  const ship = useShip();
  const [viewMode, setViewMode] = useState<'command' | 'board'>('command');
  const [branch, setBranch] = useState<string | null>(null);
  const [runtimeWorkspaces, setRuntimeWorkspaces] = useState<Workspace[]>([]);
  const [availableEditors, setAvailableEditors] = useState<WorkspaceEditorInfo[]>([]);
  const [selectedBranch, setSelectedBranch] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingEditors, setLoadingEditors] = useState(true);
  const [loadingChanges, setLoadingChanges] = useState(false);
  const [loadingSessions, setLoadingSessions] = useState(false);
  const [openingEditorId, setOpeningEditorId] = useState<string | null>(null);
  const [syncing, setSyncing] = useState(false);
  const [activating, setActivating] = useState(false);
  const [creating, setCreating] = useState(false);
  const [updatingLinks, setUpdatingLinks] = useState(false);
  const [deletingWorkspace, setDeletingWorkspace] = useState(false);
  const [workspaceKeyInput, setWorkspaceKeyInput] = useState('');
  const [createWorkspaceType, setCreateWorkspaceType] = useState<WorkspaceGraphRow['workspaceType']>('feature');
  const [linkFeatureId, setLinkFeatureId] = useState<string>(NO_LINK_VALUE);
  const [linkSpecId, setLinkSpecId] = useState<string>(NO_LINK_VALUE);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [groupBy, setGroupBy] = useState<WorkspaceGroupBy>('type');
  const [showDetails, setShowDetails] = useState(false);
  const [workspaceChanges, setWorkspaceChanges] = useState<WorkspaceFileChange[]>([]);
  const [activeSession, setActiveSession] = useState<WorkspaceSessionInfo | null>(null);
  const [recentSessions, setRecentSessions] = useState<WorkspaceSessionInfo[]>([]);

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      const currentBranch = await getCurrentBranchCmd();
      setBranch(currentBranch);

      const result = await listWorkspacesCmd();
      if (result.status === 'ok') {
        setRuntimeWorkspaces(result.data);
      } else {
        setError(result.error || 'Failed to load workspaces.');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  useEffect(() => {
    let cancelled = false;

    const loadEditors = async () => {
      setLoadingEditors(true);
      const result = await listWorkspaceEditorsCmd();
      if (cancelled) return;

      if (result.status === 'ok') {
        setAvailableEditors(result.data);
      } else {
        setAvailableEditors([]);
      }
      setLoadingEditors(false);
    };

    void loadEditors();

    return () => {
      cancelled = true;
    };
  }, []);

  const rows = useMemo<WorkspaceRow[]>(() => {
    const statusRank: Record<WorkspaceGraphStatus, number> = {
      active: 0,
      review: 1,
      idle: 2,
      planned: 3,
      merged: 4,
      archived: 5,
    };

    return runtimeWorkspaces
      .map((workspace) => {
        const rw = workspace as RuntimeWorkspace;
        const status = normalizeWorkspaceStatus(
          rw.status ?? (workspace.branch === branch ? 'active' : 'idle'),
        );
        return {
          id: workspace.branch || 'unknown',
          branch: workspace.branch,
          workspaceType: normalizeWorkspaceType(rw.workspace_type),
          featureId: workspace.feature_id ?? null,
          specId: workspace.spec_id ?? null,
          releaseId: rw.release_id ?? null,
          activeMode: workspace.active_mode ?? null,
          providers: workspace.providers ?? [],
          resolvedAt: workspace.resolved_at,
          isWorktree: workspace.is_worktree,
          worktreePath: workspace.worktree_path ?? null,
          lastActivatedAt: rw.last_activated_at ?? null,
          contextHash: rw.context_hash ?? null,
          status,
        };
      })
      .sort((a, b) => {
        if (a.status !== b.status) return statusRank[a.status] - statusRank[b.status];
        return b.resolvedAt.localeCompare(a.resolvedAt);
      });
  }, [runtimeWorkspaces, branch]);

  const featureTitleLookup = useMemo(() => {
    const map = new Map<string, string>();
    for (const feature of ship.features) {
      map.set(feature.id, feature.title);
      map.set(feature.file_name, feature.title);
    }
    return map;
  }, [ship.features]);

  const specTitleLookup = useMemo(() => {
    const map = new Map<string, string>();
    for (const spec of ship.specs) {
      map.set(spec.id, spec.spec.metadata.title);
      map.set(spec.file_name, spec.spec.metadata.title);
    }
    return map;
  }, [ship.specs]);

  const releaseTitleLookup = useMemo(() => {
    const map = new Map<string, string>();
    for (const release of ship.releases) {
      map.set(release.id, release.version);
      map.set(release.file_name, release.version);
      map.set(release.version, release.version);
    }
    return map;
  }, [ship.releases]);

  const filteredRows = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    return rows.filter((row) => {
      if (!query) return true;

      const haystack = [
        row.branch,
        row.featureId ?? '',
        featureTitleLookup.get(row.featureId ?? '') ?? '',
        row.specId ?? '',
        specTitleLookup.get(row.specId ?? '') ?? '',
        row.releaseId ?? '',
        releaseTitleLookup.get(row.releaseId ?? '') ?? '',
        row.workspaceType,
        row.status,
        row.activeMode ?? '',
        row.providers.join(' '),
      ]
        .join(' ')
        .toLowerCase();
      return haystack.includes(query);
    });
  }, [rows, searchQuery, featureTitleLookup, specTitleLookup, releaseTitleLookup]);


  useEffect(() => {
    if (filteredRows.length === 0) {
      setSelectedBranch(null);
      return;
    }
    if (selectedBranch && filteredRows.some((entry) => entry.branch === selectedBranch)) return;
    if (branch && filteredRows.some((entry) => entry.branch === branch)) {
      setSelectedBranch(branch);
      return;
    }
    setSelectedBranch(filteredRows[0].branch);
  }, [filteredRows, selectedBranch, branch]);

  const detail = useMemo(() => {
    if (!selectedBranch) return null;
    return filteredRows.find((entry) => entry.branch === selectedBranch) ?? null;
  }, [filteredRows, selectedBranch]);

  useEffect(() => {
    if (detail) {
      setShowDetails(true);
    }
  }, [detail?.branch]);

  useEffect(() => {
    if (!detail) {
      setWorkspaceChanges([]);
      setActiveSession(null);
      setRecentSessions([]);
      return;
    }

    let cancelled = false;
    const targetBranch = detail.branch;

    const loadWorkspaceContext = async () => {
      setLoadingChanges(true);
      setLoadingSessions(true);

      const [changesResult, activeSessionResult, sessionsResult] = await Promise.all([
        listWorkspaceChangesCmd(targetBranch),
        getActiveWorkspaceSessionCmd(targetBranch),
        listWorkspaceSessionsCmd(targetBranch, 8),
      ]);

      if (cancelled) return;

      if (changesResult.status === 'ok') {
        setWorkspaceChanges(changesResult.data);
      } else {
        setWorkspaceChanges([]);
      }

      if (activeSessionResult.status === 'ok') {
        setActiveSession(activeSessionResult.data);
      } else {
        setActiveSession(null);
      }

      if (sessionsResult.status === 'ok') {
        setRecentSessions(sessionsResult.data);
      } else {
        setRecentSessions([]);
      }

      setLoadingChanges(false);
      setLoadingSessions(false);
    };

    void loadWorkspaceContext();

    return () => {
      cancelled = true;
    };
  }, [detail?.branch]);

  const linkedFeature = useMemo(() => {
    if (!detail) return null;
    return (
      ship.features.find((entry) => entry?.branch === detail.branch) ??
      ship.features.find((entry) => entry?.id === detail.featureId) ??
      ship.features.find((entry) => entry?.file_name === detail.featureId) ??
      null
    );
  }, [detail, ship.features]);

  const linkedSpec = useMemo(() => {
    if (!detail) return null;
    return (
      ship.specs.find((entry) => entry.id === detail.specId) ??
      ship.specs.find((entry) => entry.file_name === detail.specId) ??
      ship.specs.find((entry) => entry.id === linkedFeature?.spec_id) ??
      ship.specs.find((entry) => entry.file_name === linkedFeature?.spec_id) ??
      null
    );
  }, [detail, linkedFeature?.spec_id, ship.specs]);

  const linkedRelease = useMemo(() => {
    if (!detail) return null;
    return (
      ship.releases.find((entry) => entry.id === detail.releaseId) ??
      ship.releases.find((entry) => entry.file_name === detail.releaseId) ??
      ship.releases.find((entry) => entry.id === linkedFeature?.release_id) ??
      ship.releases.find((entry) => entry.file_name === linkedFeature?.release_id) ??
      ship.releases.find((entry) => entry.version === detail.releaseId) ??
      null
    );
  }, [detail, linkedFeature?.release_id, ship.releases]);

  useEffect(() => {
    if (!detail) {
      setLinkFeatureId(NO_LINK_VALUE);
      setLinkSpecId(NO_LINK_VALUE);
      return;
    }

    const normalizedFeatureId =
      ship.features.find((entry) => entry.id === detail.featureId || entry.file_name === detail.featureId)
        ?.id ?? NO_LINK_VALUE;
    const normalizedSpecId =
      ship.specs.find((entry) => entry.id === detail.specId || entry.file_name === detail.specId)
        ?.id ?? NO_LINK_VALUE;

    setLinkFeatureId(normalizedFeatureId);
    setLinkSpecId(normalizedSpecId);
  }, [detail?.branch, detail?.featureId, detail?.specId, ship.features, ship.specs]);

  const featureLinkOptions = useMemo(() => {
    return [...ship.features].sort((a, b) => a.title.localeCompare(b.title));
  }, [ship.features]);

  const specLinkOptions = useMemo(() => {
    return [...ship.specs].sort((a, b) =>
      a.spec.metadata.title.localeCompare(b.spec.metadata.title),
    );
  }, [ship.specs]);

  const syncCurrentWorkspace = async () => {
    if (!branch) return;
    setSyncing(true);
    setError(null);
    try {
      const result = await syncWorkspaceCmd(branch);
      if (result.status === 'ok') {
        setSelectedBranch(result.data.branch);
        await load();
      } else {
        setError(result.error || 'Failed to sync workspace.');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setSyncing(false);
    }
  };

  const activateSelectedWorkspace = async () => {
    if (!detail) return;
    setActivating(true);
    setError(null);
    try {
      const result = await activateWorkspaceCmd(detail.branch);
      if (result.status === 'ok') {
        setSelectedBranch(result.data.branch);
        await load();
      } else {
        setError(result.error || 'Failed to activate workspace.');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setActivating(false);
    }
  };

  const createWorkspaceFromInput = async (
    workspaceTypeOverride?: WorkspaceGraphRow['workspaceType'],
  ) => {
    const workspaceType = workspaceTypeOverride ?? createWorkspaceType;
    let key = workspaceKeyInput.trim() || branch?.trim() || '';
    if (!key) {
      setError('Provide a workspace key (branch/id).');
      return;
    }

    if (!key.includes('/') && workspaceType !== 'feature') {
      key = `${workspaceType}/${key}`;
    }

    setCreating(true);
    setError(null);
    try {
      const linkedFeatureForBranch =
        ship.features.find((entry) => entry?.branch === key) ?? null;
      const result = await createWorkspaceCmd(key, {
        workspaceType,
        activate: true,
        featureId: linkedFeatureForBranch?.id ?? null,
        specId: linkedFeatureForBranch?.spec_id ?? null,
        releaseId: linkedFeatureForBranch?.release_id ?? null,
      });
      if (result.status === 'ok') {
        setSelectedBranch(result.data.branch);
        setWorkspaceKeyInput('');
        await load();
      } else {
        setError(result.error || 'Failed to create workspace.');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setCreating(false);
    }
  };

  const applyWorkspaceLinks = async () => {
    if (!detail) return;
    setUpdatingLinks(true);
    setError(null);
    try {
      const selectedFeature =
        linkFeatureId === NO_LINK_VALUE
          ? null
          : ship.features.find((entry) => entry.id === linkFeatureId) ?? null;
      const selectedSpec =
        linkSpecId === NO_LINK_VALUE
          ? null
          : ship.specs.find((entry) => entry.id === linkSpecId) ?? null;

      const result = await createWorkspaceCmd(detail.branch, {
        workspaceType: detail.workspaceType,
        featureId: selectedFeature?.id ?? detail.featureId ?? null,
        specId: selectedSpec?.id ?? selectedFeature?.spec_id ?? detail.specId ?? null,
        releaseId: selectedFeature?.release_id ?? detail.releaseId ?? null,
        activate: detail.status === 'active',
      });

      if (result.status === 'ok') {
        setSelectedBranch(result.data.branch);
        await load();
      } else {
        setError(result.error || 'Failed to update workspace links.');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setUpdatingLinks(false);
    }
  };

  const deleteSelectedWorkspace = async () => {
    if (!detail) return;
    if (typeof window !== 'undefined') {
      const confirmed = window.confirm(
        `Delete workspace '${detail.branch}'?\n\nThis removes workspace state and session history for this branch.`,
      );
      if (!confirmed) return;
    }

    setDeletingWorkspace(true);
    setError(null);
    try {
      const result = await deleteWorkspaceCmd(detail.branch);
      if (result.status === 'ok') {
        setSelectedBranch(null);
        await load();
      } else {
        setError(result.error || 'Failed to delete workspace.');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setDeletingWorkspace(false);
    }
  };

  const clearFilters = () => {
    setSearchQuery('');
  };

  const openFeature = () => {
    if (!linkedFeature) return;
    void navigate({ to: FEATURES_ROUTE });
    void ship.handleSelectFeature(linkedFeature);
  };

  const openSpec = () => {
    if (!linkedSpec) return;
    void navigate({ to: FEATURES_ROUTE });
    void ship.handleSelectSpec(linkedSpec);
  };

  const openAgentProviders = () => {
    void navigate({ to: AGENTS_PROVIDERS_ROUTE });
  };

  const openWorkspaceEditorForBranch = async (targetBranch: string, editorId: string) => {
    setOpeningEditorId(editorId);
    const result = await openWorkspaceEditorCmd(targetBranch, editorId);
    setOpeningEditorId(null);
    if (result.status === 'error') {
      setError(result.error || `Failed to open ${editorId}.`);
    }
  };

  const openWorkspaceEditor = async (editorId: string) => {
    if (!detail) return;
    await openWorkspaceEditorForBranch(detail.branch, editorId);
  };

  const matchingWorkspaceEvents = useMemo(() => {
    if (!detail) return [];
    const tokens = [detail.branch, detail.featureId, detail.specId, detail.releaseId]
      .filter((value): value is string => Boolean(value))
      .map((value) => value.toLowerCase());

    if (tokens.length === 0) return [];

    return workspaceUi.eventEntries
      .filter((event) => {
        const haystack = `${event.subject} ${event.details ?? ''}`.toLowerCase();
        return tokens.some((token) => haystack.includes(token));
      })
      .slice(-12)
      .reverse();
  }, [detail, workspaceUi.eventEntries]);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="size-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center p-8">
        <Alert variant="destructive" className="max-w-lg">
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      </div>
    );
  }

  if (rows.length === 0) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-5 p-8">
        <div className="flex size-16 items-center justify-center rounded-2xl border-2 border-dashed border-muted-foreground/20">
          <GitBranch className="size-7 text-muted-foreground/40" />
        </div>
        <div className="text-center">
          <h3 className="text-lg font-semibold">No Workspaces Yet</h3>
          <p className="mt-1 max-w-sm text-sm text-muted-foreground">
            Create a workspace to start context-aware agent execution.
          </p>
          {branch && (
            <p className="mt-2 text-xs text-muted-foreground">
              Current branch:{' '}
              <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{branch}</code>
            </p>
          )}
        </div>
        <div className="flex items-center gap-2">
          <Select
            value={createWorkspaceType}
            onValueChange={(value) => setCreateWorkspaceType(value as WorkspaceGraphRow['workspaceType'])}
          >
            <SelectTrigger size="sm" className="h-9 w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {WORKSPACE_TYPE_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Input
            value={workspaceKeyInput}
            onChange={(event: ChangeEvent<HTMLInputElement>) => setWorkspaceKeyInput(readInputValue(event))}
            onKeyDown={(event: KeyboardEvent<HTMLInputElement>) => {
              if (isEnterKey(event)) {
                event.preventDefault();
                void createWorkspaceFromInput();
              }
            }}
            placeholder={branch ? `key (default: ${branch})` : 'workspace key'}
            className="h-9 w-56"
          />
          <Button onClick={() => void createWorkspaceFromInput()} disabled={creating}>
            <Plus className="size-4" />
            {creating ? 'Creating…' : 'Create Workspace'}
          </Button>
        </div>
      </div>
    );
  }

  const hasActiveFilters = searchQuery.trim().length > 0;
  const latestSession = activeSession ?? recentSessions[0] ?? null;

  const workspaceList = (
    <section className="flex h-full min-h-0 flex-col rounded-lg border border-border/70 bg-card/30">
      <div className="flex items-center justify-between border-b border-border/50 px-3 py-2">
        <h3 className="text-sm font-semibold">Workspace Roster</h3>
        <span className="text-[11px] text-muted-foreground">{filteredRows.length}</span>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto p-2">
        <div className="space-y-1.5">
          {filteredRows.map((row) => {
            const selected = row.branch === selectedBranch;
            return (
              <button
                key={row.id}
                type="button"
                className={`w-full rounded-md border px-3 py-2 text-left transition-colors ${
                  selected
                    ? 'border-primary/50 bg-primary/10'
                    : 'border-border/60 bg-background/50 hover:bg-muted/40'
                }`}
                onClick={() => setSelectedBranch(row.branch)}
              >
                <div className="flex items-start justify-between gap-2">
                  <span className="truncate text-sm font-semibold">{row.branch}</span>
                  <div className="flex items-center gap-1">
                    {availableEditors.length > 0 && (
                      <span
                        className="inline-flex h-5 w-5 cursor-pointer items-center justify-center rounded-sm hover:bg-muted/70"
                        title={`Open in ${availableEditors[0].name}`}
                        onClick={(event) => {
                          event.stopPropagation();
                          void openWorkspaceEditorForBranch(row.branch, availableEditors[0].id);
                        }}
                      >
                        <TerminalSquare className="size-3" />
                      </span>
                    )}
                    <Badge variant={statusVariant(row.status)} className="h-5 px-1.5 text-[10px]">
                      {row.status}
                    </Badge>
                  </div>
                </div>
                <div className="mt-1 flex flex-wrap gap-1">
                  <Badge variant="outline" className="h-5 px-1.5 text-[10px]">
                    {row.workspaceType}
                  </Badge>
                  <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
                    {row.activeMode ?? 'default'}
                  </Badge>
                  <Badge variant="outline" className="h-5 px-1.5 text-[10px]">
                    {row.providers.length} provider{row.providers.length === 1 ? '' : 's'}
                  </Badge>
                </div>
              </button>
            );
          })}
        </div>
      </div>
    </section>
  );

  const detailSections = detail ? (
    <div className="space-y-4 p-4">
      <section>
        <div className="mb-2 flex items-center justify-between">
          <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">Session Metadata</p>
          <Badge variant={latestSession?.status === 'active' ? 'default' : 'outline'} className="h-5 px-1.5 text-[10px]">
            {latestSession?.status ?? 'no-session'}
          </Badge>
        </div>
        <div className="rounded-lg border bg-background/50 p-3 text-[11px]">
          {loadingSessions ? (
            <p className="text-muted-foreground">Loading sessions…</p>
          ) : latestSession ? (
            <div className="space-y-1.5">
              <p><span className="text-muted-foreground">ID:</span> <code className="font-mono text-[10px]">{latestSession.id}</code></p>
              <p><span className="text-muted-foreground">Started:</span> {new Date(latestSession.started_at).toLocaleString()}</p>
              {latestSession.primary_provider && (
                <p><span className="text-muted-foreground">Provider:</span> {latestSession.primary_provider}</p>
              )}
              {latestSession.mode_id && (
                <p><span className="text-muted-foreground">Mode:</span> {latestSession.mode_id}</p>
              )}
              {latestSession.goal && (
                <p className="line-clamp-3"><span className="text-muted-foreground">Goal:</span> {latestSession.goal}</p>
              )}
              {latestSession.compile_error ? (
                <p className="text-destructive line-clamp-2">
                  <span className="text-muted-foreground">Compile:</span> {latestSession.compile_error}
                </p>
              ) : (
                <p><span className="text-muted-foreground">Compiled:</span> {latestSession.compiled_at ? new Date(latestSession.compiled_at).toLocaleString() : 'n/a'}</p>
              )}
            </div>
          ) : (
            <p className="text-muted-foreground">No workspace sessions recorded yet.</p>
          )}
        </div>
      </section>

      <section>
        <div className="mb-2 flex items-center justify-between">
          <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">Linked Context</p>
          <div className="flex items-center gap-1.5">
            <Button
              size="xs"
              variant="outline"
              className="h-6 gap-1 px-1.5 text-[10px]"
              onClick={() => void applyWorkspaceLinks()}
              disabled={updatingLinks}
            >
              {updatingLinks ? 'Saving…' : 'Save Links'}
            </Button>
            {linkedFeature && (
              <Button size="xs" variant="ghost" className="h-6 gap-1 px-1.5 text-[10px]" onClick={openFeature}>
                <ExternalLink className="size-3" />
                Feature
              </Button>
            )}
            {linkedSpec && (
              <Button size="xs" variant="ghost" className="h-6 gap-1 px-1.5 text-[10px]" onClick={openSpec}>
                <NotebookPen className="size-3" />
                Spec
              </Button>
            )}
          </div>
        </div>
        <div className="space-y-2">
          <div className="rounded-md border bg-background/60 p-2">
            <p className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">Feature</p>
            <Select
              value={linkFeatureId}
              onValueChange={(value) => setLinkFeatureId(value ?? NO_LINK_VALUE)}
            >
              <SelectTrigger size="sm" className="h-7 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={NO_LINK_VALUE}>No change</SelectItem>
                {featureLinkOptions.map((entry) => (
                  <SelectItem key={entry.id} value={entry.id}>
                    {entry.title}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="rounded-md border bg-background/60 p-2">
            <p className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">Spec</p>
            <Select
              value={linkSpecId}
              onValueChange={(value) => setLinkSpecId(value ?? NO_LINK_VALUE)}
            >
              <SelectTrigger size="sm" className="h-7 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={NO_LINK_VALUE}>No change</SelectItem>
                {specLinkOptions.map((entry) => (
                  <SelectItem key={entry.id} value={entry.id}>
                    {entry.spec.metadata.title}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="flex items-center justify-between rounded-md border bg-background/60 px-3 py-2">
            <span className="text-xs text-muted-foreground">Spec</span>
            {linkedSpec ? (
              <span className="truncate max-w-[220px] text-xs">{linkedSpec.spec.metadata.title}</span>
            ) : (
              <span className="text-xs text-muted-foreground/50 italic">none</span>
            )}
          </div>
          <div className="flex items-center justify-between rounded-md border bg-background/60 px-3 py-2">
            <span className="text-xs text-muted-foreground">Feature</span>
            {linkedFeature ? (
              <span className="truncate max-w-[220px] text-xs">{linkedFeature.title}</span>
            ) : (
              <span className="text-xs text-muted-foreground/50 italic">none</span>
            )}
          </div>
          <div className="flex items-center justify-between rounded-md border bg-background/60 px-3 py-2">
            <span className="text-xs text-muted-foreground">Release</span>
            {linkedRelease ? (
              <span className="truncate max-w-[220px] text-xs">{linkedRelease.version}</span>
            ) : (
              <span className="text-xs text-muted-foreground/50 italic">none</span>
            )}
          </div>
        </div>
      </section>

      <section>
        <div className="mb-2 flex items-center justify-between">
          <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">Workspace Modes</p>
          <Button size="xs" variant="ghost" onClick={openAgentProviders} className="h-5 gap-1 px-1 text-[10px]">
            <Settings2 className="size-3" />
            Configure
          </Button>
        </div>
        <div className="rounded-lg border bg-gradient-to-br from-primary/5 to-transparent p-3 space-y-2">
          <div className="flex flex-wrap gap-1">
            <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
              <Zap className="size-2.5 mr-0.5" />
              {detail.activeMode ?? workspaceUi.activeModeId ?? 'default'}
            </Badge>
            <Badge variant="outline" className="h-5 px-1.5 text-[10px]">
              {detail.providers.length} provider{detail.providers.length === 1 ? '' : 's'}
            </Badge>
            <Badge variant="outline" className="h-5 px-1.5 text-[10px]">
              <GitFork className="size-2.5 mr-0.5" />
              {detail.isWorktree ? 'worktree' : 'checkout'}
            </Badge>
          </div>
          <div className="flex flex-wrap gap-1">
            <Button
              size="xs"
              variant={workspaceUi.activeModeId === null ? 'secondary' : 'outline'}
              className="h-6 px-2 text-[10px]"
              onClick={() => void workspaceUi.handleSetActiveMode(null)}
            >
              Default
            </Button>
            {workspaceUi.modes.map((mode) => (
              <Button
                key={mode.id}
                size="xs"
                variant={workspaceUi.activeModeId === mode.id ? 'secondary' : 'outline'}
                className="h-6 px-2 text-[10px]"
                onClick={() => void workspaceUi.handleSetActiveMode(mode.id)}
              >
                {mode.name}
              </Button>
            ))}
          </div>
        </div>
      </section>

      <section>
        <p className="mb-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">Editor Links</p>
        <div className="rounded-lg border bg-background/50 p-3">
          {loadingEditors ? (
            <p className="text-[11px] text-muted-foreground">Discovering editors…</p>
          ) : availableEditors.length === 0 ? (
            <p className="text-[11px] text-muted-foreground">No supported editors found in PATH (cursor, vscode, zed).</p>
          ) : (
            <div className="flex flex-wrap gap-1.5">
              {availableEditors.map((editor) => (
                <Button
                  key={editor.id}
                  size="xs"
                  variant="outline"
                  className="h-6 gap-1.5 px-2 text-[10px]"
                  onClick={() => void openWorkspaceEditor(editor.id)}
                  disabled={openingEditorId === editor.id}
                >
                  <TerminalSquare className="size-3" />
                  {openingEditorId === editor.id ? 'Opening…' : editor.name}
                </Button>
              ))}
            </div>
          )}
        </div>
      </section>

      <section>
        <p className="mb-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">Agent Logs</p>
        <div className="rounded-lg border bg-background/50 p-3">
          {matchingWorkspaceEvents.length === 0 ? (
            <p className="text-[11px] text-muted-foreground">No matching workspace events yet.</p>
          ) : (
            <div className="space-y-1.5">
              {matchingWorkspaceEvents.slice(0, 8).map((event) => (
                <div key={event.seq} className="rounded border border-border/50 px-2 py-1.5">
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-[10px] font-semibold uppercase tracking-wide">{event.entity}.{event.action}</span>
                    <span className="text-[10px] text-muted-foreground">{new Date(event.timestamp).toLocaleTimeString()}</span>
                  </div>
                  <p className="mt-0.5 truncate text-[11px]">{event.subject}</p>
                </div>
              ))}
            </div>
          )}
        </div>
      </section>

      <section>
        <p className="mb-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">Affected Files</p>
        <div className="rounded-lg border bg-background/50 p-3">
          {loadingChanges ? (
            <p className="text-[11px] text-muted-foreground">Scanning git status…</p>
          ) : workspaceChanges.length === 0 ? (
            <p className="text-[11px] text-muted-foreground">No modified files in this workspace.</p>
          ) : (
            <div className="space-y-1">
              {workspaceChanges.slice(0, 20).map((change, idx) => (
                <div key={`${change.path}-${idx}`} className="flex items-center gap-2 text-[11px]">
                  <Badge variant="outline" className="h-5 min-w-9 justify-center px-1.5 text-[10px]">
                    {change.status || '--'}
                  </Badge>
                  <span className="truncate font-mono">{change.path}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </section>

      <section>
        <p className="mb-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">Details</p>
        <div className="space-y-1 text-[11px]">
          <div className="flex justify-between py-1 border-b border-border/30">
            <span className="text-muted-foreground">Branch</span>
            <code className="font-mono text-[10px] truncate max-w-[180px]">{detail.branch}</code>
          </div>
          {detail.worktreePath && (
            <div className="flex justify-between py-1 border-b border-border/30">
              <span className="text-muted-foreground">Worktree</span>
              <code className="font-mono text-[10px] truncate max-w-[180px]">{detail.worktreePath}</code>
            </div>
          )}
          <div className="flex justify-between py-1 border-b border-border/30">
            <span className="text-muted-foreground">Resolved</span>
            <span className="text-muted-foreground">{new Date(detail.resolvedAt).toLocaleDateString()}</span>
          </div>
          <div className="flex justify-between py-1 border-b border-border/30">
            <span className="text-muted-foreground">Last Active</span>
            <span className="text-muted-foreground">
              {detail.lastActivatedAt ? new Date(detail.lastActivatedAt).toLocaleDateString() : 'never'}
            </span>
          </div>
          {detail.contextHash && (
            <div className="flex justify-between py-1 border-b border-border/30">
              <span className="text-muted-foreground">Context Hash</span>
              <code className="font-mono text-[10px] truncate max-w-[120px]" title={detail.contextHash}>
                {shortToken(detail.contextHash, 12)}
              </code>
            </div>
          )}
          <div className="flex justify-between py-1">
            <span className="text-muted-foreground">ID</span>
            <code className="font-mono text-[10px] truncate max-w-[180px]">{detail.id}</code>
          </div>
        </div>
      </section>
    </div>
  ) : (
    <div className="flex h-full min-h-[20rem] items-center justify-center p-8 text-center">
      <div>
        <Clock3 className="mx-auto size-6 text-muted-foreground/40" />
        <p className="mt-2 text-sm text-muted-foreground">Select a workspace to view command-center details.</p>
      </div>
    </div>
  );

  return (
    <div className="flex h-full min-h-0 flex-col">
      {/* Toolbar */}
      <div className="shrink-0 border-b border-border/50 px-3 py-2">
        <div className="flex flex-wrap items-center gap-2">
          <div className="relative h-7 min-w-[180px] flex-1 max-w-[26rem]">
            <Search className="absolute left-2 top-1/2 size-3 -translate-y-1/2 text-muted-foreground" />
            <Input
              value={searchQuery}
              onChange={(event: ChangeEvent<HTMLInputElement>) => setSearchQuery(readInputValue(event))}
              placeholder="Search workspaces…"
              className="h-7 pl-7 text-xs"
            />
          </div>

          <div className="flex h-7 items-center gap-0.5 rounded-md border bg-muted/30 p-0.5">
            {GROUP_BY_OPTIONS.map((option) => (
              <Button
                key={option.key}
                type="button"
                size="xs"
                variant={groupBy === option.key ? 'secondary' : 'ghost'}
                className="h-5 px-2 text-[10px]"
                onClick={() => setGroupBy(option.key)}
              >
                {option.label}
              </Button>
            ))}
          </div>

          <div className="flex h-7 items-center gap-0.5 rounded-md border bg-muted/30 p-0.5">
            <Button
              type="button"
              size="xs"
              variant={viewMode === 'command' ? 'secondary' : 'ghost'}
              className="h-5 px-2 text-[10px]"
              onClick={() => setViewMode('command')}
            >
              Command
            </Button>
            <Button
              type="button"
              size="xs"
              variant={viewMode === 'board' ? 'secondary' : 'ghost'}
              className="h-5 px-2 text-[10px]"
              onClick={() => setViewMode('board')}
            >
              Board
            </Button>
          </div>

          {hasActiveFilters && (
            <Button variant="ghost" size="xs" onClick={clearFilters} className="h-7">
              <X className="size-3" />
            </Button>
          )}

          <span className="ml-auto whitespace-nowrap text-[10px] text-muted-foreground">
            {filteredRows.length} of {rows.length}
          </span>
        </div>

        <div className="mt-2 flex flex-wrap items-center gap-1.5">
          <Select
            value={createWorkspaceType}
            onValueChange={(value) => setCreateWorkspaceType(value as WorkspaceGraphRow['workspaceType'])}
          >
            <SelectTrigger size="sm" className="h-7 w-32 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {WORKSPACE_TYPE_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Input
            value={workspaceKeyInput}
            onChange={(event: ChangeEvent<HTMLInputElement>) => setWorkspaceKeyInput(readInputValue(event))}
            onKeyDown={(event: KeyboardEvent<HTMLInputElement>) => {
              if (isEnterKey(event)) {
                event.preventDefault();
                void createWorkspaceFromInput();
              }
            }}
            placeholder={branch ? `new (default: ${branch})` : 'new workspace'}
            className="h-7 w-44 text-xs"
          />
          <Button variant="outline" size="xs" className="h-7 gap-1.5" onClick={() => void createWorkspaceFromInput()} disabled={creating}>
            <Plus className={`size-3 ${creating ? 'animate-pulse' : ''}`} />
            Create
          </Button>
          <Button
            variant="outline"
            size="xs"
            className="h-7 gap-1.5"
            onClick={() => void createWorkspaceFromInput('hotfix')}
            disabled={creating}
          >
            Hotfix
          </Button>
          <Button
            variant="outline"
            size="xs"
            className="h-7 gap-1.5"
            onClick={() => void createWorkspaceFromInput('experiment')}
            disabled={creating}
          >
            Experiment
          </Button>
          <Button variant="outline" size="xs" className="h-7 gap-1.5" onClick={() => void syncCurrentWorkspace()} disabled={!branch || syncing}>
            <GitBranch className={`size-3 ${syncing ? 'animate-pulse' : ''}`} />
            Sync
          </Button>
          <Button variant="outline" size="xs" className="h-7 gap-1.5" onClick={() => void load()} disabled={loading}>
            <RefreshCw className={`size-3 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          {detail && viewMode === 'board' && (
            <Button
              size="xs"
              variant={showDetails ? 'secondary' : 'outline'}
              className="ml-auto h-7 gap-1.5"
              onClick={() => setShowDetails((current) => !current)}
            >
              {showDetails ? <PanelRightClose className="size-3" /> : <PanelRightOpen className="size-3" />}
              {showDetails ? 'Hide Details' : 'Show Details'}
            </Button>
          )}
        </div>
      </div>

      <div className="relative flex flex-1 min-h-0">
        {viewMode === 'command' ? (
          <div className="flex-1 min-w-0 overflow-hidden p-2">
            <div className="grid h-full gap-3 xl:grid-cols-[320px_minmax(0,1fr)]">
              <div className="min-w-0">{workspaceList}</div>
              <section className="min-h-0 min-w-0 overflow-y-auto rounded-lg border border-border/70 bg-card/40">
                {detail && (
                  <div className="sticky top-0 z-10 border-b border-border/50 bg-card/95 px-4 py-3">
                    <div className="flex items-center justify-between gap-2">
                      <div className="min-w-0">
                        <h3 className="truncate text-sm font-semibold">{detail.branch}</h3>
                        <p className="text-[11px] text-muted-foreground">
                          {detail.workspaceType} · {detail.isWorktree ? 'worktree' : 'checkout'}
                        </p>
                      </div>
                      <div className="flex items-center gap-1.5">
                        <Badge variant={statusVariant(detail.status)} className="h-6 px-2 text-[11px]">
                          {detail.status}
                        </Badge>
                        <Button
                          size="xs"
                          variant="destructive"
                          className="h-6 gap-1 px-2 text-[10px]"
                          onClick={() => void deleteSelectedWorkspace()}
                          disabled={deletingWorkspace}
                        >
                          <Trash2 className="size-3" />
                          {deletingWorkspace ? 'Deleting…' : 'Delete'}
                        </Button>
                        {detail.status !== 'active' && (
                          <Button
                            size="xs"
                            onClick={() => void activateSelectedWorkspace()}
                            disabled={activating}
                          >
                            {activating ? '…' : 'Activate'}
                          </Button>
                        )}
                      </div>
                    </div>
                  </div>
                )}
                {detailSections}
              </section>
            </div>
          </div>
        ) : (
          <>
            {/* Kanban */}
            <div className="flex-1 min-w-0 overflow-auto p-1">
              <WorkspaceLifecycleGraph
                rows={filteredRows}
                selectedBranch={selectedBranch}
                onSelectBranch={setSelectedBranch}
                groupBy={groupBy}
              />
            </div>

            {/* Overlay Detail Panel */}
            {detail && showDetails && (
              <aside className="absolute bottom-3 right-3 top-3 z-20 w-[420px] overflow-y-auto rounded-lg border border-border/70 bg-card/95 shadow-2xl backdrop-blur-sm">
            {/* Detail Header */}
              <div className="sticky top-0 z-10 border-b border-border/50 bg-card/95 px-4 py-3">
                <div className="flex items-center justify-between gap-2">
                  <div className="min-w-0">
                    <h3 className="truncate text-sm font-semibold">{detail.branch}</h3>
                    <p className="text-[11px] text-muted-foreground">
                      {detail.workspaceType} · {detail.isWorktree ? 'worktree' : 'checkout'}
                    </p>
                  </div>
                  <div className="flex items-center gap-1.5">
                    <Badge variant={statusVariant(detail.status)} className="h-6 px-2 text-[11px]">
                      {detail.status}
                    </Badge>
                    <Button
                      size="xs"
                      variant="destructive"
                      className="h-6 gap-1 px-2 text-[10px]"
                      onClick={() => void deleteSelectedWorkspace()}
                      disabled={deletingWorkspace}
                    >
                      <Trash2 className="size-3" />
                      {deletingWorkspace ? '…' : 'Delete'}
                    </Button>
                    <Button
                      size="xs"
                      variant="ghost"
                      className="h-6 w-6 p-0"
                      onClick={() => setShowDetails(false)}
                    >
                      <X className="size-3.5" />
                    </Button>
                  </div>
                </div>
              </div>
              {detailSections}
              </aside>
            )}
          </>
        )}
      </div>
    </div>
  );
}
