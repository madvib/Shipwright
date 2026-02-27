import { Compass, Plus } from 'lucide-react';
import { AdrEntry } from '@/bindings';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';

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

  const formatDate = (value: string) => {
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) return value;
    return parsed.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  return (
    <PageFrame>
      <PageHeader
        title="Architecture Decisions"
        description={`${adrs.length} recorded decision${adrs.length !== 1 ? 's' : ''}`}
        actions={
          <Button onClick={onNewAdr}>
            <Plus className="size-4" />
            New Decision
          </Button>
        }
      />

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
            <CardDescription>Title first, with date and status at a glance.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            {sortedAdrs.map((entry) => (
              <div
                key={entry.path}
                className="hover:bg-muted/40 rounded-md border p-2.5 transition-colors"
                title={entry.path}
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <p className="truncate text-sm font-medium">{entry.adr.metadata.title}</p>
                    <div className="mt-1 flex flex-wrap items-center gap-2">
                      <span className="text-muted-foreground text-xs">{formatDate(entry.adr.metadata.date)}</span>
                      <Badge
                        variant="outline"
                        className={`w-fit ${STATUS_COLORS[entry.adr.metadata.status] ?? 'text-muted-foreground'}`}
                      >
                        {entry.adr.metadata.status}
                      </Badge>
                    </div>
                  </div>
                  <Button size="sm" variant="outline" onClick={() => onSelectAdr(entry)}>
                    Open
                  </Button>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      )}
    </PageFrame>
  );
}
