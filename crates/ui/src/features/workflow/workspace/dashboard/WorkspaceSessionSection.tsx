import {
  Badge,
  Button,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@ship/ui';
import { Clock3, Info, Play, RefreshCw, Square, Zap } from 'lucide-react';
import {
  type WorkspaceProviderMatrix,
  type WorkspaceSessionInfo,
} from '@/lib/platform/tauri/commands';

interface WorkspaceSessionSectionProps {
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
  sessionProvider: string | null;
  setSessionProvider: (provider: string | null) => void;
  linkFeatureId: string;
  linkSpecId: string;
  linkReleaseId: string;
  noLinkValue: string;
  currentConfigGeneration: number;
}

export function WorkspaceSessionSection({
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
  sessionProvider,
  setSessionProvider,
  linkFeatureId,
  linkSpecId,
  linkReleaseId,
  noLinkValue,
  currentConfigGeneration,
}: WorkspaceSessionSectionProps) {
  const hasActiveSession = activeSession?.status === 'active';
  const hasSessionProviders = (providerMatrix?.allowed_providers.length ?? 0) > 0;
  const sessionNotes = recentSessions
    .filter((session) => Boolean(session.summary) || Boolean(session.goal))
    .slice(0, 3);

  return (
    <section className="space-y-3 rounded-lg border bg-card p-3">
      <div className="flex items-center gap-2 px-1">
        <div className="rounded bg-muted p-1">
          <Clock3 className="size-3 text-muted-foreground" />
        </div>
        <p className="text-[11px] font-semibold text-muted-foreground">
          Session Control
        </p>
        <Tooltip>
          <TooltipTrigger asChild>
            <Info className="size-3 cursor-help text-muted-foreground/30 transition-colors hover:text-muted-foreground" />
          </TooltipTrigger>
          <TooltipContent side="top" className="max-w-xs">
            Start an agent session for structured tracking. Starting the console
            will also auto-start the session.
          </TooltipContent>
        </Tooltip>
      </div>

      <div className="grid gap-4">
        {!hasActiveSession ? (
          <div className="flex flex-col gap-3 rounded-md border bg-muted/10 p-3">
            <div className="space-y-2">
              <Input
                value={sessionGoalInput}
                onChange={(e) => setSessionGoalInput(e.target.value)}
                placeholder="Session goal (optional)"
                className="h-8 text-[11px]"
              />
              <p className="text-[10px] text-muted-foreground">
                Workspace context links: feature{' '}
                <code className="rounded bg-muted px-1">
                  {linkFeatureId === noLinkValue ? 'none' : linkFeatureId}
                </code>{' '}
                · spec{' '}
                <code className="rounded bg-muted px-1">
                  {linkSpecId === noLinkValue ? 'none' : linkSpecId}
                </code>
                {' '}· release{' '}
                <code className="rounded bg-muted px-1">
                  {linkReleaseId === noLinkValue ? 'none' : linkReleaseId}
                </code>
              </p>
              <p className="text-[10px] text-muted-foreground">
                Session providers ({providerMatrix?.source ?? 'unknown'}):{' '}
                <code className="rounded bg-muted px-1">
                  {(providerMatrix?.allowed_providers ?? []).join(', ') || 'none'}
                </code>
              </p>
              <div className="space-y-1">
                <p className="text-[10px] text-muted-foreground">Session provider</p>
                <Select
                  value={sessionProvider ?? ''}
                  onValueChange={(value) => setSessionProvider(value || null)}
                  disabled={!hasSessionProviders}
                >
                  <SelectTrigger size="sm" className="h-8 text-[11px]">
                    <SelectValue placeholder="No allowed providers" />
                  </SelectTrigger>
                  <SelectContent>
                    {(providerMatrix?.allowed_providers ?? []).map((provider) => (
                      <SelectItem key={provider} value={provider}>
                        {provider}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <Button
              size="sm"
              className="h-8 gap-1.5"
              onClick={onStartSession}
              disabled={startingSession || !hasSessionProviders}
            >
              {startingSession ? (
                <RefreshCw className="size-3 animate-spin" />
              ) : (
                <Play className="size-3 fill-current" />
              )}
              Start Agent Session
            </Button>
          </div>
        ) : (
          <div className="flex flex-col gap-3 rounded-md border bg-muted/10 p-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="flex size-6 items-center justify-center rounded-full bg-primary/10 text-primary">
                  <Zap className="size-3 fill-current" />
                </div>
                <span className="text-xs font-semibold">
                  ACTIVE SESSION
                </span>
              </div>
              <Badge
                variant="outline"
                className="text-[9px] font-semibold"
              >
                {activeSession?.id?.slice(0, 8) || 'unknown'}
              </Badge>
            </div>
            <div className="space-y-2">
              <p className="text-[10px] text-muted-foreground">
                Provider:{' '}
                <code className="rounded bg-muted px-1">
                  {activeSession?.primary_provider ?? 'none'}
                </code>{' '}
                · generation at start:{' '}
                <code className="rounded bg-muted px-1">
                  {activeSession?.config_generation_at_start ?? 'n/a'}
                </code>{' '}
                · current:{' '}
                <code className="rounded bg-muted px-1">
                  {currentConfigGeneration}
                </code>
              </p>
              {activeSession?.stale_context && (
                <div className="flex items-center justify-between gap-2 rounded-md border border-amber-500/30 bg-amber-500/5 px-2 py-1.5">
                  <p className="text-[10px] text-amber-700">
                    Session context is stale. Restart to align with latest workspace compile.
                  </p>
                </div>
              )}
              <Tooltip>
                <TooltipTrigger asChild>
                  <Input
                    value={sessionSummaryInput}
                    onChange={(e) => setSessionSummaryInput(e.target.value)}
                    placeholder="Session summary for end (optional)"
                    className="h-8 text-[11px]"
                  />
                </TooltipTrigger>
                <TooltipContent side="left">
                  Describe what was achieved in this session for project
                  tracking.
                </TooltipContent>
              </Tooltip>
            </div>
            <div className="grid gap-2 sm:grid-cols-2">
              <Button
                size="sm"
                variant="outline"
                className="h-8 gap-1.5"
                onClick={onRestartSession}
                disabled={restartingSession || endingSession}
              >
                {restartingSession ? (
                  <RefreshCw className="size-3 animate-spin" />
                ) : (
                  <RefreshCw className="size-3" />
                )}
                Sync + Restart
              </Button>
              <Button
                size="sm"
                variant="outline"
                className="h-8 gap-1.5"
                onClick={onEndSession}
                disabled={endingSession || restartingSession}
              >
                {endingSession ? (
                  <RefreshCw className="size-3 animate-spin" />
                ) : (
                  <Square className="size-3 fill-current" />
                )}
                End Session
              </Button>
            </div>
          </div>
        )}
      </div>

      {sessionNotes.length > 0 && (
        <div className="rounded-md border bg-muted/10 p-3">
          <p className="mb-2 text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
            Recent Session Notes
          </p>
          <div className="space-y-2">
            {sessionNotes.map((session) => (
              <div key={session.id} className="rounded border bg-background/80 p-2">
                <p className="text-[10px] text-muted-foreground">
                  {new Date(session.updated_at).toLocaleString()} · <code>{session.id.slice(0, 8)}</code>
                </p>
                {session.summary && (
                  <p className="mt-1 text-[11px] text-foreground">
                    {session.summary}
                  </p>
                )}
                {!session.summary && session.goal && (
                  <p className="mt-1 text-[11px] text-muted-foreground">
                    Goal: {session.goal}
                  </p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}
