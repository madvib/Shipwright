import { useCallback, useEffect, useMemo, useState } from 'react';
import { FeatureInfo as FeatureEntry, ReleaseDocument as ReleaseEntry } from '@/bindings';
import {
  ArrowLeft,
  CheckCircle2,
  Edit3,
  ExternalLink,
  FileClock,
  Save,
  Target,
  X,
} from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import MarkdownEditor from '@/components/editor';
import { ReleaseHeaderMetadata } from './ReleaseHeaderMetadata';
import { readFrontmatterSummary, setFrontmatterStringField, setFrontmatterStringListField } from '@ship/ui';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@ship/ui';
import { Progress } from '@ship/ui';
import { splitFrontmatterDocument, stripAllFrontmatter } from '@ship/ui';
import { featureStatusFallbackReadiness, formatStatusLabel } from '@/features/planning/common/hub/utils/featureMetrics';
import { cn } from '@/lib/utils';

interface ReleaseDetailProps {
  release: ReleaseEntry;
  features: FeatureEntry[];
  mcpEnabled?: boolean;
  onClose: () => void;
  onSelectFeature: (feature: FeatureEntry) => void;
  onSave: (fileName: string, content: string) => Promise<void> | void;
}



function featureTileClasses(status: string) {
  switch (status) {
    case 'implemented':
      return {
        tile: 'border-status-green/35 bg-status-green/5',
        progress: 'bg-status-green',
      };
    case 'in-progress':
      return {
        tile: 'border-status-blue/35 bg-status-blue/5',
        progress: 'bg-status-blue',
      };
    case 'planned':
      return {
        tile: 'border-status-yellow/35 bg-status-yellow/5',
        progress: 'bg-status-yellow',
      };
    default:
      return {
        tile: 'border-muted-foreground/25 bg-muted/20',
        progress: 'bg-primary',
      };
  }
}

