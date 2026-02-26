import { Compass, Plus } from 'lucide-react';
import { AdrEntry } from '@/bindings';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

interface AdrListProps {
  adrs: AdrEntry[];
  onNewAdr: () => void;
  onSelectAdr: (entry: AdrEntry) => void;
}

const STATUS_COLORS: Record<string, string> = {
  accepted: 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-300',
  rejected: 'bg-red-500/15 text-red-600 dark:text-red-300',
  superseded: 'bg-amber-500/15 text-amber-600 dark:text-amber-300',
  proposed: 'bg-blue-500/15 text-blue-600 dark:text-blue-300',
};

export default function AdrList({ adrs, onNewAdr, onSelectAdr }: AdrListProps) {
  const sortedAdrs = [...adrs].sort((a, b) => {
    const aTime = new Date(a.adr.metadata.date).getTime();
    const bTime = new Date(b.adr.metadata.date).getTime();
    return bTime - aTime;
  });

  return (
    <div className="mx-auto flex w-full max-w-6xl flex-col gap-4 p-5 md:p-6">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Architecture Decisions</h2>
          <p className="text-muted-foreground text-sm">
            {adrs.length} recorded decision{adrs.length !== 1 ? 's' : ''}
          </p>
        </div>
        <Button onClick={onNewAdr}>
          <Plus className="size-4" />
          New Decision
        </Button>
      </header>

      {adrs.length === 0 ? (
        <Card size="sm">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Compass className="size-4" />
              No decisions yet
            </CardTitle>
            <CardDescription>
              Document your architecture decisions to keep the team aligned.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={onNewAdr}>
              <Plus className="size-4" />
              Record First Decision
            </Button>
          </CardContent>
        </Card>
      ) : (
        <Card size="sm">
          <CardHeader className="pb-3">
            <CardTitle className="text-sm">Decision Register</CardTitle>
            <CardDescription>Status, date, and rationale in one place.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <div className="text-muted-foreground hidden grid-cols-[9rem_9rem_1fr_auto] gap-3 px-2 text-xs md:grid">
              <span>Status</span>
              <span>Date</span>
              <span>Title</span>
              <span />
            </div>
            {sortedAdrs.map((entry) => (
              <div
                key={entry.path}
                className="hover:bg-muted/40 grid gap-2 rounded-md border p-3 transition-colors md:grid-cols-[9rem_9rem_1fr_auto] md:items-center md:gap-3"
                title={entry.path}
              >
                <Badge
                  variant="outline"
                  className={`w-fit ${STATUS_COLORS[entry.adr.metadata.status] ?? 'text-muted-foreground'}`}
                >
                  {entry.adr.metadata.status}
                </Badge>
                <span className="text-muted-foreground text-xs">
                  {new Date(entry.adr.metadata.date).toLocaleDateString('en-US', {
                    month: 'short',
                    day: 'numeric',
                    year: 'numeric',
                  })}
                </span>
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium">{entry.adr.metadata.title}</p>
                  <p className="text-muted-foreground line-clamp-2 text-xs">{entry.adr.body}</p>
                </div>
                <Button size="sm" variant="outline" onClick={() => onSelectAdr(entry)}>
                  Open
                </Button>
              </div>
            ))}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
