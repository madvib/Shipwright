import { FormEvent, useCallback, useEffect, useState } from 'react';
import { ADR } from '@/bindings';
import { generateAdrCmd } from '@/lib/platform/tauri/commands';
import AdrEditor from './AdrEditor';
import DetailSheet from './DetailSheet';
import { loadProjectTemplate } from '@/components/editor/templateLoader';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { deriveAdrHeaderTitle } from './adrTitle';

interface NewAdrModalProps {
  onClose: () => void;
  tagSuggestions: string[];
  specSuggestions: string[];
  adrSuggestions?: string[];
  onSubmit: (
    title: string,
    details: string,
    options?: {
      status?: string;
      date?: string;
      spec?: string | null;
      tags?: string[];
    }
  ) => void | Promise<void>;
}

function createInitialAdr(): ADR {
  return {
    metadata: {
      id: '',
      title: '',
      date: new Date().toISOString().slice(0, 10),
      tags: [],
      spec_id: null,
      status: 'proposed',
    } as any,
    body: '',
  };
}

export default function NewAdrModal({ onClose, onSubmit, tagSuggestions, specSuggestions, adrSuggestions }: NewAdrModalProps) {
  const [draft, setDraft] = useState<ADR>(() => createInitialAdr());
  const [error, setError] = useState<string | null>(null);
  const headerTitle = deriveAdrHeaderTitle(draft, 'New ADR');

  const submit = useCallback(async () => {
    const title = draft.metadata.title.trim();
    if (!title) {
      setError('Title is required.');
      return;
    }
    if (!draft.body.trim()) {
      setError('Details are required.');
      return;
    }
    const meta = draft.metadata as any;
    await onSubmit(title, draft.body.trim(), {
      status: meta.status,
      date: draft.metadata.date,
      spec: meta.spec_id?.trim() ? meta.spec_id.trim() : null,
      tags: Array.from(new Set((draft.metadata.tags ?? []).map((tag) => tag.trim()).filter(Boolean))),
    });
  }, [draft, onSubmit]);

  const insertTemplate = useCallback(async () => {
    const template = await loadProjectTemplate('adr', { bodyOnly: true });
    const snippet = template?.trim().replace(/^\s*#\s+Decision\s*\n+/i, '');
    if (!snippet) return;
    setDraft((current) => {
      const nextBody = current.body.trimEnd() ? `${current.body.trimEnd()}\n\n${snippet}` : snippet;
      return {
        ...current,
        body: nextBody,
      };
    });
    setError(null);
  }, []);

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    await submit();
  };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
        event.preventDefault();
        void submit();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onClose, submit]);

  return (
    <DetailSheet
      title={<h2 className="truncate text-lg font-semibold tracking-tight">{headerTitle}</h2>}
      meta={null}
      onClose={onClose}
      className="max-w-[1800px]"
      headerClassName="px-3 py-2.5 md:px-4 md:py-3"
      bodyScrollable={false}
      bodyClassName="overflow-hidden p-0"
      footerClassName="px-3 py-2 md:px-4 md:py-2.5"
      inlineHeader
      footer={
        <div className="flex flex-wrap items-center justify-end gap-2">
          <Button type="button" variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" form="new-adr-form">
            Create ADR
          </Button>
        </div>
      }
    >
      <form id="new-adr-form" onSubmit={handleSubmit} className="flex h-full min-h-0 flex-col gap-2.5 p-3">
        {error && (
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        <div className="min-h-0 flex-1">
          <AdrEditor
            adr={draft}
            onChange={(next) => {
              setDraft(next);
              setError(null);
            }}
            specSuggestions={specSuggestions}
            tagSuggestions={tagSuggestions}
            adrSuggestions={adrSuggestions}
            onInsertTemplate={insertTemplate}
            mcpEnabled={false}
            sampleLabel="Generate Draft"
            sampleRequiresMcp={false}
            onMcpSample={async () => {
              try {
                const title = draft.metadata.title.trim() || 'Untitled ADR';
                return await generateAdrCmd(title, draft.body.trim());
              } catch (err) {
                setError(String(err));
                return null;
              }
            }}
          />
        </div>
      </form>
    </DetailSheet>
  );
}
