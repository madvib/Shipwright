import { Clock3 } from 'lucide-react';
import {
  type WorkspaceProviderMatrix,
  type WorkspaceRepairReport,
  type WorkspaceSessionInfo,
  type WorkspaceFileChange,
  type WorkspaceGitStatusSummary,
} from '@/lib/platform/tauri/commands';
import { WorkspaceGraphStatus } from '../components/WorkspaceLifecycleGraph';
import { WorkspaceLinksSection } from './dashboard/WorkspaceLinksSection.tsx';
import { WorkspaceProviderPreflightSection } from './dashboard/WorkspaceProviderPreflightSection.tsx';
import { WorkspaceSessionSection } from './dashboard/WorkspaceSessionSection.tsx';
import { WorkspaceStatusCard } from './dashboard/WorkspaceStatusCard.tsx';
import { WorkspaceRow } from './types';
import { ProviderInfo } from '@/bindings';

interface WorkspaceDashboardProps {
  detail: WorkspaceRow | null;
  statusVariant: (status: WorkspaceGraphStatus) => 'default' | 'secondary' | 'outline';
  linkedFeature: any;
  linkedSpec: any;
  linkedRelease: any;
  linkFeatureId: string;
  setLinkFeatureId: (id: string) => void;
  linkSpecId: string;
  setLinkSpecId: (id: string) => void;
  linkReleaseId: string;
  setLinkReleaseId: (id: string) => void;
  featureLinkOptions: any[];
  specLinkOptions: any[];
  releaseLinkOptions: any[];
  updatingLinks: boolean;
  onApplyLinks: () => void;
  onOpenFeature: () => void;
  onOpenSpec: () => void;
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
  NO_LINK_VALUE: string;
  onSync: () => void;
  syncing: boolean;
  onActivate: () => void;
  activating: boolean;
  onArchive: () => void;
  archiving: boolean;
  onRepair: () => void;
  repairing: boolean;
  lastRepairReport: WorkspaceRepairReport | null;
  loading: boolean;
  onRefreshProviders: () => void;
}

