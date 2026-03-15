import { useMemo, useState, useEffect } from 'react';
import { useNavigate } from '@tanstack/react-router';
import {
  FolderOpen,
  RefreshCcw,
  Search,
  Settings2,
  Sparkles,
  Plus,
  ChevronLeft,
  ChevronRight,
  GitBranch,
  CheckCircle2,
  Circle,
  Bot,
  Layout,
} from 'lucide-react';
import {
  ModeConfig,
  ProjectDiscovery as Project,
  StatusConfig,
  WorkspaceGitStatusSummary,
  ProviderInfo,
} from '@/bindings';
import { DEFAULT_STATUSES } from '@/lib/workspace-ui';
import { getGitStatusForPathCmd, listModesCmd, listProvidersCmd } from '@/lib/platform/tauri/commands';
import { AlertDialog, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from '@ship/primitives';
import { Badge } from '@ship/primitives';
import { Button } from '@ship/primitives';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@ship/primitives';
import { Input } from '@ship/primitives';
import { Textarea } from '@ship/primitives';
import { PageFrame, PageHeader } from '@ship/primitives';


export interface CreateProjectInput {
  name: string;
  description?: string;
  directory: string;
  useDefaults: boolean;
  selectedStatuses: string[];
  enabledAgents: string[];
  selectedPreset?: string;
  gitCommitCategories: string[];
  selectedModes: string[];
}

interface ProjectOnboardingProps {
  detectedProject: Project | null;
  detectingProject: boolean;
  creatingProject: boolean;
  recentProjects: Project[];
  onRefreshDetection: () => void;
  onOpenProject: () => void;
  onCreateProject: (input: CreateProjectInput) => Promise<any>;
  onPickDirectory: () => Promise<string | null>;
  onSelectProject: (project: Project) => void;
  onOpenSettings: (tab?: 'global' | 'project' | 'agents' | 'modules') => void;
}

export default function ProjectOnboarding({
  detectedProject,
  detectingProject,
  creatingProject,
  recentProjects,
  onRefreshDetection,
  onOpenProject,
  onCreateProject,
  onPickDirectory,
  onSelectProject,
  onOpenSettings,
}: ProjectOnboardingProps) {
  const navigate = useNavigate();
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [currentStep, setCurrentStep] = useState(0);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [directory, setDirectory] = useState('');
  const [selectedStatuses, setSelectedStatuses] = useState<string[]>(
    DEFAULT_STATUSES.map((status: StatusConfig) => status.id)
  );
  const [enabledAgents, setEnabledAgents] = useState<string[]>(['claude']);
  const [gitCommitCategories, setGitCommitCategories] = useState<string[]>([]);
  const [gitStatus, setGitStatus] = useState<WorkspaceGitStatusSummary | null>(null);
  const [loadingGitStatus, setLoadingGitStatus] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [availableProviders, setAvailableProviders] = useState<ProviderInfo[]>([]);
  const [availableModes, setAvailableModes] = useState<ModeConfig[]>([]);
  const [selectedModes, setSelectedModes] = useState<string[]>(['frontend-react']);
  const [loadingInitialData, setLoadingInitialData] = useState(false);

  // Reset step when dialog opens/closes
  useEffect(() => {
    if (!createDialogOpen) {
      setCurrentStep(0);
      setFormError(null);
    } else {
      // Fetch initial data when opening
      void fetchInitialData();
    }
  }, [createDialogOpen]);

  const fetchInitialData = async () => {
    setLoadingInitialData(true);
    try {
      const [providersRes, modesRes] = await Promise.all([
        listProvidersCmd(),
        listModesCmd()
      ]);
      
      if (providersRes.status === 'ok') {
        setAvailableProviders(providersRes.data);
      }
      
      setAvailableModes(modesRes);
    } catch (err) {
      console.error('Failed to fetch initial onboarding data', err);
    } finally {
      setLoadingInitialData(false);
    }
  };

  const projectOptions = useMemo(() => {
    const byPath = new Map<string, Project>();
    if (detectedProject) {
      byPath.set(detectedProject.path, detectedProject);
    }
    for (const project of recentProjects) {
      if (!byPath.has(project.path)) {
        byPath.set(project.path, project);
      }
    }
    return Array.from(byPath.values());
  }, [detectedProject, recentProjects]);


  const toggleAgent = (agentId: string) => {
    setEnabledAgents((prev) =>
      prev.includes(agentId) ? prev.filter((id) => id !== agentId) : [...prev, agentId]
    );
  };

  const toggleMode = (modeId: string) => {
    setSelectedModes((prev) => {
      if (prev.includes(modeId)) {
        if (prev.length === 1) return prev;
        return prev.filter((id) => id !== modeId);
      }
      return [...prev, modeId];
    });
  };

  const toggleCommitCategory = (category: string) => {
    setGitCommitCategories((prev) => {
      if (prev.includes(category)) {
        return prev.filter((id) => id !== category);
      }
      return [...prev, category];
    });
  };

  const fetchGitStatus = async (path: string) => {
    setLoadingGitStatus(true);
    const result = await getGitStatusForPathCmd(path);
    if (result.status === 'ok') {
      setGitStatus(result.data);
    } else {
      setGitStatus(null);
    }
    setLoadingGitStatus(false);
  };

  const handleNext = async () => {
    if (currentStep === 0) {
      if (!name.trim() || !directory.trim()) {
        setFormError('Name and directory are required.');
        return;
      }
      setFormError(null);
      await fetchGitStatus(directory);
      setCurrentStep(1);
    } else if (currentStep === 1) {
      setCurrentStep(2);
    }
  };

  const handleBack = () => {
    setCurrentStep((prev) => Math.max(0, prev - 1));
  };


  const pickDirectory = async () => {
    const picked = await onPickDirectory();
    if (picked) {
      setDirectory(picked);
    }
  };

  const handleCreate = async () => {
    if (currentStep !== 2) return;

    setFormError(null);

    const cleanName = name.trim();
    if (!cleanName) {
      setFormError('Project name is required.');
      setCurrentStep(0);
      return;
    }

    if (!directory.trim()) {
      setFormError('Choose a directory for this project.');
      setCurrentStep(0);
      return;
    }

    try {
      const info = await onCreateProject({
        name: cleanName,
        description: description.trim() || undefined,
        directory,
        useDefaults: false,
        selectedStatuses,
        enabledAgents,
        selectedPreset: selectedModes[0], // Keep active_mode as first selection
        gitCommitCategories,
        selectedModes, // Pass all selected modes
      });

      if (info) {
        setCreateDialogOpen(false);
        // Reset state
        setName('');
        setDescription('');
        setDirectory('');
        setSelectedStatuses(DEFAULT_STATUSES.map((status: StatusConfig) => status.id));
        setCurrentStep(0);

        // Navigate to the overview of the newly created project
        void navigate({ to: '/project/overview' });
      }
    } catch (error) {
      setFormError(String(error));
    }
  };

  return (
    <PageFrame width="wide">
      <PageHeader
        title="Select a project"
        description="Open an existing project or create a new one to get started."
        actions={
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="icon" onClick={() => onOpenSettings()}>
              <Settings2 className="size-4" />
            </Button>
          </div>
        }
      />

      <Card size="sm" className="overflow-hidden">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <Search className="size-4" />
            Existing Projects
          </CardTitle>
          <CardDescription>
            Open one from your registry, or detect a nearby project in this workspace.
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-3 !pt-5">
          <div className="flex flex-wrap items-center gap-2">
            <Button className="gap-2" onClick={onOpenProject}>
              <FolderOpen className="size-4" />
              Open Project Folder
            </Button>

            <AlertDialog open={createDialogOpen} onOpenChange={setCreateDialogOpen}>
              <AlertDialogTrigger render={
                <Button variant="outline" className="gap-2">
                  <Plus className="size-4" />
                  Create New Project
                </Button>
              } />

              <AlertDialogContent size="default" className="w-[min(96vw,880px)] max-w-none">
                <AlertDialogHeader className="place-items-start text-left">
                  <AlertDialogTitle>Create New Project</AlertDialogTitle>
                  <AlertDialogDescription>
                    Initialize a <code>.ship/</code> workspace in your project directory.
                  </AlertDialogDescription>
                </AlertDialogHeader>

                <div className="flex flex-col gap-6 min-h-[400px]">
                  {/* Step Indicator */}
                  <div className="flex items-center justify-between px-1">
                    {[
                      { id: 0, label: 'Basics', icon: Sparkles },
                      { id: 1, label: 'Version Control', icon: GitBranch },
                      { id: 2, label: 'Agents & Presets', icon: Bot },
                    ].map((step, idx) => (
                      <div key={step.id} className="flex items-center gap-2">
                        <div
                          className={[
                            'flex size-8 items-center justify-center rounded-full border transition-all',
                            currentStep === step.id
                              ? 'border-primary bg-primary text-primary-foreground shadow-sm'
                              : currentStep > step.id
                              ? 'border-primary bg-primary/10 text-primary'
                              : 'border-muted bg-muted/30 text-muted-foreground',
                          ].join(' ')}
                        >
                          {currentStep > step.id ? (
                            <CheckCircle2 className="size-4" />
                          ) : (
                            <step.icon className="size-4" />
                          )}
                        </div>
                        <div className="hidden sm:block">
                          <p
                            className={[
                              'text-xs font-medium',
                              currentStep === step.id ? 'text-foreground' : 'text-muted-foreground',
                            ].join(' ')}
                          >
                            {step.label}
                          </p>
                        </div>
                        {idx < 2 && (
                          <div className="mx-2 hidden h-px w-8 bg-muted sm:block" />
                        )}
                      </div>
                    ))}
                  </div>

                  {loadingInitialData ? (
                    <div className="flex flex-col items-center justify-center py-20 gap-3">
                      <RefreshCcw className="size-8 animate-spin text-primary" />
                      <p className="text-sm text-muted-foreground">Loading configuration options...</p>
                    </div>
                  ) : (
                    <>
                      {currentStep === 0 && (
                        <div className="space-y-5 animate-in fade-in slide-in-from-bottom-2">
                          <div className="grid gap-4 md:grid-cols-2">
                            <div className="space-y-1.5">
                              <label className="text-sm font-medium">Project Name</label>
                              <Input
                                value={name}
                                onChange={(event) => setName(event.target.value)}
                                onKeyDown={(event) => {
                                  if (event.key === 'Enter') {
                                    event.preventDefault();
                                    handleNext();
                                  }
                                }}
                                placeholder="Acme Web App"
                                disabled={creatingProject}
                              />
                            </div>

                            <div className="space-y-1.5">
                              <label className="text-sm font-medium">Directory</label>
                              <div className="flex gap-2">
                                <Input value={directory} readOnly placeholder="Choose a folder…" />
                                <Button
                                  type="button"
                                  variant="outline"
                                  onClick={pickDirectory}
                                  disabled={creatingProject}
                                >
                                  <FolderOpen className="size-4" />
                                </Button>
                              </div>
                            </div>
                          </div>

                          <div className="space-y-1.5">
                            <label className="text-sm font-medium">
                              Description <span className="text-muted-foreground font-normal">(optional)</span>
                            </label>
                            <Textarea
                              value={description}
                              onChange={(event) => setDescription(event.target.value)}
                              placeholder="What is this project for?"
                              className="min-h-[72px] resize-none"
                              disabled={creatingProject}
                            />
                          </div>
                        </div>
                      )}

                      {currentStep === 1 && (
                        <div className="space-y-5 animate-in fade-in slide-in-from-bottom-2">
                          <Card className="bg-muted/30">
                            <CardHeader className="p-4 pb-2">
                              <div className="flex items-center justify-between">
                                <CardTitle className="flex items-center gap-2 text-sm">
                                  <GitBranch className="size-4 text-primary" />
                                  Local Repository Status
                                </CardTitle>
                                {loadingGitStatus && <RefreshCcw className="size-3 animate-spin text-muted-foreground" />}
                              </div>
                            </CardHeader>
                            <CardContent className="p-4 pt-0">
                              {gitStatus ? (
                                <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
                                  <div className="space-y-0.5">
                                    <p className="text-[10px] uppercase tracking-wider text-muted-foreground font-semibold">Branch</p>
                                    <p className="text-sm font-medium">{gitStatus.branch}</p>
                                  </div>
                                  <div className="space-y-0.5">
                                    <p className="text-[10px] uppercase tracking-wider text-muted-foreground font-semibold">Changes</p>
                                    <p className="text-sm font-medium">{gitStatus.touched_files} files</p>
                                  </div>
                                  <div className="space-y-0.5">
                                    <p className="text-[10px] uppercase tracking-wider text-muted-foreground font-semibold">Delta</p>
                                    <p className="text-sm font-medium text-green-600">+{gitStatus.insertions} <span className="text-red-600">-{gitStatus.deletions}</span></p>
                                  </div>
                                  <div className="space-y-0.5">
                                    <p className="text-[10px] uppercase tracking-wider text-muted-foreground font-semibold">Remote</p>
                                    <p className="text-sm font-medium truncate">{gitStatus.upstream || 'None'}</p>
                                  </div>
                                </div>
                              ) : (
                                <div className="flex items-center gap-2 py-2 text-sm text-muted-foreground">
                                  {loadingGitStatus ? 'Analyzing repository status...' : 'No git repository detected in this directory.'}
                                </div>
                              )}
                            </CardContent>
                          </Card>

                          <div className="space-y-3">
                            <div>
                              <p className="text-sm font-medium">Track in Version Control</p>
                              <p className="text-muted-foreground text-xs">
                                These categories are git-ignored by default. Select which ones Ship should automatically stage and commit to your repository.
                              </p>
                            </div>
                            <div className="flex flex-wrap gap-2">
                              {[
                                { id: 'releases', label: 'Releases' },
                                { id: 'features', label: 'Features' },
                                { id: 'adrs', label: 'ADRs' },
                                { id: 'vision', label: 'Vision' },
                              ].map((cat) => {
                                const active = gitCommitCategories.includes(cat.id);
                                return (
                                  <button
                                    key={cat.id}
                                    type="button"
                                    onClick={() => toggleCommitCategory(cat.id)}
                                    className={[
                                      'inline-flex items-center gap-2 rounded-lg border px-3 py-1.5 text-xs font-medium transition-all',
                                      active
                                        ? 'border-primary bg-primary/5 text-primary shadow-sm ring-1 ring-primary/20'
                                        : 'border-muted bg-background text-muted-foreground hover:border-muted-foreground/30',
                                    ].join(' ')}
                                  >
                                    {active ? (
                                      <CheckCircle2 className="size-3.5" />
                                    ) : (
                                      <Circle className="size-3.5" />
                                    )}
                                    {cat.label}
                                  </button>
                                );
                              })}
                            </div>
                          </div>
                          
                          <div className="rounded-md bg-primary/5 border border-primary/20 p-3 flex items-center gap-3">
                            <CheckCircle2 className="size-4 text-primary shrink-0" />
                            <div className="text-xs">
                              <p className="font-semibold text-primary">ship.toml is always tracked</p>
                              <p className="text-muted-foreground">The core project configuration is automatically versioned.</p>
                            </div>
                          </div>
                        </div>
                      )}

                      {currentStep === 2 && (
                        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-2">
                          <div className="space-y-3">
                            <div>
                              <p className="text-sm font-medium">Enabled Agents</p>
                              <p className="text-muted-foreground text-xs">
                                Choose the AI models you want to use for this project.
                              </p>
                            </div>
                            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                              {availableProviders.map((provider) => {
                                const active = enabledAgents.includes(provider.id);
                                return (
                                  <button
                                    key={provider.id}
                                    type="button"
                                    onClick={() => toggleAgent(provider.id)}
                                    className={[
                                      'flex flex-col items-start gap-1 rounded-xl border p-3 text-left transition-all',
                                      active
                                        ? 'border-primary bg-primary/5 shadow-sm ring-1 ring-primary/20'
                                        : 'border-muted bg-background hover:bg-muted/20',
                                    ].join(' ')}
                                  >
                                    <div className="flex w-full items-center justify-between">
                                      <span className={['text-xs font-bold uppercase tracking-wider', active ? 'text-primary' : 'text-muted-foreground'].join(' ')}>{provider.id}</span>
                                      {active && <CheckCircle2 className="size-3.5 text-primary" />}
                                    </div>
                                    <span className="text-sm font-medium">{provider.name}</span>
                                    <div className="flex flex-wrap gap-1 mt-1">
                                      {provider.models.slice(0, 2).map(m => (
                                        <Badge key={m.id} variant="secondary" className="text-[9px] px-1 py-0 h-auto font-normal">
                                          {m.name}
                                        </Badge>
                                      ))}
                                      {provider.models.length > 2 && (
                                        <span className="text-[9px] text-muted-foreground">+{provider.models.length - 2} more</span>
                                      )}
                                    </div>
                                  </button>
                                );
                              })}
                            </div>
                          </div>

                          <div className="space-y-3">
                            <div>
                              <p className="text-sm font-medium">Project Preset</p>
                              <p className="text-muted-foreground text-xs">
                                Pre-configured settings optimized for your stack.
                              </p>
                            </div>
                            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                              {availableModes.map((mode) => {
                                const active = selectedModes.includes(mode.id);
                                return (
                                  <button
                                    key={mode.id}
                                    type="button"
                                    onClick={() => toggleMode(mode.id)}
                                    className={[
                                      'flex items-center gap-3 rounded-xl border p-3 text-left transition-all',
                                      active
                                        ? 'border-primary bg-primary/5 shadow-sm ring-1 ring-primary/20'
                                        : 'border-muted bg-background hover:bg-muted/20',
                                    ].join(' ')}
                                  >
                                    <div className={['flex size-10 shrink-0 items-center justify-center rounded-lg border', active ? 'bg-primary/20 border-primary/40' : 'bg-muted/30'].join(' ')}>
                                      <Layout className={['size-5', active ? 'text-primary' : 'text-muted-foreground'].join(' ')} />
                                    </div>
                                    <div className="min-w-0 flex-1">
                                      <p className="text-sm font-medium truncate">{mode.name}</p>
                                      <p className="text-[10px] text-muted-foreground line-clamp-2">{mode.description}</p>
                                    </div>
                                    {active && <CheckCircle2 className="ml-auto size-4 text-primary" />}
                                  </button>
                                );
                              })}
                            </div>
                          </div>
                        </div>
                      )}
                    </>
                  )}

                  {formError && (
                    <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                      {formError}
                    </div>
                  )}
                </div>

                <AlertDialogFooter className="border-t pt-4">
                  <div className="flex w-full items-center justify-between">
                    <Button
                      type="button"
                      variant="ghost"
                      onClick={handleBack}
                      disabled={currentStep === 0 || creatingProject}
                    >
                      <ChevronLeft className="size-4 mr-1" />
                      Back
                    </Button>

                    <div className="flex items-center gap-2">
                      <AlertDialogCancel disabled={creatingProject} className="mr-2">Cancel</AlertDialogCancel>
                      {currentStep < 2 ? (
                        <Button type="button" onClick={handleNext} disabled={creatingProject}>
                          Next
                          <ChevronRight className="size-4 ml-1" />
                        </Button>
                      ) : (
                        <Button type="button" onClick={handleCreate} disabled={creatingProject} className="bg-primary text-primary-foreground">
                          {creatingProject ? 'Creating…' : 'Create Project'}
                          <CheckCircle2 className="size-4 ml-2" />
                        </Button>
                      )}
                    </div>
                  </div>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>

            {!detectedProject ? (
              <Button variant="ghost" size="sm" onClick={onRefreshDetection} disabled={detectingProject} className="text-xs text-muted-foreground">
                {detectingProject ? (
                  <>
                    <RefreshCcw className="mr-2 size-3 animate-spin" />
                    Detecting...
                  </>
                ) : (
                  <>
                    <Sparkles className="mr-2 size-3" />
                    Detect Nearby Project
                  </>
                )}
              </Button>
            ) : (
              <div className="text-muted-foreground inline-flex items-center gap-2 rounded-md border bg-muted/30 px-2 py-1 text-[10px]">
                <Sparkles className="size-3 text-primary" />
                Nearby project detected
                <Button
                  variant="ghost"
                  size="xs"
                  onClick={onRefreshDetection}
                  disabled={detectingProject}
                  className="h-auto px-1 py-0"
                >
                  Refresh
                </Button>
              </div>
            )}
          </div>

          {projectOptions.length === 0 ? (
            <div className="text-muted-foreground rounded-lg border border-dashed p-4 text-sm">
              No known projects yet. Open a project folder or create a new project.
            </div>
          ) : (
            <div className="grid gap-2 md:grid-cols-2">
              {projectOptions.map((project) => {
                const isDetected = detectedProject?.path === project.path;
                return (
                  <button
                    key={project.path}
                    type="button"
                    onClick={() => onSelectProject(project)}
                    className="hover:bg-muted/60 focus-visible:ring-ring/50 flex flex-col items-start gap-1 rounded-lg border p-3 text-left outline-none transition focus-visible:ring-2"
                    title={project.path}
                  >
                    <div className="flex w-full items-center justify-between gap-2">
                      <span className="text-sm font-medium">{project.name}</span>
                      {isDetected && <Badge variant="secondary">Nearby</Badge>}
                    </div>
                    <span className="text-muted-foreground w-full truncate text-xs">{project.path}</span>
                  </button>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>
    </PageFrame>
  );
}
