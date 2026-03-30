// Events tab for the skills preview panel.
// Shows Ship built-in event refs and custom events with direction badges.

import { Badge } from '@ship/primitives'
import type { LibrarySkill } from './useSkillsLibrary'

interface EventsJson {
  ship?: string[]
  custom?: Array<{
    id: string
    direction?: 'in' | 'out' | 'both'
    label?: string
    description?: string
    schema?: {
      properties?: Record<string, { type?: string }>
      required?: string[]
      [key: string]: unknown
    }
  }>
}

export function SkillEventsTab({ skill }: { skill: LibrarySkill }) {
  const raw = skill.eventsSchema
  if (!raw) {
    return (
      <div className="py-8 text-center px-4">
        <p className="text-xs text-muted-foreground">No events.json found for this skill.</p>
        <p className="text-[10px] text-muted-foreground mt-1">Add assets/events.json to define events.</p>
      </div>
    )
  }

  const data = raw as EventsJson
  const shipRefs = data.ship ?? []
  const custom = data.custom ?? []

  return (
    <div className="space-y-4 px-4 py-3">
      {shipRefs.length > 0 && (
        <div>
          <h4 className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground mb-2">
            Ship Built-ins ({shipRefs.length})
          </h4>
          <div className="flex flex-wrap gap-1">
            {shipRefs.map((ref) => (
              <Badge key={ref} variant="secondary" className="text-[10px]">ship.{ref}</Badge>
            ))}
          </div>
        </div>
      )}

      {custom.length > 0 && (
        <div>
          <h4 className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground mb-2">
            Custom Events ({custom.length})
          </h4>
          <div className="space-y-2">
            {custom.map((ev) => (
              <div key={ev.id} className="rounded border border-border/50 bg-card/40 p-2 space-y-1">
                <div className="flex items-center gap-1.5">
                  <span className="text-[11px] font-mono text-foreground/90">{ev.id}</span>
                  {ev.direction && (
                    <Badge
                      variant={ev.direction === 'out' ? 'default' : ev.direction === 'in' ? 'outline' : 'secondary'}
                      className="text-[9px]"
                    >
                      {ev.direction}
                    </Badge>
                  )}
                </div>
                {ev.label && <p className="text-[10px] text-foreground/70">{ev.label}</p>}
                {ev.description && (
                  <p className="text-[10px] text-muted-foreground leading-relaxed">{ev.description}</p>
                )}
                {ev.schema?.properties && (
                  <div className="pt-1">
                    <p className="text-[9px] font-semibold uppercase tracking-wide text-muted-foreground mb-1">
                      Payload
                    </p>
                    <div className="font-mono text-[9px] text-foreground/60 space-y-0.5">
                      {Object.entries(ev.schema.properties).map(([k, v]) => (
                        <div key={k} className="flex gap-1">
                          <span className="text-foreground/80">{k}</span>
                          <span className="text-muted-foreground">{v.type ?? 'any'}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {shipRefs.length === 0 && custom.length === 0 && (
        <p className="text-xs text-muted-foreground text-center py-4">No events defined.</p>
      )}
    </div>
  )
}
