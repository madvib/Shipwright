import { useState, useMemo, useEffect, useRef } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { Button, Alert, AlertDescription, AlertTitle } from '@ship/ui';
import {
  createWorkspaceCmd,
  deleteWorkspaceCmd,
  openWorkspaceEditorCmd,
  repairWorkspaceCmd,
  setWorkspaceModeCmd,
  startWorkspaceSessionCmd,
  endWorkspaceSessionCmd,
  syncWorkspaceCmd,
  activateWorkspaceCmd,
  type WorkspaceRepairReport,
} from '@/lib/platform/tauri/commands';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { FEATURES_ROUTE, OVERVIEW_ROUTE } from '@/lib/constants/routes';

// Subcomponents
import { WorkspaceSidebar } from './workspace/WorkspaceSidebar';
import { WorkspaceHeader } from './workspace/WorkspaceHeader';
import { WorkspaceDashboard } from './workspace/WorkspaceDashboard';
import { WorkspaceTerminalTray } from './workspace/WorkspaceTerminalTray';
import { WorkspaceHeaderActions } from './workspace/WorkspaceHeaderActions';

// Hooks
import { useWorkspaceState } from './workspace/useWorkspaceState';
import { useWorkspaceTerminal } from './workspace/useWorkspaceTerminal';
import { useRuntimePerf } from './workspace/useRuntimePerf';

import { WorkspaceGraphStatus } from './components/WorkspaceLifecycleGraph';
import { cn } from '@/lib/utils';

const NO_LINK_VALUE = '__none__';
type SessionErrorSurface = 'alert' | 'silent';

