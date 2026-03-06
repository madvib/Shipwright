import { useMemo, useState } from 'react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import { Badge } from '@ship/ui';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@ship/ui';
import { cn } from '@/lib/utils';

export type WorkspaceGraphStatus =
  | 'planned'
  | 'active'
  | 'idle'
  | 'review'
  | 'merged'
  | 'archived';

export type WorkspaceGroupBy = 'status' | 'type' | 'release';

export interface WorkspaceGraphRow {
  branch: string;
  status: WorkspaceGraphStatus;
  workspaceType: 'feature' | 'refactor' | 'experiment' | 'hotfix';
  specId: string | null;
  featureId: string | null;
  releaseId: string | null;
  providers: string[];
  isWorktree: boolean;
  activeMode: string | null;
  contextHash?: string | null;
}

const STATUS_ORDER: WorkspaceGraphStatus[] = [
  'planned',
  'active',
  'idle',
  'review',
  'merged',
  'archived',
];

const TYPE_ORDER: WorkspaceGraphRow['workspaceType'][] = [
  'feature',
  'refactor',
  'experiment',
  'hotfix',
];

const UNASSIGNED_RELEASE_KEY = '__unassigned_release__';

const STATUS_META: Record<
  WorkspaceGraphStatus,
  { label: string; tone: string; rail: string }
> = {
  planned: {
    label: 'Planned',
    tone: 'border-sky-300 bg-sky-50 text-sky-900 dark:border-sky-500/35 dark:bg-sky-500/12 dark:text-sky-200',
    rail: 'bg-sky-600 dark:bg-sky-500/80',
  },
  active: {
    label: 'Active',
    tone: 'border-emerald-300 bg-emerald-50 text-emerald-900 dark:border-emerald-500/35 dark:bg-emerald-500/12 dark:text-emerald-200',
    rail: 'bg-emerald-600 dark:bg-emerald-500/80',
  },
  idle: {
    label: 'Idle',
    tone: 'border-zinc-300 bg-zinc-50 text-zinc-900 dark:border-zinc-500/35 dark:bg-zinc-500/12 dark:text-zinc-200',
    rail: 'bg-zinc-500 dark:bg-zinc-400/75',
  },
  review: {
    label: 'Review',
    tone: 'border-amber-300 bg-amber-50 text-amber-900 dark:border-amber-500/35 dark:bg-amber-500/12 dark:text-amber-200',
    rail: 'bg-amber-600 dark:bg-amber-400/85',
  },
  merged: {
    label: 'Merged',
    tone: 'border-violet-300 bg-violet-50 text-violet-900 dark:border-violet-500/35 dark:bg-violet-500/12 dark:text-violet-200',
    rail: 'bg-violet-600 dark:bg-violet-400/85',
  },
  archived: {
    label: 'Archived',
    tone: 'border-slate-300 bg-slate-50 text-slate-900 dark:border-slate-500/35 dark:bg-slate-500/12 dark:text-slate-200',
    rail: 'bg-slate-600 dark:bg-slate-400/80',
  },
};

const TYPE_META: Record<
  WorkspaceGraphRow['workspaceType'],
  { label: string; tone: string; rail: string }
> = {
  feature: {
    label: 'Feature',
    tone: 'border-cyan-300 bg-cyan-50 text-cyan-900 dark:border-cyan-500/35 dark:bg-cyan-500/12 dark:text-cyan-200',
    rail: 'bg-cyan-600 dark:bg-cyan-500/80',
  },
  refactor: {
    label: 'Refactor',
    tone: 'border-indigo-300 bg-indigo-50 text-indigo-900 dark:border-indigo-500/35 dark:bg-indigo-500/12 dark:text-indigo-200',
    rail: 'bg-indigo-600 dark:bg-indigo-500/80',
  },
  experiment: {
    label: 'Experiment',
    tone: 'border-fuchsia-300 bg-fuchsia-50 text-fuchsia-900 dark:border-fuchsia-500/35 dark:bg-fuchsia-500/12 dark:text-fuchsia-200',
    rail: 'bg-fuchsia-600 dark:bg-fuchsia-500/85',
  },
  hotfix: {
    label: 'Hotfix',
    tone: 'border-rose-300 bg-rose-50 text-rose-900 dark:border-rose-500/35 dark:bg-rose-500/12 dark:text-rose-200',
    rail: 'bg-rose-600 dark:bg-rose-500/85',
  },
};

