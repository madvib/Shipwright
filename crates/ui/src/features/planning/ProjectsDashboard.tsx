import { FolderOpen, FolderPlus, ShipWheel } from 'lucide-react';
import { ProjectDiscovery as Project } from '@/bindings';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

interface ProjectsDashboardProps {
  projects: Project[];
  activeProject: Project | null;
  onSelectProject: (project: Project) => void;
  onOpenProject: () => void;
  onNewProject: () => void;
}

export default function ProjectsDashboard({
  projects,
  activeProject,
  onSelectProject,
  onOpenProject,
  onNewProject,
}: ProjectsDashboardProps) {
  return (
    <div className="mx-auto flex w-full max-w-6xl flex-col gap-4 p-5 md:p-6">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Projects</h1>
          <p className="text-muted-foreground text-sm">Switch context across tracked workspaces.</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={onOpenProject}>
            <FolderOpen className="size-4" />
            Open Existing
          </Button>
          <Button onClick={onNewProject}>
            <FolderPlus className="size-4" />
            New Project
          </Button>
        </div>
      </header>

      {projects.length === 0 ? (
        <Card size="sm">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <ShipWheel className="size-4" />
              No projects tracked yet
            </CardTitle>
            <CardDescription>Open a repository to add it to your Ship workspace.</CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={onOpenProject}>
              <FolderOpen className="size-4" />
              Open Project
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {projects.map((project) => {
            const isActive = activeProject?.path === project.path;
            return (
              <button
                key={project.path}
                className="text-left"
                onClick={() => onSelectProject(project)}
                title={project.path}
              >
                <Card
                  size="sm"
                  className={`h-full transition-colors ${isActive ? 'border-primary/40 bg-primary/5' : 'hover:bg-muted/60'}`}
                >
                  <CardHeader className="pb-2">
                    <div className="flex items-start justify-between gap-2">
                      <CardTitle className="truncate text-sm">{project.name}</CardTitle>
                      {isActive && <Badge className="text-[10px]">Active</Badge>}
                    </div>
                    <CardDescription className="line-clamp-2 break-all text-xs">{project.path}</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <p className="text-muted-foreground text-xs">
                      {typeof project.issue_count === 'number'
                        ? `${project.issue_count} issues`
                        : 'Issue count unavailable'}
                    </p>
                  </CardContent>
                </Card>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
