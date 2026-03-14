import { useEffect, useMemo, useState } from 'react';
import { Bot, Link2, Loader2, Lock, Search, Server, Sparkles } from 'lucide-react';
import {
  Badge,
  Button,
  Checkbox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
} from '@ship/primitives';
import { type ProviderInfo } from '@/bindings';
import { useAgentAssetInventory } from '@/features/agents/shared/useAgentAssetInventory';
import { useMcpRegistrySearch } from '@/features/agents/shared/useMcpRegistrySearch';

type WorkspaceAgentDialogProps = {
  open: boolean;
  branch: string;
  workspaceType: 'feature' | 'patch' | 'service';
  providerInfos: ProviderInfo[];
  currentProviders: string[];
  currentMcpServers: string[];
  currentSkills: string[];
  saving: boolean;
  onOpenChange: (open: boolean) => void;
  onOpenMcpSettings?: () => void;
  onOpenPermissionsSettings?: () => void;
  onSave: (input: { providers: string[]; mcpServers: string[]; skills: string[] }) => Promise<void>;
};

const REQUIRED_SKILLS_BY_WORKSPACE_TYPE: Record<'feature' | 'patch' | 'service', string[]> = {
  feature: ['ship-workflow', 'task-policy', 'start-session'],
  patch: ['ship-workflow', 'task-policy'],
  service: ['ship-workflow', 'task-policy', 'workspace-session-lifecycle'],
};

function toggleList(current: string[], value: string): string[] {
  const normalized = value.trim();
  if (!normalized) return current;
  if (current.includes(normalized)) {
    return current.filter((entry) => entry !== normalized);
  }
  return [...current, normalized];
}

function normalizeUnique(values: string[]): string[] {
  const next: string[] = [];
  for (const value of values) {
    const trimmed = value.trim();
    if (!trimmed) continue;
    if (!next.includes(trimmed)) {
      next.push(trimmed);
    }
  }
  return next;
}

