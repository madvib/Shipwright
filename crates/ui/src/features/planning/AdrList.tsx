import { useMemo, useState } from 'react';
import {
  Compass,
  Plus,
  CheckCircle2,
  XCircle,
  AlertCircle,
  RefreshCcw,
  HelpCircle
} from 'lucide-react';
import { AdrEntry } from '@/bindings';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { EmptyState } from '@/components/ui/empty-state';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';
import TemplateEditorButton from './TemplateEditorButton';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';
import { getAdrStatusClasses } from '@/lib/workspace-ui';
import { formatDate } from '@/lib/date';
import { StatusFilter } from '@/components/app/StatusFilter';

interface AdrListProps {
  adrs: AdrEntry[];
  onNewAdr: () => void;
  onSelectAdr: (entry: AdrEntry) => void;
}

type AdrSort = 'newest' | 'oldest' | 'status';
const ADR_SORT_OPTIONS: Array<{ value: AdrSort; label: string }> = [
  { value: 'newest', label: 'Newest first' },
  { value: 'oldest', label: 'Oldest first' },
  { value: 'status', label: 'Status' },
];

const ADR_STATUS_OPTIONS = [
  { value: 'Proposed', label: 'Proposed', icon: HelpCircle },
  { value: 'Accepted', label: 'Accepted', icon: CheckCircle2 },
  { value: 'Rejected', label: 'Rejected', icon: XCircle },
  { value: 'Superseded', label: 'Superseded', icon: RefreshCcw },
  { value: 'Deprecated', label: 'Deprecated', icon: AlertCircle },
];

export default function AdrList({ adrs, onNewAdr, onSelectAdr }: AdrListProps) {
  const [sortBy, setSortBy] = useState<AdrSort>('newest');
  const [search, setSearch] = useState('');
  const [selectedStatuses, setSelectedStatuses] = useState<Set<string>>(new Set());

  const sortedAdrs = useMemo(() => {
    const needle = search.trim().toLowerCase();
    const next = adrs.filter((entry) => {
      // Search filter
      const matchesSearch = !needle || (
        entry.adr.metadata.title.toLowerCase().includes(needle) ||
        entry.status.toLowerCase().includes(needle) ||
        entry.file_name.toLowerCase().includes(needle)
      );

      // Status filter
      const matchesStatus = selectedStatuses.size === 0 || selectedStatuses.has(entry.status);

      return matchesSearch && matchesStatus;
    });
    next.sort((a, b) => {
      const dateA = new Date(a.adr.metadata.date || 0).getTime();
      const dateB = new Date(b.adr.metadata.date || 0).getTime();

      switch (sortBy) {
        case 'oldest':
          return (Number.isNaN(dateA) ? 0 : dateA) - (Number.isNaN(dateB) ? 0 : dateB) || a.file_name.localeCompare(b.file_name);
        case 'status': {
          const statusCompare = (a.status || '').localeCompare(b.status || '', undefined, {
            sensitivity: 'base',
          });
          return statusCompare || (Number.isNaN(dateB) ? 0 : dateB) - (Number.isNaN(dateA) ? 0 : dateA);
        }
        case 'newest':
        default:
          return (Number.isNaN(dateB) ? 0 : dateB) - (Number.isNaN(dateA) ? 0 : dateA) || a.file_name.localeCompare(b.file_name);
      }
    });
    return next;
  }, [adrs, search, sortBy, selectedStatuses]);


  return (
    <PageFrame>
      <PageHeader
        title="Architecture Decisions"
        description={`${adrs.length} recorded decision${adrs.length !== 1 ? 's' : ''} `}
        actions={
          <div className="flex items-center gap-2">
            <TemplateEditorButton kind="adr" />
            <Button onClick={onNewAdr}>
              <Plus className="size-4" />
              New Decision
            </Button>
          </div>
        }
      />

      {adrs.length === 0 ? (
        <EmptyState
          icon={<Compass className="size-4" />}
          title="No decisions yet"
          description="Document your architecture decisions to keep the team aligned."
          action={
            <Button onClick={onNewAdr}>
              <Plus className="mr-2 size-4" />
              Record First Decision
            </Button>
          }
        />
      ) : (
        <Card size="sm">
          <CardHeader className="pb-3">
            <div className="flex items-start justify-between gap-3">
              <div>
                <CardTitle className="text-sm">Decision Register</CardTitle>
                <CardDescription>Title first, with date and status at a glance.</CardDescription>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Input
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                  placeholder="Search decisions"
                  className="h-8 w-[200px]"
                />

                <StatusFilter
                  label="Status"
                  options={ADR_STATUS_OPTIONS}
                  selectedValues={selectedStatuses}
                  onSelect={setSelectedStatuses}
                />

                <Select value={sortBy} onValueChange={(value) => setSortBy(value as AdrSort)}>
                  <SelectTrigger size="sm" className="h-8 w-[150px]">
                    <SelectValue>
                      {ADR_SORT_OPTIONS.find((option) => option.value === sortBy)?.label}
                    </SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    {ADR_SORT_OPTIONS.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
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
                      {entry.status && (
                        <Badge
                          variant="outline"
                          className={cn("h-5 px-1.5 text-[10px] font-semibold uppercase tracking-wider", getAdrStatusClasses(entry.status))}
                        >
                          {entry.status}
                        </Badge>
                      )}
                      {(entry.adr.metadata.tags ?? []).filter(Boolean).map((tag: string) => (
                        <Badge key={tag} variant="secondary" className="h-4 px-1 text-[9px]">
                          {tag}
                        </Badge>
                      ))}
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
