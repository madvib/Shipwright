import { FileSearch, Info, LockIcon, Save, Shield, ShieldAlert, Terminal, Zap } from 'lucide-react';
import { Badge } from '@ship/primitives';
import { Button } from '@ship/primitives';
import { Card, CardContent } from '@ship/primitives';
import { Label } from '@ship/primitives';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@ship/primitives';
import { Tooltip, TooltipTrigger, TooltipContent } from '@ship/primitives';
import { cn } from '@/lib/utils';
import type { AgentDiscoveryCache, Permissions } from '@/bindings';
import type { ScopeKey } from '../agents.types';
import { PatternListEditor } from '../PatternListEditor';

const PERMISSION_PRESETS: Array<{
  id: string;
  name: string;
  description: string;
  icon: React.ElementType;
  colorClass: string;
  apply: () => Permissions;
}> = [
  {
    id: 'readonly',
    name: 'Read-only',
    description: 'Read files and run read-only MCP tools. No writes, no shell execution.',
    icon: FileSearch,
    colorClass: 'text-blue-500',
    apply: () => ({
      tools: { allow: ['mcp__*__read*', 'mcp__*__list*', 'mcp__*__get*', 'mcp__*__search*'], deny: ['mcp__*__write*', 'mcp__*__delete*', 'mcp__*__create*', 'mcp__*__exec*'] },
      filesystem: { allow: ['**/*'], deny: [] },
      commands: { allow: [], deny: ['*'] },
      network: { policy: 'none', allow_hosts: [] },
      agent: { require_confirmation: [] },
    }),
  },
  {
    id: 'standard',
    name: 'Ship Guarded',
    description: 'Ship-first baseline — read + Ship MCP by default, risky mutations require explicit opt-in.',
    icon: Shield,
    colorClass: 'text-emerald-500',
    apply: () => ({
      tools: {
        allow: ['Read', 'Glob', 'Grep', 'mcp__ship__*', 'mcp__*__read*', 'mcp__*__list*', 'mcp__*__search*'],
        deny: ['Bash', 'Write', 'Edit', 'MultiEdit', 'mcp__*__write*', 'mcp__*__delete*', 'mcp__*__exec*'],
      },
      filesystem: { allow: ['**/*'], deny: ['/etc/**', '/sys/**', '/proc/**', '~/.ssh/**', '~/.gnupg/**'] },
      commands: {
        allow: ['git status', 'git diff', 'git log', 'ls', 'cat', 'rg', 'find', 'pwd'],
        deny: ['rm -rf', 'git push --force', 'npm publish', 'cargo publish'],
      },
      network: { policy: 'none', allow_hosts: [] },
      agent: {
        require_confirmation: ['Bash', 'Write', 'Edit', 'MultiEdit', 'mcp__*__write*', 'mcp__*__delete*', 'mcp__*__exec*'],
      },
    }),
  },
  {
    id: 'yolo',
    name: 'Full Access',
    description: 'No restrictions. Agent can do anything. Use only in trusted environments.',
    icon: ShieldAlert,
    colorClass: 'text-rose-500',
    apply: () => ({
      tools: { allow: ['*'], deny: [] },
      filesystem: { allow: ['**/*'], deny: [] },
      commands: { allow: ['*'], deny: [] },
      network: { policy: 'unrestricted', allow_hosts: [] },
      agent: { require_confirmation: [] },
    }),
  },
];

export interface PermissionsSectionProps {
  agentScope: ScopeKey;
  activePermissions: Permissions | null;
  permissionsDirty: boolean;
  permissionsTab: 'tools' | 'commands' | 'filesystem';
  setPermissionsTab: (tab: 'tools' | 'commands' | 'filesystem') => void;
  permissionValidationIssues: string[];
  toolAllowPatterns: string[];
  toolDenyPatterns: string[];
  permissionToolSuggestions: Array<{ value: string; label?: string; keywords?: string[] }>;
  commandPatternSuggestions: Array<{ value: string }>;
  filesystemPathSuggestions: Array<{ value: string }>;
  discoveryCache: AgentDiscoveryCache | null;
  savePermissionsIsPending: boolean;
  refreshDiscoveryCacheIsPending: boolean;
  onApplyPreset: (permissions: Permissions) => void;
  onUpdatePermissions: (updater: (current: Permissions) => Permissions) => void;
  onSavePermissions: () => void;
  onRefreshDiscoveryCache: () => void;
}

