import { useEffect, useState } from "react";
import {
  ArrowLeft,
  Bot,
  FileCode2,
  FileCog,
  FileStack,
  GitBranch,
  Globe2,
  Package,
  Settings,
  RefreshCw,
} from "lucide-react";
import { cn } from '@/lib/utils';
import { Button } from '@ship/primitives';
import { Tooltip, TooltipContent, TooltipTrigger } from '@ship/primitives';
import { getVersion } from '@tauri-apps/api/app';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from '@ship/primitives';
import SettingsPanel from '@/features/agents/SettingsPanel';
import AgentsPanel, { type AgentSection } from '@/features/agents/AgentsPanel';
import { ProjectConfig, ProjectDiscovery } from '@/bindings';
import { Config } from '@/lib/workspace-ui';
import ProjectSwitcherMenuContent from '@/components/app/ProjectSwitcherMenuContent';

export type SettingsSection =
  | 'global'
  | 'project'
  | 'appearance'
  | 'providers'
  | 'mcp'
  | 'skills'
  | 'rules'
  | 'permissions';

interface SettingsSidebarItem {
  id: SettingsSection;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  group: "general" | "agents";
}

const SETTINGS_ITEMS: SettingsSidebarItem[] = [
  { id: 'global', label: 'General', icon: Globe2, group: 'general' },
  { id: 'project', label: 'Project', icon: GitBranch, group: 'general' },
  { id: 'providers', label: 'Providers', icon: Bot, group: 'agents' },
  { id: 'mcp', label: 'MCP Servers', icon: Package, group: 'agents' },
  { id: 'skills', label: 'Skills', icon: FileStack, group: 'agents' },
  { id: 'rules', label: 'Rules', icon: FileCode2, group: 'agents' },
  { id: 'permissions', label: 'Permissions', icon: FileCog, group: 'agents' },
];

const AGENT_SECTIONS: SettingsSection[] = ['providers', 'mcp', 'skills', 'rules', 'permissions'];

interface SettingsLayoutProps {
  config: Config;
  activeProject: ProjectDiscovery | null;
  projectConfig: ProjectConfig | null;
  globalAgentConfig: ProjectConfig | null;
  recentProjects?: ProjectDiscovery[];
  onOpenProject?: () => void;
  onSelectProject?: (project: ProjectDiscovery) => void;
  activeSection: SettingsSection;
  onSectionChange: (section: SettingsSection) => void;
  onThemePreview: (theme: "light" | "dark" | undefined) => void;
  onSave: (config: Config) => Promise<void>;
  onSaveProject: (config: ProjectConfig) => Promise<void>;
  onSaveGlobalAgentConfig: (config: ProjectConfig) => Promise<void>;
  onDone: () => void;
}

