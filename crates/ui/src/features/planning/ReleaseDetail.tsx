import { useCallback, useEffect, useState } from 'react';
import { FeatureInfo, ReleaseDocument } from '@/bindings';
import { Target, ExternalLink } from 'lucide-react';
import DetailSheet from './DetailSheet';
import MarkdownEditor from '@/components/editor';
import ReleaseMetadataPanel from '@/components/editor/ReleaseMetadataPanel';
import { Button } from '@/components/ui/button';

interface ReleaseDetailProps {
  release: ReleaseDocument;
  features: FeatureInfo[];
  mcpEnabled?: boolean;
  onClose: () => void;
  onSelectFeature: (feature: FeatureInfo) => void;
  onSave: (fileName: string, content: string) => Promise<void> | void;
}

export default function ReleaseDetail({
  release,
  features,
  mcpEnabled = false,
  onClose,
  onSelectFeature,
  onSave,
}: ReleaseDetailProps) {
  const [content, setContent] = useState(release.content);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setContent(release.content);
    setDirty(false);
    setSaving(false);
  }, [release]);

  const saveRelease = useCallback(async () => {
    if (!dirty || saving) return;
    setSaving(true);
    try {
      await onSave(release.file_name, content);
      setDirty(false);
    } finally {
      setSaving(false);
    }
  }, [content, dirty, onSave, release.file_name, saving]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
        event.preventDefault();
        void saveRelease();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onClose, saveRelease]);

  return (
    <DetailSheet
      label="Release"
      title={<h2 className="text-xl font-semibold tracking-tight">{release.version}</h2>}
      meta={
        <p className="text-muted-foreground text-xs">
          {release.file_name} · {release.status}
        </p>
      }
      onClose={onClose}
      className="max-w-[1800px]"
      bodyScrollable={false}
      bodyClassName="overflow-hidden p-0"
      footerClassName="px-3 py-2 md:px-4 md:py-2.5"
      footer={
        <div className="flex flex-wrap items-center justify-end gap-2">
          <Button variant="outline" onClick={onClose}>
            Close
          </Button>
        </div>
      }
    >
      <div className="h-full min-h-0 p-2">
        <MarkdownEditor
          label={undefined}
          toolbarStart={
            <Button size="xs" className="h-7 px-2 text-xs" onClick={() => void saveRelease()} disabled={!dirty || saving}>
              {saving ? 'Saving…' : 'Save Release'}
            </Button>
          }
          value={content}
          onChange={(next) => {
            setContent(next);
            setDirty(true);
          }}
          frontmatterPanel={({ frontmatter, delimiter, onChange }) => (
            <ReleaseMetadataPanel
              frontmatter={frontmatter}
              delimiter={delimiter}
              defaultVersion={release.version}
              defaultStatus={release.status}
              onChange={onChange}
            />
          )}
          mcpEnabled={mcpEnabled}
          showStats={false}
          fillHeight
          rows={18}
          defaultMode="doc"
        />
      </div>

      {/* Associated Features Section */}
      <div className="border-t bg-muted/20 p-4">
        <div className="mb-3 flex items-center justify-between">
          <h3 className="flex items-center gap-2 text-sm font-semibold text-muted-foreground">
            <Target className="size-4 text-primary" />
            Planned Features
          </h3>
          <span className="text-muted-foreground text-xs uppercase tracking-wider">
            {features.filter(f => f.release_id === release.file_name).length} Features
          </span>
        </div>
        <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
          {features
            .filter((f) => f.release_id === release.file_name)
            .map((feature) => (
              <button
                key={feature.file_name}
                onClick={() => onSelectFeature(feature)}
                className="group flex flex-col items-start gap-1 rounded-md border bg-card p-3 text-left transition-colors hover:border-primary/50 hover:bg-accent/50"
              >
                <div className="flex w-full items-start justify-between gap-2">
                  <span className="truncate text-sm font-medium">{feature.title}</span>
                  <ExternalLink className="size-3 opacity-0 transition-opacity group-hover:opacity-100" />
                </div>
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <span className="capitalize">{feature.status}</span>
                </div>
              </button>
            ))}
          {features.filter((f) => f.release_id === release.file_name).length === 0 && (
            <div className="col-span-full py-6 text-center">
              <p className="text-muted-foreground text-sm">No features linked to this release yet.</p>
              <p className="text-muted-foreground text-xs mt-1">
                Edit a feature to associate it with {release.version}.
              </p>
            </div>
          )}
        </div>
      </div>
    </DetailSheet>
  );
}