const RELEASE_TONES: Array<{ tone: string; rail: string }> = [
  {
    tone: 'border-blue-300 bg-blue-50 text-blue-900 dark:border-blue-500/35 dark:bg-blue-500/12 dark:text-blue-200',
    rail: 'bg-blue-600 dark:bg-blue-500/80',
  },
  {
    tone: 'border-teal-300 bg-teal-50 text-teal-900 dark:border-teal-500/35 dark:bg-teal-500/12 dark:text-teal-200',
    rail: 'bg-teal-600 dark:bg-teal-500/80',
  },
  {
    tone: 'border-orange-300 bg-orange-50 text-orange-900 dark:border-orange-500/35 dark:bg-orange-500/12 dark:text-orange-200',
    rail: 'bg-orange-600 dark:bg-orange-500/80',
  },
  {
    tone: 'border-purple-300 bg-purple-50 text-purple-900 dark:border-purple-500/35 dark:bg-purple-500/12 dark:text-purple-200',
    rail: 'bg-purple-600 dark:bg-purple-500/80',
  },
  {
    tone: 'border-lime-300 bg-lime-50 text-lime-900 dark:border-lime-500/35 dark:bg-lime-500/12 dark:text-lime-200',
    rail: 'bg-lime-600 dark:bg-lime-500/80',
  },
];

interface WorkspaceColumn {
  id: string;
  label: string;
  tone: string;
  rail: string;
  rows: WorkspaceGraphRow[];
  isSelected: boolean;
}

interface WorkspaceLifecycleGraphProps {
  rows: WorkspaceGraphRow[];
  selectedBranch: string | null;
  onSelectBranch: (branch: string) => void;
  groupBy?: WorkspaceGroupBy;
}

function shortToken(value: string, size = 8): string {
  return value.length <= size ? value : `${value.slice(0, size)}…`;
}

