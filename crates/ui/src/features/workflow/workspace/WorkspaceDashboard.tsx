import { useEffect, useMemo, useState } from 'react';
import { Clock3, GitBranch, Minus, Plus } from 'lucide-react';
import { Badge, Button, Tooltip, TooltipContent, TooltipTrigger } from '@ship/ui';
import {
  type BranchDetailSummary,
  type WorkspaceFileChange,
  type WorkspaceGitStatusSummary,
  type WorkspaceProviderMatrix,
  type WorkspaceRepairReport,
  type WorkspaceSessionInfo,
} from '@/lib/platform/tauri/commands';
import { WorkspaceGraphStatus } from '../components/WorkspaceLifecycleGraph';
import { WorkspaceProviderPreflightSection } from './dashboard/WorkspaceProviderPreflightSection.tsx';
import { WorkspaceSessionSection } from './dashboard/WorkspaceSessionSection.tsx';
import { WorkspaceStatusCard } from './dashboard/WorkspaceStatusCard.tsx';
import { WorkspaceRow } from './types';
import { ProviderInfo } from '@/bindings';
import { cn } from '@/lib/utils';

interface WorkspaceDashboardProps {
  detail: WorkspaceRow | null;
  statusVariant: (status: WorkspaceGraphStatus) => 'default' | 'secondary' | 'outline';
  linkedFeature: any;
  linkedRelease: any;
  linkFeatureId: string | null;
  setLinkFeatureId: (id: string | null) => void;
  linkReleaseId: string | null;
  setLinkReleaseId: (id: string | null) => void;
  featureLinkOptions: any[];
  releaseLinkOptions: any[];
  updatingLinks: boolean;
  onUpdateLinks: (featureId: string | null, releaseId: string | null) => void;
  onOpenFeature: () => void;
  onOpenRelease: () => void;
  activeSession: WorkspaceSessionInfo | null;
  recentSessions: WorkspaceSessionInfo[];
  startingSession: boolean;
  endingSession: boolean;
  restartingSession: boolean;
  onStartSession: () => void;
  onEndSession: () => void;
  onRestartSession: () => void;
  sessionGoalInput: string;
  setSessionGoalInput: (val: string) => void;
  sessionSummaryInput: string;
  setSessionSummaryInput: (val: string) => void;
  providerMatrix: WorkspaceProviderMatrix | null;
  providerInfos: ProviderInfo[];
  workspaceChanges: WorkspaceFileChange[];
  workspaceGitSummary: WorkspaceGitStatusSummary | null;
  sessionProvider: string | null;
  setSessionProvider: (provider: string | null) => void;
  onSync: () => void;
  syncing: boolean;
  onArchive: () => void;
  archiving: boolean;
  onRepair: () => void;
  repairing: boolean;
  lastRepairReport: WorkspaceRepairReport | null;
  loading: boolean;
  onRefreshProviders: () => void;
  onCreateFromBranch: () => void;
  creatingWorkspace: boolean;
  branchDetail: BranchDetailSummary | null;
  branchDiffPath: string | null;
  setBranchDiffPath: (path: string | null) => void;
  branchFileDiff: string;
  loadingBranchDiff: boolean;
}

function changeStatusColors(status: string): string {
  const key = status.trim().toUpperCase();
  if (key.startsWith('A')) return 'bg-emerald-500/10 text-emerald-700 border-emerald-500/30';
  if (key.startsWith('D')) return 'bg-red-500/10 text-red-700 border-red-500/30';
  if (key.startsWith('R')) return 'bg-sky-500/10 text-sky-700 border-sky-500/30';
  if (key.startsWith('M')) return 'bg-amber-500/10 text-amber-700 border-amber-500/30';
  return 'bg-muted/40 text-muted-foreground border-border';
}

function diffLineClass(line: string): string {
  if (line.startsWith('@@')) return 'bg-sky-500/8 text-sky-700 dark:text-sky-300';
  if (line.startsWith('+') && !line.startsWith('+++')) return 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300';
  if (line.startsWith('-') && !line.startsWith('---')) return 'bg-red-500/10 text-red-700 dark:text-red-300';
  if (line.startsWith('diff --git') || line.startsWith('index ') || line.startsWith('---') || line.startsWith('+++')) {
    return 'bg-muted/40 text-muted-foreground';
  }
  return 'text-foreground';
}

