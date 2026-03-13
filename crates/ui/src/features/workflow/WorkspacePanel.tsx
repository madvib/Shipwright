import { useState, useMemo, useEffect, useRef } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { Button, Alert, AlertDescription, AlertTitle } from '@ship/ui';
import {
  createWorkspaceCmd,
  openWorkspaceEditorCmd,
  repairWorkspaceCmd,
  startWorkspaceSessionCmd,
  transitionWorkspaceCmd,
  endWorkspaceSessionCmd,
  syncWorkspaceCmd,
  type WorkspaceRepairReport,
} from '@/lib/platform/tauri/commands';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import {
  AGENTS_MCP_ROUTE,
  AGENTS_PERMISSIONS_ROUTE,
  FEATURES_ROUTE,
  OVERVIEW_ROUTE,
  RELEASES_ROUTE,
} from '@/lib/constants/routes';

// Subcomponents
import { WorkspaceSidebar } from './workspace/WorkspaceSidebar';
import { WorkspaceHeader } from './workspace/WorkspaceHeader';
import { WorkspaceDashboard } from './workspace/WorkspaceDashboard';
import { WorkspaceTerminalTray } from './workspace/WorkspaceTerminalTray';
import { WorkspaceHeaderActions } from './workspace/WorkspaceHeaderActions';
import { WorkspaceAgentDialog } from './workspace/WorkspaceAgentDialog';

// Hooks
import { useWorkspaceState } from './workspace/useWorkspaceState';
import { useWorkspaceTerminal } from './workspace/useWorkspaceTerminal';
import { useRuntimePerf } from './workspace/useRuntimePerf';

import { WorkspaceGraphStatus } from './components/WorkspaceLifecycleGraph';
import { cn } from '@/lib/utils';

type SessionErrorSurface = 'alert' | 'silent';

function normalizeIdList(values: string[]): string[] {
  const normalized = values.map((value) => value.trim()).filter((value) => value.length > 0);
  return Array.from(new Set(normalized)).sort((a, b) => a.localeCompare(b));
}