export function WorkspaceAgentDialog({
  open,
  branch,
  workspaceType,
  providerInfos,
  currentProviders,
  currentMcpServers,
  currentSkills,
  saving,
  onOpenChange,
  onOpenMcpSettings,
  onOpenPermissionsSettings,
  onSave,
}: WorkspaceAgentDialogProps) {
  const [selectedProviders, setSelectedProviders] = useState<string[]>([]);
  const [selectedMcpServers, setSelectedMcpServers] = useState<string[]>([]);
  const [selectedSkills, setSelectedSkills] = useState<string[]>([]);

  const [mcpSearch, setMcpSearch] = useState('');
  const [skillSearch, setSkillSearch] = useState('');
  const [registrySearch, setRegistrySearch] = useState('');

  const requiredSkillIds = useMemo(
    () => REQUIRED_SKILLS_BY_WORKSPACE_TYPE[workspaceType] ?? [],
    [workspaceType],
  );
  const requiredSkillSet = useMemo(() => new Set(requiredSkillIds), [requiredSkillIds]);

  useEffect(() => {
    if (!open) return;
    setSelectedProviders(normalizeUnique(currentProviders));
    setSelectedMcpServers(normalizeUnique(currentMcpServers));
    setSelectedSkills(normalizeUnique([...requiredSkillIds, ...currentSkills]));
  }, [open, branch, currentProviders, currentMcpServers, currentSkills, requiredSkillIds]);

  const {
    mcpServers: localMcpOptions,
    skills: skillOptions,
    loading: loadingLocalData,
    error: inventoryError,
  } = useAgentAssetInventory({
    enabled: open,
    includeProviders: false,
    includeMcpServers: true,
    includeSkills: true,
    skillScope: null,
  });

  const registryQuery = useMcpRegistrySearch({
    query: registrySearch,
    enabled: open,
    limit: 20,
  });
  const registryResults = registryQuery.data ?? [];
  const loadingRegistry = registryQuery.isFetching;

  const filteredMcpOptions = useMemo(() => {
    const query = mcpSearch.trim().toLowerCase();
    if (!query) return localMcpOptions;
    return localMcpOptions.filter((server) => {
      return server.id.toLowerCase().includes(query) || server.label.toLowerCase().includes(query);
    });
  }, [localMcpOptions, mcpSearch]);

  const filteredSkills = useMemo(() => {
    const query = skillSearch.trim().toLowerCase();
    if (!query) return skillOptions;
    return skillOptions.filter((skill) => {
      return (
        skill.id.toLowerCase().includes(query) ||
        skill.name.toLowerCase().includes(query) ||
        (skill.description ?? '').toLowerCase().includes(query)
      );
    });
  }, [skillOptions, skillSearch]);

  const knownMcpIds = useMemo(() => new Set(localMcpOptions.map((server) => server.id)), [localMcpOptions]);
  const unresolvedMcpIds = selectedMcpServers.filter((id) => !knownMcpIds.has(id));

  const handleSave = async () => {
    await onSave({
      providers: normalizeUnique(selectedProviders),
      mcpServers: normalizeUnique(selectedMcpServers),
      skills: normalizeUnique([...requiredSkillIds, ...selectedSkills]),
    });
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="h-[min(84vh,760px)] w-[min(1120px,calc(100vw-1.5rem))] max-w-none overflow-hidden p-0 sm:max-w-none">
        <div className="flex h-full min-h-0 flex-col">
          <DialogHeader className="shrink-0 border-b px-6 py-5">
            <DialogTitle className="flex items-center gap-2">
              <Bot className="size-4 text-primary" />
              Workspace Agent Configuration
            </DialogTitle>
            <DialogDescription>
              Workspace-specific provider, MCP, and skill overrides for <code>{branch}</code>.
            </DialogDescription>
          </DialogHeader>

          <div className="grid min-h-0 flex-1 gap-4 overflow-hidden p-4 lg:grid-cols-3">
            <section className="flex min-h-0 flex-col gap-2 rounded-lg border bg-muted/20 p-3">
              <div className="flex items-center justify-between">
                <p className="text-xs font-semibold">Providers</p>
                <Badge variant="secondary">{selectedProviders.length}</Badge>
              </div>
              <div className="min-h-0 flex-1 space-y-1 overflow-y-auto pr-1">
                {providerInfos.map((provider) => {
                  const checked = selectedProviders.includes(provider.id);
                  return (
                    <label key={provider.id} className="flex cursor-pointer items-center gap-2 rounded px-1 py-1 text-xs hover:bg-background/70">
                      <Checkbox
                        checked={checked}
                        onCheckedChange={() => setSelectedProviders((current) => toggleList(current, provider.id))}
                      />
                      <span className="flex-1 truncate">{provider.name}</span>
                      {provider.enabled ? <Badge variant="outline">connected</Badge> : null}
                    </label>
                  );
                })}
                {providerInfos.length === 0 ? (
                  <p className="text-[11px] text-muted-foreground">No providers detected yet.</p>
                ) : null}
              </div>
            </section>

            <section className="flex min-h-0 flex-col gap-2 rounded-lg border bg-muted/20 p-3">
              <div className="flex items-center justify-between">
                <p className="text-xs font-semibold">MCP Servers</p>
                <Badge variant="secondary">{selectedMcpServers.length}</Badge>
              </div>

              <div className="space-y-2">
                <div className="relative">
                  <Search className="pointer-events-none absolute left-2 top-2.5 size-3.5 text-muted-foreground" />
                  <Input
                    value={mcpSearch}
                    onChange={(event) => setMcpSearch(event.target.value)}
                    placeholder="Search local MCP..."
                    className="h-8 pl-7"
                  />
                </div>
              </div>

              <div className="min-h-0 flex-1 space-y-3 overflow-y-auto pr-1">
                <div className="space-y-1">
                  {filteredMcpOptions.map((server) => {
                    const checked = selectedMcpServers.includes(server.id);
                    return (
                      <label key={server.id} className="flex cursor-pointer items-center gap-2 rounded px-1 py-1 text-xs hover:bg-background/70">
                        <Checkbox
                          checked={checked}
                          onCheckedChange={() => setSelectedMcpServers((current) => toggleList(current, server.id))}
                        />
                        <span className="truncate">{server.label}</span>
                        <span className="ml-auto truncate text-[10px] text-muted-foreground">{server.id}</span>
                      </label>
                    );
                  })}
                  {filteredMcpOptions.length === 0 ? (
                    <p className="text-[11px] text-muted-foreground">No local MCP servers.</p>
                  ) : null}
                </div>

                <div className="space-y-2 border-t pt-2">
                  <div className="relative">
                    <Server className="pointer-events-none absolute left-2 top-2.5 size-3.5 text-muted-foreground" />
                    <Input
                      value={registrySearch}
                      onChange={(event) => setRegistrySearch(event.target.value)}
                      placeholder="Search MCP registry..."
                      className="h-8 pl-7"
                    />
                  </div>
                  <div className="space-y-1">
                    {loadingRegistry ? (
                      <p className="text-[11px] text-muted-foreground">Searching registry…</p>
                    ) : registryResults.length > 0 ? (
                      registryResults.map((entry) => (
                        <div key={entry.id} className="flex items-center gap-2 rounded border bg-background/70 px-2 py-1.5 text-[11px]">
                          <span className="truncate">{entry.server_name || entry.title}</span>
                          <span className="ml-auto truncate text-[10px] text-muted-foreground">{entry.id}</span>
                          <Button
                            size="xs"
                            variant="outline"
                            className="h-6 px-2"
                            onClick={() => setSelectedMcpServers((current) => toggleList(current, entry.id))}
                          >
                            Add
                          </Button>
                        </div>
                      ))
                    ) : registrySearch.trim().length >= 2 ? (
                      <p className="text-[11px] text-muted-foreground">No registry matches.</p>
                    ) : (
                      <p className="text-[11px] text-muted-foreground">Type at least 2 characters.</p>
                    )}
                  </div>
                </div>

                {unresolvedMcpIds.length > 0 ? (
                  <div className="space-y-1 rounded border border-amber-400/40 bg-amber-400/10 p-2">
                    <p className="text-[10px] font-semibold uppercase tracking-wide text-amber-800 dark:text-amber-200">
                      Unresolved IDs
                    </p>
                    <div className="flex flex-wrap gap-1">
                      {unresolvedMcpIds.map((id) => (
                        <Button
                          key={id}
                          size="xs"
                          variant="outline"
                          className="h-6 border-amber-500/40 bg-background/80 px-2 text-[10px]"
                          onClick={() =>
                            setSelectedMcpServers((current) => current.filter((entry) => entry !== id))
                          }
                        >
                          {id}
                        </Button>
                      ))}
                    </div>
                  </div>
                ) : null}
              </div>
            </section>

            <section className="flex min-h-0 flex-col gap-2 rounded-lg border bg-muted/20 p-3">
              <div className="flex items-center justify-between">
                <p className="text-xs font-semibold">Skills</p>
                <Badge variant="secondary">{selectedSkills.length}</Badge>
              </div>
              <div className="relative">
                <Sparkles className="pointer-events-none absolute left-2 top-2.5 size-3.5 text-muted-foreground" />
                <Input
                  value={skillSearch}
                  onChange={(event) => setSkillSearch(event.target.value)}
                  placeholder="Search skills..."
                  className="h-8 pl-7"
                />
              </div>
              <div className="flex flex-wrap gap-1">
                {requiredSkillIds.map((skillId) => (
                  <Badge key={skillId} variant="outline" className="h-5 px-1.5 text-[10px] font-medium">
                    <Lock className="mr-1 size-3" />
                    {skillId}
                  </Badge>
                ))}
              </div>
              <div className="min-h-0 flex-1 space-y-1 overflow-y-auto pr-1">
                {filteredSkills.map((skill) => {
                  const checked = selectedSkills.includes(skill.id);
                  const required = requiredSkillSet.has(skill.id);
                  return (
                    <label key={skill.id} className="block cursor-pointer rounded border bg-background/70 p-2 text-xs hover:bg-background">
                      <div className="flex items-start gap-2">
                        <Checkbox
                          checked={checked}
                          disabled={required}
                          onCheckedChange={() => {
                            if (required) return;
                            setSelectedSkills((current) => toggleList(current, skill.id));
                          }}
                        />
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-1.5">
                            <p className="truncate font-medium">{skill.name}</p>
                            {required ? (
                              <Badge variant="outline" className="h-5 gap-1 px-1.5 text-[10px] font-medium">
                                <Lock className="size-3" />
                                Required
                              </Badge>
                            ) : null}
                          </div>
                          <p className="truncate text-[10px] text-muted-foreground">{skill.id}</p>
                          {skill.description ? (
                            <p className="mt-0.5 line-clamp-2 text-[10px] text-muted-foreground">{skill.description}</p>
                          ) : null}
                        </div>
                      </div>
                    </label>
                  );
                })}
                {filteredSkills.length === 0 ? (
                  <p className="text-[11px] text-muted-foreground">No skills available.</p>
                ) : null}
              </div>
            </section>
          </div>

          <div className="shrink-0 space-y-3 border-t px-4 py-3">
            {loadingLocalData ? (
              <div className="flex items-center gap-2 rounded border bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
                <Loader2 className="size-3.5 animate-spin" />
                Loading workspace agent assets...
              </div>
            ) : null}
            {inventoryError ? (
              <div className="rounded border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                {inventoryError}
              </div>
            ) : null}

            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex flex-wrap items-center gap-1 text-[11px] text-muted-foreground">
                <span>Need manual setup?</span>
                <Button
                  size="xs"
                  variant="link"
                  className="h-auto p-0 text-[11px]"
                  onClick={() => {
                    onOpenChange(false);
                    onOpenMcpSettings?.();
                  }}
                >
                  <Link2 className="size-3" />
                  MCP settings
                </Button>
                <span>or</span>
                <Button
                  size="xs"
                  variant="link"
                  className="h-auto p-0 text-[11px]"
                  onClick={() => {
                    onOpenChange(false);
                    onOpenPermissionsSettings?.();
                  }}
                >
                  permissions
                </Button>
              </div>

              <DialogFooter>
                <Button variant="outline" onClick={() => onOpenChange(false)} disabled={saving}>
                  Cancel
                </Button>
                <Button onClick={() => void handleSave()} disabled={saving}>
                  {saving ? <Loader2 className="size-3.5 animate-spin" /> : null}
                  Save Workspace Agent
                </Button>
              </DialogFooter>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
