import { useCallback, useContext, useEffect, useMemo, useState, ChangeEvent, MouseEvent } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import {
  FileText,
  Plus,
  Save,
  Trash2,
  X,
  Edit3,
  Compass,
} from 'lucide-react';
import { ADR, AdrEntry, AdrStatus } from '@/bindings';
import { FacetedFilter, PageFrame, PageHeader } from '@ship/primitives';
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  EmptyState,
  Input,
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
  Tooltip,
  TooltipTrigger,
  TooltipContent,
  Badge,
} from '@ship/primitives';
import { cn } from '@/lib/utils';
import {
  ADR_STATUS_OPTIONS,
  formatStatusLabel,
  getAdrStatusClasses,
} from '@/lib/workspace-ui';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@ship/primitives';
import { PageChromeContext } from '@ship/primitives';
import AdrEditor from './AdrEditor';
import TemplateEditorButton from '../common/TemplateEditorButton';
import { AdrHeaderMetadata } from './AdrHeaderMetadata';
import { AdrContextDialog } from './AdrContextDialog';
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
      tags?: string[];
    }
  ) => Promise<AdrEntry | void> | AdrEntry | void;
  onSelectAdr: (entry: AdrEntry) => void | Promise<void>;
  onMoveAdr: (id: string, status: AdrStatus) => void | Promise<void>;
  onSaveAdr: (id: string, adr: ADR) => void | Promise<void>;
  onDeleteAdr: (id: string) => void | Promise<void>;
  tagSuggestions: string[];
  adrSuggestions: { id: string; title: string }[];
  mcpEnabled: boolean;
  onBackToGlobal?: () => void;
}

type AdrSort = 'newest' | 'oldest' | 'status' | 'title';

const ADR_SORT_OPTIONS: Array<{ value: AdrSort; label: string }> = [
  { value: 'newest', label: 'Newest first' },
  { value: 'oldest', label: 'Oldest first' },
  { value: 'status', label: 'Status' },
  { value: 'title', label: 'Title' },
];

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
      supersedes_id: null,
    },
    context: '',
    decision: '',
  };
}