function hasListChanged(previous: string[], next: string[]): boolean {
  const prevNormalized = normalizeIdList(previous);
  const nextNormalized = normalizeIdList(next);
  if (prevNormalized.length !== nextNormalized.length) return true;
  for (let index = 0; index < prevNormalized.length; index += 1) {
    if (prevNormalized[index] !== nextNormalized[index]) {
      return true;
    }
  }
  return false;
}

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
  const [archivingWorkspace, setArchivingWorkspace] = useState(false);
  const [restartingSession, setRestartingSession] = useState(false);
  const [agentDialogOpen, setAgentDialogOpen] = useState(false);
  const [savingWorkspaceAgent, setSavingWorkspaceAgent] = useState(false);
  const terminalResizerRef = useRef(false);

  const createIntentNonceRef = useRef(0);
  const [createIntent, setCreateIntent] = useState<{ nonce: number; branch: string | null } | null>(null);
  const state = useWorkspaceState(workspaceUi, ship);
  const terminal = useWorkspaceTerminal(state.detail?.branch, workspaceUi.activeModeId, 'command');
  const runtimePerf = useRuntimePerf(import.meta.env.DEV);
  const showTerminalTray = Boolean(state.detail);
  const openCreateWorkspaceDialog = (targetBranch?: string | null) => {
    createIntentNonceRef.current += 1;
    setCreateIntent({
      nonce: createIntentNonceRef.current,
      branch: targetBranch ?? state.branch ?? null,
    });
  };
  const terminalReservedHeight = showTerminalTray && !terminalMaximized ? Math.max(terminalHeight, 140) : 0;

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
    if (status === 'archived') return 'secondary';
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
      const res = await endWorkspaceSessionCmd(
        state.detail.branch,
        summary.length > 0 ? summary : null,
        state.linkFeatureId ? [state.linkFeatureId] : []
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
        state.linkFeatureId
          ? [state.linkFeatureId]
          : []
      );
      if (endRes.status === 'error') {
        state.setError(endRes.error || 'Failed to end current session for restart.');
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

  const applyWorkspaceConfig = async (
    branch: string,
    input: {
      workspaceType: 'feature' | 'patch' | 'service';
      environmentId?: string | null;
      providers?: string[];
      mcpServers?: string[];
      skills?: string[];
      featureId?: string | null;
      releaseId?: string | null;
      isWorktree?: boolean;
      worktreePath?: string | null;
    },
    fallbackError: string,
  ) => {
    const result = await createWorkspaceCmd(branch, input);
    if (result.status === 'error') {
      state.setError(result.error || fallbackError);
      return false;
    }
    return true;
  };

  const handleUpdateLinks = async (nextFeatureId: string | null, nextReleaseId: string | null) => {
    if (!state.detail) return;
    if (nextFeatureId && nextReleaseId) {
      state.setError('Choose either a feature anchor or a release anchor for this workspace.');
      return;
    }

    state.setLinkFeatureId(nextFeatureId);
    state.setLinkReleaseId(nextReleaseId);
    state.setUpdatingLinks(true);

    try {
      const releaseId = nextReleaseId
        ? ship.releases.find(
            (release) =>
              release.id === nextReleaseId ||
              release.file_name === nextReleaseId ||
              release.version === nextReleaseId
          )?.id ?? nextReleaseId
        : null;

      const ok = await applyWorkspaceConfig(state.detail.branch, {
        workspaceType: state.detail.workspaceType,
        environmentId: state.detail.environmentId,
        featureId: nextFeatureId,
        releaseId,
      }, 'Failed to update workspace links.');
      if (!ok) {
        return;
      }
      await state.load();
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


  const handleArchive = async () => {
    if (!state.detail) return;
    setArchivingWorkspace(true);
    try {
      const result = await transitionWorkspaceCmd(state.detail.branch, 'archived');
      if (result.status === 'error') {
        state.setError(result.error || 'Failed to archive workspace.');
        return;
      }
      if (terminal.terminalSession?.branch === state.detail.branch) {
        await terminal.stopWorkspaceTerminal();
      }
      await state.load();
    } finally {
      setArchivingWorkspace(false);
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
    workspaceType: 'feature' | 'patch' | 'service';
    environmentId: string | null;
    providers: string[];
    featureId: string | null;
    releaseId: string | null;
    isWorktree: boolean;
    worktreePath: string | null;
  }) => {
    setCreatingWorkspace(true);
    try {
      const ok = await applyWorkspaceConfig(input.branch, {
        workspaceType: input.workspaceType,
        environmentId: input.environmentId,
        providers: input.providers,
        featureId: input.featureId,
        releaseId: input.releaseId,
        isWorktree: input.isWorktree,
        worktreePath: input.worktreePath,
      }, 'Failed to create workspace.');
      if (!ok) {
        return;
      }
      await state.load();
      state.setSelectedBranch(input.branch);
    } finally {
      setCreatingWorkspace(false);
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
      document.body.style.userSelect = '';
      (document.body.style as CSSStyleDeclaration & { webkitUserSelect?: string }).webkitUserSelect = '';
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = 'default';
      document.body.style.userSelect = '';
      (document.body.style as CSSStyleDeclaration & { webkitUserSelect?: string }).webkitUserSelect = '';
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

  const linkedRelease = useMemo(() => {
    const releaseRef =
      state.linkReleaseId ?? state.detail?.releaseId ?? null;
    if (!releaseRef) return null;
    return (
      ship.releases.find(
        (release) =>
          release.id === releaseRef ||
          release.file_name === releaseRef ||
          release.version === releaseRef
      ) || null
    );
  }, [state.detail?.releaseId, state.linkReleaseId, ship.releases]);

  const featureLabels = useMemo(
    () => Object.fromEntries(ship.features.map((feature) => [feature.id, feature.title ?? feature.id])),
    [ship.features],
  );


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
    if (state.detail?.featureId) {
      const feat = ship.features.find(f => f.id === state.detail?.featureId);
      if (feat) {
        void navigate({ to: FEATURES_ROUTE });
        void ship.handleSelectFeature(feat);
      }
    }
  };

  const handleOpenRelease = () => {
    if (!linkedRelease) return;
    void navigate({ to: RELEASES_ROUTE });
    void ship.handleSelectRelease(linkedRelease);
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

  const handleUpdateAgentConfiguration = async (input: {
    providers: string[];
    mcpServers: string[];
    skills: string[];
  }) => {
    if (!state.detail) return;
    const previousProviders = state.detail.providers ?? [];
    const previousMcpServers = state.detail.mcpServers ?? [];
    const previousSkills = state.detail.skills ?? [];
    const providersChanged = hasListChanged(previousProviders, input.providers);
    const mcpChanged = hasListChanged(previousMcpServers, input.mcpServers);
    const skillsChanged = hasListChanged(previousSkills, input.skills);
    const configChanged = providersChanged || mcpChanged || skillsChanged;
    const hadActiveSession = state.activeSession?.status === 'active';
    setSavingWorkspaceAgent(true);
    try {
      const ok = await applyWorkspaceConfig(state.detail.branch, {
        workspaceType: state.detail.workspaceType,
        providers: input.providers,
        mcpServers: input.mcpServers,
        skills: input.skills,
        featureId: state.detail.featureId,
        releaseId: state.detail.releaseId,
        isWorktree: state.detail.isWorktree,
        worktreePath: state.detail.worktreePath,
      }, 'Failed to update workspace agent configuration.');
      if (!ok) {
        return;
      }
      const syncResult = await syncWorkspaceCmd(state.detail.branch);
      if (syncResult.status === 'error') {
        state.setError(syncResult.error || 'Workspace config saved but sync failed.');
        return;
      }
      await state.load();
      if (hadActiveSession && mcpChanged) {
        state.setError(
          'Workspace MCP configuration changed and was synced. Restart the active agent session to load updated MCP tools.',
        );
      } else if (hadActiveSession && configChanged) {
        state.setError(
          'Workspace agent configuration changed and was synced. Restart the active session to refresh context.',
        );
      }
    } finally {
      setSavingWorkspaceAgent(false);
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
            : 'w-[340px] opacity-100',
        )}
      >
        <WorkspaceSidebar
          rows={state.rows}
          gitBranches={state.gitBranches}
          activeSessionBranches={state.activeSessionBranches}
          selectedBranch={state.selectedBranch}
          onSelectBranch={state.setSelectedBranch}
          onConfigureBranch={openCreateWorkspaceDialog}
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
          featureLabels={featureLabels}
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
              gitBranches={state.gitBranches}
              existingWorkspaceBranches={state.rows.map((row) => row.branch)}
              creatingWorkspace={creatingWorkspace}
              environmentOptions={Array.from(
                new Set(
                  state.rows
                    .map((row) => row.environmentId)
                    .filter((value): value is string => Boolean(value)),
                ),
              ).map((id) => ({ id, label: id }))}
              featureOptions={ship.features.map((feature) => ({
                id: feature.id,
                label: feature.title,
              }))}
              releaseOptions={ship.releases.map((release) => ({
                id: release.id,
                label: release.version,
              }))}
              providerOptions={state.providerInfos}
              createIntent={createIntent}
              onCreateIntentConsumed={() => setCreateIntent(null)}
              onCreateWorkspace={handleCreateWorkspace}
              canConfigureAgent={Boolean(state.detail)}
              onOpenAgentConfig={() => setAgentDialogOpen(true)}
              currentTheme={workspaceUi.config.theme}
              onThemeChange={(theme) => workspaceUi.handleSaveSettings({ ...workspaceUi.config, theme })}
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
            linkedRelease={linkedRelease}
            linkFeatureId={state.linkFeatureId}
            setLinkFeatureId={state.setLinkFeatureId}
            linkReleaseId={state.linkReleaseId}
            setLinkReleaseId={state.setLinkReleaseId}
            featureLinkOptions={ship.features}
            releaseLinkOptions={ship.releases}
            updatingLinks={state.updatingLinks}
            onUpdateLinks={handleUpdateLinks}
            onOpenFeature={handleOpenFeature}
            onOpenRelease={handleOpenRelease}
            activeSession={state.activeSession}
            recentSessions={state.recentSessions}
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
            workspaceChanges={state.workspaceChanges}
            workspaceGitSummary={state.workspaceGitSummary}
            sessionProvider={sessionProvider}
            setSessionProvider={setSessionProvider}
            restartingSession={restartingSession}
            onRestartSession={handleRestartSession}
            onSync={handleSync}
            syncing={state.syncing}
            onArchive={handleArchive}
            archiving={archivingWorkspace}
            onRepair={handleRepair}
            repairing={repairingWorkspace}
            lastRepairReport={lastRepairReport}
            loading={state.loading}
            onRefreshProviders={() => void state.load()}
            onCreateFromBranch={() => openCreateWorkspaceDialog(state.selectedBranch ?? state.branch)}
            creatingWorkspace={creatingWorkspace}
            branchDetail={state.branchDetail}
            branchDiffPath={state.branchDiffPath}
            setBranchDiffPath={state.setBranchDiffPath}
            branchFileDiff={state.branchFileDiff}
            loadingBranchDiff={state.loadingBranchDiff}
          />
        </div>

        {state.error && (
          <div className="fixed bottom-16 right-6 z-50 w-80 animate-in fade-in slide-in-from-bottom-4">
            <Alert variant="destructive" className="pointer-events-auto shadow-lg">
              <AlertTitle>Action Error</AlertTitle>
              <AlertDescription className="flex min-w-0 flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                <span className="min-w-0 flex-1 break-words">{state.error}</span>
                <Button size="xs" variant="ghost" className="h-6 w-fit shrink-0 self-end px-2 sm:self-start" onClick={() => state.setError(null)}>
                  Dismiss
                </Button>
              </AlertDescription>
            </Alert>
          </div>
        )}

        {showTerminalTray && (
          <WorkspaceTerminalTray
            terminalSession={terminal.terminalSession}
            terminalProvider={terminal.terminalProvider}
            onProviderChange={terminal.setTerminalProvider}
            startingTerminal={terminal.startingTerminal}
            stoppingTerminal={terminal.stoppingTerminal}
            onStart={handleStartTerminal}
            onStop={terminal.stopWorkspaceTerminal}
            onRetry={handleStartTerminal}
            onMaximizedChange={setTerminalMaximized}
            maximized={terminalMaximized}
            height={terminalHeight}
            onResizerMouseDown={(_e) => {
              _e.preventDefault();
              _e.stopPropagation();
              terminalResizerRef.current = true;
              document.body.style.cursor = 'ns-resize';
              document.body.style.userSelect = 'none';
              (document.body.style as CSSStyleDeclaration & { webkitUserSelect?: string }).webkitUserSelect = 'none';
            }}
            terminalContainerRef={terminal.terminalContainerRef}
            onSendSigInt={() => terminal.sendTerminalInput('\x03')}
            activationError={terminal.terminalSession?.activation_error}
            runtimeError={terminal.runtimeError}
            hasActiveSession={state.activeSession?.status === 'active'}
            runtimePerf={runtimePerf}
          />
        )}

        {state.detail ? (
          <WorkspaceAgentDialog
            open={agentDialogOpen}
            onOpenChange={setAgentDialogOpen}
            branch={state.detail.branch}
            workspaceType={state.detail.workspaceType}
            providerInfos={state.providerInfos}
            currentProviders={state.detail.providers ?? []}
            currentMcpServers={state.detail.mcpServers ?? []}
            currentSkills={state.detail.skills ?? []}
            saving={savingWorkspaceAgent}
            onOpenMcpSettings={() => {
              void navigate({ to: AGENTS_MCP_ROUTE });
            }}
            onOpenPermissionsSettings={() => {
              void navigate({ to: AGENTS_PERMISSIONS_ROUTE });
            }}
            onSave={handleUpdateAgentConfiguration}
          />
        ) : null}
      </main>
    </div>
  );
}
