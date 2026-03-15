import { useCallback, useEffect, useMemo, useState } from 'react';
import { FeatureDocument as FeatureEntry } from '@/bindings';
import {
  AlertTriangle,
  ArrowLeft,
  CheckCircle2,
  Edit3,
  FileText,
  GitBranch,
  Save,
  X,
} from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import MarkdownEditor from '@/components/editor';
import { FeatureHeaderMetadata } from './FeatureHeaderMetadata';
import { FeatureChecklistSection } from './FeatureChecklistSection';
import {
  toggleFeatureChecklistItem,
} from '@/features/planning/common/hub/utils/featureMetrics';
import { Badge, Button } from '@ship/primitives';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@ship/primitives';
import { Progress } from '@ship/primitives';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/primitives';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@ship/primitives';
import {
  readFrontmatterStringListField,
  splitFrontmatterDocument,
  setFrontmatterStringField,
  setFrontmatterStringListField,
} from '@ship/primitives';
import { cn } from '@/lib/utils';

const DOC_STATUS_OPTIONS = ['not-started', 'draft', 'reviewed', 'published'];

interface FeatureDetailProps {
  feature: FeatureEntry;
  releaseSuggestions?: string[];
  tagSuggestions?: string[];
  mcpEnabled?: boolean;
  onClose: () => void;
  onSelectRelease: (fileName: string) => void;
  onSave: (fileName: string, content: string) => Promise<void> | void;
  onStart: (fileName: string) => Promise<void> | void;
  onDone: (fileName: string) => Promise<void> | void;
  onSaveDocumentation: (
    fileName: string,
    content: string,
    status?: string | null,
    verifyNow?: boolean
  ) => Promise<void> | void;
}

