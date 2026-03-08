import { useCallback, useEffect, useMemo, useState } from 'react';
import { FeatureDocument as FeatureEntry } from '@/bindings';
import {
  AlertTriangle,
  ArrowLeft,
  CheckCircle2,
  Edit3,
  FileText,
  GitBranch,
  Info,
  Save,
  X,
} from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import MarkdownEditor from '@/components/editor';
import { FeatureHeaderMetadata } from './FeatureHeaderMetadata';
import { FeatureChecklistSection } from './FeatureChecklistSection';
import { Button } from '@ship/ui';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@ship/ui';
import { Progress } from '@ship/ui';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@ship/ui';
import { Tooltip, TooltipContent, TooltipTrigger } from '@ship/ui';
import {
  readFrontmatterStringListField,
  splitFrontmatterDocument,
  setFrontmatterStringField,
  setFrontmatterStringListField,
} from '@ship/ui';
import { cn } from '@/lib/utils';

interface FeatureDetailProps {
  feature: FeatureEntry;
  releaseSuggestions?: string[];
  specSuggestions?: string[];
  tagSuggestions?: string[];
  mcpEnabled?: boolean;
  onClose: () => void;
  onSelectRelease: (fileName: string) => void;
  onSelectSpec: (fileName: string) => void;
  onSave: (fileName: string, content: string) => Promise<void> | void;
}

export default function FeatureDetail({
  feature,
  releaseSuggestions = [],
  specSuggestions = [],
  tagSuggestions = [],
  mcpEnabled = true,
  onClose,
  onSelectRelease,
  onSelectSpec,
  onSave,
}: FeatureDetailProps) {
  const [content, setContent] = useState(feature.content ?? '');
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [editing, setEditing] = useState(false);
  const [activeTab, setActiveTab] = useState<'feature' | 'docs'>('feature');

  useEffect(() => {
    setContent(feature.content ?? '');
    setDirty(false);
    setSaving(false);
    setEditing(false);
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
  const docsContent = useMemo(() => feature.docs_content?.trim() ?? '', [feature.docs_content]);

  const handleMetadataUpdate = useCallback((updates: {
    status?: string;
    release_id?: string;
    spec_id?: string;
    tags?: string[];
  }) => {
    let nextContent = content;
    const delimiter = documentModel.delimiter || '---';

    if (updates.status) {
      nextContent = setFrontmatterStringField(nextContent, 'status', updates.status, delimiter) || nextContent;
    }
    if (updates.release_id !== undefined) {
      nextContent = setFrontmatterStringField(nextContent, 'release_id', updates.release_id, delimiter) || nextContent;
    }
    if (updates.spec_id !== undefined) {
      nextContent = setFrontmatterStringField(nextContent, 'spec_id', updates.spec_id, delimiter) || nextContent;
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
      if (event.key === 'Escape' && editing) {
        event.preventDefault();
        cancelEditing();
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's' && editing) {
        event.preventDefault();
        void saveFeature();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [cancelEditing, editing, saveFeature]);

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
              specId={feature.spec_id || undefined}
              tags={tags}
              isEditing={editing}
              onUpdate={handleMetadataUpdate}
              releaseSuggestions={releaseSuggestions}
              specSuggestions={specSuggestions}
              tagSuggestions={tagSuggestions}
              onNavigate={(id, type) => {
                if (type === 'release') onSelectRelease(id);
                if (type === 'spec') onSelectSpec(id);
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
                  Docs: <span className="font-medium text-foreground">{feature.docs_status ?? 'not-started'}</span>
                </p>
                <p className="text-muted-foreground text-xs">
                  Revision: <span className="font-medium text-foreground">{feature.docs_revision ?? 0}</span>
                </p>
                {feature.docs_updated_at && (
                  <p className="text-muted-foreground text-xs">
                    Updated: <span className="font-medium text-foreground">{new Date(feature.docs_updated_at).toLocaleString()}</span>
                  </p>
                )}
                {!docsContent && (
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
                <div className="flex items-start justify-between gap-2">
                  <div>
                    <h3 className="text-sm font-semibold">
                      {activeTab === 'feature' ? 'Feature Document' : 'Feature Documentation'}
                    </h3>
                    <p className="text-muted-foreground inline-flex items-center gap-1.5 text-sm">
                      {activeTab === 'feature'
                        ? 'Read-first feature markdown with full-screen editing when needed.'
                        : 'Feature documentation linked to this feature.'}
                      {activeTab === 'docs' && (
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <button
                              type="button"
                              className="text-muted-foreground hover:text-foreground inline-flex"
                              aria-label="Documentation storage details"
                            >
                              <Info className="size-3.5" />
                            </button>
                          </TooltipTrigger>
                          <TooltipContent side="top">
                            Feature docs are stored in Ship state and tracked by revision.
                          </TooltipContent>
                        </Tooltip>
                      )}
                    </p>
                  </div>
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
                <TabsList className="h-8 w-fit">
                  <TabsTrigger value="feature" className="h-6 px-3 text-xs">Feature</TabsTrigger>
                  <TabsTrigger value="docs" className="h-6 px-3 text-xs">Documentation</TabsTrigger>
                </TabsList>
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
                      />
                      <FeatureChecklistSection
                        title="Acceptance Criteria"
                        items={acceptanceItems}
                        emptyLabel="No acceptance criteria defined."
                      />
                    </div>

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
                    <div className="text-muted-foreground flex flex-wrap gap-4 text-xs">
                      <span>Status: <span className="font-medium text-foreground">{feature.docs_status ?? 'not-started'}</span></span>
                      <span>Revision: <span className="font-medium text-foreground">{feature.docs_revision ?? 0}</span></span>
                    </div>
                    <article className="ship-markdown-preview rounded-md bg-background px-4 py-3">
                      {docsContent ? (
                        <ReactMarkdown remarkPlugins={[remarkGfm]}>{docsContent}</ReactMarkdown>
                      ) : (
                        <p className="text-muted-foreground text-sm italic">
                          No feature documentation content yet.
                        </p>
                      )}
                    </article>
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
