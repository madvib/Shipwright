import { createFileRoute } from '@tanstack/react-router';
import { Clock3, RefreshCcw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';
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

function ActivityRouteComponent() {
  const workspace = useWorkspace();
  const events = workspace.eventEntries;

  return (
    <div className="mx-auto flex w-full max-w-6xl flex-col gap-4 p-5 md:p-6">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Activity Log</h1>
          <p className="text-muted-foreground text-sm">
            {workspace.activeProject?.name ?? 'Project'} · append-only event stream
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={workspace.refreshEvents}>
            <RefreshCcw className="size-4" />
            Refresh
          </Button>
          <Button variant="outline" onClick={() => void workspace.ingestEvents()}>
            Ingest Filesystem
          </Button>
        </div>
      </header>

      <Card size="sm">
        <CardContent className="space-y-3">
          {events.length === 0 ? (
            <div className="rounded-md border border-dashed px-4 py-6 text-center">
              <p className="text-muted-foreground text-sm">
                No events yet. Run commands or ingest filesystem changes to populate this stream.
              </p>
            </div>
          ) : (
            <div className="relative pl-5">
              <div className="bg-border absolute bottom-0 left-1 top-0 w-px" />
              <div className="space-y-3">
                {events.map((entry) => {
                  const actor = entry.actor?.trim() ? entry.actor.trim() : 'system';
                  const style = ENTITY_STYLES[entry.entity] ?? 'border-zinc-500/30 text-zinc-300';
                  const actionLabel = `${entry.entity}.${entry.action}`;

                  return (
                    <article
                      key={entry.seq}
                      className="bg-card relative rounded-lg border px-3 py-2"
                    >
                      <span className="bg-primary absolute -left-[1.15rem] top-3 size-2.5 rounded-full border border-background" />
                      <div className="flex items-start justify-between gap-3">
                        <div>
                          <div className="text-sm font-medium">#{entry.seq} {actionLabel}</div>
                          <div className="text-muted-foreground text-sm">{entry.subject}</div>
                          {entry.details && (
                            <div className="text-muted-foreground mt-1 text-xs">{entry.details}</div>
                          )}
                        </div>
                        <div className="flex items-center gap-1.5">
                          <Badge variant="outline" className={style}>
                            {entry.entity}
                          </Badge>
                          <Badge variant="outline" className="text-muted-foreground">
                            {actor}
                          </Badge>
                        </div>
                      </div>
                      <div className="text-muted-foreground mt-2 inline-flex items-center gap-1.5 text-xs">
                        <Clock3 className="size-3.5 text-violet-400" />
                        {new Date(entry.timestamp).toLocaleString()}
                      </div>
                    </article>
                  );
                })}
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export const Route = createFileRoute('/project/activity')({
  component: ActivityRouteComponent,
});
