import { useCallback, useEffect, useState } from 'react';
import { ReleaseDocument } from '@/bindings';
import DetailSheet from './DetailSheet';
import MarkdownEditor from '@/components/editor';
import { loadProjectTemplate } from '@/components/editor/templateLoader';
import { Button } from '@/components/ui/button';

interface ReleaseDetailProps {
  release: ReleaseDocument;
  mcpEnabled?: boolean;
  onClose: () => void;
  onSave: (fileName: string, content: string) => Promise<void> | void;
}

export default function ReleaseDetail({
  release,
  mcpEnabled = false,
  onClose,
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
          <Button onClick={() => void saveRelease()} disabled={!dirty || saving}>
            {saving ? 'Saving…' : 'Save Release'}
          </Button>
        </div>
      }
    >
      <div className="h-full min-h-0 p-2">
        <MarkdownEditor
          label={undefined}
          value={content}
          onChange={(next) => {
            setContent(next);
            setDirty(true);
          }}
          mcpEnabled={mcpEnabled}
          onMcpSample={() =>
            loadProjectTemplate('release', {
              tomlValues: {
                version: release.version,
              },
            })
          }
          sampleLabel="Insert Template"
          sampleRequiresMcp={false}
          sampleInline
          showStats={false}
          fillHeight
          rows={18}
          defaultMode="doc"
        />
      </div>
    </DetailSheet>
  );
}
