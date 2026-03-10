import { useMemo, useState } from 'react';
import {
  Badge,
  Button,
  Checkbox,
  Input,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@ship/ui';
import { Clock3, Info, Play, Plus, RefreshCw, Square, Zap } from 'lucide-react';
import {
  createSpecCmd,
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
  sessionSpecIds: string[];
  setSessionSpecIds: (ids: string[]) => void;
  specLinkOptions: any[];
  providerMatrix: WorkspaceProviderMatrix | null;
  sessionProvider: string | null;
  setSessionProvider: (provider: string | null) => void;
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
  sessionSpecIds,
  setSessionSpecIds,
  specLinkOptions,
  providerMatrix,
  sessionProvider,
  setSessionProvider,
}: WorkspaceSessionSectionProps) {
  const [specSearch, setSpecSearch] = useState('');
  const [creatingSpec, setCreatingSpec] = useState(false);
  const [specError, setSpecError] = useState<string | null>(null);

  const hasActiveSession = activeSession?.status === 'active';
  const allowedProviders = providerMatrix?.allowed_providers ?? [];
  const hasSessionProviders = allowedProviders.length > 0;

  const sessionNotes = recentSessions
    .filter((session) => Boolean(session.summary) || Boolean(session.goal))
    .slice(0, 3);

  const specOptions = useMemo(
    () =>
      specLinkOptions
        .map((entry) => ({
          id: entry.id,
          label: entry.spec?.metadata?.title || entry.id,
        }))
        .sort((left, right) => left.label.localeCompare(right.label)),
    [specLinkOptions],
  );

  const filteredSpecOptions = useMemo(() => {
    const query = specSearch.trim().toLowerCase();
    if (!query) return specOptions;
    return specOptions.filter((option) => {
      const label = option.label.toLowerCase();
      const id = option.id.toLowerCase();
      return label.includes(query) || id.includes(query);
    });
  }, [specOptions, specSearch]);

  const selectedSpecBadges = sessionSpecIds
    .map((id) => specOptions.find((option) => option.id === id))
    .filter((value): value is { id: string; label: string } => Boolean(value))
    .slice(0, 8);

  const toggleSpecSelection = (specId: string) => {
    setSessionSpecIds(
      sessionSpecIds.includes(specId)
        ? sessionSpecIds.filter((id) => id !== specId)
        : [...sessionSpecIds, specId],
    );
  };

  const canCreateSpec = specSearch.trim().length > 0;

  const handleCreateSpec = async () => {
    const title = specSearch.trim();
    if (!title) return;

    setCreatingSpec(true);
    setSpecError(null);
    try {
      const template = `# ${title}\n\n## Context\n\n## Session Plan\n\n## Validation\n`;
      const res = await createSpecCmd(title, template);
      if (res.status === 'error') {
        setSpecError(res.error || 'Failed to create spec.');
        return;
      }
      setSessionSpecIds(Array.from(new Set([...sessionSpecIds, res.data.id])));
      setSpecSearch('');
    } finally {
      setCreatingSpec(false);
    }
  };

  const renderSpecPicker = () => (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between gap-2">
        <p className="text-[10px] text-muted-foreground">Session Specs</p>
        {sessionSpecIds.length > 0 ? (
          <Button
            size="xs"
            variant="ghost"
            className="h-6 px-1.5 text-[10px]"
            onClick={() => setSessionSpecIds([])}
          >
            Clear
          </Button>
        ) : null}
      </div>

      <Popover>
        <PopoverTrigger>
          <Button size="sm" variant="outline" className="h-8 w-full justify-between text-[11px]">
            <span className="truncate">
              {sessionSpecIds.length > 0
                ? `${sessionSpecIds.length} spec${sessionSpecIds.length === 1 ? '' : 's'} linked`
                : 'Create Session Spec or Link Existing'}
            </span>
            <Badge variant="outline" className="h-5 px-1.5 text-[10px]">
              {sessionSpecIds.length}
            </Badge>
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-[min(520px,94vw)] p-3" align="start" sideOffset={8}>
          <div className="space-y-2">
            <Input
              value={specSearch}
              onChange={(event) => setSpecSearch(event.target.value)}
              placeholder="Search existing specs or type a new title..."
              className="h-8"
            />

            <Button
              size="xs"
              variant="outline"
              className="h-7 w-full justify-start gap-1.5"
              onClick={() => void handleCreateSpec()}
              disabled={!canCreateSpec || creatingSpec}
            >
              {creatingSpec ? <RefreshCw className="size-3 animate-spin" /> : <Plus className="size-3" />}
              Create spec "{specSearch.trim() || '...'}"
            </Button>

            <div className="max-h-52 space-y-1 overflow-y-auto">
              {filteredSpecOptions.map((option) => (
                <label
                  key={option.id}
                  className="flex cursor-pointer items-center justify-between gap-3 rounded-md border border-transparent px-2 py-1.5 text-[11px] hover:border-border"
                >
                  <div className="min-w-0">
                    <p className="truncate text-foreground">{option.label}</p>
                    <p className="truncate text-[10px] text-muted-foreground">{option.id}</p>
                  </div>
                  <Checkbox
                    checked={sessionSpecIds.includes(option.id)}
                    onCheckedChange={() => toggleSpecSelection(option.id)}
                  />
                </label>
              ))}
              {filteredSpecOptions.length === 0 ? (
                <p className="px-1 text-[10px] text-muted-foreground">No specs match this search.</p>
              ) : null}
            </div>

            {specError ? (
              <p className="rounded border border-status-red/30 bg-status-red/5 px-2 py-1 text-[10px] text-status-red">
                {specError}
              </p>
            ) : null}
          </div>
        </PopoverContent>
      </Popover>

      {selectedSpecBadges.length > 0 ? (
        <div className="flex flex-wrap gap-1">
          {selectedSpecBadges.map((spec) => (
            <Badge key={spec.id} variant="outline" className="text-[9px]">
              {spec.label}
            </Badge>
          ))}
        </div>
      ) : (
        <p className="text-[10px] text-muted-foreground">No session specs linked yet.</p>
      )}
    </div>
  );

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
            Workspaces hold long-lived context. Sessions are start/stop runtime windows with audit capture.
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
                placeholder="What should this session accomplish?"
                className="h-8 text-[11px]"
              />

              <div className="space-y-1">
                <p className="text-[10px] text-muted-foreground">Agent provider</p>
                <Select
                  value={sessionProvider ?? ''}
                  onValueChange={(value) => setSessionProvider(value || null)}
                  disabled={!hasSessionProviders}
                >
                  <SelectTrigger size="sm" className="h-8 text-[11px]">
                    <SelectValue placeholder="No allowed providers" />
                  </SelectTrigger>
                  <SelectContent>
                    {allowedProviders.map((provider) => (
                      <SelectItem key={provider} value={provider}>
                        {provider}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {!hasSessionProviders ? (
                  <p className="text-[10px] text-amber-700">
                    No providers are currently allowed for this workspace context.
                  </p>
                ) : null}
              </div>

              {renderSpecPicker()}
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
              Start Session
            </Button>
          </div>
        ) : (
          <div className="flex flex-col gap-3 rounded-md border bg-muted/10 p-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="flex size-6 items-center justify-center rounded-full bg-primary/10 text-primary">
                  <Zap className="size-3 fill-current" />
                </div>
                <span className="text-xs font-semibold">ACTIVE SESSION</span>
              </div>
              <Badge variant="outline" className="text-[9px] font-semibold">
                {activeSession?.id?.slice(0, 8) || 'unknown'}
              </Badge>
            </div>

            <div className="space-y-2">
              <p className="text-[10px] text-muted-foreground">
                started {activeSession?.started_at ? new Date(activeSession.started_at).toLocaleString() : 'unknown'}
                {activeSession?.primary_provider ? (
                  <>
                    {' '}
                    · provider <code className="rounded bg-muted px-1">{activeSession.primary_provider}</code>
                  </>
                ) : null}
              </p>

              {activeSession?.stale_context && (
                <div className="flex items-center justify-between gap-2 rounded-md border border-amber-500/30 bg-amber-500/5 px-2 py-1.5">
                  <p className="text-[10px] text-amber-700">
                    Workspace context changed since session start. Restart to refresh context.
                  </p>
                </div>
              )}

              <Tooltip>
                <TooltipTrigger asChild>
                  <Input
                    value={sessionSummaryInput}
                    onChange={(e) => setSessionSummaryInput(e.target.value)}
                    placeholder="What was accomplished in this session?"
                    className="h-8 text-[11px]"
                  />
                </TooltipTrigger>
                <TooltipContent side="left">
                  Saved as a session completion note.
                </TooltipContent>
              </Tooltip>

              {renderSpecPicker()}
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