export function WorkspaceLifecycleGraph({
  rows,
  selectedBranch,
  onSelectBranch,
  groupBy = 'status',
}: WorkspaceLifecycleGraphProps) {
  const groupedByStatus = useMemo(() => {
    const grouped: Record<WorkspaceGraphStatus, WorkspaceGraphRow[]> = {
      planned: [],
      active: [],
      idle: [],
      review: [],
      merged: [],
      archived: [],
    };
    for (const row of rows) grouped[row.status].push(row);
    return grouped;
  }, [rows]);

  const groupedByType = useMemo(() => {
    const grouped: Record<WorkspaceGraphRow['workspaceType'], WorkspaceGraphRow[]> = {
      feature: [],
      refactor: [],
      experiment: [],
      hotfix: [],
    };
    for (const row of rows) grouped[row.workspaceType].push(row);
    return grouped;
  }, [rows]);

  const groupedByRelease = useMemo(() => {
    const grouped = new Map<string, WorkspaceGraphRow[]>();
    for (const row of rows) {
      const key = row.releaseId?.trim() ? row.releaseId.trim() : UNASSIGNED_RELEASE_KEY;
      const list = grouped.get(key) ?? [];
      list.push(row);
      grouped.set(key, list);
    }
    if (!grouped.has(UNASSIGNED_RELEASE_KEY)) {
      grouped.set(UNASSIGNED_RELEASE_KEY, []);
    }
    return grouped;
  }, [rows]);

  const releaseKeys = useMemo(() => {
    const keys = Array.from(groupedByRelease.keys());
    keys.sort((a, b) => {
      if (a === UNASSIGNED_RELEASE_KEY) return 1;
      if (b === UNASSIGNED_RELEASE_KEY) return -1;
      return a.localeCompare(b);
    });
    return keys;
  }, [groupedByRelease]);

  const selected = rows.find((row) => row.branch === selectedBranch) ?? null;
  const selectedReleaseKey = selected?.releaseId?.trim() || UNASSIGNED_RELEASE_KEY;
  const [collapsedColumns, setCollapsedColumns] = useState<Record<string, boolean>>({});

  const columns = useMemo<WorkspaceColumn[]>(() => {
    if (groupBy === 'type') {
      return TYPE_ORDER.map((key) => {
        const meta = TYPE_META[key];
        return {
          id: `type:${key}`,
          label: meta.label,
          tone: meta.tone,
          rail: meta.rail,
          rows: groupedByType[key],
          isSelected: selected?.workspaceType === key,
        };
      });
    }

    if (groupBy === 'release') {
      return releaseKeys.map((releaseKey, index) => {
        const toneMeta = RELEASE_TONES[index % RELEASE_TONES.length];
        return {
          id: `release:${releaseKey}`,
          label: releaseKey === UNASSIGNED_RELEASE_KEY ? 'Unassigned' : releaseKey,
          tone: toneMeta.tone,
          rail: toneMeta.rail,
          rows: groupedByRelease.get(releaseKey) ?? [],
          isSelected: selectedReleaseKey === releaseKey,
        };
      });
    }

    return STATUS_ORDER.map((key) => {
      const meta = STATUS_META[key];
      return {
        id: `status:${key}`,
        label: meta.label,
        tone: meta.tone,
        rail: meta.rail,
        rows: groupedByStatus[key],
        isSelected: selected?.status === key,
      };
    });
  }, [
    groupBy,
    groupedByType,
    groupedByRelease,
    groupedByStatus,
    releaseKeys,
    selected?.status,
    selected?.workspaceType,
    selectedReleaseKey,
  ]);

  const title =
    groupBy === 'type'
      ? 'Workspaces by Type'
      : groupBy === 'release'
        ? 'Workspaces by Release'
        : 'Workspaces by Status';

  const description =
    groupBy === 'type'
      ? 'Cards grouped by workspace type.'
      : groupBy === 'release'
        ? 'Cards grouped by linked release.'
        : 'Cards grouped by lifecycle status.';

  const isCollapsed = (id: string, count: number) => {
    if (id in collapsedColumns) return !!collapsedColumns[id];
    return count === 0;
  };

  const toggleColumn = (id: string, count: number) => {
    setCollapsedColumns((prev) => ({
      ...prev,
      [id]: !isCollapsed(id, count),
    }));
  };

  const rowSubtitle = (row: WorkspaceGraphRow) => {
    const base =
      groupBy === 'type'
        ? row.status
        : groupBy === 'release'
          ? `${row.status} · ${row.workspaceType}`
          : row.workspaceType;
    return `${base}${row.specId ? ` · ${row.specId}` : row.featureId ? ` · ${row.featureId}` : ''}`;
  };

  return (
    <Card size="sm" className="min-w-0">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm">{title}</CardTitle>
        <CardDescription className="text-xs">{description}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex items-stretch gap-2.5 overflow-x-auto pb-1">
          {columns.map((column) => {
            const collapsed = isCollapsed(column.id, column.rows.length);

            if (collapsed) {
              return (
                <div
                  key={column.id}
                  role="button"
                  tabIndex={0}
                  className={cn(
                    'flex w-16 shrink-0 cursor-pointer flex-col items-center rounded-lg border py-4 transition-all hover:w-20',
                    column.tone
                  )}
                  onClick={() => toggleColumn(column.id, column.rows.length)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter' || event.key === ' ') {
                      event.preventDefault();
                      toggleColumn(column.id, column.rows.length);
                    }
                  }}
                  title={`Expand ${column.label}`}
                >
                  <span className="mb-3 rounded border border-border/70 bg-background/90 px-2 py-1 text-sm font-bold leading-none text-foreground">
                    {column.rows.length}
                  </span>
                  <span
                    className="flex-1 text-sm font-semibold tracking-wider text-foreground/85"
                    style={{ writingMode: 'vertical-rl', transform: 'rotate(180deg)' }}
                  >
                    {column.label}
                  </span>
                  <ChevronDown className="mt-3 size-4 text-foreground/75" />
                </div>
              );
            }

            return (
              <div key={column.id} className="flex min-w-[300px] flex-1 flex-col">
                <div
                  className={cn(
                    'flex h-full min-h-[20rem] max-h-[calc(100vh-17rem)] flex-col rounded-lg border bg-muted/20',
                    column.isSelected && 'ring-1 ring-primary/35'
                  )}
                >
                  <button
                    type="button"
                    onClick={() => toggleColumn(column.id, column.rows.length)}
                    className="flex w-full items-center justify-between gap-2 border-b px-3.5 py-3 text-left transition-colors hover:bg-muted/25"
                    title={`Collapse ${column.label}`}
                  >
                    <div className="flex min-w-0 items-center gap-1.5">
                      <span
                        className={cn(
                          'size-3.5 shrink-0 rounded-full border border-background shadow-sm',
                          column.isSelected ? 'bg-primary' : column.rail
                        )}
                      />
                      <p className="truncate text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                        {column.label}
                      </p>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <Badge
                        variant="outline"
                        className={cn('h-7 min-w-9 justify-center px-2.5 text-sm font-bold leading-none', column.tone)}
                      >
                        {column.rows.length}
                      </Badge>
                      <ChevronUp className="size-4 text-muted-foreground" />
                    </div>
                  </button>

                  <div className="min-h-0 flex-1 space-y-2.5 overflow-auto p-2.5">
                    {column.rows.length === 0 ? (
                      <p className="px-1 py-2 text-sm italic text-muted-foreground/70">
                        no workspaces
                      </p>
                    ) : (
                      column.rows.map((row) => {
                        const isSelected = row.branch === selectedBranch;
                        return (
                          <button
                            key={row.branch}
                            type="button"
                            onClick={() => onSelectBranch(row.branch)}
                            className={cn(
                              'relative w-full overflow-hidden rounded-lg border px-3 py-2.5 text-left transition-all',
                              isSelected
                                ? 'border-primary/55 bg-primary/10 shadow-[inset_0_1px_0_rgba(255,255,255,0.08)]'
                                : 'bg-background/65 hover:bg-muted/45'
                            )}
                          >
                            <span
                              className={cn(
                                'absolute inset-y-0 left-0 w-0.5',
                                isSelected ? 'bg-primary' : 'bg-border'
                              )}
                            />
                            <div className="flex items-start justify-between gap-2 pl-1">
                              <p className="truncate text-[15px] font-semibold leading-tight">{row.branch}</p>
                              {row.status === 'active' && (
                                <Badge variant="default" className="h-5 px-1.5 text-[10px]">
                                  live
                                </Badge>
                              )}
                            </div>
                            <div className="mt-1.5 flex flex-wrap gap-1 pl-1">
                              <Badge variant="outline" className="h-5 px-1.5 text-[10px]">
                                {row.workspaceType}
                              </Badge>
                              <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
                                {row.activeMode ?? 'default'}
                              </Badge>
                              <Badge variant="outline" className="h-5 px-1.5 text-[10px]">
                                {row.providers.length} provider{row.providers.length === 1 ? '' : 's'}
                              </Badge>
                              <Badge
                                variant={row.isWorktree ? 'secondary' : 'ghost'}
                                className="h-5 px-1.5 text-[10px]"
                              >
                                {row.isWorktree ? 'worktree' : 'checkout'}
                              </Badge>
                              {row.releaseId && (
                                <Badge variant="ghost" className="h-5 max-w-[10.5rem] px-1.5 text-[10px]">
                                  rel {shortToken(row.releaseId, 14)}
                                </Badge>
                              )}
                              {row.contextHash && (
                                <Badge
                                  variant="ghost"
                                  className="h-5 px-1.5 text-[10px] cursor-help"
                                  title={`Context hash: ${row.contextHash}`}
                                >
                                  ctx {shortToken(row.contextHash, 7)}
                                </Badge>
                              )}
                            </div>
                            <p className="truncate pt-1 pl-1 text-[11px] text-muted-foreground">
                              {rowSubtitle(row)}
                            </p>
                          </button>
                        );
                      })
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>

        {selected && (
          <div className="grid gap-1 rounded-md border bg-muted/15 px-2.5 py-2 text-[11px] md:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)]">
            <p className="truncate">
              <span className="text-muted-foreground">Selected:</span>{' '}
              <span className="font-medium">{selected.branch}</span>
            </p>
            <p className="hidden text-muted-foreground md:block">|</p>
            <p className="truncate text-muted-foreground">
              {selected.status} · {selected.workspaceType} · mode: {selected.activeMode ?? 'default'}
              {' · '}providers: {selected.providers.length}
              {selected.featureId ? ` · feature ${selected.featureId}` : ''}
              {selected.releaseId ? ` · release ${selected.releaseId}` : ''}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
