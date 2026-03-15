import { Bot, ChevronDown, ChevronRight, Download, Info, Upload } from 'lucide-react';
import { Badge } from '@ship/primitives';
import { Button } from '@ship/primitives';
import { Card } from '@ship/primitives';
import { Tooltip, TooltipTrigger, TooltipContent } from '@ship/primitives';
import { cn } from '@/lib/utils';
import {
  type ProviderRow,
  type ScopeKey,
  HOOK_EVENTS,
  PROVIDER_LOGO,
  PROVIDER_STATUS_COPY,
  getProviderSyncSummary,
  providerStatusBadgeClass,
} from '../agents.types';
import type { McpValidationReport } from '@/bindings';

export interface ProvidersSectionProps {
  providerRows: ProviderRow[];
  providersPending: boolean;
  providersError: Error | null;
  refreshProviders: () => void;
  expandedProviderId: string;
  setExpandedProviderId: (id: string) => void;
  hasActiveProject: boolean;
  agentScope: ScopeKey;
  importStatus: Record<string, 'idle' | 'loading' | 'ok' | 'error'>;
  exportStatus: Record<string, 'idle' | 'loading' | 'ok' | 'error'>;
  importSummary: Record<string, string>;
  onImport: (target: string) => void;
  onExport: (target: string) => void;
  mcpValidationReport: McpValidationReport | null;
  mcpValidationIsFetching: boolean;
  onRefetchMcpValidation: () => void;
}

