import { FormEvent, useMemo, useState } from 'react';
import {
  FolderOpen,
  RefreshCcw,
  Search,
  Settings2,
  Sparkles,
  Plus,
  RotateCcw,
} from 'lucide-react';
import { ProjectDiscovery as Project, StatusConfig } from '@/bindings';
import { DEFAULT_STATUSES } from '@/lib/workspace-ui';
import { AlertDialog, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from '@ship/ui';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@ship/ui';
import { Input } from '@ship/ui';
import { Textarea } from '@ship/ui';
import { PageFrame, PageHeader } from '@ship/ui';

const STATUS_COLORS: Record<string, { dot: string; chip: string; active: string }> = {
  gray:   { dot: 'bg-gray-400',   chip: 'border-gray-200 bg-gray-50 text-gray-600',         active: 'border-gray-400 bg-gray-100 text-gray-800' },
  blue:   { dot: 'bg-blue-500',   chip: 'border-blue-200 bg-blue-50 text-blue-700',          active: 'border-blue-400 bg-blue-100 text-blue-900' },
  yellow: { dot: 'bg-yellow-400', chip: 'border-yellow-200 bg-yellow-50 text-yellow-700',    active: 'border-yellow-400 bg-yellow-100 text-yellow-900' },
  red:    { dot: 'bg-red-500',    chip: 'border-red-200 bg-red-50 text-red-700',             active: 'border-red-400 bg-red-100 text-red-900' },
  green:  { dot: 'bg-green-500',  chip: 'border-green-200 bg-green-50 text-green-700',       active: 'border-green-400 bg-green-100 text-green-900' },
  orange: { dot: 'bg-orange-400', chip: 'border-orange-200 bg-orange-50 text-orange-700',    active: 'border-orange-400 bg-orange-100 text-orange-900' },
  purple: { dot: 'bg-purple-500', chip: 'border-purple-200 bg-purple-50 text-purple-700',    active: 'border-purple-400 bg-purple-100 text-purple-900' },
};

export interface CreateProjectInput {
  name: string;
  description?: string;
  directory: string;
  useDefaults: boolean;
  selectedStatuses: string[];
}

interface ProjectOnboardingProps {
  detectedProject: Project | null;
  detectingProject: boolean;
  creatingProject: boolean;
  recentProjects: Project[];
  onRefreshDetection: () => void;
  onOpenProject: () => void;
  onCreateProject: (input: CreateProjectInput) => Promise<void>;
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
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [directory, setDirectory] = useState('');
  const [selectedStatuses, setSelectedStatuses] = useState<string[]>(
    DEFAULT_STATUSES.map((status: StatusConfig) => status.id)
  );
  const [formError, setFormError] = useState<string | null>(null);

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

  const toggleStatus = (statusId: string) => {
    setSelectedStatuses((prev) => {
      if (prev.includes(statusId)) {
        if (prev.length === 1) return prev; // keep at least one
        return prev.filter((id) => id !== statusId);
      }
      return [...prev, statusId];
    });
  };

  const resetStatuses = () => {
    setSelectedStatuses(DEFAULT_STATUSES.map((s: StatusConfig) => s.id));
  };

  const pickDirectory = async () => {
    const picked = await onPickDirectory();
    if (picked) {
      setDirectory(picked);
    }
  };

  const submitCreateProject = async (event: FormEvent) => {
    event.preventDefault();
    setFormError(null);

    const cleanName = name.trim();
    if (!cleanName) {
      setFormError('Project name is required.');
      return;
    }

    if (!directory.trim()) {
      setFormError('Choose a directory for this project.');
      return;
    }

    try {
      await onCreateProject({
        name: cleanName,
        description: description.trim() || undefined,
        directory,
        useDefaults: false,
        selectedStatuses,
      });
      setName('');
      setDescription('');
      setDirectory('');
      setSelectedStatuses(DEFAULT_STATUSES.map((status: StatusConfig) => status.id));
      setCreateDialogOpen(false);
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
              <AlertDialogTrigger render={<Button variant="outline" className="gap-2" />}>
                <Plus className="size-4" />
                Create New Project
              </AlertDialogTrigger>

              <AlertDialogContent size="default" className="w-[min(96vw,880px)] max-w-none">
                <AlertDialogHeader className="place-items-start text-left">
                  <AlertDialogTitle>Create New Project</AlertDialogTitle>
                  <AlertDialogDescription>
                    Initialize a <code>.ship/</code> workspace in your project directory.
                  </AlertDialogDescription>
                </AlertDialogHeader>

                <form onSubmit={submitCreateProject} className="space-y-5">
                  {/* Name + Directory */}
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-1.5">
                      <label className="text-sm font-medium">Project Name</label>
                      <Input
                        value={name}
                        onChange={(event) => setName(event.target.value)}
                        placeholder="Acme Web App"
                        disabled={creatingProject}
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label className="text-sm font-medium">Directory</label>
                      <div className="flex gap-2">
                        <Input value={directory} readOnly placeholder="Choose a folder…" />
                        <Button type="button" variant="outline" onClick={pickDirectory} disabled={creatingProject}>
                          <FolderOpen className="size-4" />
                        </Button>
                      </div>
                    </div>
                  </div>

                  {/* Description */}
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

                  {/* Workflow Statuses */}
                  <div className="space-y-2">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm font-medium">Workflow Statuses</p>
                        <p className="text-muted-foreground text-xs">
                          Workflow statuses used to track work across this project.
                        </p>
                      </div>
                      <button
                        type="button"
                        onClick={resetStatuses}
                        disabled={creatingProject}
                        className="text-muted-foreground hover:text-foreground inline-flex items-center gap-1 text-xs transition-colors disabled:pointer-events-none disabled:opacity-50"
                      >
                        <RotateCcw className="size-3" />
                        Reset
                      </button>
                    </div>

                    <div className="flex flex-wrap gap-2">
                      {DEFAULT_STATUSES.map((status) => {
                        const active = selectedStatuses.includes(status.id);
                        const colors =
                          STATUS_COLORS[(status.color ?? 'gray') as keyof typeof STATUS_COLORS] ??
                          STATUS_COLORS.gray;
                        return (
                          <button
                            key={status.id}
                            type="button"
                            onClick={() => toggleStatus(status.id)}
                            disabled={creatingProject}
                            className={[
                              'inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-medium transition-all',
                              'disabled:pointer-events-none disabled:opacity-50',
                              active ? colors.active : `${colors.chip} opacity-50`,
                            ].join(' ')}
                          >
                            <span className={['size-2 rounded-full', colors.dot].join(' ')} />
                            {status.name}
                          </button>
                        );
                      })}
                    </div>
                  </div>

                  {formError && (
                    <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                      {formError}
                    </div>
                  )}

                  <AlertDialogFooter className="pt-1">
                    <AlertDialogCancel disabled={creatingProject}>Cancel</AlertDialogCancel>
                    <Button type="submit" disabled={creatingProject}>
                      {creatingProject ? 'Creating…' : 'Create Project'}
                    </Button>
                  </AlertDialogFooter>
                </form>
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
