import { Bot, Server, BookOpen, ScrollText, Shield } from 'lucide-react'
import { ProvidersForm } from '#/features/compiler/sections/ProvidersForm'
import { McpServersForm } from '#/features/compiler/sections/McpServersForm'
import { SkillsForm } from '#/features/compiler/sections/SkillsForm'
import { RulesForm } from '#/features/compiler/sections/RulesForm'
import { PermissionsForm } from '#/features/compiler/sections/PermissionsForm'
import { DEFAULT_PERMISSIONS } from '#/features/compiler/types'
import type { ProjectLibrary } from '#/features/compiler/types'

export type ComposerSection = 'providers' | 'mcp' | 'skills' | 'rules' | 'permissions'

const COMPOSER_TABS: Array<{ id: ComposerSection; label: string; icon: React.ElementType }> = [
  { id: 'providers', label: 'Providers', icon: Bot },
  { id: 'mcp', label: 'MCP', icon: Server },
  { id: 'skills', label: 'Skills', icon: BookOpen },
  { id: 'rules', label: 'Rules', icon: ScrollText },
  { id: 'permissions', label: 'Permissions', icon: Shield },
]

const SECTION_HELP: Record<ComposerSection, { title: string; description: string }> = {
  providers: { title: 'Target providers', description: 'Choose which AI coding assistants to build for.' },
  mcp: { title: 'MCP servers', description: 'Tools, APIs, and services your agents can call during a session.' },
  skills: { title: 'Skills', description: 'Instruction files injected into agent context — workflows, domain knowledge, repeated tasks.' },
  rules: { title: 'Rules', description: 'Always-active instructions included in every session.' },
  permissions: { title: 'Permissions', description: 'Control what tools, paths, and commands your agents can access.' },
}

function SectionHeader({ section }: { section: ComposerSection }) {
  const { title, description } = SECTION_HELP[section]
  return (
    <div className="mb-5">
      <h2 className="font-display text-sm font-semibold text-foreground">{title}</h2>
      <p className="mt-0.5 text-[11px] text-muted-foreground">{description}</p>
    </div>
  )
}

export interface ComposerPanelProps {
  library: ProjectLibrary
  activeSection: ComposerSection
  selectedProviders: string[]
  onSectionChange: (section: ComposerSection) => void
  onLibraryChange: (patch: Partial<ProjectLibrary>) => void
  onToggleProvider: (id: string) => void
}

export function ComposerPanel({
  library,
  activeSection,
  selectedProviders,
  onSectionChange,
  onLibraryChange,
  onToggleProvider,
}: ComposerPanelProps) {
  const mcpCount = library.mcp_servers.length
  const skillCount = library.skills.length
  const ruleCount = library.rules.length

  return (
    <div className="flex flex-1 min-w-0 flex-col border-r border-border/60">
      <div className="flex items-center gap-0.5 border-b border-border/60 bg-muted/20 px-2 py-1.5 shrink-0 overflow-x-auto [scrollbar-width:none]">
        {COMPOSER_TABS.map(({ id, label, icon: Icon }) => {
          const count = id === 'mcp' ? mcpCount : id === 'skills' ? skillCount : id === 'rules' ? ruleCount : 0
          return (
            <button
              key={id}
              onClick={() => onSectionChange(id)}
              className={`flex shrink-0 items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium transition ${
                activeSection === id
                  ? 'bg-card text-foreground shadow-sm'
                  : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'
              }`}
            >
              <Icon className="size-3.5" />
              {label}
              {count > 0 && (
                <span className="rounded-full bg-primary/15 px-1.5 py-0.5 text-[9px] font-bold text-primary">
                  {count}
                </span>
              )}
            </button>
          )
        })}
      </div>

      {activeSection === 'skills' ? (
        <div className="flex flex-1 min-h-0 flex-col p-4 lg:p-5">
          <SectionHeader section={activeSection} />
          <div className="flex-1 min-h-0">
            <SkillsForm
              skills={library.skills}
              onChange={(skills) => onLibraryChange({ skills })}
            />
          </div>
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto">
          <div className="mx-auto max-w-3xl p-4 lg:p-6">
            <SectionHeader section={activeSection} />
            {activeSection === 'providers' && (
              <ProvidersForm selected={selectedProviders} onToggle={onToggleProvider} />
            )}
            {activeSection === 'mcp' && (
              <McpServersForm
                servers={library.mcp_servers}
                onChange={(mcp_servers) => onLibraryChange({ mcp_servers })}
              />
            )}
            {activeSection === 'rules' && (
              <RulesForm
                rules={library.rules}
                onChange={(rules) => onLibraryChange({ rules })}
              />
            )}
            {activeSection === 'permissions' && (
              <PermissionsForm
                permissions={library.permissions ?? DEFAULT_PERMISSIONS}
                onChange={(permissions) => onLibraryChange({ permissions })}
              />
            )}
          </div>
        </div>
      )}
    </div>
  )
}
