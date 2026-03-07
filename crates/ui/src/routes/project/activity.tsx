import { createFileRoute } from '@tanstack/react-router';
import { useEffect, useMemo, useState } from 'react';
import { Clock3, RefreshCcw } from 'lucide-react';
import { Button } from '@ship/ui';
import { Badge } from '@ship/ui';
import { Card, CardContent } from '@ship/ui';
import { PageFrame, PageHeader, Tooltip, TooltipTrigger, TooltipContent } from '@ship/ui';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';

const ENTITY_STYLES: Record<string, string> = {
  issue: 'border-amber-500/30 text-amber-300',
  spec: 'border-sky-500/30 text-sky-300',
  adr: 'border-emerald-500/30 text-emerald-300',
  project: 'border-violet-500/30 text-violet-300',
  config: 'border-fuchsia-500/30 text-fuchsia-300',
  mode: 'border-cyan-500/30 text-cyan-300',
  feature: 'border-lime-500/30 text-lime-300',
  release: 'border-orange-500/30 text-orange-300',
};

const ACTOR_STYLES: Record<'user' | 'agent' | 'system', string> = {
  user: 'border-blue-500/30 text-blue-600 dark:text-blue-300',
  agent: 'border-emerald-500/30 text-emerald-600 dark:text-emerald-300',
  system: 'border-zinc-500/30 text-zinc-600 dark:text-zinc-300',
};

function classifyActor(raw: string): { kind: 'user' | 'agent' | 'system'; label: string } {
  const value = raw.trim().toLowerCase();
  if (!value || value === 'system' || value === 'hook') {
    return { kind: 'system', label: 'System' };
  }
  if (value.includes('agent') || value.includes('mcp') || value.includes('ai')) {
    return { kind: 'agent', label: 'Agent' };
  }
  return { kind: 'user', label: 'User' };
}

