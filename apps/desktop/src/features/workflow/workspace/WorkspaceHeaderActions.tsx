import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Check,
  ChevronsUpDown,
  CircleHelp,
  Link2,
  Bot,
  Plus,
  RefreshCw,
  Settings2,
  Sun,
  Moon,
  Wrench,
} from 'lucide-react';
import {
  Badge,
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Switch,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@ship/primitives';
import { type GitBranchInfo } from '@/lib/platform/tauri/commands';
import { type ProviderInfo } from '@/bindings';

type WorkspaceTypeOption = 'feature' | 'patch' | 'service';
type EnvironmentMode = 'template' | 'custom';
type CreateWorkspaceStep = 'intent' | 'runtime' | 'branch';

interface CreateWorkspaceInput {
  branch: string;
  workspaceType: WorkspaceTypeOption;
  environmentId: string | null;
  providers: string[];
  featureId: string | null;
  releaseId: string | null;
  isWorktree: boolean;
  worktreePath: string | null;
}

interface WorkspaceLinkOption {
  id: string;
  label: string;
}

interface CreateWorkspaceIntent {
  nonce: number;
  branch: string | null;
}

interface WorkspaceHeaderActionsProps {
  gitBranches: GitBranchInfo[];
  existingWorkspaceBranches: string[];
  creatingWorkspace: boolean;
  environmentOptions: WorkspaceLinkOption[];
  providerOptions: ProviderInfo[];
  featureOptions: WorkspaceLinkOption[];
  releaseOptions: WorkspaceLinkOption[];
  createIntent: CreateWorkspaceIntent | null;
  onCreateIntentConsumed: () => void;
  onCreateWorkspace: (input: CreateWorkspaceInput) => Promise<void>;
  canConfigureAgent: boolean;
  onOpenAgentConfig: () => void;
  currentTheme?: string;
  onThemeChange?: (theme: 'light' | 'dark') => void;
}

const WORKSPACE_TYPE_LABELS: Record<WorkspaceTypeOption, string> = {
  feature: 'Feature',
  patch: 'Patch',
  service: 'Service',
};

const CREATE_WORKSPACE_STEPS: Array<{ id: CreateWorkspaceStep; label: string }> = [
  { id: 'intent', label: 'Intent' },
  { id: 'runtime', label: 'Runtime' },
  { id: 'branch', label: 'Branch' },
];

function labelForOption(option: WorkspaceLinkOption | undefined): string {
  if (!option) return '';
  return option.label?.trim() || option.id;
}

function matchesQuery(option: WorkspaceLinkOption, query: string): boolean {
  if (!query) return true;
  const label = labelForOption(option).toLowerCase();
  const id = option.id.toLowerCase();
  return label.includes(query) || id.includes(query);
}

function slugify(input: string): string {
  return input
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 64);
}

