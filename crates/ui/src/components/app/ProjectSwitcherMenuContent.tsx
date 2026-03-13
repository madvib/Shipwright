import {
  FolderOpen,
  FolderPlus,
  Globe2,
  History,
  Target,
} from 'lucide-react';
import {
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
} from '@ship/ui';

export interface ProjectSwitcherItem {
  name: string;
  path: string;
}

interface ProjectSwitcherMenuContentProps {
  activeProject?: ProjectSwitcherItem | null;
  projects?: ProjectSwitcherItem[];
  onSelectProject: (project: ProjectSwitcherItem) => void;
  onOpenProject?: () => void;
  onNewProject?: () => void;
  onOpenGlobalNotes?: () => void;
  maxProjects?: number;
  showActiveSection?: boolean;
  showActions?: boolean;
}

function dedupeByPath(projects: ProjectSwitcherItem[]): ProjectSwitcherItem[] {
  const seen = new Set<string>();
  const output: ProjectSwitcherItem[] = [];
  for (const project of projects) {
    if (!project.path || seen.has(project.path)) continue;
    seen.add(project.path);
    output.push(project);
  }
  return output;
}

export default function ProjectSwitcherMenuContent({
  activeProject = null,
  projects = [],
  onSelectProject,
  onOpenProject,
  onNewProject,
  onOpenGlobalNotes,
  maxProjects = 3,
  showActiveSection = true,
  showActions = true,
}: ProjectSwitcherMenuContentProps) {
  const uniqueProjects = dedupeByPath(projects);
  const otherProjects = uniqueProjects
    .filter((project) => project.path !== activeProject?.path)
    .slice(0, maxProjects);

  return (
    <>
      {showActiveSection ? (
        <DropdownMenuGroup className="p-1">
          <DropdownMenuLabel className="flex items-center gap-2 px-2 pb-2 opacity-50 uppercase text-[9px] tracking-[0.2em] font-black">
            <Target className="size-3" />
            Current Project
          </DropdownMenuLabel>
          {activeProject ? (
            <div className="bg-gradient-to-br from-amber-500/15 to-amber-600/5 mb-1.5 rounded-lg border border-amber-500/30 px-3.5 py-3 shadow-inner">
              <p className="truncate text-sm font-bold text-foreground leading-tight">{activeProject.name}</p>
              <p className="text-muted-foreground truncate text-[10px] opacity-60 font-mono mt-1 flex items-center gap-1">
                <span className="opacity-40">path:</span> {activeProject.path}
              </p>
            </div>
          ) : (
            <div className="text-muted-foreground mb-1.5 rounded-lg border border-dashed border-sidebar-border px-3.5 py-3 text-xs italic">
              No active project selected.
            </div>
          )}
        </DropdownMenuGroup>
      ) : null}

      {(showActiveSection || showActions) ? (
        <DropdownMenuSeparator className="mx-1 my-1 opacity-50" />
      ) : null}

      <DropdownMenuGroup className="p-1">
        <DropdownMenuLabel className="flex items-center gap-2 px-2 pb-2 opacity-50 uppercase text-[9px] tracking-[0.2em] font-black">
          <History className="size-3" />
          Recent Projects
        </DropdownMenuLabel>
        {otherProjects.length === 0 ? (
          <div className="text-muted-foreground rounded-lg px-2.5 py-3 text-xs italic opacity-60">
            No recent projects.
          </div>
        ) : (
          <div className="space-y-1">
            {otherProjects.map((project) => (
              <DropdownMenuItem
                key={project.path}
                className="cursor-pointer rounded-md px-3 py-2.5 transition-all active:scale-[0.98] hover:bg-sidebar-accent"
                onClick={() => onSelectProject(project)}
              >
                <div className="min-w-0">
                  <p className="truncate text-sm font-semibold leading-tight">{project.name}</p>
                  <p className="text-muted-foreground truncate text-[9px] opacity-50 font-mono mt-0.5">
                    {project.path}
                  </p>
                </div>
              </DropdownMenuItem>
            ))}
          </div>
        )}
      </DropdownMenuGroup>

      {showActions ? (
        <>
          <DropdownMenuSeparator className="mx-1 my-1 opacity-50" />
          <DropdownMenuGroup className="p-1 space-y-0.5">
            {onOpenGlobalNotes ? (
              <DropdownMenuItem
                onClick={onOpenGlobalNotes}
                className="cursor-pointer gap-2 py-2 rounded-md hover:bg-sidebar-accent"
              >
                <Globe2 className="size-4 opacity-60" />
                <span className="text-sm font-medium">Global Notes</span>
              </DropdownMenuItem>
            ) : null}
            {(onOpenProject || onNewProject) ? <DropdownMenuSeparator className="my-1" /> : null}
            {onOpenProject ? (
              <DropdownMenuItem
                onClick={onOpenProject}
                className="cursor-pointer gap-2 py-2 rounded-md hover:bg-sidebar-accent"
              >
                <FolderOpen className="size-4 opacity-60" />
                <span className="text-sm font-medium">Open Folder...</span>
              </DropdownMenuItem>
            ) : null}
            {onNewProject ? (
              <DropdownMenuItem
                onClick={onNewProject}
                className="cursor-pointer gap-2 py-2 rounded-md hover:bg-sidebar-accent"
              >
                <FolderPlus className="size-4 opacity-60" />
                <span className="text-sm font-medium">New Project...</span>
              </DropdownMenuItem>
            ) : null}
          </DropdownMenuGroup>
        </>
      ) : null}
    </>
  );
}
