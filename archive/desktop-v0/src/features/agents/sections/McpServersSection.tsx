import { ExternalLink, Package, PenLine, Plus, Trash2, Wrench } from 'lucide-react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Badge } from '@ship/primitives';
import { Button } from '@ship/primitives';
import { Card } from '@ship/primitives';
import { Input } from '@ship/primitives';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/primitives';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@ship/primitives';
import { Tooltip, TooltipTrigger, TooltipContent } from '@ship/primitives';
import { cn } from '@/lib/utils';
import type {
  CatalogEntry,
  McpProbeReport,
  McpRegistryEntry,
  McpServerConfig,
  McpValidationReport,
  Permissions,
} from '@/bindings';
import {
  type McpEditDraft,
  type ScopeKey,
  MCP_STDIO_ONLY_ALPHA,
  EMPTY_MCP_SERVER,
  inferMcpServerId,
} from '../agents.types';
import { McpServerForm } from '../McpServerForm';
import { ExplorerDialog } from '@/features/agents/shared/ExplorerDialog';

function formatEpochSeconds(value: string): string {
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return value;
  return new Date(numeric * 1000).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
  });
}

function mcpToolPattern(serverId: string, toolName: string): string {
  return `mcp__${serverId}__${toolName}`;
}

function isMcpToolDenied(permissions: Permissions | undefined, serverId: string, toolName: string): boolean {
  if (!permissions) return false;
  const deny = permissions.tools?.deny ?? [];
  const exact = mcpToolPattern(serverId, toolName);
  return deny.includes(exact) || deny.includes(`mcp__${serverId}__*`) || deny.includes('mcp__*__*');
}

function isMcpServerDenied(permissions: Permissions | undefined, serverId: string): boolean {
  if (!permissions) return false;
  return (permissions.tools?.deny ?? []).includes(`mcp__${serverId}__*`);
}

export interface McpServersSectionProps {
  activeAgentConfig: { mcp_servers?: McpServerConfig[] };
  agentScope: ScopeKey;
  mcpEditDraft: McpEditDraft | null;
  setMcpEditDraft: (draft: McpEditDraft | null) => void;
  mcpCatalogInput: string;
  setMcpCatalogInput: (value: string) => void;
  mcpExplorerOpen: boolean;
  setMcpExplorerOpen: (open: boolean) => void;
  mcpExplorerFilter: 'recommended' | 'catalog' | 'registry' | 'all';
  setMcpExplorerFilter: (filter: 'recommended' | 'catalog' | 'registry' | 'all') => void;
  mcpDiagnosticsOpen: boolean;
  setMcpDiagnosticsOpen: (open: boolean) => void;
  recommendedMcpCatalogEntries: CatalogEntry[];
  filteredMcpCatalogEntries: CatalogEntry[];
  mcpRegistryEntries: McpRegistryEntry[];
  mcpRegistryIsFetching: boolean;
  mcpRegistryIsError: boolean;
  installedMcpServerIdSet: Set<string>;
  hasMcpSearchQuery: boolean;
  mcpValidationReport: McpValidationReport | null;
  mcpValidationIsFetching: boolean;
  mcpValidationIsError: boolean;
  mcpValidationError: unknown;
  mcpProbeReport: McpProbeReport | null;
  mcpProbeIsFetching: boolean;
  mcpProbeIsError: boolean;
  mcpProbeError: unknown;
  mcpDiagnosticsIssueCount: number;
  hasNoReachableServers: boolean;
  mcpProbeByServerId: Map<string, { server_id: string; server_name: string; status: string; transport: string; duration_ms: number; message: string | null; warnings: string[]; discovered_tools: Array<{ name: string; description?: string | null }>; }>;
  cachedMcpToolsByServerId: Map<string, Array<{ name: string; description?: string | null }>>;
  activePermissions: Permissions | null;
  savePermissionsIsPending: boolean;
  mcpIdOptions: Array<{ value: string; label?: string; keywords?: (string | null | undefined)[] }>;
  mcpCommandOptions: Array<{ value: string }>;
  mcpEnvKeyOptions: Array<{ value: string }>;
  onInstallCatalogMcpEntry: (entry: CatalogEntry) => void;
  onInstallRegistryEntry: (entry: McpRegistryEntry) => void;
  onRemoveMcpServer: (idx: number) => void;
  onSaveMcpServer: () => void;
  onToggleDiscoveredToolPolicy: (serverId: string, toolName: string) => void;
  onToggleServerToolBlock: (serverId: string) => void;
  onRefetchMcpValidation: () => void;
  onRefetchMcpProbe: () => void;
}

