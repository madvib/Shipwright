import { FormEvent, useCallback, useEffect, useState } from 'react';
import { ADR, AdrStatus } from '@/bindings';
import AdrEditor from './AdrEditor';
import { loadProjectTemplate } from '@/components/editor/templateLoader';
import { Alert, AlertDescription, Button, DetailSheet, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/primitives';
import { deriveAdrHeaderTitle } from './adrTitle';

interface NewAdrModalProps {
  onClose: () => void;
  tagSuggestions: string[];
  adrSuggestions?: { id: string; title: string }[];
  onSubmit: (
    title: string,
    context: string,
    decision: string,
    options?: {
      status?: string;
      date?: string;
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
      supersedes_id: null,
    },
    context: '',
    decision: '',
  };
}

const ADR_STATUSES: AdrStatus[] = [
  'proposed',
  'accepted',
  'rejected',
  'superseded',
  'deprecated',
];

export default function NewAdrModal({ onClose, onSubmit, tagSuggestions, adrSuggestions }: NewAdrModalProps) {
  const [draft, setDraft] = useState<ADR>(() => createInitialAdr());
  const [status, setStatus] = useState<AdrStatus>('proposed');
  const [error, setError] = useState<string | null>(null);
  const headerTitle = deriveAdrHeaderTitle(draft, 'New ADR');

  const submit = useCallback(async () => {
    const title = draft.metadata.title.trim();
    if (!title) {
      setError('Title is required.');
      return;
    }
    if (!draft.decision.trim()) {
      setError('Decision is required.');
      return;
    }
    await onSubmit(title, draft.context.trim(), draft.decision.trim(), {
      status,
      date: draft.metadata.date,
      tags: Array.from(new Set((draft.metadata.tags ?? []).map((tag) => tag.trim()).filter(Boolean))),
    });
  }, [draft, onSubmit, status]);

  const insertTemplate = useCallback(async () => {
    const template = await loadProjectTemplate('adr', { bodyOnly: true });
    const snippet = template?.trim().replace(/^\s*#\s+Decision\s*\n+/i, '');
    if (!snippet) return;
    setDraft((current) => {
      const nextDecision = current.decision.trimEnd()
        ? `${current.decision.trimEnd()}\n\n${snippet}`
        : snippet;
      return {
        ...current,
        decision: nextDecision,
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
          <div className="mr-auto flex items-center gap-2">
            <span className="text-muted-foreground text-xs uppercase tracking-wide">Status</span>
            <Select value={status} onValueChange={(next) => setStatus(next as AdrStatus)}>
              <SelectTrigger size="sm" className="h-8 w-36 capitalize">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {ADR_STATUSES.map((option) => (
                  <SelectItem key={option} value={option} className="capitalize">
                    {option}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
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
            tagSuggestions={tagSuggestions}
            adrSuggestions={adrSuggestions}
            onInsertTemplate={insertTemplate}
          />
        </div>
      </form>
    </DetailSheet>
  );
}
