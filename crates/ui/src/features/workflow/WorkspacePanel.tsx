import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import {
  ExternalLink,
  GitBranch,
  GitFork,
  Loader2,
  Plus,
  Search,
  RefreshCw,
  Settings2,
  X,
  Zap,
} from 'lucide-react';
import {
  activateWorkspaceCmd,
  createWorkspaceCmd,
  getCurrentBranchCmd,
  listWorkspacesCmd,
  syncWorkspaceCmd,
} from '@/lib/platform/tauri/commands';
import { Workspace } from '@/bindings';
import { RuntimeWorkspace } from '@/lib/types/workspace';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Alert, AlertDescription, AlertTitle } from '@ship/ui';
import { Input } from '@ship/ui';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { FEATURES_ROUTE, SETTINGS_ROUTE } from '@/lib/constants/routes';
import {
  WorkspaceLifecycleGraph,
  type WorkspaceGroupBy,
  type WorkspaceGraphRow,
  type WorkspaceGraphStatus,
} from './components/WorkspaceLifecycleGraph';

const GROUP_BY_OPTIONS: Array<{ key: WorkspaceGroupBy; label: string }> = [
  { key: 'status', label: 'Status' },
  { key: 'type', label: 'Type' },
  { key: 'release', label: 'Release' },
];

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

