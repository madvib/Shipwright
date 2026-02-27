import { createFileRoute } from '@tanstack/react-router';
import { useEffect, useMemo, useState } from 'react';
import { Clock3, RefreshCcw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';
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
            <Button size="xs" variant="outline" onClick={workspace.refreshEvents}>
              <RefreshCcw className="size-4" />
              Refresh
            </Button>
            <Button size="xs" variant="outline" onClick={() => void workspace.ingestEvents()}>
              Ingest Filesystem
            </Button>
          </>
        }
      />

      <Card size="sm">
        <CardContent className="space-y-2 px-2 py-2">
          {events.length === 0 ? (
            <div className="rounded-md border border-dashed px-4 py-4 text-center">
              <p className="text-muted-foreground text-sm">
                No events yet. Run commands or ingest filesystem changes to populate this stream.
              </p>
            </div>
          ) : (
            <>
              <div className="space-y-1">
                {pagedEvents.map((entry) => {
                  const actor = entry.actor?.trim() ? entry.actor.trim() : 'system';
                  const style = ENTITY_STYLES[entry.entity] ?? 'border-zinc-500/30 text-zinc-300';
                  const actionLabel = `${entry.entity}.${entry.action}`;

                  return (
                    <article
                      key={entry.seq}
                      className="bg-card grid grid-cols-[auto_1fr_auto] items-start gap-2 rounded-md border px-2.5 py-2"
                    >
                      <span className="text-muted-foreground min-w-10 text-xs font-medium">#{entry.seq}</span>
                      <div className="min-w-0 space-y-0.5">
                        <div className="truncate text-xs font-medium">{actionLabel}</div>
                        <div className="text-muted-foreground truncate text-xs">{entry.subject}</div>
                        {entry.details && (
                          <div className="text-muted-foreground line-clamp-2 text-[11px]">{entry.details}</div>
                        )}
                      </div>
                      <div className="flex min-w-0 flex-col items-end gap-1">
                        <div className="text-muted-foreground inline-flex items-center gap-1 text-[11px]">
                          <Clock3 className="size-3 text-violet-400" />
                          <span>{new Date(entry.timestamp).toLocaleString()}</span>
                        </div>
                        <div className="flex items-center gap-1">
                          <Badge variant="outline" className={style}>
                            {entry.entity}
                          </Badge>
                          <Badge variant="outline" className="text-muted-foreground">
                            {actor}
                          </Badge>
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
