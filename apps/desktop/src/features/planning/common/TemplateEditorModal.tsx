import { FormEvent, useCallback, useEffect, useMemo, useState } from 'react';
import MarkdownEditor from '@/components/editor';
import { Alert, AlertDescription, Button, DetailSheet } from '@ship/primitives';
import { getTemplateCmd, saveTemplateCmd, TemplateKind } from '@/lib/platform/tauri/commands';

interface TemplateEditorModalProps {
  kind: TemplateKind;
  title?: string;
  onClose: () => void;
}

function titleForKind(kind: TemplateKind): string {
  switch (kind) {
    case 'adr':
      return 'ADR Template';
    case 'feature':
      return 'Feature Template';
    case 'vision':
      return 'Vision Template';
    default:
      return 'Template';
  }
}

export default function TemplateEditorModal({ kind, title, onClose }: TemplateEditorModalProps) {
  const [content, setContent] = useState('');
  const [initialContent, setInitialContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const resolvedTitle = useMemo(() => title ?? titleForKind(kind), [kind, title]);
  const dirty = content !== initialContent;

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      try {
        setLoading(true);
        setError(null);
        const template = await getTemplateCmd(kind);
        if (cancelled) return;
        setContent(template);
        setInitialContent(template);
      } catch (err) {
        if (!cancelled) {
          setError(String(err));
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    };
    void load();
    return () => {
      cancelled = true;
    };
  }, [kind]);

  const save = useCallback(async () => {
    try {
      setSaving(true);
      setError(null);
      await saveTemplateCmd(kind, content);
      setInitialContent(content);
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }, [content, kind, onClose]);

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    await save();
  };

  return (
    <DetailSheet
      title={<h2 className="truncate text-lg font-semibold tracking-tight">{resolvedTitle}</h2>}
      meta={null}
      onClose={onClose}
      className="max-w-[1800px]"
      bodyScrollable={false}
      bodyClassName="overflow-hidden p-0"
      inlineHeader
      footer={
        <div className="flex items-center justify-end gap-2">
          <Button type="button" variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" form="template-editor-form" disabled={!dirty || saving || loading}>
            {saving ? 'Saving…' : 'Save Template'}
          </Button>
        </div>
      }
    >
      <form id="template-editor-form" onSubmit={handleSubmit} className="flex h-full min-h-0 flex-col gap-2 p-2">
        {error && (
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {loading ? (
          <div className="text-muted-foreground flex min-h-0 flex-1 items-center justify-center text-sm">
            Loading template...
          </div>
        ) : (
          <div className="min-h-0 flex-1">
            <MarkdownEditor
              value={content}
              onChange={setContent}
              showStats={false}
              showFrontmatter={false}
              defaultMode="doc"
              fillHeight
            />
          </div>
        )}
      </form>
    </DetailSheet>
  );
}