export function ProvidersSection({
  providerRows,
  providersPending,
  providersError,
  refreshProviders,
  expandedProviderId,
  setExpandedProviderId,
  hasActiveProject,
  agentScope,
  importStatus,
  exportStatus,
  importSummary,
  onImport,
  onExport,
  mcpValidationReport,
  mcpValidationIsFetching,
  onRefetchMcpValidation,
}: ProvidersSectionProps) {
  return (
    <div className="grid gap-4">
      {/* ── AI Clients ── */}
      <Card size="sm" className="overflow-hidden">
        <div className="flex items-center justify-between border-b px-4 py-3">
          <div className="flex items-center gap-3">
            <div className="flex size-7 items-center justify-center rounded-lg border border-primary/20 bg-primary/10">
              <Bot className="size-3.5 text-primary" />
            </div>
            <div>
              <h3 className="text-sm font-semibold">AI Clients</h3>
              <p className="text-[11px] text-muted-foreground">Ship manages provider configs and native exports.</p>
            </div>
          </div>
          {providersPending && (
            <Badge variant="outline" className="text-[10px] text-muted-foreground">
              checking PATH...
            </Badge>
          )}
        </div>
        {providersError && (
          <div className="flex items-center justify-between gap-2 border-b bg-rose-500/5 px-4 py-2.5">
            <p className="text-[11px] text-rose-700 dark:text-rose-300">
              Provider detection failed. Showing supported providers with unknown install status.
            </p>
            <Button
              type="button"
              variant="ghost"
              size="xs"
              className="h-6 px-2 text-[10px]"
              onClick={() => refreshProviders()}
            >
              Retry
            </Button>
          </div>
        )}
        <div className="divide-y divide-border/50">
          {providerRows.map((provider) => {
            const syncSummary = getProviderSyncSummary(provider, true, mcpValidationReport);
            const isExpanded = expandedProviderId === provider.id;
            const providerHookEvents = HOOK_EVENTS.filter((event) => event.providers.includes(provider.id));
            const issueCount = syncSummary.issues.length;
            const nextIssue = syncSummary.issues.find((issue) => issue.level !== 'info') ?? syncSummary.issues[0] ?? null;
            const importDisabledReason =
              !hasActiveProject
                ? 'Open or create a project to import provider config files.'
                : agentScope !== 'project'
                  ? 'Switch to Project scope to import into this project.'
                  : provider.id === 'claude'
                    ? 'Imports from .mcp.json first, then ~/.claude.json if missing.'
                    : provider.id === 'gemini'
                      ? 'Imports from .gemini/settings.json first, then ~/.gemini/settings.json if missing.'
                      : 'Imports from .codex/config.toml in this project.';
            const exportTargetReason = hasActiveProject
              ? provider.id === 'claude'
                ? 'Writes .mcp.json, CLAUDE.md, .claude/settings.json, and .claude/skills.'
                : provider.id === 'gemini'
                  ? 'Writes .gemini/settings.json, GEMINI.md, .gemini/policies/ship-permissions.toml, and .gemini/skills.'
                  : 'Writes .codex/config.toml, AGENTS.md, and .agents/skills.'
              : 'Open or create a project to export provider config files.';
            const logo = PROVIDER_LOGO[provider.id];
            return (
              <div key={provider.id} className={cn('transition-colors', isExpanded && 'bg-muted/30')}>
                <div className="flex items-start gap-3 px-4 py-3">
                  <div className="flex size-8 shrink-0 items-center justify-center rounded-lg border bg-card">
                    {logo
                      ? <img src={logo.src} alt={provider.name} className={cn('size-5 object-contain', logo.invertDark && 'dark:invert')} />
                      : <Bot className="size-4 text-muted-foreground" />
                    }
                  </div>

                  <div className="min-w-0 flex-1 space-y-1">
                    <div className="flex flex-wrap items-center gap-1.5">
                      <p className="text-sm font-medium">{provider.name}</p>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Badge variant="outline" className={cn('cursor-default text-[10px]', providerStatusBadgeClass(syncSummary.status))}>
                            {PROVIDER_STATUS_COPY[syncSummary.status]}
                          </Badge>
                        </TooltipTrigger>
                        <TooltipContent className="max-w-xs">
                          Provider sync health for this scope.
                        </TooltipContent>
                      </Tooltip>
                      {provider.checking ? (
                        <Badge variant="outline" className="cursor-default text-[10px] text-muted-foreground">
                          checking...
                        </Badge>
                      ) : provider.installed ? (
                        <Badge variant="outline" className="cursor-default text-[10px] text-muted-foreground">
                          {provider.version ?? 'installed'}
                        </Badge>
                      ) : (
                        <Badge variant="outline" className="cursor-default text-[10px] text-muted-foreground">
                          not found
                        </Badge>
                      )}
                      {issueCount > 0 ? (
                        <Badge variant="outline" className="cursor-default border-amber-500/40 bg-amber-500/10 text-[10px] text-amber-800 dark:text-amber-300">
                          {issueCount} warning{issueCount === 1 ? '' : 's'}
                        </Badge>
                      ) : null}
                    </div>
                    {importSummary[provider.id] && (
                      <p className="text-[11px] text-muted-foreground">{importSummary[provider.id]}</p>
                    )}
                  </div>

                  <div className="flex shrink-0 items-center gap-1.5">
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          type="button"
                          variant="outline"
                          size="xs"
                          disabled={!hasActiveProject || agentScope !== 'project' || importStatus[provider.id] === 'loading'}
                          onClick={() => onImport(provider.id)}
                          className="h-6 px-2 text-[10px]"
                        >
                          <Download className="mr-1 size-3" />
                          {importStatus[provider.id] === 'loading' ? 'Importing…'
                            : importStatus[provider.id] === 'ok' ? 'Imported ✓'
                            : 'Import'}
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        {importDisabledReason}
                      </TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          type="button"
                          variant="outline"
                          size="xs"
                          disabled={!hasActiveProject || exportStatus[provider.id] === 'loading'}
                          onClick={() => onExport(provider.id)}
                          className="h-6 px-2 text-[10px]"
                        >
                          <Upload className="mr-1 size-3" />
                          {exportStatus[provider.id] === 'loading' ? 'Exporting…'
                            : exportStatus[provider.id] === 'ok' ? 'Exported ✓'
                            : 'Export'}
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        {exportTargetReason}
                      </TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          type="button"
                          variant="ghost"
                          size="xs"
                          className="h-6 w-6 p-0"
                          onClick={() => setExpandedProviderId(isExpanded ? '' : provider.id)}
                        >
                          {isExpanded
                            ? <ChevronDown className="size-3.5" />
                            : <ChevronRight className="size-3.5" />
                          }
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        {isExpanded ? 'Hide advanced settings' : 'Show advanced settings'}
                      </TooltipContent>
                    </Tooltip>
                  </div>
                </div>

                {isExpanded && (
                  <div className="space-y-3 border-t bg-muted/20 px-4 py-4">
                    {(!provider.installed || nextIssue) && (
                      <div className={cn(
                        'rounded-md border px-3 py-2 text-[11px]',
                        !provider.installed
                          ? 'border-amber-500/30 bg-amber-500/5 text-amber-800 dark:text-amber-200'
                          : 'border-border/70 bg-background/80 text-muted-foreground'
                      )}>
                        {!provider.installed ? (
                          <div className="flex items-center justify-between gap-2">
                            <p>
                              Install <code>{provider.binary}</code> and then rescan provider detection.
                            </p>
                            <Button
                              type="button"
                              variant="outline"
                              size="xs"
                              className="h-6 px-2 text-[10px]"
                              onClick={() => refreshProviders()}
                            >
                              Rescan
                            </Button>
                          </div>
                        ) : nextIssue ? (
                          <div className="space-y-1">
                            <p className="font-medium text-foreground">Next step</p>
                            <p>{nextIssue.message}</p>
                            {nextIssue.hint ? <p>Hint: {nextIssue.hint}</p> : null}
                          </div>
                        ) : null}
                      </div>
                    )}

                    <div className="space-y-1.5">
                      <div className="flex items-center gap-1.5">
                        <span className="text-xs font-medium">Hook Surface</span>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Info className="size-3 cursor-default text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent>
                            Hook support by provider. Ship stores hooks centrally and only exports native events where supported.
                          </TooltipContent>
                        </Tooltip>
                      </div>
                      {providerHookEvents.length > 0 ? (
                        <div className="flex flex-wrap gap-1.5">
                          {providerHookEvents.map((event) => (
                            <Tooltip key={`${provider.id}-${event.value}`}>
                              <TooltipTrigger asChild>
                                <Badge variant="secondary" className="text-[10px] cursor-default">
                                  {event.label}
                                </Badge>
                              </TooltipTrigger>
                              <TooltipContent className="max-w-xs">{event.description}</TooltipContent>
                            </Tooltip>
                          ))}
                        </div>
                      ) : (
                        <p className="text-[11px] text-muted-foreground">
                          No native hook export support for this provider yet.
                        </p>
                      )}
                    </div>

                    <div className="space-y-2 rounded-md border bg-background/80 p-2.5">
                      <div className="flex items-center justify-between gap-2">
                        <p className="text-[11px] font-medium">
                          Diagnostics
                        </p>
                        <Button
                          type="button"
                          variant="outline"
                          size="xs"
                          className="h-6 px-2 text-[10px]"
                          onClick={() => onRefetchMcpValidation()}
                          disabled={mcpValidationIsFetching}
                        >
                          {mcpValidationIsFetching ? 'Checking…' : 'Validate'}
                        </Button>
                      </div>
                      {syncSummary.issues.length === 0 ? (
                        <p className="text-[11px] text-emerald-700 dark:text-emerald-300">
                          No diagnostics warnings for this provider.
                        </p>
                      ) : (
                        <div className="max-h-40 space-y-1 overflow-auto pr-1">
                          {syncSummary.issues.map((issue, idx) => (
                            <div
                              key={`${provider.id}-${issue.code}-${issue.server_id ?? issue.provider_id ?? idx}`}
                              className={cn(
                                'rounded border px-2 py-1.5 text-[11px]',
                                issue.level === 'error'
                                  ? 'border-rose-500/30 bg-rose-500/5 text-rose-700 dark:text-rose-300'
                                  : issue.level === 'warning'
                                    ? 'border-amber-500/30 bg-amber-500/5 text-amber-700 dark:text-amber-300'
                                    : 'border-border/60 bg-muted/30 text-muted-foreground'
                              )}
                            >
                              <p className="font-medium">{issue.message}</p>
                              {issue.hint && <p className="opacity-90">Hint: {issue.hint}</p>}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </Card>
    </div>
  );
}