function cleanEventText(value: string | null | undefined): string {
  const raw = (value ?? '').trim();
  if (!raw) return '';
  return raw
    .replace(
      /\b(?:title|size|mtime|path|created|updated)\s*=\s*("[^"]*"|'[^']*'|[^,\s;|]+)/gi,
      ''
    )
    .replace(/\s*[|,;]\s*/g, ' · ')
    .replace(/\s*·\s*·+\s*/g, ' · ')
    .replace(/^·\s*|\s*·$/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

function parseDetailMap(value: string | null | undefined): Record<string, string> {
  const input = (value ?? '').trim();
  if (!input) return {};

  const map: Record<string, string> = {};
  const pairs = input.match(/([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*("[^"]*"|'[^']*'|[^\s|,;]+)/g) ?? [];
  for (const pair of pairs) {
    const match = pair.match(/^([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+)$/);
    if (!match) continue;
    const key = match[1].trim().toLowerCase();
    const valueText = match[2].trim().replace(/^["']|["']$/g, '');
    map[key] = valueText;
  }
  return map;
}

function humanizeToken(value: string): string {
  return value
    .replace(/\.md$/i, '')
    .replace(/[-_]+/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

function formatMainLabel(entity: string, action: string, subject: string): string {
  const entityLabel = humanizeToken(entity);
  const actionLabel = humanizeToken(action);
  if (!subject) return `${entityLabel} ${actionLabel}`;
  return `${entityLabel} ${actionLabel}: ${humanizeToken(subject)}`;
}

function formatSecondaryLabel(details: Record<string, string>): string {
  const from = details.from ?? '';
  const to = details.to ?? '';
  const status = details.status ?? '';
  const target = details.target ?? '';

  if (from && to) {
    return `${humanizeToken(from)} -> ${humanizeToken(to)}`;
  }
  if (status) {
    return `Status: ${humanizeToken(status)}`;
  }
  if (target) {
    return humanizeToken(target);
  }
  return '';
}

const PAGE_SIZE = 50;

function ActivityRouteComponent() {
  const workspace = useWorkspace();
  const events = workspace.eventEntries;
  const [page, setPage] = useState(1);

  const totalPages = Math.max(1, Math.ceil(events.length / PAGE_SIZE));
  const pagedEvents = useMemo(() => {
    const start = (page - 1) * PAGE_SIZE;
    return events.slice(start, start + PAGE_SIZE);
  }, [events, page]);

  useEffect(() => {
    setPage((current) => Math.min(current, totalPages));
  }, [totalPages]);

  return (
    <PageFrame>
      <PageHeader
        title="Activity Log"
        description={`Append-only runtime stream (${events.length} event${events.length === 1 ? '' : 's'})`}
        actions={
          <>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button size="xs" variant="outline" onClick={workspace.refreshEvents}>
                  <RefreshCcw className="size-4" />
                  Refresh
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom">Refresh the activity log from the server.</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <Button size="xs" variant="outline" onClick={() => void workspace.ingestEvents()}>
                  Ingest Filesystem
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom">Scan the filesystem for new events and append them.</TooltipContent>
            </Tooltip>
          </>
        }
      />

      <Card size="sm">
        <CardContent className="space-y-1.5 px-1.5 py-1.5">
          {events.length === 0 ? (
            <div className="rounded-md border border-dashed px-4 py-4 text-center">
              <p className="text-muted-foreground text-sm">
                No events yet. Run commands or ingest filesystem changes to populate this stream.
              </p>
            </div>
          ) : (
            <>
              <div className="space-y-0.5">
                {pagedEvents.map((entry) => {
                  const actor = entry.actor?.trim() ? entry.actor.trim() : 'system';
                  const style = ENTITY_STYLES[entry.entity] ?? 'border-zinc-500/30 text-zinc-300';
                  const actorInfo = classifyActor(actor);
                  const subject = cleanEventText(entry.subject);
                  const detailMap = parseDetailMap(entry.details);
                  const mainLabel = formatMainLabel(entry.entity, entry.action, subject);
                  const secondaryLabel = formatSecondaryLabel(detailMap);

                  return (
                    <article
                      key={entry.seq}
                      className="bg-card rounded-md border px-2 py-1.5"
                    >
                      <div className="grid min-w-0 grid-cols-[1fr_auto] items-center gap-1 text-[11px] leading-4">
                        <div className="min-w-0 truncate">
                          <span className="text-muted-foreground mr-2 shrink-0 font-medium">#{entry.seq}</span>
                          {mainLabel}
                          {secondaryLabel ? (
                            <span className="text-muted-foreground font-normal"> · {secondaryLabel}</span>
                          ) : null}
                        </div>
                        <div className="ml-auto flex shrink-0 items-center gap-1">
                          <Badge variant="outline" className={`h-4.5 px-1.5 text-[10px] ${style}`}>
                            {entry.entity}
                          </Badge>
                          <Badge
                            variant="outline"
                            className={`h-4.5 px-1.5 text-[10px] ${ACTOR_STYLES[actorInfo.kind]}`}
                          >
                            {actorInfo.label}
                          </Badge>
                          <span className="text-muted-foreground inline-flex items-center gap-1">
                            <Clock3 className="size-2.5 text-violet-400" />
                            <span>{new Date(entry.timestamp).toLocaleString()}</span>
                          </span>
                        </div>
                      </div>
                    </article>
                  );
                })}
              </div>

              {events.length > PAGE_SIZE && (
                <div className="flex items-center justify-between border-t px-1 pt-2">
                  <p className="text-muted-foreground text-[11px]">
                    Page {page} of {totalPages}
                  </p>
                  <div className="flex items-center gap-1">
                    <Button
                      size="xs"
                      variant="outline"
                      disabled={page <= 1}
                      onClick={() => setPage((current) => Math.max(1, current - 1))}
                    >
                      Prev
                    </Button>
                    <Button
                      size="xs"
                      variant="outline"
                      disabled={page >= totalPages}
                      onClick={() => setPage((current) => Math.min(totalPages, current + 1))}
                    >
                      Next
                    </Button>
                  </div>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>
    </PageFrame>
  );
}

export const Route = createFileRoute('/project/activity')({
  component: ActivityRouteComponent,
});