export default function WorkspacePanel() {
  const navigate = useNavigate();
  const workspaceUi = useWorkspace();
  const ship = useShip();

  const [workspaceSidebarCollapsed, setWorkspaceSidebarCollapsed] = useState(false);
  const [terminalMaximized, setTerminalMaximized] = useState(false);
  const [terminalHeight, setTerminalHeight] = useState(320);
  const [repairingWorkspace, setRepairingWorkspace] = useState(false);
  const [lastRepairReport, setLastRepairReport] = useState<WorkspaceRepairReport | null>(null);
  const [sessionProvider, setSessionProvider] = useState<string | null>(null);
  const [creatingWorkspace, setCreatingWorkspace] = useState(false);
  const [deletingWorkspace, setDeletingWorkspace] = useState(false);
  const [updatingWorkspaceMode, setUpdatingWorkspaceMode] = useState(false);
  const [restartingSession, setRestartingSession] = useState(false);
  const terminalResizerRef = useRef(false);

  const state = useWorkspaceState(workspaceUi, ship);
  const terminal = useWorkspaceTerminal(state.detail?.branch, workspaceUi.activeModeId, 'command');
  const runtimePerf = useRuntimePerf(import.meta.env.DEV);
  const terminalReservedHeight = terminalMaximized ? 0 : Math.max(terminalHeight, 140);

  const isDarkTheme = useMemo(() => {
    if (workspaceUi.config.theme === 'dark') return true;
    if (workspaceUi.config.theme === 'light') return false;
    if (typeof document !== 'undefined') {
      return document.documentElement.classList.contains('dark');
    }
    return false;
  }, [workspaceUi.config.theme]);

  const statusVariant = (status: WorkspaceGraphStatus): 'default' | 'secondary' | 'outline' => {
    if (status === 'active') return 'default';
    if (status === 'review') return 'secondary';
    return 'outline';
  };

  const handleOpenEditor = async (targetBranch: string, editorId: string) => {
    const result = await openWorkspaceEditorCmd(targetBranch, editorId);
    if (result.status === 'error') {
      state.setError(result.error || `Failed to open ${editorId}.`);
    }
  };

  const startSessionInternal = async ({
    preferredProvider,
    errorSurface = 'alert',
  }: {
    preferredProvider?: string | null;
    errorSurface?: SessionErrorSurface;
  } = {}): Promise<{ ok: boolean; error: string | null; provider: string | null }> => {
    if (!state.detail) {
      return { ok: false, error: 'Select a workspace before starting a session.', provider: null };
    }

    const allowedProviders = state.providerMatrix?.allowed_providers ?? [];
    let provider = preferredProvider ?? sessionProvider;
    if (!provider || !allowedProviders.includes(provider)) {
      provider = allowedProviders[0] ?? null;
    }

    if (!provider) {
      const message =
        `No allowed providers resolved for workspace session (${state.providerMatrix?.source ?? 'unknown source'}). ` +
        'Open workspace settings and add at least one provider.';
      if (errorSurface === 'alert') {
        state.setError(message);
      }
      return { ok: false, error: message, provider: null };
    }

    state.setStartingSession(true);
    try {
      const goal = state.sessionGoalInput.trim();
      const res = await startWorkspaceSessionCmd(
        state.detail.branch,
        goal.length > 0 ? goal : null,
        state.detail.activeMode ?? workspaceUi.activeModeId ?? null,
        provider
      );
      if (res.status === 'ok') {
        setSessionProvider(provider);
        await state.load();
        return { ok: true, error: null, provider };
      } else {
        const message = res.error || 'Failed to start workspace session.';
        if (errorSurface === 'alert') {
          state.setError(message);
        }
        return { ok: false, error: message, provider };
      }
    } finally {
      state.setStartingSession(false);
    }
  };

  const handleStartSession = async () => {
    await startSessionInternal();
  };

  const handleEndSession = async () => {
    if (!state.detail || !state.activeSession) return;
    state.setEndingSession(true);
    try {
      const summary = state.sessionSummaryInput.trim();
      const updatedFeatureIds =
        state.linkFeatureId && state.linkFeatureId !== NO_LINK_VALUE
          ? [state.linkFeatureId]
          : [];
      const updatedSpecIds =
        state.linkSpecId && state.linkSpecId !== NO_LINK_VALUE
          ? [state.linkSpecId]
          : [];
      const res = await endWorkspaceSessionCmd(
        state.detail.branch,
        summary.length > 0 ? summary : null,
        updatedFeatureIds,
        updatedSpecIds
      );
      if (res.status === 'ok') {
        if (terminal.terminalSession?.branch === state.detail.branch) {
          await terminal.stopWorkspaceTerminal();
        }
        state.load();
      } else {
        state.setError(res.error || 'Failed to end workspace session.');
      }
    } finally {
      state.setEndingSession(false);
    }
  };

  const handleRestartSession = async () => {
    if (!state.detail || !state.activeSession) return;
    const detail = state.detail;
    const activeSession = state.activeSession;
    const hadTerminal = terminal.terminalSession?.branch === detail.branch;
    const allowedProviders = state.providerMatrix?.allowed_providers ?? [];
    const preferredProviders = [
      activeSession.primary_provider ?? null,
      sessionProvider,
    ].filter((value): value is string => Boolean(value));
    const restartProvider =
      preferredProviders.find((candidate) =>
        allowedProviders.includes(candidate)
      ) ?? allowedProviders[0] ?? null;

    if (!restartProvider) {
      state.setError(
        'Cannot restart session: no allowed providers resolved for this workspace.'
      );
      return;
    }

    setRestartingSession(true);
    try {
      const endRes = await endWorkspaceSessionCmd(
        detail.branch,
        'Session restarted to apply updated workspace context.',
        state.linkFeatureId && state.linkFeatureId !== NO_LINK_VALUE
          ? [state.linkFeatureId]
          : [],
        state.linkSpecId && state.linkSpecId !== NO_LINK_VALUE
          ? [state.linkSpecId]
          : []
      );
      if (endRes.status === 'error') {
        state.setError(endRes.error || 'Failed to end current session for restart.');
        return;
      }

      const activateRes = await activateWorkspaceCmd(detail.branch);
      if (activateRes.status === 'error') {
        state.setError(
          activateRes.error || 'Failed to activate workspace before restart.'
        );
        return;
      }

      const startRes = await startWorkspaceSessionCmd(
        detail.branch,
        activeSession.goal ?? null,
        detail.activeMode ?? workspaceUi.activeModeId ?? null,
        restartProvider
      );
      if (startRes.status === 'error') {
        state.setError(startRes.error || 'Failed to restart workspace session.');
        return;
      }

      if (hadTerminal) {
        if (terminal.terminalSession) {
          await terminal.stopWorkspaceTerminal();
        }
        await terminal.startWorkspaceTerminal();
      }

      await state.load();
    } finally {
      setRestartingSession(false);
    }
  };

  const handleApplyLinks = async () => {
    if (!state.detail) return;
    state.setUpdatingLinks(true);
    try {
      const featureId =
        state.linkFeatureId === NO_LINK_VALUE ? null : state.linkFeatureId;
      const specId = state.linkSpecId === NO_LINK_VALUE ? null : state.linkSpecId;
      const res = await createWorkspaceCmd(state.detail.branch, {
        workspaceType: state.detail.workspaceType,
        featureId,
        specId,
        releaseId: state.detail.releaseId ?? null,
        modeId: state.detail.activeMode ?? null,
      });
      if (res.status === 'error') {
        state.setError(res.error || 'Failed to update workspace links.');
        return;
      }
      state.load();
    } finally {
      state.setUpdatingLinks(false);
    }
  };

  const handleSync = async () => {
    if (!state.detail) return;
    state.setSyncing(true);
    try {
      const result = await syncWorkspaceCmd(state.detail.branch);
      if (result.status === 'error') {
        state.setError(result.error || 'Failed to sync workspace.');
        return;
      }
      await state.load();
    } finally {
      state.setSyncing(false);
    }
  };

  const handleActivate = async () => {
    if (!state.detail) return;
    state.setActivating(true);
    try {
      const result = await activateWorkspaceCmd(state.detail.branch);
      if (result.status === 'error') {
        state.setError(result.error || 'Failed to activate workspace.');
        return;
      }
      await state.load();
    } finally {
      state.setActivating(false);
    }
  };

  const handleRepair = async () => {
    if (!state.detail) return;
    setRepairingWorkspace(true);
    try {
      const res = await repairWorkspaceCmd(state.detail.branch, false);
      if (res.status === 'ok') {
        setLastRepairReport(res.data);
        state.load();
      } else {
        state.setError(res.error || 'Workspace repair failed.');
      }
    } finally {
      setRepairingWorkspace(false);
    }
  };

  const handleCreateWorkspace = async (input: {
    branch: string;
    workspaceType: 'feature' | 'refactor' | 'experiment' | 'hotfix';
    modeId: string | null;
  }) => {
    setCreatingWorkspace(true);
    try {
      const result = await createWorkspaceCmd(input.branch, {
        workspaceType: input.workspaceType,
        modeId: input.modeId,
        activate: true,
      });
      if (result.status === 'error') {
        state.setError(result.error || 'Failed to create workspace.');
        return;
      }
      await state.load();
      state.setSelectedBranch(input.branch);
    } finally {
      setCreatingWorkspace(false);
    }
  };

  const handleDeleteWorkspace = async (branch: string) => {
    setDeletingWorkspace(true);
    try {
      const result = await deleteWorkspaceCmd(branch);
      if (result.status === 'error') {
        state.setError(result.error || 'Failed to delete workspace.');
        return;
      }
      if (terminal.terminalSession?.branch === branch) {
        await terminal.stopWorkspaceTerminal();
      }
      await state.load();
    } finally {
      setDeletingWorkspace(false);
    }
  };

  const handleUpdateWorkspaceMode = async (modeId: string | null) => {
    if (!state.detail) return;
    setUpdatingWorkspaceMode(true);
    try {
      const result = await setWorkspaceModeCmd(state.detail.branch, modeId);
      if (result.status === 'error') {
        state.setError(result.error || 'Failed to update workspace mode.');
        return;
      }
      await state.load();
    } finally {
      setUpdatingWorkspaceMode(false);
    }
  };

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!terminalResizerRef.current) return;
      const height = window.innerHeight - e.clientY;
      setTerminalHeight(Math.max(44, Math.min(height, window.innerHeight - 100)));
    };
    const handleMouseUp = () => {
      terminalResizerRef.current = false;
      document.body.style.cursor = 'default';
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

  useEffect(() => {
    if (!terminal.terminalSession && terminalMaximized) {
      setTerminalMaximized(false);
    }
  }, [terminal.terminalSession, terminalMaximized]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'b') {
        event.preventDefault();
        setWorkspaceSidebarCollapsed((previous) => !previous);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  const linkedFeature = useMemo(() => {
    if (!state.detail) return null;
    return ship.features.find((feature) => feature.id === state.detail?.featureId) || null;
  }, [state.detail, ship.features]);

  const linkedSpec = useMemo(() => {
    if (!state.detail) return null;
    return ship.specs.find((s: any) => s.id === state.detail?.specId) || null;
  }, [state.detail, ship.specs]);

  useEffect(() => {
    const allowedProviders = state.providerMatrix?.allowed_providers ?? [];
    setSessionProvider((previous) => {
      if (allowedProviders.length === 0) {
        return null;
      }
      if (previous && allowedProviders.includes(previous)) {
        return previous;
      }
      return allowedProviders[0];
    });
  }, [state.providerMatrix]);

  const handleOpenFeature = () => {
    if (linkedFeature) {
      void navigate({ to: FEATURES_ROUTE });
      void ship.handleSelectFeature(linkedFeature);
    }
  };

  const handleOpenSpec = () => {
    if (!linkedSpec) return;
    const relatedFeature =
      ship.features.find(
        (entry) =>
          entry.spec_id === linkedSpec.id || entry.spec_id === linkedSpec.file_name
      ) ?? null;
    if (!relatedFeature) {
      state.setError(
        `Spec ${linkedSpec.id} is linked, but no feature currently references it.`
      );
      return;
    }
    void navigate({ to: FEATURES_ROUTE });
    void ship.handleSelectFeature(relatedFeature);
  };

  const handleStartTerminal = async () => {
    if (!state.detail) return;
    terminal.setRuntimeError(null);
    let sessionWarning: string | null = null;
    if (!state.activeSession) {
      if (terminal.terminalProvider === 'shell') {
        sessionWarning =
          'Console is running in shell mode without a tracked workspace session. Start a session to capture lifecycle metadata.';
      } else {
        const sessionResult = await startSessionInternal({
          preferredProvider: terminal.terminalProvider,
          errorSurface: 'silent',
        });
        if (!sessionResult.ok) {
          sessionWarning =
            `Session tracking was not started: ${sessionResult.error ?? 'no allowed provider resolved'}`;
        }
      }
    }
    await terminal.startWorkspaceTerminal();
    if (sessionWarning) {
      terminal.setRuntimeError(sessionWarning);
    }
  };

  useEffect(() => {
    workspaceUi.setIsWorkspaceFocusMode(true);
    return () => workspaceUi.setIsWorkspaceFocusMode(false);
  }, [workspaceUi]);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background">
      <aside
        className={cn(
          'h-full shrink-0 overflow-hidden border-r border-border transition-[width,opacity] duration-200',
          workspaceSidebarCollapsed
            ? 'pointer-events-none w-0 opacity-0'
            : 'w-[340px] opacity-100'
        )}
      >
        <WorkspaceSidebar
          filteredRows={state.filteredRows}
          selectedBranch={state.selectedBranch}
          onSelectBranch={state.setSelectedBranch}
          availableEditors={state.availableEditors}
          isDarkTheme={isDarkTheme}
          onOpenEditor={handleOpenEditor}
          searchQuery={state.searchQuery}
          onSearchChange={state.setSearchQuery}
          loading={state.loading}
          onRefresh={state.load}
          onHome={() => {
            workspaceUi.setIsWorkspaceFocusMode(false);
            void navigate({ to: OVERVIEW_ROUTE });
          }}
          onCollapse={() => setWorkspaceSidebarCollapsed(true)}
          statusVariant={statusVariant}
        />
      </aside>

      <main className="relative flex flex-1 flex-col overflow-hidden bg-muted/5">
        <WorkspaceHeader
          branch={state.detail?.branch ?? 'Select Workspace'}
          sidebarCollapsed={workspaceSidebarCollapsed}
          onHome={() => {
            workspaceUi.setIsWorkspaceFocusMode(false);
            void navigate({ to: OVERVIEW_ROUTE });
          }}
          onExpandSidebar={() => setWorkspaceSidebarCollapsed(false)}
          actions={
            <WorkspaceHeaderActions
              detail={state.detail}
              modeOptions={state.modeOptions}
              creatingWorkspace={creatingWorkspace}
              deletingWorkspace={deletingWorkspace}
              updatingWorkspaceMode={updatingWorkspaceMode}
              onCreateWorkspace={handleCreateWorkspace}
              onDeleteWorkspace={handleDeleteWorkspace}
              onUpdateWorkspaceMode={handleUpdateWorkspaceMode}
            />
          }
        />

        <div
          className="flex-1 overflow-y-auto custom-scrollbar"
          style={{ paddingBottom: terminalReservedHeight ? terminalReservedHeight + 8 : 0 }}
        >
          <WorkspaceDashboard
            detail={state.detail}
            statusVariant={statusVariant}
            linkedFeature={linkedFeature}
            linkedSpec={linkedSpec}
            linkFeatureId={state.linkFeatureId}
            setLinkFeatureId={state.setLinkFeatureId}
            linkSpecId={state.linkSpecId}
            setLinkSpecId={state.setLinkSpecId}
            featureLinkOptions={ship.features}
            specLinkOptions={ship.specs}
            updatingLinks={state.updatingLinks}
            onApplyLinks={handleApplyLinks}
            onOpenFeature={handleOpenFeature}
            onOpenSpec={handleOpenSpec}
            activeSession={state.activeSession}
            startingSession={state.startingSession}
            endingSession={state.endingSession}
            onStartSession={handleStartSession}
            onEndSession={handleEndSession}
            sessionGoalInput={state.sessionGoalInput}
            setSessionGoalInput={state.setSessionGoalInput}
            sessionSummaryInput={state.sessionSummaryInput}
            setSessionSummaryInput={state.setSessionSummaryInput}
            providerMatrix={state.providerMatrix}
            providerInfos={state.providerInfos}
            sessionProvider={sessionProvider}
            setSessionProvider={setSessionProvider}
            restartingSession={restartingSession}
            onRestartSession={handleRestartSession}
            NO_LINK_VALUE={NO_LINK_VALUE}
            onSync={handleSync}
            syncing={state.syncing}
            onActivate={handleActivate}
            activating={state.activating}
            onRepair={handleRepair}
            repairing={repairingWorkspace}
            lastRepairReport={lastRepairReport}
            loading={state.loading}
            onRefreshProviders={() => void state.load()}
          />
        </div>

        {state.error && (
          <div className="fixed bottom-16 right-6 z-50 w-80 animate-in fade-in slide-in-from-bottom-4">
            <Alert variant="destructive" className="pointer-events-auto shadow-lg">
              <AlertTitle>Action Error</AlertTitle>
              <AlertDescription className="flex items-center justify-between gap-3">
                <span>{state.error}</span>
                <Button size="xs" variant="ghost" className="h-6 px-2" onClick={() => state.setError(null)}>
                  Dismiss
                </Button>
              </AlertDescription>
            </Alert>
          </div>
        )}

        <WorkspaceTerminalTray
          terminalSession={terminal.terminalSession}
          terminalProvider={terminal.terminalProvider}
          onProviderChange={terminal.setTerminalProvider}
          startingTerminal={terminal.startingTerminal}
          stoppingTerminal={terminal.stoppingTerminal}
          onStart={handleStartTerminal}
          onStop={terminal.stopWorkspaceTerminal}
          onMaximizedChange={setTerminalMaximized}
          maximized={terminalMaximized}
          height={terminalHeight}
          onResizerMouseDown={(_e) => {
            terminalResizerRef.current = true;
            document.body.style.cursor = 'ns-resize';
          }}
          terminalContainerRef={terminal.terminalContainerRef}
          onSendSigInt={() => terminal.sendTerminalInput('\x03')}
          activationError={terminal.terminalSession?.activation_error}
          runtimeError={terminal.runtimeError}
          hasActiveSession={state.activeSession?.status === 'active'}
          runtimePerf={runtimePerf}
        />
      </main>
    </div>
  );
}
