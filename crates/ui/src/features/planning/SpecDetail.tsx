import { useCallback, useEffect, useState } from 'react';
import { SpecDocument } from '@/bindings';
import DetailSheet from './DetailSheet';
import MarkdownEditor from '@/components/editor';
import { loadProjectTemplate } from '@/components/editor/templateLoader';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';

interface SpecDetailProps {
  spec: SpecDocument;
  mcpEnabled?: boolean;
  onClose: () => void;
  onSave: (fileName: string, content: string) => Promise<void> | void;
  onDelete: (fileName: string) => Promise<void> | void;
}

export default function SpecDetail({ spec, mcpEnabled = false, onClose, onSave, onDelete }: SpecDetailProps) {
  const [content, setContent] = useState(spec.content);
  const [dirty, setDirty] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setContent(spec.content);
    setDirty(false);
    setConfirmDelete(false);
    setSaving(false);
  }, [spec]);

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

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
        event.preventDefault();
        void saveSpec();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onClose, saveSpec]);

  return (
    <DetailSheet
      label="Spec"
      title={<h2 className="text-xl font-semibold tracking-tight">{spec.title}</h2>}
      meta={<p className="text-muted-foreground text-xs">{spec.file_name}</p>}
      onClose={onClose}
      footer={
        <div className="flex flex-wrap items-center justify-end gap-2">
          <Button variant="outline" onClick={onClose}>
            Close
          </Button>
          <Button onClick={() => void saveSpec()} disabled={!dirty || saving}>
            {saving ? 'Saving…' : 'Save Spec'}
          </Button>
          {!confirmDelete ? (
            <Button variant="destructive" onClick={() => setConfirmDelete(true)}>
              Delete
            </Button>
          ) : (
            <Card size="sm" className="w-full border-destructive/30 md:w-auto">
              <CardContent className="flex items-center gap-2 py-2">
                <span className="text-sm">Delete this spec?</span>
                <Button variant="destructive" size="xs" onClick={() => void onDelete(spec.file_name)}>
                  Yes
                </Button>
                <Button variant="outline" size="xs" onClick={() => setConfirmDelete(false)}>
                  Cancel
                </Button>
              </CardContent>
            </Card>
          )}
        </div>
      }
    >
      <MarkdownEditor
        label="Spec Content"
        value={content}
        onChange={(next) => {
          setContent(next);
          setDirty(true);
        }}
        mcpEnabled={mcpEnabled}
        sampleLabel="Insert Template"
        sampleRequiresMcp={false}
        onMcpSample={() =>
          loadProjectTemplate('spec', {
            tomlValues: {
              title: spec.title,
            },
          })
        }
        rows={18}
        defaultMode="doc"
      />
    </DetailSheet>
  );
}
