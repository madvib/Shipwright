import { ComponentType, useCallback, useEffect, useMemo, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import {
  AlertCircle,
  CheckCircle2,
  Compass,
  Edit3,
  GitBranch,
  HelpCircle,
  Plus,
  RefreshCcw,
  Save,
  Shapes,
  Trash2,
  X,
  XCircle,
} from 'lucide-react';
import { ADR, AdrEntry, AdrStatus } from '@/bindings';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';
import { StatusFilter } from '@/components/app/StatusFilter';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { EmptyState } from '@/components/ui/empty-state';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { formatDate } from '@/lib/date';
import { cn } from '@/lib/utils';
import { getAdrStatusClasses } from '@/lib/workspace-ui';
import AdrEditor from './AdrEditor';
import TemplateEditorButton from './TemplateEditorButton';
import { deriveAdrDocTitle } from './adrTitle';

interface AdrListProps {
  adrs: AdrEntry[];
  selectedAdr: AdrEntry | null;
  onCreateAdr: (
    title: string,
    context: string,
    decision: string,
    options?: {
      status?: string;
      date?: string;
      spec?: string | null;
      tags?: string[];
    }
  ) => Promise<AdrEntry | void> | AdrEntry | void;
  onSelectAdr: (entry: AdrEntry) => void | Promise<void>;
  onMoveAdr: (id: string, status: AdrStatus) => void | Promise<void>;
  onSaveAdr: (id: string, adr: ADR) => void | Promise<void>;
  onDeleteAdr: (id: string) => void | Promise<void>;
  specSuggestions: string[];
  tagSuggestions: string[];
  adrSuggestions: string[];
  mcpEnabled: boolean;
}

type AdrSort = 'newest' | 'oldest' | 'status' | 'title';

const ADR_SORT_OPTIONS: Array<{ value: AdrSort; label: string }> = [
  { value: 'newest', label: 'Newest first' },
  { value: 'oldest', label: 'Oldest first' },
  { value: 'status', label: 'Status' },
  { value: 'title', label: 'Title' },
];

const ADR_STATUS_OPTIONS: Array<{
  value: AdrStatus;
  label: string;
  icon: ComponentType<{ className?: string }>;
}> = [
  { value: 'proposed', label: 'Proposed', icon: HelpCircle },
  { value: 'accepted', label: 'Accepted', icon: CheckCircle2 },
  { value: 'rejected', label: 'Rejected', icon: XCircle },
  { value: 'superseded', label: 'Superseded', icon: RefreshCcw },
  { value: 'deprecated', label: 'Deprecated', icon: AlertCircle },
];

function formatStatusLabel(status: AdrStatus): string {
  return status.charAt(0).toUpperCase() + status.slice(1);
}

function normalizeAdr(adr: ADR): ADR {
  return {
    ...adr,
    metadata: {
      ...adr.metadata,
      tags: adr.metadata.tags ?? [],
    },
  };
}

function createInitialAdrDraft(): ADR {
  return {
    metadata: {
      id: '',
      title: '',
      date: new Date().toISOString().slice(0, 10),
      tags: [],
      spec_id: null,
      supersedes_id: null,
    },
    context: '',
    decision: '',
  };
}

function MarkdownSection({ title, content, emptyFallback }: { title: string; content: string; emptyFallback: string }) {
  return (
    <section className="space-y-2">
      <h3 className="text-sm font-semibold uppercase tracking-wide text-muted-foreground">{title}</h3>
      <div className="ship-markdown-preview rounded-md border bg-background">
        <ReactMarkdown remarkPlugins={[remarkGfm]}>
          {content.trim() ? content : emptyFallback}
        </ReactMarkdown>
      </div>
    </section>
  );
}

export default function AdrList({
  adrs,
  selectedAdr,
  onCreateAdr,
  onSelectAdr,
  onMoveAdr,
  onSaveAdr,
  onDeleteAdr,
  specSuggestions,
  tagSuggestions,
  adrSuggestions,
  mcpEnabled,
}: AdrListProps) {
  const [sortBy, setSortBy] = useState<AdrSort>('newest');
  const [search, setSearch] = useState('');
  const [selectedStatuses, setSelectedStatuses] = useState<Set<string>>(new Set());
  const [movingIds, setMovingIds] = useState<Set<string>>(new Set());
  const [editMode, setEditMode] = useState(false);
  const [creating, setCreating] = useState(false);
  const [draft, setDraft] = useState<ADR | null>(null);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [createStatus, setCreateStatus] = useState<AdrStatus>('proposed');
  const [createError, setCreateError] = useState<string | null>(null);

  const activeEntry = useMemo(() => {
    if (!selectedAdr) return null;
    return adrs.find((entry) => entry.id === selectedAdr.id) ?? null;
  }, [adrs, selectedAdr]);

  const stats = useMemo(() => {
    const accepted = adrs.filter((entry) => entry.status === 'accepted').length;
    const proposed = adrs.filter((entry) => entry.status === 'proposed').length;
    const linkedToSpec = adrs.filter((entry) => !!entry.adr.metadata.spec_id).length;
    const withLineage = adrs.filter((entry) => !!entry.adr.metadata.supersedes_id).length;
    return { accepted, proposed, linkedToSpec, withLineage };
  }, [adrs]);

  const sortedAdrs = useMemo(() => {
    const needle = search.trim().toLowerCase();
    const next = adrs.filter((entry) => {
      const matchesSearch =
        !needle ||
        entry.adr.metadata.title.toLowerCase().includes(needle) ||
        entry.status.toLowerCase().includes(needle) ||
        entry.file_name.toLowerCase().includes(needle) ||
        entry.id.toLowerCase().includes(needle) ||
        (entry.adr.metadata.spec_id ?? '').toLowerCase().includes(needle);

      const matchesStatus =
        selectedStatuses.size === 0 || selectedStatuses.has(entry.status);

      return matchesSearch && matchesStatus;
    });

    next.sort((a, b) => {
      const dateA = new Date(a.adr.metadata.date || 0).getTime();
      const dateB = new Date(b.adr.metadata.date || 0).getTime();

      switch (sortBy) {
        case 'oldest':
          return (
            (Number.isNaN(dateA) ? 0 : dateA) -
              (Number.isNaN(dateB) ? 0 : dateB) ||
            a.file_name.localeCompare(b.file_name)
          );
        case 'status':
          return (
            a.status.localeCompare(b.status, undefined, { sensitivity: 'base' }) ||
            (Number.isNaN(dateB) ? 0 : dateB) - (Number.isNaN(dateA) ? 0 : dateA)
          );
        case 'title':
          return (
            a.adr.metadata.title.localeCompare(b.adr.metadata.title) ||
            (Number.isNaN(dateB) ? 0 : dateB) - (Number.isNaN(dateA) ? 0 : dateA)
          );
        case 'newest':
        default:
          return (
            (Number.isNaN(dateB) ? 0 : dateB) -
              (Number.isNaN(dateA) ? 0 : dateA) ||
            a.file_name.localeCompare(b.file_name)
          );
      }
    });

    return next;
  }, [adrs, search, selectedStatuses, sortBy]);

  useEffect(() => {
    if (!selectedAdr && sortedAdrs.length > 0 && !creating) {
      void Promise.resolve(onSelectAdr(sortedAdrs[0]));
    }
  }, [creating, onSelectAdr, selectedAdr, sortedAdrs]);

  useEffect(() => {
    if (creating) return;
    if (activeEntry) {
      setDraft(normalizeAdr(activeEntry.adr));
      setDirty(false);
      setSaving(false);
      setEditMode(false);
      setCreateError(null);
      return;
    }
    setDraft(null);
    setDirty(false);
    setSaving(false);
    setEditMode(false);
    setCreateError(null);
  }, [activeEntry?.id, creating]);

  const handleMoveStatus = (entry: AdrEntry, next: string) => {
    const nextStatus = next as AdrStatus;
    if (nextStatus === entry.status || movingIds.has(entry.id)) return;

    setMovingIds((current) => new Set(current).add(entry.id));
    void Promise.resolve(onMoveAdr(entry.id, nextStatus)).finally(() => {
      setMovingIds((current) => {
        const updated = new Set(current);
        updated.delete(entry.id);
        return updated;
      });
    });
  };

  const startCreating = () => {
    setCreating(true);
    setEditMode(true);
    setDraft(createInitialAdrDraft());
    setDirty(false);
    setSaving(false);
    setCreateStatus('proposed');
    setCreateError(null);
  };

  const cancelEditing = () => {
    if (creating) {
      setCreating(false);
      setEditMode(false);
      setDraft(activeEntry ? normalizeAdr(activeEntry.adr) : null);
      setDirty(false);
      setCreateError(null);
      return;
    }

    if (!activeEntry) return;
    setDraft(normalizeAdr(activeEntry.adr));
    setDirty(false);
    setEditMode(false);
    setCreateError(null);
  };

  const saveDraft = useCallback(async () => {
    if (!draft || saving) return;
    setSaving(true);
    try {
      if (creating) {
        const nextTitle =
          draft.metadata.title.trim() ||
          deriveAdrDocTitle(draft.decision) ||
          deriveAdrDocTitle(draft.context);

        if (!nextTitle) {
          setCreateError('Title is required.');
          return;
        }
        if (!draft.decision.trim()) {
          setCreateError('Decision is required.');
          return;
        }

        const created = await onCreateAdr(nextTitle, draft.context.trim(), draft.decision.trim(), {
          status: createStatus,
          date: draft.metadata.date,
          spec: draft.metadata.spec_id ?? null,
          tags: draft.metadata.tags ?? [],
        });
        if (!created) {
          setCreateError('Failed to create ADR.');
          return;
        }
        setCreating(false);
        setDirty(false);
        setEditMode(false);
        setCreateError(null);
        await Promise.resolve(onSelectAdr(created));
        return;
      }

      if (!activeEntry || !dirty) return;
      await onSaveAdr(activeEntry.id, draft);
      setDirty(false);
      setEditMode(false);
      setCreateError(null);
    } finally {
      setSaving(false);
    }
  }, [
    activeEntry,
    createStatus,
    creating,
    dirty,
    draft,
    onCreateAdr,
    onSaveAdr,
    onSelectAdr,
    saving,
  ]);

  const showReadPane = !creating && !editMode;
  const displayEntry = creating ? null : activeEntry;
  const displayTitle = creating
    ? (draft?.metadata.title.trim() || 'New ADR')
    : (displayEntry?.adr.metadata.title ?? 'Select A Decision');
  const displayDate = creating ? draft?.metadata.date : displayEntry?.adr.metadata.date;

  return (
    <PageFrame width="wide">
      <PageHeader
        title="Architecture Decision Suite"
        description="Read-first decision intelligence with full-screen create and edit modes."
        actions={
          <div className="flex items-center gap-2">
            <TemplateEditorButton kind="adr" />
            <Button onClick={startCreating}>
              <Plus className="size-4" />
              New Decision
            </Button>
          </div>
        }
      />

      {adrs.length === 0 && !creating ? (
        <EmptyState
          icon={<Compass className="size-4" />}
          title="No decisions yet"
          description="Document your architecture decisions to keep the team aligned."
          action={
            <Button onClick={startCreating}>
              <Plus className="mr-2 size-4" />
              Record First Decision
            </Button>
          }
        />
      ) : (
        <div className="space-y-4">
          {showReadPane && (
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
              <Card size="sm">
                <CardHeader className="pb-2">
                  <CardDescription>Total ADRs</CardDescription>
                  <CardTitle className="text-xl">{adrs.length}</CardTitle>
                </CardHeader>
              </Card>
              <Card size="sm">
                <CardHeader className="pb-2">
                  <CardDescription>Accepted</CardDescription>
                  <CardTitle className="text-xl text-status-green">{stats.accepted}</CardTitle>
                </CardHeader>
              </Card>
              <Card size="sm">
                <CardHeader className="pb-2">
                  <CardDescription>In Proposal</CardDescription>
                  <CardTitle className="text-xl text-status-blue">{stats.proposed}</CardTitle>
                </CardHeader>
              </Card>
              <Card size="sm">
                <CardHeader className="pb-2">
                  <CardDescription>Coverage</CardDescription>
                  <CardTitle className="text-sm font-semibold">
                    {stats.linkedToSpec} specs · {stats.withLineage} lineage links
                  </CardTitle>
                </CardHeader>
              </Card>
            </div>
          )}

          <div
            className={cn(
              'grid min-h-0 gap-4',
              showReadPane && 'lg:grid-cols-[360px_minmax(0,1fr)] lg:h-[calc(100vh-19rem)]'
            )}
          >
            {showReadPane && (
              <aside className="h-full min-h-0">
                <Card size="sm" className="flex h-full min-h-[60vh] flex-col">
                  <CardHeader className="space-y-2 pb-2">
                    <CardTitle className="text-sm">Decision Register</CardTitle>
                    <div className="space-y-2">
                      <Input
                        value={search}
                        onChange={(event) => setSearch(event.target.value)}
                        placeholder="Search title, id, spec"
                        className="h-8"
                      />
                      <div className="flex items-center gap-2">
                        <StatusFilter
                          label="Status"
                          options={ADR_STATUS_OPTIONS}
                          selectedValues={selectedStatuses}
                          onSelect={setSelectedStatuses}
                          className="h-8"
                        />
                        <Select value={sortBy} onValueChange={(value) => setSortBy(value as AdrSort)}>
                          <SelectTrigger size="sm" className="h-8 w-[138px]">
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
                  <CardContent className="min-h-0 flex-1 space-y-2 overflow-auto">
                    {sortedAdrs.map((entry) => (
                      <button
                        key={entry.id}
                        type="button"
                        className={cn(
                          'hover:bg-muted/40 w-full rounded-md border p-2 text-left transition-colors',
                          activeEntry?.id === entry.id && 'border-primary/50 bg-primary/5'
                        )}
                        onClick={() => void onSelectAdr(entry)}
                        title={entry.path}
                      >
                        <p className="truncate text-sm font-medium">{entry.adr.metadata.title}</p>
                        <div className="mt-1 flex flex-wrap items-center gap-1.5">
                          <Badge variant="secondary" className="h-5 px-1.5 font-mono text-[10px]">
                            {entry.id}
                          </Badge>
                          <Badge
                            variant="outline"
                            className={cn(
                              'h-5 px-1.5 text-[10px] font-semibold uppercase tracking-wider',
                              getAdrStatusClasses(entry.status)
                            )}
                          >
                            {formatStatusLabel(entry.status)}
                          </Badge>
                        </div>
                      </button>
                    ))}
                  </CardContent>
                </Card>
              </aside>
            )}

            <section className="h-full min-h-0">
              <Card size="sm" className="flex h-full min-h-[60vh] flex-col">
              <CardHeader className="gap-3 border-b pb-3">
                {!creating && !displayEntry ? (
                  <div>
                    <CardTitle>Select A Decision</CardTitle>
                    <CardDescription>
                      Choose an ADR from the register to read or edit it.
                    </CardDescription>
                  </div>
                ) : (
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="min-w-0 space-y-1">
                      <CardTitle className="truncate">{displayTitle}</CardTitle>
                      <CardDescription className="flex flex-wrap items-center gap-2">
                        {displayDate && <span>{formatDate(displayDate)}</span>}
                        {!creating && displayEntry && (
                          <>
                            <span className="inline-flex items-center gap-1">
                              <Shapes className="size-3.5" />
                              {displayEntry.adr.metadata.spec_id
                                ? displayEntry.adr.metadata.spec_id
                                : 'No linked spec'}
                            </span>
                            <span className="inline-flex items-center gap-1">
                              <GitBranch className="size-3.5" />
                              {displayEntry.adr.metadata.supersedes_id
                                ? `Supersedes ${displayEntry.adr.metadata.supersedes_id}`
                                : 'No lineage link'}
                            </span>
                          </>
                        )}
                      </CardDescription>
                    </div>

                    <div className="flex flex-wrap items-center gap-2">
                      {creating ? (
                        <Select value={createStatus} onValueChange={(next) => next && setCreateStatus(next as AdrStatus)}>
                          <SelectTrigger size="sm" className="h-8 w-36">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {ADR_STATUS_OPTIONS.map((option) => (
                              <SelectItem key={option.value} value={option.value}>
                                {option.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      ) : displayEntry ? (
                        <Select
                          value={displayEntry.status}
                          onValueChange={(next) => {
                            if (next) handleMoveStatus(displayEntry, next);
                          }}
                          disabled={movingIds.has(displayEntry.id)}
                        >
                          <SelectTrigger size="sm" className="h-8 w-36">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {ADR_STATUS_OPTIONS.map((option) => (
                              <SelectItem key={option.value} value={option.value}>
                                {option.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      ) : null}

                      {(creating || editMode) ? (
                        <>
                          <Button variant="outline" size="sm" onClick={cancelEditing} disabled={saving}>
                            <X className="size-4" />
                            Cancel
                          </Button>
                          <Button size="sm" onClick={() => void saveDraft()} disabled={saving || (!dirty && !creating)}>
                            <Save className="size-4" />
                            {creating ? (saving ? 'Creating…' : 'Create') : (saving ? 'Saving…' : 'Save')}
                          </Button>
                        </>
                      ) : displayEntry ? (
                        <Button size="sm" onClick={() => setEditMode(true)}>
                          <Edit3 className="size-4" />
                          Edit Full Screen
                        </Button>
                      ) : null}

                      {!creating && displayEntry && (
                        <AlertDialog>
                          <AlertDialogTrigger
                            render={
                              <Button
                                size="sm"
                                variant="outline"
                                className="border-destructive/40 text-destructive hover:bg-destructive/10"
                              />
                            }
                          >
                            <Trash2 className="size-4" />
                          </AlertDialogTrigger>
                          <AlertDialogContent size="sm">
                            <AlertDialogHeader>
                              <AlertDialogTitle>Delete this ADR?</AlertDialogTitle>
                              <AlertDialogDescription>
                                This will permanently remove the decision document.
                              </AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                              <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
                              <AlertDialogAction
                                size="sm"
                                variant="destructive"
                                onClick={() => void onDeleteAdr(displayEntry.id)}
                              >
                                Delete
                              </AlertDialogAction>
                            </AlertDialogFooter>
                          </AlertDialogContent>
                        </AlertDialog>
                      )}
                    </div>
                  </div>
                )}
              </CardHeader>

              <CardContent className="min-h-0 flex-1 overflow-hidden p-2 md:p-3">
                {createError && (
                  <div className="mb-3 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                    {createError}
                  </div>
                )}

                {creating || editMode ? (
                  draft ? (
                    <div className="h-full min-h-0">
                      <AdrEditor
                        adr={draft}
                        onChange={(next) => {
                          setDraft(next);
                          setDirty(true);
                          setCreateError(null);
                        }}
                        specSuggestions={specSuggestions}
                        tagSuggestions={tagSuggestions}
                        adrSuggestions={adrSuggestions}
                        mcpEnabled={mcpEnabled}
                      />
                    </div>
                  ) : (
                    <EmptyState
                      icon={<Compass className="size-4" />}
                      title="No Draft Loaded"
                      description="Start a new ADR or select an existing one."
                    />
                  )
                ) : displayEntry ? (
                  <div className="h-full overflow-auto space-y-4">
                    <MarkdownSection
                      title="Context"
                      content={displayEntry.adr.context}
                      emptyFallback="_No context captured yet._"
                    />
                    <MarkdownSection
                      title="Decision"
                      content={displayEntry.adr.decision}
                      emptyFallback="_No decision recorded yet._"
                    />
                  </div>
                ) : (
                  <EmptyState
                    icon={<Compass className="size-4" />}
                    title="No ADR Selected"
                    description="Pick a decision from the register to inspect context and decision markdown."
                  />
                )}
              </CardContent>
              </Card>
            </section>
          </div>
        </div>
      )}
    </PageFrame>
  );
}