export default function ReleaseDetail({
  release,
  features,
  mcpEnabled = false,
  onClose,
  onSelectFeature,
  onSave,
}: ReleaseDetailProps) {
  const [content, setContent] = useState(release.content ?? '');
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [editing, setEditing] = useState(false);
  const [showAllLinkedFeatures, setShowAllLinkedFeatures] = useState(false);

  useEffect(() => {
    setContent(release.content ?? '');
    setDirty(false);
    setSaving(false);
    setEditing(false);
    setShowAllLinkedFeatures(false);
  }, [release]);

  const saveRelease = useCallback(async () => {
    if (!dirty || saving) return;
    setSaving(true);
    try {
      await onSave(release.file_name, content);
      setDirty(false);
      setEditing(false);
    } finally {
      setSaving(false);
    }
  }, [content, dirty, onSave, release.file_name, saving]);

  const cancelEditing = useCallback(() => {
    setContent(release.content ?? '');
    setDirty(false);
    setEditing(false);
  }, [release.content]);

  const linkedFeatures = useMemo(
    () =>
      features.filter(
        (feature) => feature.release_id === release.file_name || feature.release_id === release.version
      ),
    [features, release.file_name, release.version]
  );

  const linkedStatusSummary = useMemo(() => {
    const implemented = linkedFeatures.filter((entry) => entry.status === 'implemented').length;
    const inProgress = linkedFeatures.filter((entry) => entry.status === 'in-progress').length;
    const planned = linkedFeatures.filter((entry) => entry.status === 'planned').length;
    const averageReadiness =
      linkedFeatures.length === 0
        ? 0
        : Math.round(
          linkedFeatures.reduce(
            (sum, entry) => sum + featureStatusFallbackReadiness(entry.status),
            0
          ) / linkedFeatures.length
        );
    return {
      implemented,
      inProgress,
      planned,
      averageReadiness,
    };
  }, [linkedFeatures]);

  const visibleLinkedFeatures = useMemo(
    () => (showAllLinkedFeatures ? linkedFeatures : linkedFeatures.slice(0, 8)),
    [linkedFeatures, showAllLinkedFeatures]
  );

  const hiddenLinkedFeatureCount = Math.max(linkedFeatures.length - 8, 0);

  const documentModel = useMemo(() => splitFrontmatterDocument(content), [content]);
  const summary = useMemo(() => readFrontmatterSummary(release.content), [release.content]);

  const handleMetadataUpdate = useCallback((updates: {
    version?: string;
    status?: string;
    target_date?: string;
    tags?: string[];
  }) => {
    let nextContent = content;
    const delimiter = documentModel.delimiter || '+++';

    if (updates.version !== undefined) nextContent = setFrontmatterStringField(nextContent, 'version', updates.version, delimiter) ?? nextContent;
    if (updates.status !== undefined) nextContent = setFrontmatterStringField(nextContent, 'status', updates.status, delimiter) ?? nextContent;
    if (updates.target_date !== undefined) nextContent = setFrontmatterStringField(nextContent, 'target_date', updates.target_date, delimiter) ?? nextContent;
    if (updates.tags !== undefined) nextContent = setFrontmatterStringListField(nextContent, 'tags', updates.tags, delimiter) ?? nextContent;

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
        void saveRelease();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [cancelEditing, editing, saveRelease]);

  return (
    <div className="space-y-4">
      <Card size="sm" className="border-primary/20">
        <CardContent className="space-y-3 py-3">
          <div className="flex flex-col items-center gap-3">
            <div className="flex w-full items-center justify-between">
              <div className="flex-1">
                <Button variant="ghost" size="sm" className="h-7 px-2" onClick={onClose}>
                  <ArrowLeft className="size-4" />
                  Back To Hub
                </Button>
              </div>

              <h2 className="px-4 text-center text-xl font-bold tracking-tight text-foreground">
                {summary.version || release.version}
              </h2>

              <div className="flex flex-1 justify-end gap-2">
                {!editing ? (
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-8 gap-2 border-primary/30 text-primary/80 hover:text-primary"
                    onClick={() => setEditing(true)}
                  >
                    <Edit3 className="size-4" />
                    Edit Full Screen
                  </Button>
                ) : (
                  <>
                    <Button variant="outline" size="sm" className="h-8" onClick={cancelEditing} disabled={saving}>
                      <X className="size-4" />
                      Cancel
                    </Button>
                    <Button size="sm" className="h-8 shadow-sm" onClick={() => void saveRelease()} disabled={!dirty || saving}>
                      <Save className="size-4" />
                      {saving ? 'Saving…' : 'Save'}
                    </Button>
                  </>
                )}
              </div>
            </div>

            <ReleaseHeaderMetadata
              version={summary.version || release.version}
              status={summary.status || release.status}
              targetDate={summary.target_date}
              tags={summary.tags}
              isEditing={editing}
              onUpdate={handleMetadataUpdate}
            />
          </div>
        </CardContent>
      </Card>

      {editing ? (
        <Card size="sm" className="flex min-h-[calc(100vh-15.5rem)] flex-col">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Release Editor</CardTitle>
            <CardDescription>Editing {release.file_name}</CardDescription>
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
              showStats={false}
            />
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-3">
          <Card size="sm" className="flex min-h-[calc(100vh-20rem)] flex-col">
            <CardHeader className="border-b pb-3">
              <div className="flex items-start justify-between gap-2">
                <div>
                  <CardTitle className="text-sm">Release Document</CardTitle>
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
                {stripAllFrontmatter(documentModel.body).trim() ? (
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>{stripAllFrontmatter(documentModel.body)}</ReactMarkdown>
                ) : (
                  <p className="text-muted-foreground text-sm italic">No body content yet.</p>
                )}
              </article>
            </CardContent>
          </Card>

          <div className="grid min-h-0 gap-3 xl:grid-cols-[300px_minmax(0,1fr)]">
            <Card size="sm">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm">Release Summary</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2 text-xs">
                <p className="text-muted-foreground inline-flex items-center gap-1.5">
                  <Target className="size-3.5 text-primary" />
                  {linkedFeatures.length} linked features
                </p>
                <p className="text-muted-foreground inline-flex items-center gap-1.5">
                  <CheckCircle2 className="size-3.5 text-emerald-500" />
                  {linkedStatusSummary.implemented} implemented
                </p>
                <p className="text-muted-foreground inline-flex items-center gap-1.5">
                  <FileClock className="size-3.5 text-amber-500" />
                  {linkedStatusSummary.inProgress} in progress · {linkedStatusSummary.planned} planned
                </p>
                <p className="text-muted-foreground">
                  Avg readiness: {linkedStatusSummary.averageReadiness}%
                </p>
              </CardContent>
            </Card>

            <Card size="sm" className="flex min-h-[260px] flex-col">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm">Planned Features</CardTitle>
                <CardDescription className="text-xs">
                  Features mapped to this release
                </CardDescription>
              </CardHeader>
              <CardContent className="min-h-0 flex-1 space-y-2 overflow-auto">
                {linkedFeatures.length > 0 ? (
                  <>
                    <div className="grid gap-2 sm:grid-cols-2">
                      {visibleLinkedFeatures.map((feature) => {
                        const readiness = featureStatusFallbackReadiness(feature.status);
                        const tone = featureTileClasses(feature.status);

                        return (
                          <button
                            key={feature.file_name}
                            type="button"
                            onClick={() => onSelectFeature(feature)}
                            className={cn(
                              'hover:bg-muted/40 w-full rounded-md border px-2.5 py-2 text-left text-xs transition-colors',
                              tone.tile
                            )}
                            title={feature.file_name}
                          >
                            <div className="flex items-start justify-between gap-2">
                              <div className="min-w-0">
                                <p className="truncate font-medium">{feature.title}</p>
                                <p className="text-muted-foreground text-[11px]">
                                  {feature.file_name}
                                </p>
                              </div>
                              <ExternalLink className="size-3.5 shrink-0 text-muted-foreground" />
                            </div>

                            <div className="mt-2 space-y-1">
                              <div className="flex items-center justify-between">
                                <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
                                  {formatStatusLabel(feature.status)}
                                </Badge>
                                <span className="text-muted-foreground text-[11px]">{readiness}%</span>
                              </div>
                              <Progress value={readiness} indicatorClassName={tone.progress} />
                            </div>
                          </button>
                        );
                      })}
                    </div>
                    {hiddenLinkedFeatureCount > 0 && (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="mt-1 h-7 w-full text-xs"
                        onClick={() => setShowAllLinkedFeatures((current) => !current)}
                      >
                        {showAllLinkedFeatures
                          ? 'Show less'
                          : `Show ${hiddenLinkedFeatureCount} more`}
                      </Button>
                    )}
                  </>
                ) : (
                  <p className="text-muted-foreground py-8 text-center text-xs italic">
                    No features linked to this release yet.
                  </p>
                )}
              </CardContent>
            </Card>
          </div>
        </div>
      )}
    </div>
  );
}
