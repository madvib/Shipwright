import { FormEvent, useMemo, useState } from 'react';
import {
  FolderOpen,
  RefreshCcw,
  Search,
  Settings2,
  Sparkles,
  Plus,
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
  const [useDefaults, setUseDefaults] = useState(true);
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
        return prev.filter((id) => id !== statusId);
      }
      return [...prev, statusId];
    });
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

    if (!useDefaults && selectedStatuses.length === 0) {
      setFormError('Select at least one status or use defaults.');
      return;
    }

    try {
      await onCreateProject({
        name: cleanName,
        description: description.trim() || undefined,
        directory,
        useDefaults,
        selectedStatuses,
      });
      setName('');
      setDescription('');
      setDirectory('');
      setUseDefaults(true);
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
                    Choose a directory, then initialize `.ship` with defaults or custom statuses.
                  </AlertDialogDescription>
                </AlertDialogHeader>

                <form onSubmit={submitCreateProject} className="space-y-4">
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <label className="text-sm font-medium">Project Name</label>
                      <Input
                        value={name}
                        onChange={(event) => setName(event.target.value)}
                        placeholder="Acme Web App"
                        disabled={creatingProject}
                      />
                    </div>

                    <div className="space-y-2">
                      <label className="text-sm font-medium">Directory</label>
                      <div className="flex gap-2">
                        <Input value={directory} readOnly placeholder="Choose a folder" />
                        <Button type="button" variant="outline" onClick={pickDirectory} disabled={creatingProject}>
                          Browse
                        </Button>
                      </div>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium">Description (optional)</label>
                    <Textarea
                      value={description}
                      onChange={(event) => setDescription(event.target.value)}
                      placeholder="What is this project for?"
                      className="min-h-20"
                      disabled={creatingProject}
                    />
                  </div>

                  <div className="space-y-2 rounded-md border p-3">
                    <div className="flex items-center justify-between">
                      <label className="text-sm font-medium">Project Settings</label>
                      <label className="text-muted-foreground inline-flex items-center gap-2 text-xs">
                        <input
                          type="checkbox"
                          checked={useDefaults}
                          onChange={(event) => setUseDefaults(event.target.checked)}
                          className="accent-primary"
                          disabled={creatingProject}
                        />
                        Use defaults
                      </label>
                    </div>

                    {!useDefaults && (
                      <div className="grid grid-cols-2 gap-2 md:grid-cols-3">
                        {DEFAULT_STATUSES.map((status) => (
                          <label
                            key={status.id}
                            className="text-muted-foreground inline-flex items-center gap-2 text-xs"
                          >
                            <input
                              type="checkbox"
                              checked={selectedStatuses.includes(status.id)}
                              onChange={() => toggleStatus(status.id)}
                              className="accent-primary"
                              disabled={creatingProject}
                            />
                            {status.name}
                          </label>
                        ))}
                      </div>
                    )}
                  </div>

                  {formError && (
                    <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                      {formError}
                    </div>
                  )}

                  <AlertDialogFooter className="pt-1">
                    <AlertDialogCancel disabled={creatingProject}>Cancel</AlertDialogCancel>
                    <Button type="submit" disabled={creatingProject}>
                      {creatingProject ? 'Creating Project...' : 'Create Project'}
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