export function McpServersSection({
  activeAgentConfig,
  mcpEditDraft,
  setMcpEditDraft,
  mcpCatalogInput,
  setMcpCatalogInput,
  mcpExplorerOpen,
  setMcpExplorerOpen,
  mcpExplorerFilter,
  setMcpExplorerFilter,
  mcpDiagnosticsOpen,
  setMcpDiagnosticsOpen,
  recommendedMcpCatalogEntries,
  filteredMcpCatalogEntries,
  mcpRegistryEntries,
  mcpRegistryIsFetching,
  mcpRegistryIsError,
  installedMcpServerIdSet,
  hasMcpSearchQuery,
  mcpValidationReport,
  mcpValidationIsFetching,
  mcpValidationIsError,
  mcpValidationError,
  mcpProbeReport,
  mcpProbeIsFetching,
  mcpProbeIsError,
  mcpProbeError,
  mcpDiagnosticsIssueCount,
  hasNoReachableServers,
  mcpProbeByServerId,
  cachedMcpToolsByServerId,
  activePermissions,
  savePermissionsIsPending,
  mcpIdOptions,
  mcpCommandOptions,
  mcpEnvKeyOptions,
  onInstallCatalogMcpEntry,
  onInstallRegistryEntry,
  onRemoveMcpServer,
  onSaveMcpServer,
  onToggleDiscoveredToolPolicy,
  onToggleServerToolBlock,
  onRefetchMcpValidation,
  onRefetchMcpProbe,
}: McpServersSectionProps) {
  return (
    <div className="grid gap-4">
      <Card size="sm" className="overflow-hidden">
        <div className="flex items-center gap-3 border-b bg-gradient-to-r from-violet-500/10 via-card/80 to-card/50 px-4 py-3">
          <div className="flex size-7 items-center justify-center rounded-lg border border-violet-500/20 bg-violet-500/10">
            <Package className="size-3.5 text-violet-500" />
          </div>
          <div className="flex-1">
            <div className="flex items-center gap-2 text-sm font-semibold">
              <h3 className="text-sm font-semibold">MCP Servers</h3>
              {MCP_STDIO_ONLY_ALPHA && (
                <Badge variant="secondary" className="text-[10px]">stdio-only alpha</Badge>
              )}
            </div>
            <p className="text-[11px] text-muted-foreground">Manage your MCP server library and validate connectivity.</p>
          </div>
          <Badge variant="secondary" className="shrink-0 text-[10px]">
            {(activeAgentConfig.mcp_servers ?? []).length} server{(activeAgentConfig.mcp_servers ?? []).length !== 1 ? 's' : ''}
          </Badge>
        </div>

        <div className="border-b bg-muted/20 px-4 py-3">
          <div className="flex flex-wrap items-center gap-2">
            <Button
              type="button"
              size="sm"
              variant="secondary"
              className="h-8"
              onClick={() => setMcpExplorerOpen(true)}
            >
              <Plus className="mr-1.5 size-3.5" />
              Explore
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="h-8"
              onClick={() => setMcpEditDraft({ idx: null, server: { ...EMPTY_MCP_SERVER } })}
            >
              <PenLine className="mr-1.5 size-3.5" />
              Add Manually
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="h-8"
              onClick={() => setMcpDiagnosticsOpen(true)}
            >
              <Wrench className="mr-1.5 size-3.5" />
              Diagnostics
              {mcpDiagnosticsIssueCount > 0 ? (
                <Badge variant="secondary" className="ml-1.5 h-4 px-1.5 py-0 text-[10px]">
                  {mcpDiagnosticsIssueCount}
                </Badge>
              ) : null}
            </Button>
          </div>
          <div className="mt-2 flex flex-wrap items-center gap-2 text-[11px]">
            {hasNoReachableServers ? (
              <Badge variant="outline" className="border-rose-500/40 bg-rose-500/10 text-rose-700 dark:text-rose-300">
                {mcpProbeReport?.reachable_servers ?? 0}/{mcpProbeReport?.checked_servers ?? 0} reachable
              </Badge>
            ) : mcpProbeReport ? (
              <Badge variant="outline" className="text-muted-foreground">
                {mcpProbeReport.reachable_servers}/{mcpProbeReport.checked_servers} reachable
              </Badge>
            ) : null}
            {mcpValidationReport ? (
              <Badge variant="outline" className={mcpValidationReport.ok ? 'text-emerald-700 dark:text-emerald-300' : 'text-amber-700 dark:text-amber-300'}>
                Preflight {mcpValidationReport.ok ? 'ready' : 'needs attention'}
              </Badge>
            ) : null}
          </div>
        </div>

        <ExplorerDialog
          open={mcpExplorerOpen}
          onOpenChange={setMcpExplorerOpen}
          title="MCP Explorer"
          icon={<Package className="size-4 text-violet-500" />}
        >
          <div className="space-y-3">
              <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_180px]">
                <Input
                  value={mcpCatalogInput}
                  onChange={(event) => {
                    const next = event.target.value;
                    setMcpCatalogInput(next);
                    if (next.trim().length > 0 && mcpExplorerFilter === 'recommended') {
                      setMcpExplorerFilter('all');
                    }
                  }}
                  placeholder="Search by name, ID, tag, author"
                  className="h-8 text-xs"
                  autoCapitalize="none"
                  autoCorrect="off"
                  spellCheck={false}
                />
                <Select
                  value={mcpExplorerFilter}
                  onValueChange={(value) => setMcpExplorerFilter(value as 'recommended' | 'catalog' | 'registry' | 'all')}
                >
                  <SelectTrigger size="sm" className="h-8 text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="recommended">Recommended</SelectItem>
                    <SelectItem value="catalog">Catalog</SelectItem>
                    <SelectItem value="registry">Registry</SelectItem>
                    <SelectItem value="all">All</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {(mcpExplorerFilter === 'recommended' || mcpExplorerFilter === 'all') && !hasMcpSearchQuery && (
                <div className="space-y-2 rounded-md border bg-muted/15 p-2.5">
                  <div className="space-y-0.5">
                    <p className="text-[11px] font-semibold">Recommended</p>
                    <p className="text-[10px] text-muted-foreground">
                      Based on embedded catalog entries tagged official.
                    </p>
                  </div>
                  <div className="grid gap-2 sm:grid-cols-2">
                    {recommendedMcpCatalogEntries.map((entry) => {
                      const serverId = inferMcpServerId({
                        id: entry.id.startsWith('mcp-') ? entry.id.slice(4) : entry.id,
                        name: entry.name,
                        command: entry.command ?? '',
                        args: entry.args ?? [],
                        env: {},
                        scope: 'project',
                        server_type: 'stdio',
                        url: null,
                        disabled: false,
                        timeout_secs: null,
                      }).toLowerCase();
                      const installed = installedMcpServerIdSet.has(serverId);
                      return (
                        <div key={`recommended-${entry.id}`} className="rounded border bg-background/70 p-2">
                          <div className="flex items-start justify-between gap-2">
                            <div className="min-w-0">
                              <p className="truncate text-xs font-medium">{entry.name}</p>
                              <p className="truncate text-[10px] text-muted-foreground">{entry.id}</p>
                            </div>
                            <div className="flex items-center gap-1">
                              {entry.source_url ? (
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Button
                                      type="button"
                                      size="icon-sm"
                                      variant="ghost"
                                      className="h-6 w-6"
                                      onClick={() => {
                                        void openUrl(entry.source_url as string);
                                      }}
                                    >
                                      <ExternalLink className="size-3.5" />
                                    </Button>
                                  </TooltipTrigger>
                                  <TooltipContent>Open source docs</TooltipContent>
                                </Tooltip>
                              ) : null}
                              <Button
                                type="button"
                                size="xs"
                                variant="outline"
                                disabled={installed}
                                onClick={() => onInstallCatalogMcpEntry(entry)}
                              >
                                {installed ? 'Installed' : 'Add'}
                              </Button>
                            </div>
                          </div>
                          <p className="mt-1 line-clamp-2 text-[10px] text-muted-foreground">{entry.description}</p>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

              {(mcpExplorerFilter === 'catalog' || mcpExplorerFilter === 'all') && (
                <div className="space-y-2 rounded-md border bg-muted/15 p-2.5">
                  <div className="flex items-center justify-between gap-2">
                    <p className="text-[11px] font-semibold">Catalog</p>
                    <Badge variant="secondary" className="text-[10px]">
                      {filteredMcpCatalogEntries.length} results
                    </Badge>
                  </div>
                  <div className="max-h-[48vh] space-y-1 overflow-y-auto pr-1">
                    {filteredMcpCatalogEntries.map((entry) => {
                      const serverId = inferMcpServerId({
                        id: entry.id.startsWith('mcp-') ? entry.id.slice(4) : entry.id,
                        name: entry.name,
                        command: entry.command ?? '',
                        args: entry.args ?? [],
                        env: {},
                        scope: 'project',
                        server_type: 'stdio',
                        url: null,
                        disabled: false,
                        timeout_secs: null,
                      }).toLowerCase();
                      const installed = installedMcpServerIdSet.has(serverId);
                      return (
                        <div key={`catalog-${entry.id}`} className="flex items-center justify-between gap-2 rounded border bg-background/70 px-2 py-1.5 text-xs">
                          <div className="min-w-0">
                            <p className="truncate font-medium">{entry.name}</p>
                            <p className="truncate text-[10px] text-muted-foreground">{entry.id}</p>
                          </div>
                          <div className="flex items-center gap-1">
                            {entry.source_url ? (
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Button
                                    type="button"
                                    size="icon-sm"
                                    variant="ghost"
                                    className="h-6 w-6"
                                    onClick={() => {
                                      void openUrl(entry.source_url as string);
                                    }}
                                  >
                                    <ExternalLink className="size-3.5" />
                                  </Button>
                                </TooltipTrigger>
                                <TooltipContent>Open source docs</TooltipContent>
                              </Tooltip>
                            ) : null}
                            <Button
                              type="button"
                              size="xs"
                              variant="outline"
                              disabled={installed}
                              onClick={() => onInstallCatalogMcpEntry(entry)}
                            >
                              {installed ? 'Installed' : 'Add'}
                            </Button>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

              {(mcpExplorerFilter === 'registry' || mcpExplorerFilter === 'all') && (
                <div className="space-y-2 rounded-md border bg-muted/15 p-2.5">
                  <div className="flex items-center justify-between gap-2">
                    <p className="text-[11px] font-semibold">Registry</p>
                    {mcpRegistryIsFetching ? (
                      <Badge variant="outline" className="text-[10px]">Searching…</Badge>
                    ) : null}
                  </div>
                  {mcpCatalogInput.trim().length < 2 ? (
                    <p className="text-[11px] text-muted-foreground">Type at least 2 characters to search the registry.</p>
                  ) : mcpRegistryIsError ? (
                    <p className="text-[11px] text-amber-700 dark:text-amber-300">
                      Registry lookup unavailable right now.
                    </p>
                  ) : (
                    <div className="max-h-[48vh] space-y-1 overflow-y-auto pr-1">
                      {mcpRegistryEntries.map((entry) => {
                        const requiredEnv = entry.required_env ?? [];
                        const requiredHeaders = entry.required_headers ?? [];
                        const serverId = inferMcpServerId({
                          id: (entry.id || entry.server_name || entry.title || 'mcp-server').toLowerCase().replace(/[^a-z0-9-]/g, '-').replace(/-+/g, '-').replace(/^-|-$/g, ''),
                          name: entry.title || entry.server_name || entry.id || 'MCP Server',
                          command: entry.command ?? '',
                          args: entry.args ?? [],
                          env: {},
                          scope: 'project',
                          server_type: 'stdio',
                          url: null,
                          disabled: false,
                          timeout_secs: null,
                        }).toLowerCase();
                        const installed = installedMcpServerIdSet.has(serverId);
                        return (
                          <div key={`registry-${entry.id}`} className="flex items-center justify-between gap-2 rounded border bg-background/70 px-2 py-1.5 text-xs">
                            <div className="min-w-0">
                              <p className="truncate font-medium">{entry.title}</p>
                              <p className="truncate text-[10px] text-muted-foreground">
                                {entry.server_name} • {entry.transport} • v{entry.version}
                              </p>
                              {(requiredEnv.length > 0 || requiredHeaders.length > 0) && (
                                <p className="truncate text-[10px] text-amber-700 dark:text-amber-300">
                                  Requires: {requiredEnv.length > 0 ? `${requiredEnv.length} env` : ''}{requiredEnv.length > 0 && requiredHeaders.length > 0 ? ', ' : ''}{requiredHeaders.length > 0 ? `${requiredHeaders.length} headers` : ''}
                                </p>
                              )}
                            </div>
                            <Button
                              type="button"
                              size="xs"
                              variant="outline"
                              disabled={installed}
                              onClick={() => onInstallRegistryEntry(entry)}
                            >
                              {installed ? 'Installed' : 'Add'}
                            </Button>
                          </div>
                        );
                      })}
                      {mcpRegistryEntries.length === 0 ? (
                        <p className="py-2 text-[11px] text-muted-foreground">No registry matches for this query.</p>
                      ) : null}
                    </div>
                  )}
                </div>
              )}
          </div>
        </ExplorerDialog>

        <ExplorerDialog
          open={mcpDiagnosticsOpen}
          onOpenChange={setMcpDiagnosticsOpen}
          title="MCP Diagnostics"
          icon={<Wrench className="size-4 text-violet-500" />}
        >
          <div className="flex h-full min-h-0 flex-col gap-3">
            <div className="flex flex-wrap items-center gap-2 rounded-md border bg-background/70 p-2.5">
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-8"
                onClick={() => onRefetchMcpValidation()}
                disabled={mcpValidationIsFetching}
              >
                {mcpValidationIsFetching ? 'Validating…' : 'Run Validate'}
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-8"
                onClick={() => onRefetchMcpProbe()}
                disabled={(activeAgentConfig.mcp_servers ?? []).length === 0 || mcpProbeIsFetching}
              >
                {mcpProbeIsFetching ? 'Probing…' : 'Run Probe'}
              </Button>
              <span className="text-[11px] text-muted-foreground">
                Validate checks config shape. Probe checks reachability and discovered tools.
              </span>
            </div>

            <Tabs defaultValue="preflight" className="flex min-h-0 flex-1 flex-col gap-3">
              <TabsList className="h-8 w-fit">
                <TabsTrigger value="preflight" className="text-xs">Preflight</TabsTrigger>
                <TabsTrigger value="runtime" className="text-xs">Runtime Probe</TabsTrigger>
              </TabsList>

              <TabsContent value="preflight" className="min-h-0 flex-1">
                <div className="flex h-full min-h-0 flex-col gap-2 rounded-md border bg-background/70 p-3">
                  {mcpValidationIsError ? (
                    <p className="text-[11px] text-destructive">{String(mcpValidationError)}</p>
                  ) : null}
                  {mcpValidationReport ? (
                    <>
                      <div className="flex flex-wrap items-center justify-between gap-2 text-[11px]">
                        <span className="font-semibold">
                          Preflight: {mcpValidationReport.ok ? 'ready' : 'needs attention'}
                        </span>
                        <span className="text-muted-foreground">
                          {mcpValidationReport.checked_servers} servers • {mcpValidationReport.checked_provider_configs} provider configs
                        </span>
                      </div>
                      {mcpValidationReport.issues.length === 0 ? (
                        <p className="text-[11px] text-emerald-700 dark:text-emerald-300">No issues found.</p>
                      ) : (
                        <div className="min-h-0 flex-1 space-y-1 overflow-y-auto pr-1">
                          {mcpValidationReport.issues.map((issue, idx) => (
                            <div
                              key={`${issue.code}-${issue.server_id ?? issue.provider_id ?? idx}`}
                              className={cn(
                                'rounded border px-2 py-1.5 text-[11px]',
                                issue.level === 'error'
                                  ? 'border-rose-500/30 bg-rose-500/5 text-rose-700 dark:text-rose-300'
                                  : issue.level === 'warning'
                                    ? 'border-amber-500/30 bg-amber-500/5 text-amber-700 dark:text-amber-300'
                                    : 'border-border/60 bg-muted/30 text-muted-foreground'
                              )}
                            >
                              <p className="font-medium">
                                {issue.level.toUpperCase()} {issue.server_id ? `• ${issue.server_id}` : issue.provider_id ? `• ${issue.provider_id}` : ''}
                              </p>
                              <p>{issue.message}</p>
                              {issue.hint && <p className="opacity-90">Hint: {issue.hint}</p>}
                              {issue.source_path && <p className="font-mono opacity-80">{issue.source_path}</p>}
                            </div>
                          ))}
                        </div>
                      )}
                    </>
                  ) : (
                    <p className="text-[11px] text-muted-foreground">Run Validate to see preflight diagnostics.</p>
                  )}
                </div>
              </TabsContent>

              <TabsContent value="runtime" className="min-h-0 flex-1">
                <div className="flex h-full min-h-0 flex-col gap-2 rounded-md border bg-background/70 p-3">
                  {mcpProbeIsError ? (
                    <p className="text-[11px] text-destructive">{String(mcpProbeError)}</p>
                  ) : null}
                  {mcpProbeReport ? (
                    <>
                      <div className="flex flex-wrap items-center justify-between gap-2 text-[11px]">
                        <span className="font-semibold">
                          Runtime probe: {mcpProbeReport.reachable_servers}/{mcpProbeReport.checked_servers} reachable • {mcpProbeReport.discovered_tools} tools
                        </span>
                        <span className="text-muted-foreground">{formatEpochSeconds(mcpProbeReport.generated_at)}</span>
                      </div>
                      {(mcpProbeReport.results ?? []).length === 0 ? (
                        <p className="text-[11px] text-muted-foreground">No probe results yet.</p>
                      ) : (
                        <div className="min-h-0 flex-1 space-y-1 overflow-y-auto pr-1">
                          {(mcpProbeReport.results ?? []).map((result) => {
                            const probeTools = result.discovered_tools ?? [];
                            const probeWarnings = result.warnings ?? [];
                            return (
                              <div key={`probe-${result.server_id}`} className="rounded border bg-muted/30 px-2 py-1.5 text-[11px]">
                                <div className="flex items-center justify-between gap-2">
                                  <p className="font-medium">{result.server_name || result.server_id}</p>
                                  <Badge
                                    variant="outline"
                                    className={cn(
                                      'cursor-default text-[10px]',
                                      result.status === 'ready'
                                        ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                                        : result.status === 'partial'
                                          ? 'border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300'
                                          : result.status === 'disabled'
                                            ? 'text-muted-foreground'
                                            : 'border-rose-500/30 bg-rose-500/10 text-rose-700 dark:text-rose-300'
                                    )}
                                  >
                                    {result.status.toUpperCase()}
                                  </Badge>
                                </div>
                                <p className="text-muted-foreground">
                                  {result.transport} • {probeTools.length} tools • {result.duration_ms}ms
                                </p>
                                {result.message ? <p>{result.message}</p> : null}
                                {probeWarnings.slice(0, 1).map((warning) => (
                                  <p key={warning} className="text-amber-700 dark:text-amber-300">{warning}</p>
                                ))}
                              </div>
                            );
                          })}
                        </div>
                      )}
                    </>
                  ) : (
                    <p className="text-[11px] text-muted-foreground">Run Probe to inspect runtime diagnostics.</p>
                  )}
                </div>
              </TabsContent>
            </Tabs>
          </div>
        </ExplorerDialog>

        <div className="divide-y divide-border/50">
          {mcpEditDraft?.idx === null && (
            <McpServerForm
              draft={mcpEditDraft.server}
              onChange={(s) => setMcpEditDraft({ idx: null, server: s })}
              onSave={onSaveMcpServer}
              onCancel={() => setMcpEditDraft(null)}
              idOptions={mcpIdOptions}
              commandOptions={mcpCommandOptions}
              envKeyOptions={mcpEnvKeyOptions}
              isNew
            />
          )}

          {(activeAgentConfig.mcp_servers ?? []).length === 0 && mcpEditDraft === null && (
            <div className="flex flex-col items-center gap-2 px-4 py-8 text-center">
              <Package className="size-8 text-muted-foreground opacity-30" />
              <p className="text-sm text-muted-foreground">No MCP servers configured.</p>
            </div>
          )}

          {(activeAgentConfig.mcp_servers ?? []).map((server, idx) => {
            const serverId = (server.id ?? server.name).trim();
            const normalizedServerId = serverId || inferMcpServerId(server);
            const transport = server.server_type ?? 'stdio';
            const envCount = server.env ? Object.keys(server.env).length : 0;
            const probeResult = mcpProbeByServerId.get(normalizedServerId);
            const probeStatusLabel = probeResult?.status === 'needs-attention'
              ? 'Issue'
              : probeResult?.status === 'partial'
                ? 'Partial'
                : null;
            const discoveredTools = (() => {
              const probeTools = probeResult?.discovered_tools ?? [];
              const cachedTools = cachedMcpToolsByServerId.get(normalizedServerId) ?? [];
              const byName = new Map<string, { name: string; description?: string | null }>();
              for (const tool of [...cachedTools, ...probeTools]) {
                if (!tool?.name) continue;
                byName.set(tool.name, {
                  name: tool.name,
                  description: tool.description ?? byName.get(tool.name)?.description ?? null,
                });
              }
              return Array.from(byName.values());
            })();
            const allToolsBlocked = isMcpServerDenied(activePermissions ?? undefined, normalizedServerId);
            return (
              <div key={`${normalizedServerId}-${idx}`} className={cn('transition-colors', mcpEditDraft?.idx === idx && 'bg-muted/30')}>
                <div
                  className="flex cursor-pointer items-center gap-3 px-4 py-3 transition-colors hover:bg-muted/30"
                  role="button"
                  tabIndex={0}
                  onClick={() => setMcpEditDraft({ idx, server: { ...server } })}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter' || event.key === ' ') {
                      event.preventDefault();
                      setMcpEditDraft({ idx, server: { ...server } });
                    }
                  }}
                >
                  <div className="flex size-7 shrink-0 items-center justify-center rounded-lg border bg-muted/40">
                    <Package className="size-3.5 text-muted-foreground" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <p className="text-sm font-medium">{server.name}</p>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Badge variant="outline" className="cursor-default px-1.5 py-0 text-[9px]">
                            {transport}
                          </Badge>
                        </TooltipTrigger>
                        <TooltipContent>
                          {transport === 'stdio' ? 'Spawned as a local process over stdin/stdout'
                            : transport === 'sse' ? 'Connected via Server-Sent Events (SSE) stream'
                            : 'Connected via HTTP request/response'}
                        </TooltipContent>
                      </Tooltip>
                      {MCP_STDIO_ONLY_ALPHA && transport !== 'stdio' && (
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Badge variant="secondary" className="cursor-default px-1.5 py-0 text-[9px]">
                              discovery-only
                            </Badge>
                          </TooltipTrigger>
                          <TooltipContent>
                            HTTP/SSE endpoints are validated and probed, but active stdio execution is the current alpha path.
                          </TooltipContent>
                        </Tooltip>
                      )}
                      {probeResult && probeStatusLabel ? (
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Badge
                              variant="outline"
                              className={cn(
                                'cursor-default px-1.5 py-0 text-[10px]',
                                probeResult.status === 'partial'
                                    ? 'border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300'
                                  : 'border-rose-500/30 bg-rose-500/10 text-rose-700 dark:text-rose-300'
                              )}
                            >
                              {probeStatusLabel}
                            </Badge>
                          </TooltipTrigger>
                          <TooltipContent className="max-w-xs">
                            {probeResult.message || 'Open diagnostics for details.'}
                          </TooltipContent>
                        </Tooltip>
                      ) : null}
                      {discoveredTools.length > 0 && (
                        <Badge variant="secondary" className="cursor-default px-1.5 py-0 text-[9px]">
                          {discoveredTools.length} tools
                        </Badge>
                      )}
                    </div>
                    <p className="truncate font-mono text-[11px] text-muted-foreground">
                      {transport === 'stdio'
                        ? [server.command, ...(server.args ?? [])].join(' ')
                        : server.url ?? server.command}
                    </p>
                  </div>

                  {envCount > 0 && (
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Badge variant="secondary" className="shrink-0 cursor-default text-[10px]">
                          {envCount} env
                        </Badge>
                      </TooltipTrigger>
                      <TooltipContent>
                        Env vars: {Object.keys(server.env ?? {}).join(', ')}
                      </TooltipContent>
                    </Tooltip>
                  )}

                  <div className="flex shrink-0 items-center gap-1">
                    {discoveredTools.length > 0 && (
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            variant={allToolsBlocked ? 'secondary' : 'outline'}
                            size="xs"
                            className="h-6 px-2 text-[10px]"
                            onClick={(event) => {
                              event.stopPropagation();
                              onToggleServerToolBlock(normalizedServerId);
                            }}
                            disabled={!activePermissions || savePermissionsIsPending}
                          >
                            {allToolsBlocked ? 'All Blocked' : 'Block All'}
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          {allToolsBlocked
                            ? `Allow all discovered tools for ${normalizedServerId}`
                            : `Block all discovered tools for ${normalizedServerId}`}
                        </TooltipContent>
                      </Tooltip>
                    )}
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="ghost"
                          size="xs"
                          className="h-6 w-6 p-0 text-destructive hover:bg-destructive/10 hover:text-destructive"
                          onClick={(event) => {
                            event.stopPropagation();
                            onRemoveMcpServer(idx);
                          }}
                        >
                          <Trash2 className="size-3.5" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>Remove server</TooltipContent>
                    </Tooltip>
                  </div>
                </div>

                {discoveredTools.length > 0 && (
                  <div className="border-t bg-background/40 px-4 py-2">
                    <p className="text-[11px] font-medium">Discovered Tools</p>
                    <div className="mt-1.5 flex flex-wrap gap-1">
                      {discoveredTools.slice(0, 24).map((tool) => {
                        const denied = isMcpToolDenied(activePermissions ?? undefined, normalizedServerId, tool.name);
                        const pattern = mcpToolPattern(normalizedServerId, tool.name);
                        return (
                          <Tooltip key={`${normalizedServerId}-${tool.name}`}>
                            <TooltipTrigger asChild>
                              <button
                                type="button"
                                className={cn(
                                  'rounded border px-1.5 py-0.5 text-[10px] font-mono transition-colors',
                                  denied
                                    ? 'border-rose-500/40 bg-rose-500/10 text-rose-700 dark:text-rose-300'
                                    : 'border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                                )}
                                onClick={() => onToggleDiscoveredToolPolicy(normalizedServerId, tool.name)}
                                disabled={!activePermissions || savePermissionsIsPending}
                              >
                                {tool.name}
                              </button>
                            </TooltipTrigger>
                            <TooltipContent className="max-w-xs">
                              <p>{denied ? 'Blocked by deny list' : 'Allowed (not denied)'}</p>
                              <p className="font-mono text-[10px]">{pattern}</p>
                              {tool.description && (
                                <p className="mt-1 text-[10px] text-muted-foreground">{tool.description}</p>
                              )}
                            </TooltipContent>
                          </Tooltip>
                        );
                      })}
                      {discoveredTools.length > 24 && (
                        <Badge variant="outline" className="text-[10px]">
                          +{discoveredTools.length - 24} more
                        </Badge>
                      )}
                    </div>
                  </div>
                )}
                {mcpEditDraft?.idx === idx && (
                  <McpServerForm
                    draft={mcpEditDraft.server}
                    onChange={(s) => setMcpEditDraft({ idx, server: s })}
                    onSave={onSaveMcpServer}
                    onCancel={() => setMcpEditDraft(null)}
                    idOptions={mcpIdOptions}
                    commandOptions={mcpCommandOptions}
                    envKeyOptions={mcpEnvKeyOptions}
                  />
                )}
              </div>
            );
          })}
        </div>
      </Card>
    </div>
  );
}
