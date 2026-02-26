import { FormEvent, useCallback, useEffect, useState } from 'react';
import { generateAdrCmd } from '@/lib/platform/tauri/commands';
import DetailSheet from './DetailSheet';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import MarkdownEditor from '@/components/editor';

interface NewAdrModalProps {
  onClose: () => void;
  onSubmit: (title: string, decision: string) => void | Promise<void>;
}

export default function NewAdrModal({ onClose, onSubmit }: NewAdrModalProps) {
  const [title, setTitle] = useState('');
  const [decision, setDecision] = useState('');
  const [error, setError] = useState<string | null>(null);

  const submit = useCallback(async () => {
    if (!title.trim()) {
      setError('Title is required.');
      return;
    }
    if (!decision.trim()) {
      setError('Decision text is required.');
      return;
    }
    await onSubmit(title.trim(), decision.trim());
  }, [decision, onSubmit, title]);

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
      label="New ADR"
      title={<h2 className="text-xl font-semibold tracking-tight">Record Decision</h2>}
      meta={
        <p className="text-muted-foreground text-xs">
          Capture the decision and the trade-offs behind it.
        </p>
      }
      onClose={onClose}
      className="max-w-[1400px]"
      footer={
        <div className="flex flex-wrap items-center justify-end gap-2">
          <span className="text-muted-foreground mr-auto text-xs">Cmd/Ctrl+Enter to record</span>
          <Button type="button" variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" form="new-adr-form">
            Record Decision
          </Button>
        </div>
      }
    >
      <form id="new-adr-form" onSubmit={handleSubmit} className="space-y-4">
        {error && (
          <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {error}
          </div>
        )}

        <div className="space-y-2">
          <Label htmlFor="adr-title">
            Decision Title <span className="text-destructive">*</span>
          </Label>
          <Input
            id="adr-title"
            autoFocus
            value={title}
            placeholder="Use PostgreSQL for persistence"
            onChange={(event) => {
              setTitle(event.target.value);
              setError(null);
            }}
          />
        </div>

        <MarkdownEditor
          label="Decision Details *"
          value={decision}
          onChange={(next) => {
            setDecision(next);
            setError(null);
          }}
          placeholder="Why this decision? What are the trade-offs?"
          rows={22}
          defaultMode="doc"
          mcpEnabled={!!title.trim()}
          sampleLabel="Generate Draft"
          onMcpSample={async () => {
            try {
              return await generateAdrCmd(title.trim(), '');
            } catch (err) {
              setError(String(err));
              return null;
            }
          }}
        />
      </form>
    </DetailSheet>
  );
}
