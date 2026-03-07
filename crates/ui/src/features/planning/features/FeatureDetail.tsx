import { useCallback, useEffect, useMemo, useState } from 'react';
import { FeatureDocument as FeatureEntry } from '@/bindings';
import {
  AlertTriangle,
  ArrowLeft,
  CheckCircle2,
  Edit3,
  GitBranch,
  Save,
  X,
} from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import MarkdownEditor from '@/components/editor';
import { FeatureHeaderMetadata } from './FeatureHeaderMetadata';
import { Button } from '@ship/ui';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@ship/ui';
import { Progress } from '@ship/ui';
import {
  readFrontmatterStringListField,
  splitFrontmatterDocument,
  setFrontmatterStringField,
  setFrontmatterStringListField,
} from '@ship/ui';
import {
  deriveFeatureChecklistMetrics,
} from '@/features/planning/common/hub/utils/featureMetrics';
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

  useEffect(() => {
    setContent(feature.content ?? '');
    setDirty(false);
    setSaving(false);
    setEditing(false);
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
  const readiness = useMemo(
    () => deriveFeatureChecklistMetrics(content, feature.status),
    [content, feature.status]
  );
  const tags = useMemo(
    () => readFrontmatterStringListField(documentModel.frontmatter, 'tags'),
    [documentModel.frontmatter]
  );

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
        <div className="grid min-h-0 gap-3 xl:grid-cols-[320px_minmax(0,1fr)]">
          <aside className="space-y-3">
            <Card size="sm">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm">Delivery Readiness</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-muted-foreground">Readiness</span>
                  <span className="font-semibold">{readiness.readinessPercent}%</span>
                </div>
                <Progress
                  value={readiness.readinessPercent}
                  indicatorClassName={cn(readiness.blocking ? 'bg-amber-500' : 'bg-emerald-500')}
                />
                <div className="text-muted-foreground space-y-1 text-xs">
                  <p>
                    Todos: {readiness.todos.done}/{readiness.todos.total}
                  </p>
                  <p>
                    Acceptance: {readiness.acceptance.done}/{readiness.acceptance.total}
                  </p>
                </div>
              </CardContent>
            </Card>

            {feature.branch && (
              <Card size="sm">
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm">VCS Context</CardTitle>
                </CardHeader>
                <CardContent>
                  <p className="text-muted-foreground inline-flex items-center gap-1.5 text-xs">
                    <GitBranch className="size-3.5" />
                    {feature?.branch}
                  </p>
                </CardContent>
              </Card>
            )}
          </aside>

          <Card size="sm" className="flex min-h-[calc(100vh-15.5rem)] flex-col">
            <CardHeader className="border-b pb-3">
              <div className="flex items-start justify-between gap-2">
                <div>
                  <CardTitle className="text-sm">Feature Document</CardTitle>
                  <CardDescription>
                    Read-first view. Use Edit Full Screen when you need to update markdown or metadata.
                  </CardDescription>
                </div>
                <Button
                  variant="outline"
                  size="icon-sm"
                  className="border-primary/30 text-primary/80 hover:text-primary"
                  onClick={() => setEditing(true)}
                  title="Edit Full Screen"
                  aria-label="Edit Full Screen"
                >
                  <Edit3 className="size-4" />
                </Button>
              </div>
            </CardHeader>
            <CardContent className="min-h-0 flex-1 overflow-auto p-3">
              <article className="ship-markdown-preview rounded-md border bg-background px-4 py-3">
                {documentModel.body.trim() ? (
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>{documentModel.body}</ReactMarkdown>
                ) : (
                  <p className="text-muted-foreground text-sm italic">
                    No body content yet.
                  </p>
                )}
              </article>
              {readiness.blocking && (
                <p className="mt-3 inline-flex items-center gap-1.5 text-xs text-amber-600">
                  <AlertTriangle className="size-3.5" />
                  Acceptance criteria still have open items.
                </p>
              )}
              {!readiness.blocking && (
                <p className="mt-3 inline-flex items-center gap-1.5 text-xs text-emerald-600">
                  <CheckCircle2 className="size-3.5" />
                  Acceptance criteria currently satisfy readiness checks.
                </p>
              )}
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}
