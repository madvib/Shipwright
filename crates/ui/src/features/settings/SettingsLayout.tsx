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
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@ship/ui';
import { Tooltip, TooltipContent, TooltipTrigger } from '@ship/ui';
import SettingsPanel from '@/features/agents/SettingsPanel';
import AgentsPanel, { type AgentSection } from '@/features/agents/AgentsPanel';
import { ProjectConfig } from '@/bindings';
import { Config } from '@/lib/workspace-ui';

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
  group: 'general' | 'agents';
}

const SETTINGS_ITEMS: SettingsSidebarItem[] = [
  { id: 'global', label: 'General', icon: Globe2, group: 'general' },
  { id: 'project', label: 'Project', icon: GitBranch, group: 'general' },
  { id: 'providers', label: 'AI Providers', icon: Bot, group: 'agents' },
  { id: 'mcp', label: 'MCP Servers', icon: Package, group: 'agents' },
  { id: 'skills', label: 'Skills', icon: FileStack, group: 'agents' },
  { id: 'rules', label: 'Rules', icon: FileCode2, group: 'agents' },
  { id: 'permissions', label: 'Permissions', icon: FileCog, group: 'agents' },
];

const AGENT_SECTIONS: SettingsSection[] = ['providers', 'mcp', 'skills', 'rules', 'permissions'];

interface SettingsLayoutProps {
  config: Config;
  projectConfig: ProjectConfig | null;
  globalAgentConfig: ProjectConfig | null;
  activeSection: SettingsSection;
  onSectionChange: (section: SettingsSection) => void;
  onThemePreview: (theme: 'light' | 'dark' | undefined) => void;
  onSave: (config: Config) => Promise<void>;
  onSaveProject: (config: ProjectConfig) => Promise<void>;
  onSaveGlobalAgentConfig: (config: ProjectConfig) => Promise<void>;
  onDone: () => void;
}

export default function SettingsLayout({
  config,
  projectConfig,
  globalAgentConfig,
  activeSection,
  onSectionChange,
  onThemePreview,
  onSave,
  onSaveProject,
  onSaveGlobalAgentConfig,
  onDone,
}: SettingsLayoutProps) {
  const isAgentSection = AGENT_SECTIONS.includes(activeSection);
  const generalItems = SETTINGS_ITEMS.filter(i => i.group === 'general');
  const agentItems = SETTINGS_ITEMS.filter(i => i.group === 'agents');

  return (
    <div className="flex h-full">
      {/* Settings Sidebar */}
      <aside className="flex w-52 shrink-0 flex-col border-r border-border/50 bg-sidebar">
        <div className="flex h-10 items-center gap-2 border-b border-border/50 px-3">
          <Button variant="ghost" size="icon-xs" className="size-6" onClick={onDone}>
            <ArrowLeft className="size-3.5" />
          </Button>
          <Settings className="size-3.5 text-muted-foreground" />
          <span className="text-xs font-semibold">Settings</span>
        </div>

        <nav className="flex-1 overflow-y-auto p-2 space-y-4">
          <div>
            <p className="mb-1 px-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60">
              General
            </p>
            <div className="space-y-0.5">
              {generalItems.map(item => {
                const Icon = item.icon;
                const isActive = activeSection === item.id;
                const isDisabled = item.id === 'project' && !projectConfig;
                const button = (
                  <button
                    key={item.id}
                    disabled={isDisabled}
                    onClick={() => onSectionChange(item.id)}
                    className={cn(
                      'flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs transition-colors',
                      isActive
                        ? 'bg-accent text-accent-foreground font-medium'
                        : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground',
                      isDisabled && 'opacity-40 cursor-not-allowed'
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
                      Open or create a project to edit project settings.
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
              {agentItems.map(item => {
                const Icon = item.icon;
                const isActive = activeSection === item.id;
                return (
                  <button
                    key={item.id}
                    onClick={() => onSectionChange(item.id)}
                    className={cn(
                      'flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs transition-colors',
                      isActive
                        ? 'bg-accent text-accent-foreground font-medium'
                        : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'
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
      </aside>

      {/* Content Area */}
      <div className="flex-1 min-w-0 overflow-y-auto">
        {isAgentSection ? (
          <AgentsPanel
            projectConfig={projectConfig}
            globalAgentConfig={globalAgentConfig}
            onSaveProject={onSaveProject}
            onSaveGlobalAgentConfig={onSaveGlobalAgentConfig}
            initialSection={activeSection as AgentSection}
          />
        ) : (
          <SettingsPanel
            config={config}
            projectConfig={projectConfig}
            globalAgentConfig={globalAgentConfig}
            panelMode="settings-only"
            initialTab={activeSection === 'project' ? 'project' : 'global'}
            onThemePreview={onThemePreview}
            onSave={async (c) => {
              await onSave(c);
              onDone();
            }}
            onSaveProject={async (c) => {
              await onSaveProject(c);
              onDone();
            }}
            onSaveGlobalAgentConfig={async (c) => {
              await onSaveGlobalAgentConfig(c);
              onDone();
            }}
            onOpenAgentsModule={() => onSectionChange('providers')}
          />
        )}
      </div>
    </div>
  );
}