export default function SettingsLayout({
  config,
  activeProject,
  projectConfig,
  globalAgentConfig,
  recentProjects = [],
  onOpenProject,
  onSelectProject,
  activeSection,
  onSectionChange,
  onThemePreview,
  onSave,
  onSaveProject,
  onSaveGlobalAgentConfig,
  onDone,
}: SettingsLayoutProps) {
  const [saving, setSaving] = useState(false);
  const [appVersion, setAppVersion] = useState<string | null>(null);
  const isAgentSection = AGENT_SECTIONS.includes(activeSection);
  const skillsStudioFullPage = activeSection === 'skills';
  const generalItems = SETTINGS_ITEMS.filter((i) => i.group === "general");
  const agentItems = SETTINGS_ITEMS.filter((i) => i.group === "agents");

  const handleSave = async (c: Config) => {
    setSaving(true);
    try {
      await onSave(c);
      onDone();
    } finally {
      setSaving(false);
    }
  };

  const handleSaveProject = async (c: ProjectConfig) => {
    setSaving(true);
    try {
      await onSaveProject(c);
      if (!isAgentSection) onDone();
    } finally {
      setSaving(false);
    }
  };

  const handleSaveGlobalAgentConfig = async (c: ProjectConfig) => {
    setSaving(true);
    try {
      await onSaveGlobalAgentConfig(c);
      if (!isAgentSection) onDone();
    } finally {
      setSaving(false);
    }
  };

  useEffect(() => {
    let canceled = false;
    void getVersion()
      .then((version) => {
        if (!canceled) setAppVersion(version);
      })
      .catch(() => {
        if (!canceled) setAppVersion(null);
      });
    return () => {
      canceled = true;
    };
  }, []);

  return (
    <div className="flex h-full">
      {!skillsStudioFullPage && (
        <aside className="flex w-52 shrink-0 flex-col border-r border-border/50 bg-sidebar">
          <div className="flex h-10 items-center gap-2 border-b border-border/50 px-3">
            <Button
              variant="ghost"
              size="icon-xs"
              className="size-6"
              onClick={onDone}
            >
              <ArrowLeft className="size-3.5" />
            </Button>
            <Settings className="size-3.5 text-muted-foreground" />
            <span className="text-xs font-semibold">Settings</span>
          </div>

          <nav className="flex-1 overflow-y-auto p-2 space-y-4">
            {!activeProject && (onOpenProject || recentProjects.length > 0) ? (
              <div className="space-y-2 rounded-md border border-border/60 bg-card/60 p-2">
                <p className="px-1 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/70">
                  Project
                </p>
                {onOpenProject ? (
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    className="h-7 w-full justify-start text-xs"
                    onClick={onOpenProject}
                  >
                    Open Project…
                  </Button>
                ) : null}
                {recentProjects.length > 0 && onSelectProject ? (
                  <DropdownMenu>
                    <DropdownMenuTrigger
                      className="focus-visible:ring-ring/50 flex h-7 w-full items-center justify-between gap-2 rounded-md border border-input bg-background px-2 text-xs outline-none transition-colors hover:bg-accent/40 focus-visible:ring-[3px]"
                    >
                      <span className="truncate">Recent projects</span>
                      <GitBranch className="size-3.5 shrink-0 opacity-70" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="w-[22rem]">
                      <ProjectSwitcherMenuContent
                        activeProject={null}
                        projects={recentProjects}
                        onSelectProject={onSelectProject}
                        showActiveSection={false}
                        showActions={false}
                        maxProjects={12}
                      />
                    </DropdownMenuContent>
                  </DropdownMenu>
                ) : null}
              </div>
            ) : null}

            {activeProject ? (
              <div className="space-y-1 rounded-md border border-border/60 bg-card/60 p-2">
                <p className="px-1 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/70">
                  Active Project
                </p>
                <div className="rounded border bg-background/70 px-2 py-1.5">
                  <p className="truncate text-xs font-semibold">{activeProject.name}</p>
                  <p className="truncate text-[10px] text-muted-foreground">{activeProject.path}</p>
                </div>
              </div>
            ) : null}

            <div>
              <p className="mb-1 px-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">
                General
              </p>
              <div className="space-y-0.5">
                {generalItems.map((item) => {
                  const Icon = item.icon;
                  const isActive = activeSection === item.id;
                  const isDisabled = item.id === 'project' && !activeProject;
                  const button = (
                    <button
                      key={item.id}
                      disabled={isDisabled}
                      onClick={() => onSectionChange(item.id)}
                      className={cn(
                        "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs transition-colors",
                        isActive
                          ? "bg-accent text-accent-foreground font-medium"
                          : "text-muted-foreground hover:bg-accent/50 hover:text-foreground",
                        isDisabled && "opacity-40 cursor-not-allowed",
                      )}
                    >
                      <Icon className="size-3.5" />
                      {item.label}
                    </button>
                  );
                  if (!isDisabled) return button;
                  return (
                    <Tooltip key={item.id}>
                      <TooltipTrigger asChild>
                        <span>{button}</span>
                      </TooltipTrigger>
                      <TooltipContent>
                        Open or select a project to edit project settings.
                      </TooltipContent>
                    </Tooltip>
                  );
                })}
              </div>
            </div>

            <div>
              <p className="mb-1 px-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">
                Agents
              </p>
              <div className="space-y-0.5">
                {agentItems.map((item) => {
                  const Icon = item.icon;
                  const isActive = activeSection === item.id;
                  return (
                    <button
                      key={item.id}
                      onClick={() => onSectionChange(item.id)}
                      className={cn(
                        "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs transition-colors",
                        isActive
                          ? "bg-accent text-accent-foreground font-medium"
                          : "text-muted-foreground hover:bg-accent/50 hover:text-foreground",
                      )}
                    >
                      <Icon className="size-3.5" />
                      {item.label}
                    </button>
                  );
                })}
              </div>
            </div>
          </nav>
          <div className="border-t border-border/50 px-3 py-2 text-[10px] text-muted-foreground">
            Ship {appVersion ? `v${appVersion}` : 'version unavailable'}
          </div>
        </aside>
      )}

      {/* Content Area */}
      <div
        key={activeSection}
        className="relative flex-1 min-w-0 overflow-y-auto route-enter"
      >
        {saving && (
          <div className="absolute inset-0 z-50 flex items-center justify-center bg-background/20 backdrop-blur-[1px]">
            <div className="flex items-center gap-2 rounded-full border bg-background px-4 py-2 shadow-lg">
              <RefreshCw className="size-4 animate-spin text-primary" />
              <span className="text-sm font-medium">Saving changes...</span>
            </div>
          </div>
        )}
        {isAgentSection ? (
          <AgentsPanel
            activeProject={activeProject}
            projectConfig={projectConfig}
            globalAgentConfig={globalAgentConfig}
            onSaveProject={handleSaveProject}
            onSaveGlobalAgentConfig={handleSaveGlobalAgentConfig}
            initialSection={activeSection as AgentSection}
            onBackToSettings={skillsStudioFullPage ? () => onSectionChange('global') : undefined}
          />
        ) : (
          <SettingsPanel
            config={config}
            projectConfig={projectConfig}
            globalAgentConfig={globalAgentConfig}
            panelMode="settings-only"
            initialTab={activeSection === "project" ? "project" : "global"}
            onThemePreview={onThemePreview}
            onSave={handleSave}
            onSaveProject={handleSaveProject}
            onSaveGlobalAgentConfig={handleSaveGlobalAgentConfig}
            onOpenAgentsModule={() => onSectionChange("providers")}
          />
        )}
      </div>
    </div>
  );
}