export function WorkspaceDashboard({
  detail,
  statusVariant,
  linkedFeature,
  linkedRelease,
  linkFeatureId,
  setLinkFeatureId,
  linkReleaseId,
  setLinkReleaseId,
  featureLinkOptions,
  releaseLinkOptions,
  updatingLinks,
  onUpdateLinks,
  onOpenFeature,
  onOpenRelease,
  activeSession,
  recentSessions,
  startingSession,
  endingSession,
  restartingSession,
  onStartSession,
  onEndSession,
  onRestartSession,
  sessionGoalInput,
  setSessionGoalInput,
  sessionSummaryInput,
  setSessionSummaryInput,
  providerMatrix,
  providerInfos,
  workspaceChanges,
  workspaceGitSummary,
  sessionProvider,
  setSessionProvider,
  onSync,
  syncing,
  onArchive,
  archiving,
  onRepair,
  repairing,
  lastRepairReport,
  loading,
  onRefreshProviders,
  onCreateFromBranch,
  creatingWorkspace,
  branchDetail,
  branchDiffPath,
  setBranchDiffPath,
  branchFileDiff,
  loadingBranchDiff,
}: WorkspaceDashboardProps) {
  const [showBranchDetail, setShowBranchDetail] = useState(false);

  useEffect(() => {
    setShowBranchDetail(false);
  }, [detail?.branch]);

  const parsedDiffLines = useMemo(() => branchFileDiff.split('\n'), [branchFileDiff]);

  const renderBranchDetailPanel = (showConfigureCta: boolean) => {
    if (!branchDetail) return null;

    return (
      <section className="rounded-xl border bg-card p-3 shadow-sm">
        <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <GitBranch className="size-4 text-muted-foreground" />
              <h3 className="break-all text-sm font-semibold text-foreground">{branchDetail.branch}</h3>
              <Badge variant={branchDetail.has_workspace ? 'secondary' : 'outline'}>
                {branchDetail.has_workspace ? 'Managed' : 'Unmanaged'}
              </Badge>
            </div>
            <p className="mt-0.5 text-[11px] text-muted-foreground">compare base: {branchDetail.base_branch}</p>
          </div>
          {showConfigureCta && !branchDetail.has_workspace && (
            <Button size="sm" onClick={onCreateFromBranch} disabled={creatingWorkspace}>
              {creatingWorkspace ? 'Creating...' : 'Configure Workspace'}
            </Button>
          )}
        </div>

        <div className="mb-2 grid grid-cols-2 gap-2 md:grid-cols-5">
          <div className="rounded border bg-sky-500/10 px-2 py-1.5 text-[11px] text-sky-700 dark:text-sky-300">
            files <span className="font-semibold">{branchDetail.touched_files}</span>
          </div>
          <div className="rounded border bg-emerald-500/10 px-2 py-1.5 text-[11px] text-emerald-700 dark:text-emerald-300">
            +{branchDetail.insertions}
          </div>
          <div className="rounded border bg-red-500/10 px-2 py-1.5 text-[11px] text-red-700 dark:text-red-300">
            -{branchDetail.deletions}
          </div>
          <div className="rounded border bg-muted/20 px-2 py-1.5 text-[11px]">ahead {branchDetail.ahead}</div>
          <div className="rounded border bg-muted/20 px-2 py-1.5 text-[11px]">behind {branchDetail.behind}</div>
        </div>

        {branchDetail.changes.length === 0 ? (
          <p className="text-[11px] text-muted-foreground">No file differences against base branch.</p>
        ) : (
        <div className="grid grid-cols-1 gap-2 xl:grid-cols-[300px_minmax(0,1fr)]">
            <div className="max-h-[540px] flex flex-col gap-2">
              <div className="flex flex-wrap items-center gap-2 px-1 py-1 rounded border bg-muted/5">
                {[
                  { label: 'A', color: 'bg-emerald-500/10 text-emerald-700', full: 'Added' },
                  { label: 'D', color: 'bg-red-500/10 text-red-700', full: 'Deleted' },
                  { label: 'M', color: 'bg-amber-500/10 text-amber-700', full: 'Modified' },
                  { label: 'R', color: 'bg-sky-500/10 text-sky-700', full: 'Renamed' },
                ].map((item) => (
                  <div key={item.label} className="flex items-center gap-1">
                    <span className={cn('rounded border px-1 font-mono text-[9px] font-bold', item.color)} title={item.full}>
                      {item.label}
                    </span>
                  </div>
                ))}
              </div>
              <div className="flex-1 overflow-y-auto rounded-md border bg-muted/10 p-1.5">
              {branchDetail.changes.map((change) => {
                const selected = branchDiffPath === change.path;
                return (
                  <button
                    type="button"
                    key={`${change.status}-${change.path}`}
                    className={cn(
                      'mb-1 w-full rounded-md border px-2 py-1.5 text-left text-[11px] transition-colors',
                      selected
                        ? 'border-primary/50 bg-primary/10'
                        : 'border-border/40 bg-background hover:bg-muted/50',
                    )}
                    onClick={() => setBranchDiffPath(change.path)}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className={cn('rounded border px-1 font-mono text-[10px]', changeStatusColors(change.status))}>
                        {change.status || '--'}
                      </span>
                      <span className="flex items-center gap-2">
                        <span className="inline-flex items-center gap-1 text-emerald-700 dark:text-emerald-300">
                          <Plus className="size-3" />
                          {change.insertions}
                        </span>
                        <span className="inline-flex items-center gap-1 text-red-700 dark:text-red-300">
                          <Minus className="size-3" />
                          {change.deletions}
                        </span>
                      </span>
                    </div>
                    <p className="mt-1 truncate text-foreground">{change.path}</p>
                  </button>
                );
              })}
              </div>
            </div>

            <div className="min-h-[420px] overflow-auto rounded-md border bg-background/70 p-2">
              {loadingBranchDiff ? (
                <p className="text-xs text-muted-foreground">Loading diff...</p>
              ) : branchDiffPath ? (
                <div>
                  <p className="mb-2 truncate rounded bg-muted/30 px-2 py-1 text-[11px] font-semibold text-muted-foreground">
                    {branchDiffPath}
                  </p>
                  <div className="space-y-[1px] font-mono text-[11px] leading-relaxed">
                    {parsedDiffLines.length > 200 && branchDetail.changes.find(c => c.path === branchDiffPath)?.status === 'D' ? (
                        <div className="py-12 text-center bg-red-500/5 rounded border border-red-500/20">
                           <p className="text-muted-foreground text-xs mb-3">Large deleted file ({parsedDiffLines.length} lines)</p>
                           <Button
                             size="xs"
                             variant="outline"
                             onClick={(e) => {
                                e.currentTarget.parentElement?.classList.add('hidden');
                                e.currentTarget.parentElement?.nextElementSibling?.classList.remove('hidden');
                             }}
                            >
                             Show Diff
                           </Button>
                        </div>
                    ) : null}
                    <div className={cn(parsedDiffLines.length > 200 && branchDetail.changes.find(c => c.path === branchDiffPath)?.status === 'D' ? 'hidden' : '')}>
                        {parsedDiffLines.map((line, index) => (
                        <div key={`${index}-${line.slice(0, 12)}`} className={cn('grid grid-cols-[44px_minmax(0,1fr)] gap-2 px-1', diffLineClass(line))}>
                            <span className="select-none text-right text-[10px] text-muted-foreground/70">{index + 1}</span>
                            <span className="whitespace-pre-wrap break-words">{line || ' '}</span>
                        </div>
                        ))}
                    </div>
                  </div>
                </div>
              ) : (
                <p className="text-xs text-muted-foreground">Select a file to inspect changes.</p>
              )}
            </div>
          </div>
        )}
      </section>
    );
  };

  if (!detail) {
    if (branchDetail) {
      return <div className="space-y-6 p-4 md:p-6 lg:p-8">{renderBranchDetailPanel(true)}</div>;
    }

    return (
      <div className="flex h-full min-h-[70vh] items-center justify-center p-8 text-center bg-muted/5 rounded-xl border border-dashed border-border/60 mx-4 md:mx-6 lg:mx-8 my-4 md:my-6 lg:my-8">
        <div className="max-w-md">
          <div className="relative mx-auto mb-6 flex size-16 items-center justify-center">
            <div className="absolute inset-0 animate-pulse rounded-full bg-primary/10" />
            <Clock3 className="relative size-8 text-primary/40" />
          </div>
          <p className="text-base font-bold text-foreground">No workspace configured yet</p>
          <p className="mt-2 text-sm text-muted-foreground leading-relaxed">
            Connect this project to a git branch to unlock session tracking, runtime context, and workspace-aware AI assistants.
          </p>
          <div className="mt-8 flex items-center justify-center gap-2">
            <Button
              size="lg"
              onClick={onCreateFromBranch}
              disabled={creatingWorkspace}
              className="px-8 shadow-lg shadow-primary/20"
            >
              {creatingWorkspace ? 'Creating Workspace...' : 'Create From Branch'}
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-4 md:p-6 lg:p-8">
      <WorkspaceStatusCard
        detail={detail!}
        statusVariant={statusVariant}
        linkedFeature={linkedFeature}
        linkedRelease={linkedRelease}
        linkFeatureId={linkFeatureId}
        setLinkFeatureId={setLinkFeatureId}
        linkReleaseId={linkReleaseId}
        setLinkReleaseId={setLinkReleaseId}
        featureLinkOptions={featureLinkOptions}
        releaseLinkOptions={releaseLinkOptions}
        updatingLinks={updatingLinks}
        onUpdateLinks={onUpdateLinks}
        onOpenFeature={onOpenFeature}
        onOpenRelease={onOpenRelease}
        onSync={onSync}
        syncing={syncing}
        onArchive={onArchive}
        archiving={archiving}
        onRepair={onRepair}
        repairing={repairing}
        lastRepairReport={lastRepairReport}
        providerMatrix={providerMatrix}
      />

      <div className="grid grid-cols-1 gap-3 xl:grid-cols-[minmax(0,1fr)_360px]">
        <div className="space-y-3">
          <section className="rounded-xl border bg-card p-3 shadow-sm">
            <div className="mb-2 flex items-center justify-between gap-3">
              <div className="flex items-center gap-2">
                <p className="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">Git Snapshot</p>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Badge variant="outline" className="h-5 px-1.5 text-[9px]">
                      {workspaceGitSummary?.upstream ?? 'no upstream'}
                    </Badge>
                  </TooltipTrigger>
                  <TooltipContent>Upstream used for ahead/behind and diff stats</TooltipContent>
                </Tooltip>
              </div>
              <Button
                size="xs"
                variant="outline"
                className="h-7"
                onClick={() => setShowBranchDetail((previous) => !previous)}
                disabled={!branchDetail}
              >
                {showBranchDetail ? 'Hide Branch Detail' : 'View Branch Detail'}
              </Button>
            </div>
            <div className="mb-2 grid grid-cols-2 gap-2 md:grid-cols-5">
              <div className="rounded border bg-sky-500/10 px-2 py-1.5 text-[11px] text-sky-700 dark:text-sky-300">
                touched <span className="font-semibold">{workspaceGitSummary?.touched_files ?? workspaceChanges.length}</span>
              </div>
              <div className="rounded border bg-emerald-500/10 px-2 py-1.5 text-[11px] text-emerald-700 dark:text-emerald-300">
                +{workspaceGitSummary?.insertions ?? 0}
              </div>
              <div className="rounded border bg-red-500/10 px-2 py-1.5 text-[11px] text-red-700 dark:text-red-300">
                -{workspaceGitSummary?.deletions ?? 0}
              </div>
              <div className="rounded border bg-muted/20 px-2 py-1.5 text-[11px]">ahead {workspaceGitSummary?.ahead ?? 0}</div>
              <div className="rounded border bg-muted/20 px-2 py-1.5 text-[11px]">behind {workspaceGitSummary?.behind ?? 0}</div>
            </div>

            {workspaceChanges.length === 0 ? (
              <p className="text-[11px] text-muted-foreground">No working tree changes detected.</p>
            ) : (
              <div className="max-h-48 overflow-y-auto rounded-md border bg-muted/10 p-1.5">
                {workspaceChanges.slice(0, 24).map((change) => (
                  <div
                    key={`${change.status}-${change.path}`}
                    className="mb-1 flex items-center gap-2 rounded border border-border/50 bg-background/70 px-2 py-1 text-[11px]"
                  >
                    <span className={cn('rounded border px-1 font-mono text-[10px]', changeStatusColors(change.status))}>
                      {change.status || '--'}
                    </span>
                    <span className="truncate text-muted-foreground">{change.path}</span>
                  </div>
                ))}
              </div>
            )}
          </section>

          {showBranchDetail && renderBranchDetailPanel(false)}
        </div>

        <div className="space-y-3">
          <WorkspaceSessionSection
            activeSession={activeSession}
            recentSessions={recentSessions}
            startingSession={startingSession}
            endingSession={endingSession}
            restartingSession={restartingSession}
            onStartSession={onStartSession}
            onEndSession={onEndSession}
            onRestartSession={onRestartSession}
            sessionGoalInput={sessionGoalInput}
            setSessionGoalInput={setSessionGoalInput}
            sessionSummaryInput={sessionSummaryInput}
            setSessionSummaryInput={setSessionSummaryInput}
            providerMatrix={providerMatrix}
            sessionProvider={sessionProvider}
            setSessionProvider={setSessionProvider}
            currentConfigGeneration={detail?.configGeneration}
          />

          <WorkspaceProviderPreflightSection
            providerMatrix={providerMatrix}
            providerInfos={providerInfos}
            loading={loading}
            onRefresh={onRefreshProviders}
          />
        </div>
      </div>
    </div>
  );
}