export function WorkspaceHeaderActions({
  gitBranches,
  existingWorkspaceBranches,
  creatingWorkspace,
  environmentOptions,
  providerOptions,
  featureOptions,
  releaseOptions,
  createIntent,
  onCreateIntentConsumed,
  onCreateWorkspace,
  canConfigureAgent,
  onOpenAgentConfig,
  currentTheme,
  onThemeChange,
}: WorkspaceHeaderActionsProps) {
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [createStep, setCreateStep] = useState<CreateWorkspaceStep>('intent');

  const [attachExistingBranch, setAttachExistingBranch] = useState(false);
  const [createSourceBranch, setCreateSourceBranch] = useState<string | null>(null);
  const [createBranchSearch, setCreateBranchSearch] = useState('');
  const [createBranchOverride, setCreateBranchOverride] = useState('');
  const [patchTitle, setPatchTitle] = useState('');

  const [createType, setCreateType] = useState<WorkspaceTypeOption>('feature');

  const [createEnvironmentMode, setCreateEnvironmentMode] = useState<EnvironmentMode>('template');
  const [createEnvironmentSearch, setCreateEnvironmentSearch] = useState('');
  const [createEnvironmentId, setCreateEnvironmentId] = useState<string | null>(null);
  const [createProviders, setCreateProviders] = useState<string[]>([]);

  const [createLinkSearch, setCreateLinkSearch] = useState('');
  const [createFeatureId, setCreateFeatureId] = useState<string | null>(null);
  const [createReleaseId, setCreateReleaseId] = useState<string | null>(null);

  const [createIsWorktree, setCreateIsWorktree] = useState(true);
  const [createWorktreePath, setCreateWorktreePath] = useState('');

  const existingWorkspaceBranchSet = useMemo(
    () => new Set(existingWorkspaceBranches),
    [existingWorkspaceBranches],
  );

  const unattachedBranches = useMemo(
    () => gitBranches.filter((entry) => !existingWorkspaceBranchSet.has(entry.name)),
    [gitBranches, existingWorkspaceBranchSet],
  );

  const pickInitialBranch = useCallback(
    (preferred?: string | null): string | null => {
      const candidate = preferred?.trim();
      if (candidate) return candidate;
      return unattachedBranches[0]?.name ?? gitBranches[0]?.name ?? null;
    },
    [unattachedBranches, gitBranches],
  );

  const resetCreateWorkspaceDraft = useCallback(
    (preferredBranch?: string | null) => {
      const initialBranch = pickInitialBranch(preferredBranch);
      const shouldAttach = Boolean(preferredBranch?.trim());
      setAttachExistingBranch(shouldAttach);
      setCreateSourceBranch(initialBranch);
      setCreateBranchSearch('');
      setCreateBranchOverride('');
      setPatchTitle('');
      setCreateType('feature');
      setCreateEnvironmentMode('template');
      setCreateEnvironmentSearch('');
      setCreateEnvironmentId(null);
      const connected = providerOptions
        .filter((provider) => provider.enabled)
        .map((provider) => provider.id);
      const installed = providerOptions
        .filter((provider) => provider.installed)
        .map((provider) => provider.id);
      const defaults = connected.length > 0 ? connected : installed;
      setCreateProviders(defaults);
      setCreateLinkSearch('');
      setCreateFeatureId(null);
      setCreateReleaseId(null);
      setCreateIsWorktree(true);
      setCreateWorktreePath('');
      setCreateStep('intent');
    },
    [pickInitialBranch, providerOptions],
  );

  useEffect(() => {
    if (!createIntent?.nonce) return;
    setCreateDialogOpen(true);
    resetCreateWorkspaceDraft(createIntent.branch);
    onCreateIntentConsumed();
  }, [createIntent?.nonce, createIntent?.branch, onCreateIntentConsumed, resetCreateWorkspaceDraft]);

  const branchSearchQuery = createBranchSearch.trim().toLowerCase();

  const selectableBranches = useMemo(() => {
    const byName = new Map<string, GitBranchInfo>();
    for (const entry of unattachedBranches) {
      byName.set(entry.name, entry);
    }

    if (createSourceBranch && !byName.has(createSourceBranch)) {
      const existing = gitBranches.find((entry) => entry.name === createSourceBranch);
      if (existing) {
        byName.set(existing.name, existing);
      }
    }

    return Array.from(byName.values());
  }, [createSourceBranch, gitBranches, unattachedBranches]);

  const filteredBranches = useMemo(
    () =>
      selectableBranches.filter((entry) => {
        if (!branchSearchQuery) return true;
        return entry.name.toLowerCase().includes(branchSearchQuery);
      }),
    [selectableBranches, branchSearchQuery],
  );

  const environmentSearchQuery = createEnvironmentSearch.trim().toLowerCase();

  const filteredEnvironmentOptions = useMemo(
    () => environmentOptions.filter((option) => matchesQuery(option, environmentSearchQuery)),
    [environmentOptions, environmentSearchQuery],
  );

  const environmentLabel = useMemo(() => {
    if (createEnvironmentMode === 'custom') return 'Custom environment';
    if (!createEnvironmentId) return 'No template selected';
    return (
      labelForOption(environmentOptions.find((option) => option.id === createEnvironmentId)) ||
      createEnvironmentId
    );
  }, [createEnvironmentMode, createEnvironmentId, environmentOptions]);

  const linkSearchQuery = createLinkSearch.trim().toLowerCase();

  const filteredFeatureOptions = useMemo(
    () => featureOptions.filter((option) => matchesQuery(option, linkSearchQuery)),
    [featureOptions, linkSearchQuery],
  );

  const filteredReleaseOptions = useMemo(
    () => releaseOptions.filter((option) => matchesQuery(option, linkSearchQuery)),
    [releaseOptions, linkSearchQuery],
  );

  const linkedCount = Number(Boolean(createFeatureId || createReleaseId));
  const providerCount = createProviders.length;

  const autoBranch = useMemo(() => {
    const featureLabel = createFeatureId
      ? labelForOption(featureOptions.find((option) => option.id === createFeatureId))
      : '';
    const releaseLabel = createReleaseId
      ? labelForOption(releaseOptions.find((option) => option.id === createReleaseId))
      : '';

    const stem =
      (createType === 'patch' && patchTitle.trim()) ||
      featureLabel ||
      releaseLabel ||
      `${createType}-workspace`;

    const prefix = createType === 'service' ? 'service' : createType;
    const slug = slugify(stem) || 'workspace';
    return `${prefix}/${slug}`;
  }, [
    createFeatureId,
    createReleaseId,
    createType,
    patchTitle,
    featureOptions,
    releaseOptions,
  ]);

  const resolvedBranch = useMemo(() => {
    if (attachExistingBranch) {
      return createSourceBranch?.trim() || '';
    }
    const override = createBranchOverride.trim();
    return override || autoBranch;
  }, [attachExistingBranch, createSourceBranch, createBranchOverride, autoBranch]);

  const currentStepIndex = useMemo(
    () => CREATE_WORKSPACE_STEPS.findIndex((step) => step.id === createStep),
    [createStep],
  );
  const onFirstStep = currentStepIndex <= 0;
  const onLastStep = currentStepIndex >= CREATE_WORKSPACE_STEPS.length - 1;
  const canAdvanceStep = useMemo(() => {
    if (createStep === 'runtime') {
      return createProviders.length > 0;
    }
    return true;
  }, [createProviders.length, createStep]);

  const goToNextStep = () => {
    if (!canAdvanceStep || onLastStep) return;
    const next = CREATE_WORKSPACE_STEPS[currentStepIndex + 1];
    if (next) setCreateStep(next.id);
  };

  const goToPreviousStep = () => {
    if (onFirstStep) return;
    const prev = CREATE_WORKSPACE_STEPS[currentStepIndex - 1];
    if (prev) setCreateStep(prev.id);
  };



  const handleCreate = async () => {
    const branch = resolvedBranch.trim();
    if (!branch) return;
    if (createFeatureId && createReleaseId) {
      window.alert('Choose either a feature anchor or a release anchor before creating the workspace.');
      return;
    }

    await onCreateWorkspace({
      branch,
      workspaceType: createType,
      environmentId: createEnvironmentMode === 'template' ? createEnvironmentId : null,
      providers: createProviders,
      featureId: createFeatureId,
      releaseId: createReleaseId,
      isWorktree: createIsWorktree,
      worktreePath: createIsWorktree ? createWorktreePath.trim() || null : null,
    });

    setCreateDialogOpen(false);
    resetCreateWorkspaceDraft();
  };

  const openCreateDialog = (preferredBranch?: string | null) => {
    resetCreateWorkspaceDraft(preferredBranch);
    setCreateDialogOpen(true);
    setCreateStep('intent');
  };

  const renderLinkSection = (
    title: string,
    options: WorkspaceLinkOption[],
    selectedId: string | null,
    onSelect: (id: string | null) => void,
  ) => {
    return (
      <div className="space-y-1.5 rounded-lg border bg-muted/20 p-2.5">
        <div className="flex items-center justify-between gap-2">
          <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">{title}</p>
          <span className="truncate text-[10px] text-muted-foreground">
            {selectedId
              ? labelForOption(options.find((option) => option.id === selectedId)) || selectedId
              : 'Unlinked'}
          </span>
        </div>

        <div className="max-h-28 space-y-1 overflow-y-auto">
          <Button
            size="xs"
            variant={!selectedId ? 'secondary' : 'ghost'}
            className="h-7 w-full justify-start"
            onClick={() => onSelect(null)}
          >
            Unlinked
          </Button>
          {options.map((option) => (
            <Button
              key={option.id}
              size="xs"
              variant={selectedId === option.id ? 'secondary' : 'ghost'}
              className="h-7 w-full justify-between"
              onClick={() => onSelect(option.id)}
            >
              <span className="truncate">{labelForOption(option)}</span>
              {selectedId === option.id && <Check className="size-3" />}
            </Button>
          ))}
          {options.length === 0 ? (
            <p className="px-1 text-[10px] text-muted-foreground">No results.</p>
          ) : null}
        </div>
      </div>
    );
  };

  return (
    <div className="flex items-center gap-2">
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            size="icon-xs"
            className="size-8 bg-primary text-primary-foreground hover:bg-primary/90"
            onClick={onOpenAgentConfig}
            disabled={!canConfigureAgent}
          >
            <Bot className="size-3.5" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>Workspace Agent Configuration</TooltipContent>
      </Tooltip>

      <Dialog
        open={createDialogOpen}
        onOpenChange={(open) => {
          setCreateDialogOpen(open);
          if (!open) {
            resetCreateWorkspaceDraft();
          }
        }}
      >
        <Button
          size="sm"
          variant="outline"
          className="h-8 gap-1.5"
          onClick={() => openCreateDialog()}
        >
          <Plus className="size-3.5" />
          Create Workspace
        </Button>

        <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-[700px]">
          <DialogHeader>
            <DialogTitle>Create Workspace</DialogTitle>
            <DialogDescription>
              {createStep === 'intent'
                ? 'Choose workspace type and planning anchor.'
                : createStep === 'runtime'
                  ? 'Select runtime environment and allowed providers.'
                  : 'Choose branch/worktree strategy and confirm creation.'}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-1 rounded-md border bg-muted/20 p-1">
              {CREATE_WORKSPACE_STEPS.map((step, idx) => {
                const active = step.id === createStep;
                const completed = idx < currentStepIndex;
                return (
                  <div
                    key={step.id}
                    className={`rounded px-2 py-1.5 text-center text-[10px] font-medium transition-colors ${
                      active
                        ? 'bg-primary/15 text-primary'
                        : completed
                          ? 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300'
                          : 'text-muted-foreground'
                    }`}
                  >
                    {step.label}
                  </div>
                );
              })}
            </div>

            <div className="flex flex-wrap items-center gap-2">
              {createStep === 'intent' && (
                <>
              <Label className="mr-1">Type</Label>
              <Popover>
                <PopoverTrigger>
                  <Button size="sm" variant="outline" className="h-8 gap-2">
                    <Wrench className="size-3.5" />
                    {WORKSPACE_TYPE_LABELS[createType]}
                    <ChevronsUpDown className="size-3.5 opacity-60" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-72 p-2" align="start" sideOffset={8}>
                  <div className="space-y-1">
                    {(['feature', 'patch', 'service'] as WorkspaceTypeOption[]).map((type) => (
                      <Button
                        key={type}
                        size="xs"
                        variant={createType === type ? 'secondary' : 'ghost'}
                        className="h-8 w-full justify-between"
                        onClick={() => setCreateType(type)}
                      >
                        <span>{WORKSPACE_TYPE_LABELS[type]} workspace</span>
                        {createType === type && <Check className="size-3.5" />}
                      </Button>
                    ))}
                  </div>
                </PopoverContent>
              </Popover>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button size="icon-sm" variant="ghost" className="size-7">
                    <CircleHelp className="size-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Type controls lifecycle semantics and default branch naming.</TooltipContent>
              </Tooltip>
                </>
              )}

              {createStep === 'runtime' && (
                <>
              <Popover>
                <PopoverTrigger>
                  <Button size="sm" variant="outline" className="h-8 gap-2">
                    <Settings2 className="size-3.5" />
                    Environment
                    <ChevronsUpDown className="size-3.5 opacity-60" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-[min(520px,90vw)] p-3" align="start" sideOffset={8}>
                  <div className="space-y-3">
                    <div className="flex items-center gap-1 rounded-md border p-1">
                      <Button
                        size="xs"
                        variant={createEnvironmentMode === 'template' ? 'secondary' : 'ghost'}
                        className="h-7 flex-1"
                        onClick={() => setCreateEnvironmentMode('template')}
                      >
                        Use template
                      </Button>
                      <Button
                        size="xs"
                        variant={createEnvironmentMode === 'custom' ? 'secondary' : 'ghost'}
                        className="h-7 flex-1"
                        onClick={() => {
                          setCreateEnvironmentMode('custom');
                          setCreateEnvironmentId(null);
                        }}
                      >
                        Customize
                      </Button>
                    </div>

                    {createEnvironmentMode === 'template' ? (
                      <div className="space-y-2">
                        <Input
                          value={createEnvironmentSearch}
                          onChange={(event) => setCreateEnvironmentSearch(event.target.value)}
                          placeholder="Search environment templates..."
                          className="h-8"
                        />
                        <div className="max-h-36 space-y-1 overflow-y-auto">
                          <Button
                            size="xs"
                            variant={!createEnvironmentId ? 'secondary' : 'ghost'}
                            className="h-7 w-full justify-between"
                            onClick={() => setCreateEnvironmentId(null)}
                          >
                            No template
                            {!createEnvironmentId ? <Check className="size-3" /> : null}
                          </Button>
                          {filteredEnvironmentOptions.map((option) => (
                            <Button
                              key={option.id}
                              size="xs"
                              variant={createEnvironmentId === option.id ? 'secondary' : 'ghost'}
                              className="h-7 w-full justify-between"
                              onClick={() => setCreateEnvironmentId(option.id)}
                            >
                              <span className="truncate">{labelForOption(option)}</span>
                              {createEnvironmentId === option.id ? <Check className="size-3" /> : null}
                            </Button>
                          ))}
                          {filteredEnvironmentOptions.length === 0 ? (
                            <p className="px-1 text-[10px] text-muted-foreground">No templates found.</p>
                          ) : null}
                        </div>
                      </div>
                    ) : (
                      <div className="rounded-lg border bg-muted/20 p-3">
                        <p className="text-xs text-muted-foreground">
                          Custom environment wiring is enabled. Template selection is intentionally skipped.
                        </p>
                      </div>
                    )}

                    <p className="text-[10px] text-muted-foreground">Selected: {environmentLabel}</p>
                  </div>
                </PopoverContent>
              </Popover>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button size="icon-sm" variant="ghost" className="size-7">
                    <CircleHelp className="size-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Environment controls providers, tools, and permissions for this workspace.</TooltipContent>
              </Tooltip>
                </>
              )}

              {createStep === 'runtime' && (
                <>
              <Popover>
                <PopoverTrigger>
                  <Button size="sm" variant="outline" className="h-8 gap-2">
                    <Check className="size-3.5" />
                    Providers
                    {providerCount > 0 ? (
                      <Badge variant="secondary" className="h-4.5 px-1.5 text-[9px]">
                        {providerCount}
                      </Badge>
                    ) : null}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-[min(460px,90vw)] p-3" align="start" sideOffset={8}>
                  <div className="space-y-2">
                    <p className="text-[10px] text-muted-foreground">
                      Select providers this workspace should allow during session resolution.
                    </p>
                    <div className="max-h-40 space-y-1 overflow-y-auto">
                      {providerOptions.map((provider) => {
                        const checked = createProviders.includes(provider.id);
                        return (
                          <Button
                            key={provider.id}
                            size="xs"
                            variant={checked ? 'secondary' : 'ghost'}
                            className="h-8 w-full justify-between"
                            onClick={() => {
                              setCreateProviders((current) => {
                                if (current.includes(provider.id)) {
                                  return current.filter((value) => value !== provider.id);
                                }
                                return [...current, provider.id];
                              });
                            }}
                          >
                            <span className="truncate">
                              {provider.name} <span className="text-muted-foreground">({provider.id})</span>
                            </span>
                            {checked ? <Check className="size-3" /> : null}
                          </Button>
                        );
                      })}
                      {providerOptions.length === 0 ? (
                        <p className="px-1 text-[10px] text-muted-foreground">No providers detected yet.</p>
                      ) : null}
                    </div>
                  </div>
                </PopoverContent>
              </Popover>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button size="icon-sm" variant="ghost" className="size-7">
                    <CircleHelp className="size-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Providers selected here become workspace-level overrides.</TooltipContent>
              </Tooltip>
                </>
              )}

              {createStep === 'intent' && (
                <>
              <Popover>
                <PopoverTrigger>
                  <Button size="sm" variant="outline" className="h-8 gap-2">
                    <Link2 className="size-3.5" />
                    Links
                    {linkedCount > 0 ? (
                      <Badge variant="secondary" className="h-4.5 px-1.5 text-[9px]">
                        {linkedCount}
                      </Badge>
                    ) : null}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-[min(620px,94vw)] p-3" align="start" sideOffset={8}>
                  <div className="space-y-3">
                    <p className="text-[10px] text-muted-foreground">
                      Anchor this workspace to exactly one of feature or release.
                    </p>
                    <Input
                      value={createLinkSearch}
                      onChange={(event) => setCreateLinkSearch(event.target.value)}
                      placeholder="Search features and releases..."
                      className="h-8"
                    />
                    <div className="grid grid-cols-1 gap-2 md:grid-cols-2">
                      {renderLinkSection('Feature (anchor)', filteredFeatureOptions, createFeatureId, (id) => {
                        setCreateFeatureId(id);
                        if (id) setCreateReleaseId(null);
                      })}
                      {renderLinkSection('Release (anchor)', filteredReleaseOptions, createReleaseId, (id) => {
                        setCreateReleaseId(id);
                        if (id) setCreateFeatureId(null);
                      })}
                    </div>
                  </div>
                </PopoverContent>
              </Popover>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button size="icon-sm" variant="ghost" className="size-7">
                    <CircleHelp className="size-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Links connect this workspace to planning docs and project context.</TooltipContent>
              </Tooltip>
                </>
              )}
            </div>

            {createStep === 'intent' && createType === 'patch' && (
              <div className="space-y-2">
                <Label>Patch title</Label>
                <Input
                  value={patchTitle}
                  onChange={(event) => setPatchTitle(event.target.value)}
                  placeholder="Optional: human-readable patch title"
                />
              </div>
            )}

            {createStep === 'runtime' && createProviders.length === 0 && (
              <div className="rounded border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-[11px] text-amber-700 dark:text-amber-300">
                Select at least one provider before continuing.
              </div>
            )}

            {createStep === 'branch' && (
            <div className="rounded-lg border bg-muted/20 p-3">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <Label className="text-sm">Attach existing branch</Label>
                  <p className="text-[11px] text-muted-foreground">
                    Optional. If disabled, Ship creates a branch automatically.
                  </p>
                </div>
                <Switch checked={attachExistingBranch} onCheckedChange={setAttachExistingBranch} />
              </div>

              {attachExistingBranch ? (
                <div className="mt-3 space-y-2">
                  <Label>Select existing branch</Label>
                  <Popover>
                    <PopoverTrigger>
                      <Button variant="outline" className="h-9 w-full justify-between">
                        <span className="truncate text-left">{createSourceBranch ?? 'Choose a branch'}</span>
                        <ChevronsUpDown className="size-3.5 opacity-60" />
                      </Button>
                    </PopoverTrigger>
                    <PopoverContent className="w-[min(540px,90vw)] p-3" align="start" sideOffset={8}>
                      <div className="space-y-2">
                        <Input
                          value={createBranchSearch}
                          onChange={(event) => setCreateBranchSearch(event.target.value)}
                          placeholder="Search branches..."
                          className="h-8"
                        />
                        <div className="max-h-48 space-y-1 overflow-y-auto">
                          {filteredBranches.map((entry) => (
                            <Button
                              key={entry.name}
                              size="xs"
                              variant={createSourceBranch === entry.name ? 'secondary' : 'ghost'}
                              className="h-8 w-full justify-between"
                              onClick={() => setCreateSourceBranch(entry.name)}
                            >
                              <span className="truncate text-left">{entry.name}</span>
                              <span className="ml-3 text-[10px] text-muted-foreground">
                                ↑{entry.ahead} ↓{entry.behind}
                              </span>
                            </Button>
                          ))}
                          {filteredBranches.length === 0 ? (
                            <p className="px-1 text-xs text-muted-foreground">No branches available.</p>
                          ) : null}
                        </div>
                      </div>
                    </PopoverContent>
                  </Popover>
                </div>
              ) : (
                <div className="mt-3 space-y-2">
                  <div className="rounded border bg-background/60 px-2.5 py-2 text-xs">
                    Auto branch: <code>{autoBranch}</code>
                  </div>
                  <div className="space-y-1.5">
                    <Label>Branch override (optional)</Label>
                    <Input
                      value={createBranchOverride}
                      onChange={(event) => setCreateBranchOverride(event.target.value)}
                      placeholder="Only set if you need a custom branch"
                    />
                  </div>
                </div>
              )}
            </div>
            )}

            {createStep === 'branch' && (
            <div className="space-y-3 rounded-lg border bg-muted/20 p-3">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <Label className="text-sm">Use worktree</Label>
                  <p className="text-[11px] text-muted-foreground">
                    Default is enabled. If path is blank, Ship assigns a managed location.
                  </p>
                </div>
                <Switch checked={createIsWorktree} onCheckedChange={setCreateIsWorktree} />
              </div>

              {createIsWorktree && (
                <div className="space-y-2">
                  <Label>Worktree path override</Label>
                  <Input
                    value={createWorktreePath}
                    onChange={(event) => setCreateWorktreePath(event.target.value)}
                    placeholder="Optional (defaults to .ship/worktrees/<branch>)"
                  />
                  <p className="text-[10px] text-muted-foreground">
                    Existing git worktrees on this branch are detected and reused automatically.
                  </p>
                </div>
              )}
            </div>
            )}
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setCreateDialogOpen(false);
                resetCreateWorkspaceDraft();
              }}
              disabled={creatingWorkspace}
            >
              Cancel
            </Button>
            {!onFirstStep ? (
              <Button
                variant="outline"
                onClick={goToPreviousStep}
                disabled={creatingWorkspace}
              >
                Back
              </Button>
            ) : null}
            {!onLastStep ? (
              <Button
                onClick={goToNextStep}
                disabled={creatingWorkspace || !canAdvanceStep}
              >
                Next
              </Button>
            ) : (
              <Button onClick={() => void handleCreate()} disabled={creatingWorkspace || !resolvedBranch.trim()}>
                {creatingWorkspace ? <RefreshCw className="size-3.5 animate-spin" /> : <Plus className="size-3.5" />}
                Create Workspace
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>


      <Popover>
        <PopoverTrigger>
          <Button size="icon-xs" variant="outline" className="size-8">
            <Settings2 className="size-3.5" />
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-56 p-3" align="end" sideOffset={8}>
          <div className="space-y-2">
            <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
              Workspace Settings
            </p>
            <div className="space-y-1">
              <p className="text-[10px] text-muted-foreground">Theme</p>
              <div className="grid grid-cols-2 gap-1">
                <Button
                  size="xs"
                  variant={currentTheme === 'light' ? 'secondary' : 'outline'}
                  className="h-7 justify-start gap-1.5"
                  onClick={() => onThemeChange?.('light')}
                >
                  <Sun className="size-3" />
                  Light
                </Button>
                <Button
                  size="xs"
                  variant={currentTheme === 'dark' ? 'secondary' : 'outline'}
                  className="h-7 justify-start gap-1.5"
                  onClick={() => onThemeChange?.('dark')}
                >
                  <Moon className="size-3" />
                  Dark
                </Button>
              </div>
            </div>
          </div>
        </PopoverContent>
      </Popover>


    </div>
  );
}
