import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { Search, Plus, Check, Server, BookOpen, X } from 'lucide-react'
import { toast } from 'sonner'
import { useLibrary } from '#/features/compiler/useLibrary'
import { CURATED_MCP, CURATED_SKILLS } from '#/features/compiler/components/LibraryPanel'

export const Route = createFileRoute('/studio/templates')({ component: RegistryPage })

function RegistryPage() {
  const { library, addMcpServer, addSkill } = useLibrary()
  const [query, setQuery] = useState('')

  const addedMcpIds = new Set(library.mcp_servers.map((s) => s.name))
  const addedSkillIds = new Set(library.skills.map((s) => s.id))

  const filteredMcp = CURATED_MCP.filter(
    (m) => !query || m.displayName.toLowerCase().includes(query.toLowerCase()) || m.description.toLowerCase().includes(query.toLowerCase()),
  )
  const filteredSkills = CURATED_SKILLS.filter(
    (s) => !query || s.displayName.toLowerCase().includes(query.toLowerCase()) || s.description.toLowerCase().includes(query.toLowerCase()),
  )

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-auto p-5">

        {/* Search */}
        <div className="mb-5 max-w-sm">
          <div className="flex items-center gap-2 rounded-lg border border-border/60 bg-card px-3 py-2">
            <Search className="size-3.5 text-muted-foreground shrink-0" />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search registry..."
              className="flex-1 bg-transparent text-sm text-foreground placeholder:text-muted-foreground focus:outline-none min-w-0"
            />
            {query && (
              <button onClick={() => setQuery('')} className="text-muted-foreground hover:text-foreground">
                <X className="size-3.5" />
              </button>
            )}
          </div>
        </div>

        {/* MCP Servers section */}
        <div className="mb-8">
          <div className="flex items-center gap-2 mb-3">
            <Server className="size-3.5 text-sky-400" />
            <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">MCP Servers</h2>
            <span className="text-[10px] text-muted-foreground/40">{filteredMcp.length}</span>
          </div>
          <div className="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
            {filteredMcp.map((item) => {
              const isAdded = addedMcpIds.has(item.id)
              return (
                <div
                  key={item.id}
                  className="rounded-xl border border-border/60 bg-card p-4 flex flex-col"
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <span className="text-base">{item.icon}</span>
                      <span className="text-sm font-semibold text-foreground">{item.displayName}</span>
                    </div>
                    <button
                      onClick={() => { addMcpServer(item.config); toast.success(`Added "${item.displayName}"`) }}
                      disabled={isAdded}
                      className={`shrink-0 flex items-center gap-1 rounded-md px-2 py-1 text-[10px] font-medium transition ${
                        isAdded
                          ? 'text-emerald-500 bg-emerald-500/10'
                          : 'text-muted-foreground hover:text-primary hover:bg-primary/10 border border-border/60'
                      }`}
                    >
                      {isAdded ? <><Check className="size-3" /> Added</> : <><Plus className="size-3" /> Add</>}
                    </button>
                  </div>
                  <p className="text-[11px] text-muted-foreground leading-relaxed mb-3">{item.description}</p>
                  <div className="mt-auto">
                    <code className="text-[9px] font-mono text-muted-foreground/50 bg-muted/30 rounded px-1.5 py-0.5">
                      {item.config.command} {item.config.args?.[1] ?? item.config.args?.[0] ?? ''}
                    </code>
                  </div>
                </div>
              )
            })}
          </div>
        </div>

        {/* Skills section */}
        <div>
          <div className="flex items-center gap-2 mb-3">
            <BookOpen className="size-3.5 text-emerald-400" />
            <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Skills</h2>
            <span className="text-[10px] text-muted-foreground/40">{filteredSkills.length}</span>
          </div>
          <div className="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
            {filteredSkills.map((item) => {
              const isAdded = addedSkillIds.has(item.id)
              return (
                <div
                  key={item.id}
                  className="rounded-xl border border-border/60 bg-card p-4 flex flex-col"
                >
                  <div className="flex items-start justify-between mb-2">
                    <span className="text-sm font-semibold text-foreground">{item.displayName}</span>
                    <button
                      onClick={() => { addSkill(item.skill); toast.success(`Added "${item.displayName}"`) }}
                      disabled={isAdded}
                      className={`shrink-0 flex items-center gap-1 rounded-md px-2 py-1 text-[10px] font-medium transition ${
                        isAdded
                          ? 'text-emerald-500 bg-emerald-500/10'
                          : 'text-muted-foreground hover:text-primary hover:bg-primary/10 border border-border/60'
                      }`}
                    >
                      {isAdded ? <><Check className="size-3" /> Added</> : <><Plus className="size-3" /> Add</>}
                    </button>
                  </div>
                  <p className="text-[11px] text-muted-foreground leading-relaxed">{item.description}</p>
                </div>
              )
            })}
          </div>
        </div>

      </div>
    </div>
  )
}
