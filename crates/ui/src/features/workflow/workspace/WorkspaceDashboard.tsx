import { Clock3 } from 'lucide-react';
import { type WorkspaceProviderMatrix, type WorkspaceRepairReport, type WorkspaceSessionInfo } from '@/lib/platform/tauri/commands';
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
  linkFeatureId: string;
  setLinkFeatureId: (id: string) => void;
  linkSpecId: string;
  setLinkSpecId: (id: string) => void;
  featureLinkOptions: any[];
  specLinkOptions: any[];
  updatingLinks: boolean;
  onApplyLinks: () => void;
  onOpenFeature: () => void;
  onOpenSpec: () => void;
  activeSession: WorkspaceSessionInfo | null;
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
  sessionProvider: string | null;
  setSessionProvider: (provider: string | null) => void;
  NO_LINK_VALUE: string;
  onSync: () => void;
  syncing: boolean;
  onActivate: () => void;
  activating: boolean;
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
  linkFeatureId,
  setLinkFeatureId,
  linkSpecId,
  setLinkSpecId,
  featureLinkOptions,
  specLinkOptions,
  updatingLinks,
  onApplyLinks,
  onOpenFeature,
  onOpenSpec,
  activeSession,
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
  sessionProvider,
  setSessionProvider,
  NO_LINK_VALUE,
  onSync,
  syncing,
  onActivate,
  activating,
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
    <div className="mx-auto max-w-5xl space-y-4 p-4">
      <WorkspaceStatusCard
        detail={detail}
        statusVariant={statusVariant}
        onSync={onSync}
        syncing={syncing}
        onActivate={onActivate}
        activating={activating}
        onRepair={onRepair}
        repairing={repairing}
        lastRepairReport={lastRepairReport}
      />

      <WorkspaceSessionSection
        activeSession={activeSession}
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
        linkFeatureId={linkFeatureId}
        setLinkFeatureId={setLinkFeatureId}
        linkSpecId={linkSpecId}
        setLinkSpecId={setLinkSpecId}
        featureLinkOptions={featureLinkOptions}
        specLinkOptions={specLinkOptions}
        updatingLinks={updatingLinks}
        onApplyLinks={onApplyLinks}
        onOpenFeature={onOpenFeature}
        onOpenSpec={onOpenSpec}
        noLinkValue={NO_LINK_VALUE}
      />
    </div>
  );
}