export function WorkspaceDashboard({
  detail,
  statusVariant,
  linkedFeature,
  linkedSpec,
  linkedRelease,
  linkFeatureId,
  setLinkFeatureId,
  linkSpecId,
  setLinkSpecId,
  linkReleaseId,
  setLinkReleaseId,
  featureLinkOptions,
  specLinkOptions,
  releaseLinkOptions,
  updatingLinks,
  onApplyLinks,
  onOpenFeature,
  onOpenSpec,
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
  NO_LINK_VALUE,
  onSync,
  syncing,
  onActivate,
  activating,
  onArchive,
  archiving,
  onRepair,
  repairing,
  lastRepairReport,
  loading,
  onRefreshProviders,
}: WorkspaceDashboardProps) {
  if (!detail) {
    return (
      <div className="flex h-full min-h-[20rem] items-center justify-center p-8 text-center">
        <div>
          <Clock3 className="mx-auto size-6 text-muted-foreground/40" />
          <p className="mt-2 text-sm text-muted-foreground">
            Select a workspace to view command-center details.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4 p-4">
      <WorkspaceStatusCard
        detail={detail}
        statusVariant={statusVariant}
        onSync={onSync}
        syncing={syncing}
        onActivate={onActivate}
        activating={activating}
        onArchive={onArchive}
        archiving={archiving}
        onRepair={onRepair}
        repairing={repairing}
        lastRepairReport={lastRepairReport}
      />

      <section className="rounded-xl border bg-card p-4 shadow-sm">
        <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          Workspace Configuration
        </p>
        <div className="grid grid-cols-1 gap-2 md:grid-cols-3">
          <div className="rounded-lg border bg-muted/20 px-3 py-2">
            <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
              Environment Profile
            </p>
            <p className="mt-1 text-xs text-foreground">
              <code>{detail.environmentId ?? 'workspace-owned'}</code>
            </p>
          </div>
          <div className="rounded-lg border bg-muted/20 px-3 py-2">
            <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
              Worktree Path
            </p>
            <p className="mt-1 truncate text-xs text-foreground">
              {detail.worktreePath ?? 'checkout root'}
            </p>
          </div>
          <div className="rounded-lg border bg-muted/20 px-3 py-2">
            <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
              Providers
            </p>
            <p className="mt-1 text-xs text-foreground">
              {(detail.providers ?? []).join(', ') || 'none'}
            </p>
          </div>
        </div>
      </section>

      <section className="rounded-xl border bg-card p-4 shadow-sm">
        <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          Git Activity
        </p>
        <div className="mb-3 grid grid-cols-2 gap-2 md:grid-cols-5">
          <div className="rounded border bg-muted/20 px-2.5 py-2 text-[11px]">
            touched <span className="font-semibold">{workspaceGitSummary?.touched_files ?? workspaceChanges.length}</span>
          </div>
          <div className="rounded border bg-muted/20 px-2.5 py-2 text-[11px]">
            +{workspaceGitSummary?.insertions ?? 0}
          </div>
          <div className="rounded border bg-muted/20 px-2.5 py-2 text-[11px]">
            -{workspaceGitSummary?.deletions ?? 0}
          </div>
          <div className="rounded border bg-muted/20 px-2.5 py-2 text-[11px]">
            ahead {workspaceGitSummary?.ahead ?? 0}
          </div>
          <div className="rounded border bg-muted/20 px-2.5 py-2 text-[11px]">
            behind {workspaceGitSummary?.behind ?? 0}
          </div>
        </div>
        <p className="mb-2 text-[11px] text-muted-foreground">
          upstream: <code>{workspaceGitSummary?.upstream ?? 'none'}</code>
        </p>
        {workspaceChanges.length === 0 ? (
          <p className="text-[11px] text-muted-foreground">No working tree changes detected.</p>
        ) : (
          <div className="max-h-56 overflow-y-auto rounded-md border bg-muted/10">
            <ul className="divide-y">
              {workspaceChanges.slice(0, 24).map((change) => (
                <li key={`${change.status}-${change.path}`} className="flex items-center justify-between gap-3 px-3 py-1.5 text-[11px]">
                  <code className="rounded bg-muted px-1">{change.status || '--'}</code>
                  <span className="truncate text-right text-muted-foreground">{change.path}</span>
                </li>
              ))}
            </ul>
          </div>
        )}
      </section>

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
        linkFeatureId={linkFeatureId}
        linkSpecId={linkSpecId}
        linkReleaseId={linkReleaseId}
        noLinkValue={NO_LINK_VALUE}
        currentConfigGeneration={detail.configGeneration}
      />

      <WorkspaceProviderPreflightSection
        providerMatrix={providerMatrix}
        providerInfos={providerInfos}
        loading={loading}
        onRefresh={onRefreshProviders}
      />

      <WorkspaceLinksSection
        linkedFeature={linkedFeature}
        linkedSpec={linkedSpec}
        linkedRelease={linkedRelease}
        linkFeatureId={linkFeatureId}
        setLinkFeatureId={setLinkFeatureId}
        linkSpecId={linkSpecId}
        setLinkSpecId={setLinkSpecId}
        linkReleaseId={linkReleaseId}
        setLinkReleaseId={setLinkReleaseId}
        featureLinkOptions={featureLinkOptions}
        specLinkOptions={specLinkOptions}
        releaseLinkOptions={releaseLinkOptions}
        updatingLinks={updatingLinks}
        onApplyLinks={onApplyLinks}
        onOpenFeature={onOpenFeature}
        onOpenSpec={onOpenSpec}
        onOpenRelease={onOpenRelease}
        noLinkValue={NO_LINK_VALUE}
      />
    </div>
  );
}
