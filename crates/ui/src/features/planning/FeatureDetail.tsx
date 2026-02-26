import { useCallback, useEffect, useState } from 'react';
import { X } from 'lucide-react';
import { FeatureDocument } from '@/bindings';
import MarkdownEditor from '@/components/editor';
import { loadProjectTemplate } from '@/components/editor/templateLoader';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';

interface FeatureDetailProps {
  feature: FeatureDocument;
  mcpEnabled?: boolean;
  onClose: () => void;
  onSave: (fileName: string, content: string) => Promise<void> | void;
}

export default function FeatureDetail({
  feature,
  mcpEnabled = true,
  onClose,
  onSave,
}: FeatureDetailProps) {
  const [content, setContent] = useState(feature.content);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setContent(feature.content);
    setDirty(false);
    setSaving(false);
  }, [feature]);

  const saveFeature = useCallback(async () => {
    if (!dirty || saving) return;
    setSaving(true);
    try {
      await onSave(feature.file_name, content);
      setDirty(false);
    } finally {
      setSaving(false);
    }
  }, [content, dirty, feature.file_name, onSave, saving]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
        event.preventDefault();
        void saveFeature();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onClose, saveFeature]);

  return (
    <div
      className="fixed inset-0 z-50 bg-black/45 p-2 supports-backdrop-filter:backdrop-blur-xs md:p-4"
      onClick={onClose}
    >
      <section
        className="bg-background mx-auto flex h-full w-full max-w-[1600px] flex-col overflow-hidden rounded-2xl border shadow-2xl"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="border-b px-3 py-2 md:px-4 md:py-3">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="min-w-0">
              <h2 className="truncate text-lg font-semibold tracking-tight">{feature.title}</h2>
              <p className="text-muted-foreground truncate text-xs">{feature.file_name}</p>
              <div className="mt-1.5 flex flex-wrap items-center gap-2">
                <Badge variant="outline">{feature.status}</Badge>
                {feature.release && <Badge variant="secondary">{feature.release}</Badge>}
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Button variant="ghost" size="icon-sm" onClick={onClose} title="Close panel">
                <X className="size-4" />
              </Button>
              <Button onClick={() => void saveFeature()} disabled={!dirty || saving}>
                {saving ? 'Saving…' : dirty ? 'Save' : 'Saved'}
              </Button>
            </div>
          </div>
        </header>

        <div className="min-h-0 flex-1 p-2 md:p-3">
          <MarkdownEditor
            value={content}
            onChange={(next) => {
              setContent(next);
              setDirty(true);
            }}
            mcpEnabled={mcpEnabled}
            onMcpSample={() =>
              loadProjectTemplate('feature', {
                tomlValues: {
                  title: feature.title,
                  release: feature.release ?? '',
                },
              })
            }
            sampleLabel="Insert Template"
            sampleRequiresMcp={false}
            fillHeight
            defaultMode="doc"
          />
        </div>
      </section>
    </div>
  );
}