export default function WorkspacePanel() {
  const navigate = useNavigate();
  const workspaceUi = useWorkspace();
  const ship = useShip();
  const [branch, setBranch] = useState<string | null>(null);
  const [runtimeWorkspaces, setRuntimeWorkspaces] = useState<Workspace[]>([]);
  const [selectedBranch, setSelectedBranch] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [activating, setActivating] = useState(false);
  const [creating, setCreating] = useState(false);
  const [workspaceKeyInput, setWorkspaceKeyInput] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [groupBy, setGroupBy] = useState<WorkspaceGroupBy>('status');

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

  const filteredRows = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    return rows.filter((row) => {
      if (!query) return true;

      const haystack = [
        row.branch,
        row.featureId ?? '',
        row.specId ?? '',
        row.releaseId ?? '',
        row.activeMode ?? '',
        row.providers.join(' '),
      ]
        .join(' ')
        .toLowerCase();
      return haystack.includes(query);
    });
  }, [rows, searchQuery]);


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

  const linkedFeature = useMemo(() => {
    if (!detail) return null;
    return (
      ship.features.find((entry) => entry?.branch === detail.branch) ??
      ship.features.find((entry) => entry?.file_name === detail.featureId) ??
      null
    );
  }, [detail, ship.features]);

  const linkedSpec = useMemo(() => {
    if (!detail) return null;
    return (
      ship.specs.find((entry) => entry.file_name === detail.specId) ??
      ship.specs.find((entry) => entry.file_name === linkedFeature?.spec_id) ??
      null
    );
  }, [detail, linkedFeature?.spec_id, ship.specs]);

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

  const createWorkspaceFromInput = async () => {
    const key = workspaceKeyInput.trim() || branch?.trim() || '';
    if (!key) {
      setError('Provide a workspace key (branch/id).');
      return;
    }

    setCreating(true);
    setError(null);
    try {
      const linkedFeatureForBranch =
        ship.features.find((entry) => entry?.branch === key) ?? null;
      const result = await createWorkspaceCmd(key, {
        activate: true,
        featureId: linkedFeatureForBranch?.file_name ?? null,
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
    void ship.handleSelectSpec(linkedSpec);
  };

  const openAgentProviders = () => {
    void navigate({ to: SETTINGS_ROUTE, search: { tab: 'providers' } });
  };

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
          <Input
            value={workspaceKeyInput}
            onChange={(event) => setWorkspaceKeyInput(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
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

  return (
    <div className="flex h-full min-h-0 flex-col">
      {/* Toolbar */}
      <div className="flex shrink-0 items-center gap-2 border-b border-border/50 px-4 py-2">
        <div className="relative h-7 min-w-[180px] flex-1 max-w-xs">
          <Search className="absolute left-2 top-1/2 size-3 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
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

        {hasActiveFilters && (
          <Button variant="ghost" size="xs" onClick={clearFilters} className="h-7">
            <X className="size-3" />
          </Button>
        )}

        <div className="flex-1" />

        <span className="text-[10px] text-muted-foreground">
          {filteredRows.length} of {rows.length}
        </span>

        <div className="flex items-center gap-1">
          <Input
            value={workspaceKeyInput}
            onChange={(event) => setWorkspaceKeyInput(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                event.preventDefault();
                void createWorkspaceFromInput();
              }
            }}
            placeholder={branch ? `new (default: ${branch})` : 'new workspace'}
            className="h-7 w-40 text-xs"
          />
          <Button variant="outline" size="xs" className="h-7" onClick={() => void createWorkspaceFromInput()} disabled={creating}>
            <Plus className={`size-3 ${creating ? 'animate-pulse' : ''}`} />
          </Button>
          <Button variant="outline" size="xs" className="h-7" onClick={() => void syncCurrentWorkspace()} disabled={!branch || syncing}>
            <GitBranch className={`size-3 ${syncing ? 'animate-pulse' : ''}`} />
          </Button>
          <Button variant="outline" size="xs" className="h-7" onClick={() => void load()} disabled={loading}>
            <RefreshCw className={`size-3 ${loading ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </div>

      {/* Main split layout */}
      <div className="flex flex-1 min-h-0">
        {/* Kanban */}
        <div className="flex-1 min-w-0 overflow-auto p-3">
          <WorkspaceLifecycleGraph
            rows={filteredRows}
            selectedBranch={selectedBranch}
            onSelectBranch={setSelectedBranch}
            groupBy={groupBy}
          />
        </div>

        {/* Detail Panel */}
        {detail && (
          <aside className="w-[380px] shrink-0 overflow-y-auto border-l border-border/50 bg-card/30">
            {/* Detail Header */}
            <div className="sticky top-0 z-10 border-b border-border/50 bg-card/80 backdrop-blur-sm px-4 py-3">
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
              {branch === detail.branch && (
                <Badge variant="default" className="mt-1.5 h-5 px-1.5 text-[10px]">
                  current branch
                </Badge>
              )}
            </div>

            <div className="space-y-4 p-4">
              {/* Linked Context */}
              <section>
                <p className="mb-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">Linked Context</p>
                <div className="space-y-1.5">
                  <div className="flex items-center justify-between rounded-md border bg-background/60 px-3 py-2">
                    <span className="text-xs text-muted-foreground">Spec</span>
                    {linkedSpec ? (
                      <Button size="xs" variant="ghost" onClick={openSpec} className="h-6 gap-1 px-1.5">
                        <span className="truncate max-w-[160px] text-xs">{linkedSpec.spec.metadata.title}</span>
                        <ExternalLink className="size-3 shrink-0" />
                      </Button>
                    ) : (
                      <span className="text-xs text-muted-foreground/50 italic">none</span>
                    )}
                  </div>
                  <div className="flex items-center justify-between rounded-md border bg-background/60 px-3 py-2">
                    <span className="text-xs text-muted-foreground">Feature</span>
                    {linkedFeature ? (
                      <Button size="xs" variant="ghost" onClick={openFeature} className="h-6 gap-1 px-1.5">
                        <span className="truncate max-w-[160px] text-xs">{linkedFeature.title}</span>
                        <ExternalLink className="size-3 shrink-0" />
                      </Button>
                    ) : (
                      <span className="text-xs text-muted-foreground/50 italic">none</span>
                    )}
                  </div>
                  <div className="flex items-center justify-between rounded-md border bg-background/60 px-3 py-2">
                    <span className="text-xs text-muted-foreground">Release</span>
                    <span className="truncate max-w-[160px] text-xs font-medium">
                      {detail.releaseId || <span className="text-muted-foreground/50 italic font-normal">none</span>}
                    </span>
                  </div>
                </div>
              </section>

              {/* Runtime */}
              <section>
                <div className="mb-2 flex items-center justify-between">
                  <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">Runtime</p>
                  <Button size="xs" variant="ghost" onClick={openAgentProviders} className="h-5 gap-1 px-1 text-[10px]">
                    <Settings2 className="size-3" />
                    Configure
                  </Button>
                </div>
                <div className="rounded-lg border bg-gradient-to-br from-primary/5 to-transparent p-3 space-y-2.5">
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
                  {detail.providers.length > 0 && (
                    <div className="flex flex-wrap gap-1">
                      {detail.providers.map((provider) => (
                        <Badge key={provider} variant="secondary" className="h-5 px-1.5 text-[10px]">
                          {provider}
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>
              </section>

              {/* Details */}
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
          </aside>
        )}
      </div>
    </div>
  );
}
