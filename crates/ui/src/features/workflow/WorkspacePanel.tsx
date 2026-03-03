import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import {
  ExternalLink,
  GitBranch,
  Loader2,
  Plus,
  Search,
  RefreshCw,
  X,
} from 'lucide-react';
import {
  activateWorkspaceCmd,
  createWorkspaceCmd,
  getCurrentBranchCmd,
  listWorkspacesCmd,
  syncWorkspaceCmd,
} from '@/lib/platform/tauri/commands';
import { Workspace } from '@/bindings';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Input } from '@/components/ui/input';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import { AGENTS_PROVIDERS_ROUTE, FEATURES_ROUTE, SPECS_ROUTE } from '@/lib/constants/routes';
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
      const status = normalizeWorkspaceStatus(
        workspace.status ?? (workspace.branch === branch ? 'active' : 'idle'),
      );
        return {
        id: workspace.id,
        branch: workspace.branch,
        workspaceType: normalizeWorkspaceType(workspace.workspace_type),
        featureId: workspace.feature_id ?? null,
        specId: workspace.spec_id ?? null,
        releaseId: workspace.release_id ?? null,
        activeMode: workspace.active_mode ?? null,
        providers: workspace.providers ?? [],
        resolvedAt: workspace.resolved_at,
        isWorktree: workspace.is_worktree,
        worktreePath: workspace.worktree_path ?? null,
        lastActivatedAt: workspace.last_activated_at ?? null,
        contextHash: workspace.context_hash ?? null,
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

  const statusCounts = useMemo(() => {
    const counts: Record<WorkspaceGraphStatus, number> = {
      planned: 0,
      active: 0,
      idle: 0,
      review: 0,
      merged: 0,
      archived: 0,
    };
    for (const row of filteredRows) {
      counts[row.status] += 1;
    }
    return counts;
  }, [filteredRows]);

  const typeCounts = useMemo(() => {
    const counts: Record<WorkspaceGraphRow['workspaceType'], number> = {
      feature: 0,
      refactor: 0,
      experiment: 0,
      hotfix: 0,
    };
    for (const row of filteredRows) {
      counts[row.workspaceType] += 1;
    }
    return counts;
  }, [filteredRows]);

  const releaseCounts = useMemo(() => {
    const byRelease = new Map<string, number>();
    let unassigned = 0;

    for (const row of filteredRows) {
      const release = row.releaseId?.trim() ?? '';
      if (!release) {
        unassigned += 1;
        continue;
      }
      byRelease.set(release, (byRelease.get(release) ?? 0) + 1);
    }

    const topReleases = Array.from(byRelease.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 3);

    return {
      releaseGroups: byRelease.size,
      unassigned,
      topReleases,
    };
  }, [filteredRows]);

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
      workspaceUi.features.find((entry) => entry.branch === detail.branch) ??
      workspaceUi.features.find((entry) => entry.file_name === detail.featureId) ??
      null
    );
  }, [detail, workspaceUi.features]);

  const linkedSpec = useMemo(() => {
    if (!detail) return null;
    return (
      workspaceUi.specs.find((entry) => entry.file_name === detail.specId) ??
      workspaceUi.specs.find((entry) => entry.file_name === linkedFeature?.spec_id) ??
      null
    );
  }, [detail, linkedFeature?.spec_id, workspaceUi.specs]);

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
        workspaceUi.features.find((entry) => entry.branch === key) ?? null;
      const result = await createWorkspaceCmd(key, {
        activate: true,
        featureId: linkedFeatureForBranch?.id ?? linkedFeatureForBranch?.file_name ?? null,
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

  const actions = (
    <div className="flex items-center gap-1.5">
      <Input
        value={workspaceKeyInput}
        onChange={(event) => setWorkspaceKeyInput(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === 'Enter') {
            event.preventDefault();
            void createWorkspaceFromInput();
          }
        }}
        placeholder={branch ? `workspace key (default ${branch})` : 'workspace key (branch/id)'}
        className="h-7 w-52 text-xs"
      />
      <Button variant="outline" size="xs" onClick={() => void createWorkspaceFromInput()} disabled={creating}>
        <Plus className={`size-3.5 ${creating ? 'animate-pulse' : ''}`} />
        Create
      </Button>
      <Button
        variant="outline"
        size="xs"
        onClick={() => void syncCurrentWorkspace()}
        disabled={!branch || syncing}
      >
        <GitBranch className={`size-3.5 ${syncing ? 'animate-pulse' : ''}`} />
        Sync Git
      </Button>
      <Button variant="outline" size="xs" onClick={() => void load()} disabled={loading}>
        <RefreshCw className={`size-3.5 ${loading ? 'animate-spin' : ''}`} />
        Refresh
      </Button>
    </div>
  );

  const openFeature = () => {
    if (!linkedFeature) return;
    void navigate({ to: FEATURES_ROUTE });
    void workspaceUi.handleSelectFeature(linkedFeature);
  };

  const openSpec = () => {
    if (!linkedSpec) return;
    void navigate({ to: SPECS_ROUTE });
    void workspaceUi.handleSelectSpec(linkedSpec);
  };

  const openAgentProviders = () => {
    void navigate({ to: AGENTS_PROVIDERS_ROUTE });
  };

  if (loading) {
    return (
      <PageFrame width="wide">
        <div className="flex h-64 items-center justify-center">
          <Loader2 className="size-6 animate-spin text-muted-foreground" />
        </div>
      </PageFrame>
    );
  }

  if (error) {
    return (
      <PageFrame width="wide">
        <Alert variant="destructive">
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      </PageFrame>
    );
  }

  if (rows.length === 0) {
    return (
      <PageFrame width="wide">
        <PageHeader title="Workspaces" actions={actions} />
        <div className="flex flex-col items-center justify-center gap-4 rounded-xl border border-dashed bg-muted/20 py-16 text-center">
          <GitBranch className="size-12 text-muted-foreground/30" />
          <div>
            <h3 className="text-base font-semibold">No Workspaces Yet</h3>
            <p className="mt-1 max-w-md text-sm text-muted-foreground">
              Workspaces are the runtime unit where agents execute context-aware work.
            </p>
            {branch && (
              <p className="mt-1 text-xs text-muted-foreground">
                Current git branch:{' '}
                <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{branch}</code>
              </p>
            )}
          </div>
          <div className="flex w-full max-w-md items-center gap-2">
            <Input
              value={workspaceKeyInput}
              onChange={(event) => setWorkspaceKeyInput(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault();
                  void createWorkspaceFromInput();
                }
              }}
              placeholder={branch ? `workspace key (default ${branch})` : 'workspace key (branch/id)'}
              className="h-8"
            />
            <Button size="sm" onClick={() => void createWorkspaceFromInput()} disabled={creating}>
              {creating ? 'Creating…' : 'Create Workspace'}
            </Button>
          </div>
        </div>
      </PageFrame>
    );
  }

  const hasActiveFilters = searchQuery.trim().length > 0;

  return (
    <PageFrame width="wide" className="min-h-0">
      <PageHeader
        title="Workspaces"
        actions={actions}
        badge={
          <Badge variant="outline">
            {filteredRows.length}/{rows.length} in view
          </Badge>
        }
      />

      <Card size="sm">
        <CardContent className="space-y-3 pt-3">
          <div className="flex flex-wrap items-center gap-2">
            <div className="relative h-8 w-full min-w-[220px] flex-1 md:max-w-[340px]">
              <Search className="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={searchQuery}
                onChange={(event) => setSearchQuery(event.target.value)}
                placeholder="Search branch, feature, spec, mode, provider…"
                className="h-8 pl-8 text-xs"
              />
            </div>

            <div className="flex h-8 shrink-0 items-center gap-1 rounded-md border bg-muted/20 p-0.5">
              <span className="px-1 text-[10px] uppercase tracking-wider text-muted-foreground">Group</span>
              {GROUP_BY_OPTIONS.map((option) => (
                <Button
                  key={option.key}
                  type="button"
                  size="xs"
                  variant={groupBy === option.key ? 'secondary' : 'ghost'}
                  className="h-6 px-2 text-[11px]"
                  onClick={() => setGroupBy(option.key)}
                >
                  {option.label}
                </Button>
              ))}
            </div>

            {hasActiveFilters && (
              <Button variant="outline" size="xs" onClick={clearFilters}>
                <X className="size-3.5" />
                Clear
              </Button>
            )}
          </div>

          <div className="flex flex-wrap items-center gap-1.5 text-[10px]">
            {groupBy === 'status' ? (
              <>
                <Badge variant="outline">active {statusCounts.active}</Badge>
                <Badge variant="outline">idle {statusCounts.idle}</Badge>
                <Badge variant="outline">planned {statusCounts.planned}</Badge>
                <Badge variant="outline">review {statusCounts.review}</Badge>
                <Badge variant="outline">merged {statusCounts.merged}</Badge>
                <Badge variant="outline">archived {statusCounts.archived}</Badge>
              </>
            ) : groupBy === 'type' ? (
              <>
                <Badge variant="outline">feature {typeCounts.feature}</Badge>
                <Badge variant="outline">refactor {typeCounts.refactor}</Badge>
                <Badge variant="outline">experiment {typeCounts.experiment}</Badge>
                <Badge variant="outline">hotfix {typeCounts.hotfix}</Badge>
              </>
            ) : (
              <>
                <Badge variant="outline">releases {releaseCounts.releaseGroups}</Badge>
                <Badge variant="outline">unassigned {releaseCounts.unassigned}</Badge>
                {releaseCounts.topReleases.map(([release, count]) => (
                  <Badge key={release} variant="outline">
                    {release} {count}
                  </Badge>
                ))}
              </>
            )}
            {hasActiveFilters && (
              <span className="ml-1 text-muted-foreground">
                filtered from {rows.length} total
              </span>
            )}
          </div>
        </CardContent>
      </Card>

      {filteredRows.length === 0 ? (
        <Card size="sm">
          <CardContent className="py-10 text-center">
            <p className="text-sm font-medium">No workspaces match this search.</p>
            <p className="mt-1 text-xs text-muted-foreground">
              Adjust search terms or clear filters.
            </p>
            {hasActiveFilters && (
              <Button className="mt-3" size="sm" variant="outline" onClick={clearFilters}>
                Reset Filters
              </Button>
            )}
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-3">
          <WorkspaceLifecycleGraph
            rows={filteredRows}
            selectedBranch={selectedBranch}
            onSelectBranch={setSelectedBranch}
            groupBy={groupBy}
          />

          {detail && (
            <Card size="sm" className="min-h-0">
              <CardHeader className="pb-2">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <CardTitle className="min-w-0 truncate text-sm">{detail.branch}</CardTitle>
                  <div className="flex items-center gap-1.5">
                    <Badge variant={statusVariant(detail.status)} className="h-6 px-2 text-[11px]">
                      {detail.status}
                    </Badge>
                    {detail.isWorktree && <Badge variant="secondary">worktree</Badge>}
                    {detail.status !== 'active' && (
                      <Button
                        size="xs"
                        variant="outline"
                        onClick={() => void activateSelectedWorkspace()}
                        disabled={activating}
                      >
                        {activating ? 'Activating…' : 'Activate'}
                      </Button>
                    )}
                  </div>
                </div>
              </CardHeader>

              <CardContent className="space-y-2 pt-0 text-xs">
                <div className="flex flex-wrap items-center gap-1.5 rounded-md border bg-muted/10 px-2.5 py-1.5">
                  <span className="text-[10px] uppercase tracking-wider text-muted-foreground">Linked Context</span>
                  {linkedSpec ? (
                    <Button size="xs" variant="outline" onClick={openSpec} className="h-6 max-w-[16rem]">
                      <span className="truncate">spec {shortToken(linkedSpec.title, 24)}</span>
                      <ExternalLink className="size-3.5" />
                    </Button>
                  ) : (
                    <Badge variant="ghost" className="h-6 px-2 text-[11px]">spec none</Badge>
                  )}
                  {linkedFeature ? (
                    <Button size="xs" variant="outline" onClick={openFeature} className="h-6 max-w-[16rem]">
                      <span className="truncate">feature {shortToken(linkedFeature.title, 24)}</span>
                      <ExternalLink className="size-3.5" />
                    </Button>
                  ) : (
                    <Badge variant="ghost" className="h-6 px-2 text-[11px]">feature none</Badge>
                  )}
                  <Badge variant="outline" className="h-6 px-2 text-[11px]">
                    release {detail.releaseId ? shortToken(detail.releaseId, 20) : 'unassigned'}
                  </Badge>
                </div>

                <div className="grid gap-2 xl:grid-cols-[minmax(0,1fr)_360px]">
                  <div className="space-y-2">
                    <div className="rounded-md border bg-muted/[0.12] px-2.5 py-2 transition-colors hover:bg-muted/25">
                      <p className="text-[10px] uppercase tracking-wider text-muted-foreground">Execution Scope</p>
                      <div className="mt-1.5 grid gap-1.5 text-[11px]">
                        <div className="flex items-center justify-between rounded border bg-background/60 px-2 py-1">
                          <span className="text-muted-foreground">Spec</span>
                          <span className="truncate font-medium">
                            {linkedSpec ? shortToken(linkedSpec.file_name, 28) : 'not linked'}
                          </span>
                        </div>
                        <div className="flex items-center justify-between rounded border bg-background/60 px-2 py-1">
                          <span className="text-muted-foreground">Feature</span>
                          <span className="truncate font-medium">
                            {linkedFeature ? shortToken(linkedFeature.file_name, 28) : 'not linked'}
                          </span>
                        </div>
                        <div className="flex items-center justify-between rounded border bg-background/60 px-2 py-1">
                          <span className="text-muted-foreground">Release</span>
                          <span className="truncate font-medium">
                            {detail.releaseId ? shortToken(detail.releaseId, 28) : 'not linked'}
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="rounded-lg border bg-gradient-to-br from-primary/10 via-primary/[0.04] to-transparent px-2.5 py-2 transition-colors hover:bg-primary/[0.08]">
                    <div className="flex items-center justify-between gap-2">
                      <p className="truncate text-[10px] uppercase tracking-wider text-muted-foreground">Runtime Matrix</p>
                      <Button size="xs" variant="outline" onClick={openAgentProviders}>
                        Providers
                        <ExternalLink className="size-3.5" />
                      </Button>
                    </div>

                    <div className="mt-1.5 flex flex-wrap items-center gap-1">
                      <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
                        mode {detail.activeMode ?? workspaceUi.activeModeId ?? 'default'}
                      </Badge>
                      <Badge variant="outline" className="h-5 px-1.5 text-[10px]">
                        {detail.providers.length} provider{detail.providers.length === 1 ? '' : 's'}
                      </Badge>
                      <Badge variant={detail.isWorktree ? 'secondary' : 'outline'} className="h-5 px-1.5 text-[10px]">
                        {detail.isWorktree ? 'worktree' : 'checkout'}
                      </Badge>
                      <Badge variant="outline" className="h-5 px-1.5 text-[10px]">target local</Badge>
                      {detail.contextHash && (
                        <Badge
                          variant="ghost"
                          className="h-5 px-1.5 text-[10px] cursor-help"
                          title={`Context hash: ${detail.contextHash}`}
                        >
                          ctx {shortToken(detail.contextHash, 8)}
                        </Badge>
                      )}
                    </div>

                    <div className="mt-2 grid grid-cols-[auto_minmax(0,1fr)] gap-x-2 gap-y-1.5 text-[11px]">
                      <span className="text-muted-foreground">Branch</span>
                      <div className="flex min-w-0 items-center gap-1.5">
                        <code className="min-w-0 truncate rounded border bg-background/70 px-1.5 py-0.5 font-mono">
                          {detail.branch}
                        </code>
                        {branch === detail.branch && (
                          <Badge variant="default" className="h-5 px-1.5 text-[10px]">
                            current
                          </Badge>
                        )}
                      </div>

                      <span className="text-muted-foreground">Lifecycle</span>
                      <span className="truncate">{detail.status} · {detail.workspaceType}</span>

                      <span className="text-muted-foreground">Worktree Path</span>
                      {detail.worktreePath ? (
                        <code className="min-w-0 truncate rounded border bg-background/70 px-1.5 py-0.5 font-mono">
                          {detail.worktreePath}
                        </code>
                      ) : (
                        <span className="text-muted-foreground">repo checkout</span>
                      )}

                      <span className="text-muted-foreground">Resolved</span>
                      <span className="truncate text-muted-foreground">{new Date(detail.resolvedAt).toLocaleString()}</span>

                      <span className="text-muted-foreground">Last Active</span>
                      <span className="truncate text-muted-foreground">
                        {detail.lastActivatedAt ? new Date(detail.lastActivatedAt).toLocaleString() : 'never'}
                      </span>

                      <span className="text-muted-foreground">Workspace ID</span>
                      <code className="min-w-0 truncate rounded border bg-muted/30 px-1.5 py-0.5 font-mono">
                        {detail.id}
                      </code>
                    </div>

                    <div className="mt-2 flex flex-wrap gap-1 rounded-md border bg-background/65 px-2 py-1.5">
                      {detail.providers.length > 0 ? (
                        detail.providers.map((provider) => (
                          <Badge key={provider} variant="secondary" className="h-5 px-1.5 text-[10px]">
                            {provider}
                          </Badge>
                        ))
                      ) : (
                        <span className="text-[11px] italic text-muted-foreground">No provider snapshot captured yet.</span>
                      )}
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}
    </PageFrame>
  );
}