export default function AdrList({
  adrs,
  selectedAdr,
  onCreateAdr,
  onSelectAdr,
  onMoveAdr,
  onSaveAdr,
  onDeleteAdr,
  tagSuggestions,
  adrSuggestions,
  mcpEnabled,
  onBackToGlobal,
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
  const [contextOpen, setContextOpen] = useState(false);

  const activeEntry = useMemo(() => {
    if (!selectedAdr) return null;
    return adrs.find((entry) => entry.id === selectedAdr.id) ?? null;
  }, [adrs, selectedAdr]);

  const sortedAdrs = useMemo(() => {
    const needle = search.trim().toLowerCase();
    const next = adrs.filter((entry) => {
      const matchesSearch =
        !needle ||
        entry.adr?.metadata?.title.toLowerCase().includes(needle) ||
        entry.status.toLowerCase().includes(needle) ||
        entry.file_name.toLowerCase().includes(needle) ||
        entry.id.toLowerCase().includes(needle);

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
          const titleA = a.adr?.metadata?.title.toLowerCase() || '';
          const titleB = b.adr?.metadata?.title.toLowerCase() || '';
          return (
            titleA.localeCompare(titleB) ||
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

  const handleMoveStatus = useCallback(
    (entry: AdrEntry, nextStatus: AdrStatus) => {
      if (nextStatus === entry.status || movingIds.has(entry.id)) return;

      setMovingIds((current) => new Set(current).add(entry.id));
      void Promise.resolve(onMoveAdr(entry.id, nextStatus)).finally(() => {
        setMovingIds((current) => {
          const updated = new Set(current);
          updated.delete(entry.id);
          return updated;
        });
      });
    },
    [movingIds, onMoveAdr]
  );

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

  const RegisterContent = useMemo(() => {
    return (
      <div className="flex flex-col h-full min-h-0 py-1 px-1">
        <div className="space-y-2 mb-3">
          <Input
            value={search}
            onChange={(event: ChangeEvent<HTMLInputElement>) => setSearch(event.target.value)}
            placeholder="Filter decisions..."
            className="h-8 bg-background/50 text-xs"
          />
          <div className="flex items-center gap-2">
            <div className="h-8 flex-1">
              <FacetedFilter
                title="Status"
                options={ADR_STATUS_OPTIONS}
                selectedValues={Array.from(selectedStatuses)}
                onSelectionChange={(next: string[]) => setSelectedStatuses(new Set(next))}
              />
            </div>
            <Select value={sortBy} onValueChange={(value: string | null) => value && setSortBy(value as AdrSort)}>
              <SelectTrigger size="sm" className="h-8 w-[150px] px-2 text-xs justify-between">
                <SelectValue>
                  {ADR_SORT_OPTIONS.find((option) => option.value === sortBy)?.label}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {ADR_SORT_OPTIONS.map((option: { value: AdrSort; label: string }) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
        <div className="flex-1 overflow-auto space-y-1.5 min-h-0 no-scrollbar pb-10">
          {sortedAdrs.map((entry) => (
            <div key={entry.id} className="relative group">
              <button
                type="button"
                className={cn(
                  'w-full rounded-md border p-2 text-left transition-all hover:bg-sidebar-accent/50',
                  activeEntry?.id === entry.id
                    ? 'border-primary/30 bg-primary/5 shadow-sm'
                    : 'border-transparent bg-transparent'
                )}
                onClick={() => {
                  void onSelectAdr(entry);
                }}
                title={entry.path}
              >
                <p className={cn(
                  "truncate text-[13px] font-semibold leading-tight mb-1.5",
                  activeEntry?.id === entry.id ? "text-foreground" : "text-foreground/80"
                )}>
                  {entry.adr.metadata.title}
                </p>

                <div className="flex flex-wrap items-center gap-1.5">
                  <Badge variant="outline" className={cn(
                    "h-4.5 px-1.5 text-[9px] font-bold uppercase tracking-wider border-none bg-muted/50",
                    getAdrStatusClasses(entry.status)
                  )}
                  >
                    {formatStatusLabel(entry.status)}
                  </Badge>
                </div>
              </button>
              <div className="absolute top-1.5 right-1.5 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                <Select
                  value={entry.status}
                  onValueChange={(next: string | null) => next && handleMoveStatus(entry, next as AdrStatus)}
                >
                  <SelectTrigger size="sm" className="h-6 w-6 p-0 border-none bg-transparent hover:bg-muted/80" onClick={(e: MouseEvent) => e.stopPropagation()}>
                    <Edit3 className="size-3 text-muted-foreground" />
                  </SelectTrigger>
                  <SelectContent align="end" onClick={(e: MouseEvent) => e.stopPropagation()}>
                    {ADR_STATUS_OPTIONS.map((option: { value: string; label: string }) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          ))}
          {sortedAdrs.length === 0 && (
            <div className="py-8 text-center px-4">
              <p className="text-xs text-muted-foreground italic">No decisions found matching your filter.</p>
            </div>
          )}
        </div>
      </div>
    );
  }, [
    search,
    selectedStatuses,
    sortBy,
    sortedAdrs,
    activeEntry,
    onSelectAdr,
    handleMoveStatus
  ]);

  const { setChrome } = useContext(PageChromeContext);
  useEffect(() => {
    if (showReadPane) {
      setChrome({
        sidebar: RegisterContent,
        onBack: onBackToGlobal
      });
      return () => setChrome({ sidebar: undefined, onBack: undefined });
    }
  }, [showReadPane, RegisterContent, setChrome, onBackToGlobal]);

  return (
    <PageFrame width="full">
      <PageHeader
        title="Architecture Decision Suite"
        showGlobalChrome={false}
        actions={
          <div className="flex items-center gap-2">
            <Tooltip>
              <TooltipTrigger>
                <TemplateEditorButton kind="adr" />
              </TooltipTrigger>
              <TooltipContent side="bottom">Configure the ADR template.</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <Button onClick={startCreating}>
                  <Plus className="size-4" />
                  New Decision
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom">Document a new architecture decision.</TooltipContent>
            </Tooltip>
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
        <div className="flex-1 min-h-0 container max-w-[100rem] mx-auto">
          <div className="h-full flex flex-col pt-2 pb-6">
            <section className="flex-1 min-h-0">
              <Card size="sm" className="flex h-full flex-col border-none bg-transparent shadow-none">
                <CardHeader className="gap-3 border-b mb-6 pb-6 px-0 sticky top-0 bg-background/95 backdrop-blur-sm z-10">
                  {!displayEntry && (
                    <div className="md:hidden">
                      <CardTitle>Decision Register</CardTitle>
                      <CardDescription>
                        Explore the architecture decisions made for this project.
                      </CardDescription>
                    </div>
                  )}
                  {displayEntry && (
                    <div className="flex flex-nowrap items-start justify-between gap-3 min-w-0">
                      <div className="min-w-0 flex-1 space-y-2.5 overflow-hidden text-left">
                        <div className="flex items-center gap-2">
                          {showReadPane && (
                            <Badge
                              variant="outline"
                              className={cn(
                                'h-5 px-1.5 text-[10px] font-bold uppercase tracking-wider',
                                displayEntry && getAdrStatusClasses(displayEntry.status)
                              )}
                            >
                              {displayEntry ? formatStatusLabel(displayEntry.status) : 'Proposed'}
                            </Badge>
                          )}
                          <CardTitle className="truncate text-lg md:text-xl font-bold tracking-tight">{displayTitle}</CardTitle>
                        </div>
                        {draft && (
                          <AdrHeaderMetadata
                            adr={draft}
                            onChange={(next: ADR) => {
                              setDraft(next);
                              setDirty(true);
                            }}
                            tagSuggestions={tagSuggestions}
                            adrSuggestions={adrSuggestions}
                            onNavigate={(type: 'adr', id: string) => {
                              if (type === 'adr') {
                                const found = adrs.find(a => a.id === id);
                                if (found) void onSelectAdr(found);
                              }
                            }}
                          />
                        )}
                      </div>

                      <div className="flex flex-wrap items-center gap-2 pt-1">
                        {creating ? (
                          <Select value={createStatus} onValueChange={(next: string | null) => next && setCreateStatus(next as AdrStatus)}>
                            <SelectTrigger size="sm" className="h-8 w-36">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {ADR_STATUS_OPTIONS.map((option: { value: string; label: string }) => (
                                <SelectItem key={option.value} value={option.value}>
                                  {option.label}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        ) : displayEntry ? (
                          <Select
                            value={displayEntry.status}
                            onValueChange={(next: string | null) => {
                              if (next) handleMoveStatus(displayEntry, next as AdrStatus);
                            }}
                            disabled={movingIds.has(displayEntry.id)}
                          >
                            <SelectTrigger
                                size="sm"
                                className={cn(
                                    "h-8 w-40 font-bold uppercase tracking-wider text-[10px] transition-all",
                                    getAdrStatusClasses(displayEntry.status)
                                )}
                            >
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {ADR_STATUS_OPTIONS.map((option: { value: string; label: string }) => (
                                <SelectItem key={option.value} value={option.value} className="text-[10px] font-bold uppercase tracking-wider">
                                  {option.label}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        ) : null}

                        {(creating || editMode) || displayEntry ? (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setContextOpen(true)}
                            className="h-8 gap-1.5 shrink-0"
                          >
                            <FileText className="size-4" />
                            Decision context
                          </Button>
                        ) : null}

                        {(creating || editMode || dirty) ? (
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
                            Edit Decision
                      </Button>
                        ) : null}

                        {!creating && displayEntry && (
                          <AlertDialog>
                            <AlertDialogTrigger render={
                              <Button
                                size="sm"
                                variant="outline"
                                className="border-red-500/20 text-red-500 hover:bg-red-500/10 hover:text-red-600"
                              >
                                <Trash2 className="size-4" />
                                Delete
                              </Button>
                            } />
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

                <CardContent className="min-h-0 flex-1 overflow-auto px-0 py-8 no-scrollbar">
                  {createError && (
                    <div className="mb-6 mx-auto max-w-4xl rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                      {createError}
                    </div>
                  )}

                  {creating || editMode ? (
                    draft ? (
                      <div className="h-full min-h-0 max-w-5xl mx-auto">
                        <AdrEditor
                          key={draft?.metadata.id || 'new'}
                          adr={draft}
                          onChange={(next: ADR) => {
                            setDraft(next);
                            setDirty(true);
                            setCreateError(null);
                          }}
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
                    <div className="h-full max-w-4xl mx-auto space-y-12 pb-16 px-4">
                      <div className="space-y-4">
                        <div className="flex items-center gap-3">
                          <div className="h-px flex-1 bg-border/40" />
                          <span className="text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground/50">Decision Context</span>
                          <div className="h-px flex-1 bg-border/40" />
                        </div>
                        <div className="ship-markdown-preview min-h-[50px]">
                          <ReactMarkdown remarkPlugins={[remarkGfm]}>
                            {displayEntry.adr.context.trim() ? displayEntry.adr.context : '_No context recorded yet._'}
                          </ReactMarkdown>
                        </div>
                      </div>

                      <div className="space-y-4">
                        <div className="flex items-center gap-3">
                          <div className="h-px flex-1 bg-border/40" />
                          <span className="text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground/50">The Decision</span>
                          <div className="h-px flex-1 bg-border/40" />
                        </div>
                        <div className="ship-markdown-preview min-h-[100px]">
                          <ReactMarkdown remarkPlugins={[remarkGfm]}>
                            {displayEntry.adr.decision.trim() ? displayEntry.adr.decision : '_No decision recorded yet._'}
                          </ReactMarkdown>
                        </div>
                      </div>
                    </div>
                  ) : (
                    <div className="h-full flex items-center justify-center p-12">
                      <EmptyState
                        icon={<Compass className="size-8 opacity-20" />}
                        title="Pick A Decision"
                        description="Select an ADR from the register in the sidebar to view details."
                      />
                    </div>
                  )}
                </CardContent>
              </Card>
            </section>
          </div>
        </div>
      )
      }

      {
        draft && (
          <AdrContextDialog
            isOpen={contextOpen}
            onOpenChange={setContextOpen}
            context={draft.context}
            onContextChange={(next: string) => {
              if (creating || editMode) {
                setDraft({ ...draft, context: next });
                setDirty(true);
              }
            }}
            isEditing={creating || editMode}
          />
        )
      }
    </PageFrame>
  );
}