export function PermissionsSection({
  agentScope,
  activePermissions,
  permissionsDirty,
  permissionsTab,
  setPermissionsTab,
  permissionValidationIssues,
  toolAllowPatterns,
  toolDenyPatterns,
  permissionToolSuggestions,
  commandPatternSuggestions,
  filesystemPathSuggestions,
  discoveryCache,
  savePermissionsIsPending,
  refreshDiscoveryCacheIsPending,
  onApplyPreset,
  onUpdatePermissions,
  onSavePermissions,
  onRefreshDiscoveryCache,
}: PermissionsSectionProps) {
  return (
    <div className="grid gap-4">
      <div className="space-y-4">
        {/* Rule Sets / Presets */}
        <Card size="sm" className="overflow-hidden">
          <div className="flex items-center gap-3 border-b px-4 py-3">
            <div className="flex size-7 items-center justify-center rounded-lg border border-primary/20 bg-primary/10">
              <Zap className="size-3.5 text-primary" />
            </div>
            <div className="flex-1">
              <div className="flex items-center gap-2">
                <h3 className="text-sm font-semibold">Rule Sets</h3>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Info className="size-3 cursor-default text-muted-foreground" />
                  </TooltipTrigger>
                  <TooltipContent className="max-w-xs">
                    Presets apply a curated bundle of tool allow/deny rules. They overwrite your current permissions — customize further after applying.
                  </TooltipContent>
                </Tooltip>
              </div>
            </div>
          </div>
          <CardContent className="grid gap-3 !pt-4 sm:grid-cols-3">
            {PERMISSION_PRESETS.map((preset) => {
              const Icon = preset.icon;
              return (
                <Tooltip key={preset.id}>
                  <TooltipTrigger asChild>
                    <button
                      type="button"
                      className="flex flex-col gap-1.5 rounded-lg border p-3 text-left transition-colors hover:border-primary/40 hover:bg-primary/5"
                      onClick={() => onApplyPreset(preset.apply())}
                    >
                      <div className="flex items-center gap-2">
                        <Icon className={cn('size-3.5', preset.colorClass)} />
                        <span className="text-xs font-semibold">{preset.name}</span>
                      </div>
                      <p className="text-[11px] leading-relaxed text-muted-foreground">{preset.description}</p>
                    </button>
                  </TooltipTrigger>
                  <TooltipContent>Apply {preset.name} preset — overwrites current permissions</TooltipContent>
                </Tooltip>
              );
            })}
          </CardContent>
        </Card>

        {/* Capabilities */}
        <Card size="sm" className="overflow-hidden">
          <div className="flex items-center gap-3 border-b bg-gradient-to-r from-rose-500/10 via-card/80 to-card/50 px-4 py-3">
            <div className="flex size-7 items-center justify-center rounded-lg border border-rose-500/20 bg-rose-500/10">
              <Shield className="size-3.5 text-rose-500" />
            </div>
            <div className="flex-1">
              <h3 className="text-sm font-semibold">Capabilities</h3>
            </div>
            <div className="flex items-center gap-2">
              {discoveryCache && (
                <Badge variant="outline" className="text-[10px]">
                  {discoveryCache.shell_commands.length} commands • {discoveryCache.filesystem_paths.length} paths
                </Badge>
              )}
              {permissionsDirty ? (
                <Badge className="bg-amber-500/20 text-amber-800 hover:bg-amber-500/20 dark:text-amber-300">
                  Unsaved policy changes
                </Badge>
              ) : null}
              <Button
                type="button"
                variant="outline"
                size="xs"
                className="h-6 px-2 text-[10px]"
                onClick={() => onSavePermissions()}
                disabled={!activePermissions || !permissionsDirty || savePermissionsIsPending}
              >
                <Save className="mr-1 size-3" />
                {savePermissionsIsPending ? 'Saving…' : 'Save Policy'}
              </Button>
              <Button
                type="button"
                variant="outline"
                size="xs"
                className="h-6 px-2 text-[10px]"
                onClick={() => onRefreshDiscoveryCache()}
                disabled={refreshDiscoveryCacheIsPending}
              >
                {refreshDiscoveryCacheIsPending ? 'Refreshing…' : 'Refresh detection'}
              </Button>
            </div>
          </div>
          <CardContent className="space-y-6 !pt-5">
            {!activePermissions ? (
              <p className="py-10 text-center text-sm text-muted-foreground">Loading permissions...</p>
            ) : (
              <Tabs value={permissionsTab} onValueChange={(value) => setPermissionsTab(value as 'tools' | 'commands' | 'filesystem')}>
                <TabsList className="mb-4">
                  <TabsTrigger value="tools">MCP Tools</TabsTrigger>
                  <TabsTrigger value="commands">Shell Commands</TabsTrigger>
                  <TabsTrigger value="filesystem">Filesystem</TabsTrigger>
                </TabsList>
                {permissionValidationIssues.length > 0 && (
                  <div className="mb-3 rounded border border-amber-500/30 bg-amber-500/10 px-2.5 py-2 text-[11px] text-amber-700 dark:text-amber-300">
                    <p className="font-medium">Validation warnings: {permissionValidationIssues.length}</p>
                    <p className="mt-0.5">
                      {permissionValidationIssues[0]}
                      {permissionValidationIssues.length > 1 ? ' (fix highlighted lists before saving)' : ''}
                    </p>
                  </div>
                )}

                <TabsContent value="tools" className="space-y-6">
                  <p className="text-[11px] text-muted-foreground">
                    Built-in tools use plain IDs like <code>Edit</code>. MCP tools use <code>mcp__server__tool</code> patterns.
                  </p>
                  <div className="grid gap-6 md:grid-cols-2">
                    <div className="space-y-3">
                      <div className="flex items-center gap-2">
                        <Shield className="size-4 text-emerald-500" />
                        <Label>Allow List</Label>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Info className="size-3 cursor-default text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent className="max-w-xs">
                            Glob patterns for tools the agent is allowed to use. Use <code>*</code> to allow all, or <code>mcp__server__tool</code> to target specific tools. Allow list is checked first — deny takes precedence.
                          </TooltipContent>
                        </Tooltip>
                      </div>
                      <PatternListEditor
                        patterns={toolAllowPatterns}
                        options={permissionToolSuggestions}
                        addLabel="Add Pattern"
                        addValue="mcp__"
                        noResultsText="Type a custom tool pattern."
                        onChange={(applyAllow) => {
                          onUpdatePermissions((current) => ({
                            ...current,
                            tools: {
                              ...current.tools,
                              allow: applyAllow(current.tools?.allow || []),
                              deny: current.tools?.deny || [],
                            },
                          }));
                        }}
                      />
                    </div>

                    <div className="space-y-3">
                      <div className="flex items-center gap-2">
                        <ShieldAlert className="size-4 text-destructive" />
                        <Label>Deny List</Label>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Info className="size-3 cursor-default text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent className="max-w-xs">
                            Deny always overrides allow. Blocked tools will never execute even if they match an allow pattern. Built-in provider tools use plain IDs like <code>Edit</code> and <code>MultiEdit</code>.
                          </TooltipContent>
                        </Tooltip>
                      </div>
                      <PatternListEditor
                        patterns={toolDenyPatterns}
                        options={permissionToolSuggestions}
                        addLabel="Add Pattern"
                        addValue="mcp__*__"
                        noResultsText="Type a custom restriction pattern."
                        onChange={(applyDeny) => {
                          onUpdatePermissions((current) => ({
                            ...current,
                            tools: {
                              ...current.tools,
                              deny: applyDeny(current.tools?.deny || []),
                              allow: current.tools?.allow || ['*'],
                            },
                          }));
                        }}
                      />
                    </div>
                  </div>
                </TabsContent>

                <TabsContent value="commands" className="space-y-6">
                  <div className="grid gap-6 md:grid-cols-3">
                    <div className="space-y-3">
                      <div className="flex items-center gap-2">
                        <Terminal className="size-4 text-emerald-500" />
                        <Label>Allow Commands</Label>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Info className="size-3 cursor-default text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent className="max-w-xs">
                            Command prefixes or patterns that are explicitly allowed.
                          </TooltipContent>
                        </Tooltip>
                      </div>
                      <PatternListEditor
                        patterns={activePermissions.commands?.allow || []}
                        options={commandPatternSuggestions}
                        addLabel="Add Pattern"
                        noResultsText="Type a custom command pattern."
                        onChange={(applyAllow) => {
                          onUpdatePermissions((current) => ({
                            ...current,
                            commands: {
                              ...current.commands,
                              allow: applyAllow(current.commands?.allow || []),
                              deny: current.commands?.deny || [],
                            },
                          }));
                        }}
                      />
                    </div>

                    <div className="space-y-3">
                      <div className="flex items-center gap-2">
                        <ShieldAlert className="size-4 text-destructive" />
                        <Label>Block Commands</Label>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Info className="size-3 cursor-default text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent className="max-w-xs">
                            These command patterns are never executed.
                          </TooltipContent>
                        </Tooltip>
                      </div>
                      <PatternListEditor
                        patterns={activePermissions.commands?.deny || []}
                        options={commandPatternSuggestions}
                        addLabel="Add Pattern"
                        noResultsText="Type a custom blocked command."
                        onChange={(applyDeny) => {
                          onUpdatePermissions((current) => ({
                            ...current,
                            commands: {
                              ...current.commands,
                              deny: applyDeny(current.commands?.deny || []),
                              allow: current.commands?.allow || [],
                            },
                          }));
                        }}
                      />
                    </div>

                    <div className="space-y-3">
                      <div className="flex items-center gap-2">
                        <Info className="size-4 text-amber-500" />
                        <Label>Require Approval</Label>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Info className="size-3 cursor-default text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent className="max-w-xs">
                            Matching commands prompt for confirmation even when allowed.
                          </TooltipContent>
                        </Tooltip>
                      </div>
                      <PatternListEditor
                        patterns={activePermissions.agent?.require_confirmation || []}
                        options={commandPatternSuggestions}
                        addLabel="Add Pattern"
                        noResultsText="Type a command requiring approval."
                        onChange={(applyRequireConfirmation) => {
                          onUpdatePermissions((current) => ({
                            ...current,
                            agent: {
                              ...current.agent,
                              require_confirmation: applyRequireConfirmation(current.agent?.require_confirmation || []),
                            },
                          }));
                        }}
                      />
                    </div>
                  </div>
                </TabsContent>

                <TabsContent value="filesystem" className="space-y-6">
                  <div className="grid gap-6 md:grid-cols-2">
                    <div className="space-y-3">
                      <div className="flex items-center gap-2">
                        <FileSearch className="size-4 text-emerald-500" />
                        <Label>Read/Write Allow</Label>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Info className="size-3 cursor-default text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent className="max-w-xs">
                            Glob patterns for paths the agent can read and write. Prefer scoped directories like <code>~/projects/**</code> and add exceptions explicitly.
                          </TooltipContent>
                        </Tooltip>
                      </div>
                      <PatternListEditor
                        patterns={activePermissions.filesystem?.allow || []}
                        options={filesystemPathSuggestions}
                        addLabel="Add Path"
                        noResultsText="Type a custom path pattern."
                        onChange={(applyAllow) => {
                          onUpdatePermissions((current) => ({
                            ...current,
                            filesystem: {
                              ...current.filesystem,
                              allow: applyAllow(current.filesystem?.allow || []),
                              deny: current.filesystem?.deny || [],
                            },
                          }));
                        }}
                      />
                    </div>

                    <div className="space-y-3">
                      <div className="flex items-center gap-2">
                        <LockIcon className="size-4 text-destructive" />
                        <Label>Block List</Label>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Info className="size-3 cursor-default text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent className="max-w-xs">
                            Paths that can never be accessed, even if they match an allow pattern. Block sensitive directories like <code>~/.ssh/**</code> or <code>/etc/**</code>.
                          </TooltipContent>
                        </Tooltip>
                      </div>
                      <PatternListEditor
                        patterns={activePermissions.filesystem?.deny || []}
                        options={filesystemPathSuggestions}
                        addLabel="Add Pattern"
                        noResultsText="Type a custom blocked path."
                        onChange={(applyDeny) => {
                          onUpdatePermissions((current) => ({
                            ...current,
                            filesystem: {
                              ...current.filesystem,
                              deny: applyDeny(current.filesystem?.deny || []),
                              allow: current.filesystem?.allow || [],
                            },
                          }));
                        }}
                      />
                    </div>
                  </div>
                </TabsContent>
              </Tabs>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