export default function FeatureDetail({
  feature,
  releaseSuggestions = [],
  tagSuggestions = [],
  mcpEnabled = true,
  onClose,
  onSelectRelease,
  onSave,
  onStart,
  onDone,
  onSaveDocumentation,
}: FeatureDetailProps) {
  const [content, setContent] = useState(feature.content ?? '');
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [editing, setEditing] = useState(false);

  const [docsContent, setDocsContent] = useState(feature.docs_content ?? '');
  const [docsStatus, setDocsStatus] = useState(feature.docs_status ?? 'not-started');
  const [docsDirty, setDocsDirty] = useState(false);
  const [docsSaving, setDocsSaving] = useState(false);
  const [editingDocs, setEditingDocs] = useState(false);

  const [activeTab, setActiveTab] = useState<'feature' | 'docs'>('feature');

  useEffect(() => {
    setContent(feature.content ?? '');
    setDirty(false);
    setSaving(false);
    setEditing(false);

    setDocsContent(feature.docs_content ?? '');
    setDocsStatus(feature.docs_status ?? 'not-started');
    setDocsDirty(false);
    setDocsSaving(false);
    setEditingDocs(false);

    setActiveTab('feature');
  }, [feature]);

  const saveFeature = useCallback(async () => {
    if (!dirty || saving) return;
    setSaving(true);
    try {
      await onSave(feature.file_name, content);
      setDirty(false);
      setEditing(false);
    } finally {
      setSaving(false);
    }
  }, [content, dirty, feature.file_name, onSave, saving]);

  const cancelEditing = useCallback(() => {
    setContent(feature.content ?? '');
    setDirty(false);
    setEditing(false);
  }, [feature.content]);

  const handleStatusTransition = useCallback(async (nextStatus: string) => {
    if (nextStatus === feature.status) return;
    if (nextStatus === 'in-progress') {
      await onStart(feature.file_name);
    } else if (nextStatus === 'implemented') {
      await onDone(feature.file_name);
    }
  }, [feature.file_name, feature.status, onStart, onDone]);

  const saveDocs = useCallback(async () => {
    if (!docsDirty || docsSaving) return;
    setDocsSaving(true);
    try {
      await onSaveDocumentation(feature.file_name, docsContent, docsStatus);
      setDocsDirty(false);
      setEditingDocs(false);
    } catch {
      // Error state is set by useFeatureActions; keep editor open for correction.
    } finally {
      setDocsSaving(false);
    }
  }, [docsContent, docsDirty, docsSaving, feature.file_name, onSaveDocumentation, docsStatus]);

  const cancelDocsEditing = useCallback(() => {
    setDocsContent(feature.docs_content ?? '');
    setDocsStatus(feature.docs_status ?? 'not-started');
    setDocsDirty(false);
    setEditingDocs(false);
  }, [feature.docs_content, feature.docs_status]);

  const documentModel = useMemo(() => splitFrontmatterDocument(content), [content]);
  const todoItems = useMemo(
    () => (feature.todos ?? []).map((todo) => ({ ...todo, done: todo.completed })),
    [feature.todos]
  );
  const acceptanceItems = useMemo(
    () =>
      (feature.acceptance_criteria ?? []).map((criterion) => ({
        id: criterion.id,
        text: criterion.text,
        done: criterion.met,
      })),
    [feature.acceptance_criteria]
  );
  const readiness = useMemo(() => {
    const todosTotal = todoItems.length;
    const todosDone = todoItems.filter((item) => item.done).length;
    const todosOpen = Math.max(todosTotal - todosDone, 0);
    const acceptanceTotal = acceptanceItems.length;
    const acceptanceDone = acceptanceItems.filter((item) => item.done).length;
    const acceptanceOpen = Math.max(acceptanceTotal - acceptanceDone, 0);
    const hasChecklistCoverage = todosTotal > 0 || acceptanceTotal > 0;
    const todosPercent = todosTotal === 0 ? 0 : Math.round((todosDone / todosTotal) * 100);
    const acceptancePercent =
      acceptanceTotal === 0 ? 0 : Math.round((acceptanceDone / acceptanceTotal) * 100);
    const readinessPercent = hasChecklistCoverage
      ? Math.round(todosPercent * 0.6 + acceptancePercent * 0.4)
      : 0;
    const blocking =
      hasChecklistCoverage &&
      (acceptanceTotal > 0 ? acceptanceOpen > 0 : todosOpen > 0);

    return {
      todos: { total: todosTotal, done: todosDone, open: todosOpen },
      acceptance: { total: acceptanceTotal, done: acceptanceDone, open: acceptanceOpen },
      hasChecklistCoverage,
      readinessPercent,
      blocking,
    };
  }, [acceptanceItems, todoItems]);
  const hasChecklistCoverage = readiness.hasChecklistCoverage;
  const tags = useMemo(
    () => readFrontmatterStringListField(documentModel.frontmatter, 'tags'),
    [documentModel.frontmatter]
  );
  const docsPreview = useMemo(() => docsContent.trim(), [docsContent]);
  const docsStatusLabel = (feature.docs_status ?? 'not-started').replace(/-/g, ' ');
  const docsStatusBadgeClass = useMemo(() => {
    switch (feature.docs_status ?? 'not-started') {
      case 'draft':
        return 'border-amber-300 bg-amber-100/80 text-amber-900';
      case 'reviewed':
        return 'border-blue-300 bg-blue-100/80 text-blue-900';
      case 'published':
        return 'border-emerald-300 bg-emerald-100/80 text-emerald-900';
      default:
        return 'border-muted-foreground/30 bg-muted/60 text-muted-foreground';
    }
  }, [feature.docs_status]);
  const modelDelta = feature.model?.delta;
  const modelDeltaActionableItems = modelDelta?.actionable_items ?? [];
  const deltaSignals = useMemo(() => {
    if (modelDelta) {
      const signals: string[] = [];
      if (modelDelta.declaration_missing) {
        signals.push('Declaration is missing.');
      }
      if (modelDelta.status_missing) {
        signals.push('Status is missing.');
      }
      for (const criterion of modelDelta.unmet_acceptance_criteria ?? []) {
        signals.push(`Unmet acceptance criterion: ${criterion}`);
      }
      for (const check of modelDelta.failing_checks ?? []) {
        signals.push(`Failing status check: ${check}`);
      }
      for (const criterion of modelDelta.missing_pass_fail_criteria ?? []) {
        signals.push(`Missing PASS/FAIL condition: ${criterion}`);
      }
      return signals;
    }

    const signals: string[] = [];
    if (!hasChecklistCoverage) {
      signals.push('Declaration checklist is missing (no todos or acceptance criteria).');
    }
    if (readiness.acceptance.open > 0) {
      signals.push(
        `${readiness.acceptance.open} acceptance criteria currently unmet.`
      );
    }
    if (readiness.acceptance.open === 0 && readiness.todos.open > 0) {
      signals.push(`${readiness.todos.open} delivery todos still open.`);
    }
    return signals;
  }, [
    hasChecklistCoverage,
    modelDelta,
    readiness.acceptance.open,
    readiness.todos.open,
  ]);

  const handleToggleChecklistItem = useCallback(async (
    section: 'todos' | 'acceptance',
    itemIndex: number
  ) => {
    if (saving) return;
    const nextContent = toggleFeatureChecklistItem(content, section, itemIndex);
    if (!nextContent || nextContent === content) return;
    const previousContent = content;
    setContent(nextContent);
    setDirty(false);
    setSaving(true);
    try {
      await onSave(feature.file_name, nextContent);
    } catch {
      setContent(previousContent);
    } finally {
      setSaving(false);
    }
  }, [content, feature.file_name, onSave, saving]);

  const handleMetadataUpdate = useCallback((updates: {
    release_id?: string;
    tags?: string[];
  }) => {
    let nextContent = content;
    const delimiter = documentModel.delimiter || '---';

    if (updates.release_id !== undefined) {
      nextContent = setFrontmatterStringField(nextContent, 'release_id', updates.release_id, delimiter) || nextContent;
    }
    if (updates.tags) {
      nextContent = setFrontmatterStringListField(nextContent, 'tags', updates.tags, delimiter) || nextContent;
    }

    if (nextContent !== content) {
      setContent(nextContent);
      setDirty(true);
    }
  }, [content, documentModel.delimiter]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's' && (editing || editingDocs)) {
        event.preventDefault();
        if (editing) void saveFeature();
        if (editingDocs) void saveDocs();
        return;
      }
      if (event.key === 'Escape') {
        if (editing) {
          event.preventDefault();
          cancelEditing();
          return;
        }
        if (editingDocs) {
          event.preventDefault();
          cancelDocsEditing();
          return;
        }
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [cancelDocsEditing, cancelEditing, editing, editingDocs, saveDocs, saveFeature]);

  return (
    <div className="space-y-3">
      <Card size="sm" className="border-primary/20">
        <CardContent className="space-y-1.5 py-2.5">
          <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2">
            <div className="flex min-w-0 items-center gap-2">
              <Button variant="ghost" size="sm" className="h-7 px-2" onClick={onClose}>
                <ArrowLeft className="size-4" />
                Back To Hub
              </Button>
            </div>

            <h2 className="truncate px-2 text-center text-lg font-semibold tracking-tight">
              {feature?.title}
            </h2>

            <div className="flex min-w-0 justify-end gap-2">
              {editing && (
                <>
                  <Button variant="outline" size="sm" onClick={cancelEditing} disabled={saving}>
                    <X className="size-4" />
                    Cancel
                  </Button>
                  <Button size="sm" onClick={() => void saveFeature()} disabled={!dirty || saving}>
                    <Save className="size-4" />
                    {saving ? 'Saving…' : 'Save'}
                  </Button>
                </>
              )}
            </div>
          </div>

          <div className="flex flex-col items-center gap-1.5">
            <FeatureHeaderMetadata
              status={feature.status}
              releaseId={feature.release_id || undefined}
              tags={tags}
              isEditing={editing}
              onUpdate={handleMetadataUpdate}
              onStatusTransition={handleStatusTransition}
              releaseSuggestions={releaseSuggestions}
              tagSuggestions={tagSuggestions}
              onNavigate={(id, type) => {
                if (type === 'release') onSelectRelease(id);
              }}
            />
          </div>
        </CardContent>
      </Card>

      {editing ? (
        <Card size="sm" className="flex min-h-[calc(100vh-15.5rem)] flex-col">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Feature Editor</CardTitle>
            <CardDescription>
              Editing {feature.file_name}
            </CardDescription>
          </CardHeader>
          <CardContent className="min-h-0 flex-1 overflow-hidden p-2 md:p-3">
            <MarkdownEditor
              toolbarStart={
                <span className="text-muted-foreground text-xs">
                  {dirty ? 'Unsaved changes' : 'Saved'}
                </span>
              }
              value={content}
              onChange={(next) => {
                setContent(next);
                setDirty(true);
              }}
              mcpEnabled={mcpEnabled}
              fillHeight
              rows={24}
              defaultMode="edit"
            />
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-3">
          <div className="grid gap-2 md:grid-cols-3">
            <div className="space-y-1.5 rounded-md border bg-card px-2.5 py-2">
              <div className="flex items-center justify-between text-xs">
                <span className="text-muted-foreground">Readiness</span>
                <span className="font-semibold">{readiness.readinessPercent}%</span>
              </div>
              <Progress
                value={readiness.readinessPercent}
                indicatorClassName={cn(
                  !hasChecklistCoverage
                    ? 'bg-muted-foreground/40'
                    : readiness.blocking
                      ? 'bg-amber-500'
                      : 'bg-emerald-500'
                )}
              />
              <div className="text-muted-foreground flex flex-wrap gap-3 text-xs">
                <span>Todos {readiness.todos.done}/{readiness.todos.total}</span>
                <span>Acceptance {readiness.acceptance.done}/{readiness.acceptance.total}</span>
              </div>
              {!hasChecklistCoverage && (
                <p className="text-muted-foreground text-xs italic">
                  No checklist coverage yet.
                </p>
              )}
            </div>

            <div className="space-y-1.5 rounded-md border bg-card px-2.5 py-2">
              <p className="text-muted-foreground inline-flex items-center gap-1.5 text-xs">
                <FileText className="size-3.5" />
                Docs:
                <Badge variant="outline" className={cn('h-5 px-1.5 capitalize', docsStatusBadgeClass)}>
                  {docsStatusLabel}
                </Badge>
              </p>
              <p className="text-muted-foreground text-xs">
                Revision: <span className="font-medium text-foreground">{feature.docs_revision ?? 0}</span>
              </p>
              {feature.docs_updated_at && (
                <p className="text-muted-foreground text-xs">
                  Updated: <span className="font-medium text-foreground">{new Date(feature.docs_updated_at).toLocaleString()}</span>
                </p>
              )}
              {!docsPreview && (
                <p className="text-muted-foreground text-xs italic">
                  No docs content yet.
                </p>
              )}
            </div>

            <div className="space-y-1.5 rounded-md border bg-card px-2.5 py-2">
              <p className="text-muted-foreground text-xs">Execution Context</p>
              {feature.branch ? (
                <p className="text-muted-foreground inline-flex items-center gap-1.5 text-xs">
                  <GitBranch className="size-3.5" />
                  {feature.branch}
                </p>
              ) : (
                <p className="text-muted-foreground text-xs italic">No branch linked yet.</p>
              )}
            </div>
          </div>

          <section className="flex min-h-[calc(100vh-15.5rem)] flex-col rounded-lg border bg-card">
            <Tabs
              value={activeTab}
              onValueChange={(value) => setActiveTab(value as 'feature' | 'docs')}
              className="flex min-h-0 flex-1 flex-col"
            >
              <div className="border-b px-4 py-3">
                <div className="flex items-center justify-between gap-2">
                  <TabsList className="h-8 w-fit">
                    <TabsTrigger value="feature" className="h-6 px-3 text-xs">Feature</TabsTrigger>
                    <TabsTrigger value="docs" className="h-6 px-3 text-xs">Documentation</TabsTrigger>
                  </TabsList>
                  {activeTab === 'feature' && (
                    <Button
                      variant="outline"
                      size="sm"
                      className="border-primary/30 text-primary/80 hover:text-primary"
                      onClick={() => setEditing(true)}
                      title="Edit"
                      aria-label="Edit"
                    >
                      <Edit3 className="size-4" />
                      Edit
                    </Button>
                  )}
                </div>
              </div>

              <div className="min-h-0 flex-1 overflow-hidden p-3">
                <TabsContent value="feature" className="mt-0 h-full">
                  <div className="flex h-full flex-col gap-3">
                    <article className="ship-markdown-preview rounded-md bg-background px-4 py-3">
                      {documentModel.body.trim() ? (
                        <ReactMarkdown remarkPlugins={[remarkGfm]}>{documentModel.body}</ReactMarkdown>
                      ) : (
                        <p className="text-muted-foreground text-sm italic">
                          No body content yet.
                        </p>
                      )}
                    </article>

                    <div className="grid gap-2 md:grid-cols-2">
                      <FeatureChecklistSection
                        title="Delivery Todos"
                        items={todoItems}
                        emptyLabel="No delivery todos defined."
                        disabled={saving}
                        onToggleItem={(itemIndex) => {
                          void handleToggleChecklistItem('todos', itemIndex);
                        }}
                      />
                      <FeatureChecklistSection
                        title="Acceptance Criteria"
                        items={acceptanceItems}
                        emptyLabel="No acceptance criteria defined."
                        disabled={saving}
                        onToggleItem={(itemIndex) => {
                          void handleToggleChecklistItem('acceptance', itemIndex);
                        }}
                      />
                    </div>

                    <section className="space-y-2 rounded-md border bg-card px-3 py-2">
                      <div className="flex items-center justify-between">
                        <h4 className="text-sm font-medium">Delta</h4>
                        {deltaSignals.length > 0 ? (
                          <Badge variant="outline" className="h-5 px-1.5 text-[10px]">
                            {modelDelta ? `Drift ${modelDelta.drift_score}` : `${deltaSignals.length} Open`}
                          </Badge>
                        ) : (
                          <Badge variant="outline" className="h-5 px-1.5 text-[10px] text-emerald-700 border-emerald-300 bg-emerald-100/70">
                            Aligned
                          </Badge>
                        )}
                      </div>
                      {deltaSignals.length > 0 ? (
                        <ul className="space-y-1 text-xs text-amber-700">
                          {deltaSignals.map((signal, index) => (
                            <li key={`${signal}-${index}`} className="inline-flex items-start gap-1.5">
                              <AlertTriangle className="mt-0.5 size-3.5 shrink-0" />
                              <span>{signal}</span>
                            </li>
                          ))}
                        </ul>
                      ) : (
                        <p className="text-xs text-emerald-700 inline-flex items-center gap-1.5">
                          <CheckCircle2 className="size-3.5" />
                          Declaration and checklist status currently align.
                        </p>
                      )}
                      {modelDelta && modelDeltaActionableItems.length > 0 && (
                        <ul className="space-y-1 rounded-sm border bg-background px-2 py-1.5 text-xs text-muted-foreground">
                          {modelDeltaActionableItems.map((item, index) => (
                            <li key={`${item}-${index}`}>{item}</li>
                          ))}
                        </ul>
                      )}
                    </section>

                    {hasChecklistCoverage && readiness.blocking && (
                      <p className="inline-flex items-center gap-1.5 text-xs text-amber-600">
                        <AlertTriangle className="size-3.5" />
                        Acceptance criteria still have open items.
                      </p>
                    )}
                    {hasChecklistCoverage && !readiness.blocking && (
                      <p className="inline-flex items-center gap-1.5 text-xs text-emerald-600">
                        <CheckCircle2 className="size-3.5" />
                        Acceptance criteria currently satisfy readiness checks.
                      </p>
                      )}
                  </div>
                </TabsContent>

                <TabsContent value="docs" className="mt-0 h-full overflow-auto">
                  <div className="space-y-3">
                    <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border bg-background px-2.5 py-2">
                      <div className="text-muted-foreground flex flex-wrap items-center gap-3 text-xs">
                        <div className="flex items-center gap-1.5">
                          <span>Status:</span>
                          <Select
                            value={docsStatus}
                            onValueChange={(nextStatus) => {
                              if (!nextStatus) return;
                              setDocsStatus(nextStatus);
                              setDocsDirty(true);
                            }}
                            disabled={!editingDocs || docsSaving}
                          >
                            <SelectTrigger size="sm" className="h-7 min-w-[140px] text-xs capitalize">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {DOC_STATUS_OPTIONS.map((nextStatus) => (
                                <SelectItem key={nextStatus} value={nextStatus} className="capitalize">
                                  {nextStatus}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                        <span>
                          Revision: <span className="font-medium text-foreground">{feature.docs_revision ?? 0}</span>
                        </span>
                      </div>
                      {editingDocs ? (
                        <div className="inline-flex items-center gap-2">
                          <Button variant="outline" size="sm" onClick={cancelDocsEditing} disabled={docsSaving}>
                            <X className="size-4" />
                            Cancel
                          </Button>
                          <Button size="sm" onClick={() => void saveDocs()} disabled={!docsDirty || docsSaving}>
                            <Save className="size-4" />
                            {docsSaving ? 'Saving…' : 'Save'}
                          </Button>
                        </div>
                      ) : (
                        <Button size="sm" onClick={() => setEditingDocs(true)}>
                          <Edit3 className="size-4" />
                          Edit
                        </Button>
                      )}
                    </div>
                    {editingDocs ? (
                      <MarkdownEditor
                        value={docsContent}
                        onChange={(next) => {
                          setDocsContent(next);
                          setDocsDirty(true);
                        }}
                        mcpEnabled={mcpEnabled}
                        fillHeight
                        rows={24}
                        defaultMode="edit"
                      />
                    ) : (
                      <article className="ship-markdown-preview rounded-md bg-background px-4 py-3">
                        {docsPreview ? (
                          <ReactMarkdown remarkPlugins={[remarkGfm]}>{docsPreview}</ReactMarkdown>
                        ) : (
                          <p className="text-muted-foreground text-sm italic">
                            No feature documentation content yet.
                          </p>
                        )}
                      </article>
                    )}
                  </div>
                </TabsContent>
              </div>
            </Tabs>
          </section>
        </div>
      )}
    </div>
  );
}
