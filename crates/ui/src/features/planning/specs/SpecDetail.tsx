import { useCallback, useEffect, useMemo, useState } from 'react';
import { FeatureInfo } from '@/bindings';
import { SpecInfo } from '@/lib/types/spec';
import { Target, ExternalLink, Trash2 } from 'lucide-react';
import { SpecHeaderMetadata } from './SpecHeaderMetadata';
import {
  Button,
  Badge,
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
  DetailSheet,
  MarkdownEditor,
  readFrontmatterStringListField,
  splitFrontmatterDocument,
  setFrontmatterStringListField,
} from '@ship/ui';
import { useKeyboardShortcuts } from '@/lib/hooks/use-keyboard-shortcuts';

interface SpecDetailProps {
  spec: SpecInfo;
  features: FeatureInfo[];
  tagSuggestions?: string[];
  mcpEnabled?: boolean;
  onClose: () => void;
  onSelectFeature: (feature: FeatureInfo) => void;
  onSave: (fileName: string, content: string) => Promise<void> | void;
  onDelete: (fileName: string) => Promise<void> | void;
}

export default function SpecDetail({
  spec,
  features,
  tagSuggestions = [],
  mcpEnabled = false,
  onClose,
  onSelectFeature,
  onSave,
  onDelete,
}: SpecDetailProps) {
  const [content, setContent] = useState(spec.spec.body);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setContent(spec.spec.body);
    setDirty(false);
    setSaving(false);
  }, [spec]);

  const documentModel = useMemo(() => splitFrontmatterDocument(content), [content]);
  const tags = useMemo(
    () => readFrontmatterStringListField(documentModel.frontmatter, 'tags'),
    [documentModel.frontmatter]
  );

  const handleMetadataUpdate = useCallback((updates: {
    tags?: string[];
  }) => {
    let nextContent = content;
    const delimiter = documentModel.delimiter || '---';

    if (updates.tags) {
      nextContent = setFrontmatterStringListField(nextContent, 'tags', updates.tags, delimiter) || nextContent;
    }

    if (nextContent !== content) {
      setContent(nextContent);
      setDirty(true);
    }
  }, [content, documentModel.delimiter]);

  const saveSpec = useCallback(async () => {
    if (!dirty || saving) return;
    setSaving(true);
    try {
      await onSave(spec.file_name, content);
      setDirty(false);
    } finally {
      setSaving(false);
    }
  }, [content, dirty, onSave, saving, spec.file_name]);

  useKeyboardShortcuts({
    onEscape: onClose,
    onSave: saveSpec,
    disabled: !dirty,
  });

  const linkedFeatures = features.filter((f) => f.spec_id === spec.file_name);

  const actionButtons = (
    <>
      <Button size="xs" className="h-7 px-2 text-xs" onClick={() => void saveSpec()} disabled={!dirty || saving}>
        {saving ? 'Saving…' : 'Save Spec'}
      </Button>
      <AlertDialog>
        <AlertDialogTrigger
          render={
            <Button
              size="xs"
              variant="outline"
              className="h-7 border-destructive/40 px-2 text-destructive hover:bg-destructive/10"
              title="Delete Spec"
            />
          }
        >
          <Trash2 className="size-3.5" />
        </AlertDialogTrigger>
        <AlertDialogContent size="sm">
          <AlertDialogHeader>
            <AlertDialogTitle>Delete this spec?</AlertDialogTitle>
            <AlertDialogDescription>This will permanently remove the specification document.</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
            <AlertDialogAction size="sm" variant="destructive" onClick={() => void onDelete(spec.file_name)}>
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );

  return (
    <DetailSheet
      label="Spec"
      title={<h2 className="truncate text-lg font-semibold tracking-tight">{spec.spec.metadata.title}</h2>}
      meta={
        <SpecHeaderMetadata
          fileName={spec.file_name}
          tags={tags}
          isEditing={true}
          onUpdate={handleMetadataUpdate}
          tagSuggestions={tagSuggestions}
        />
      }
      onClose={onClose}
      className="max-w-[1800px]"
      bodyScrollable={false}
      bodyClassName="overflow-hidden p-0"
      inlineHeader
    >
      <div className="flex h-full min-h-0 flex-col">
        <div className="flex min-h-0 flex-1">
          {/* Editor — left */}
          <div className="min-w-0 flex-1 p-1.5">
            <MarkdownEditor
              key={spec.file_name || 'new'}
              label={undefined}
              toolbarStart={actionButtons}
              value={content}
              onChange={(next) => {
                setContent(next);
                setDirty(true);
              }}
              showStats={false}
              fillHeight
              rows={18}
              defaultMode="doc"
              mcpEnabled={mcpEnabled}
            />
          </div>

          {/* Features sidebar — right */}
          <aside className="flex w-[260px] shrink-0 flex-col overflow-y-auto border-l bg-muted/20">
            <div className="flex items-center gap-2 border-b bg-card/50 px-4 py-3">
              <Target className="size-3.5 text-primary" />
              <p className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">
                Implemented By
              </p>
              <Badge variant="outline" className="ml-auto text-[10px]">{linkedFeatures.length}</Badge>
            </div>
            <div className="flex flex-col gap-1.5 p-3">
              {linkedFeatures.length === 0 ? (
                <p className="py-4 text-center text-xs text-muted-foreground italic">No features linked yet.</p>
              ) : (
                linkedFeatures.map((feature) => (
                  <button
                    key={feature.file_name}
                    onClick={() => onSelectFeature(feature)}
                    className="group flex flex-col items-start gap-1 rounded-md border bg-card p-2.5 text-left transition-colors hover:border-primary/50 hover:bg-accent/50"
                  >
                    <div className="flex w-full items-start justify-between gap-2">
                      <span className="truncate text-xs font-medium">{feature.title}</span>
                      <ExternalLink className="size-3 shrink-0 opacity-0 transition-opacity group-hover:opacity-100" />
                    </div>
                    <span className="text-[10px] capitalize text-muted-foreground">{feature.status}</span>
                  </button>
                ))
              )}
            </div>
          </aside>
        </div>
      </div>
    </DetailSheet>
  );
}
