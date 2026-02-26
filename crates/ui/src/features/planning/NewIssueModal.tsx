import { FormEvent, useCallback, useEffect, useState } from 'react';
import { StatusConfig } from '@/bindings';
import { generateIssueDescriptionCmd } from '@/lib/platform/tauri/commands';
import DetailSheet from './DetailSheet';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import MarkdownEditor from '@/components/editor';

interface NewIssueModalProps {
  onClose: () => void;
  statuses: StatusConfig[];
  onSubmit: (title: string, description: string, status: string) => void | Promise<void>;
  defaultStatus?: string;
}

export default function NewIssueModal({ onClose, statuses, onSubmit, defaultStatus }: NewIssueModalProps) {
  const initialStatus = defaultStatus ?? statuses[0]?.id ?? 'backlog';
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [status, setStatus] = useState<string>(initialStatus);
  const [error, setError] = useState<string | null>(null);

  const submit = useCallback(async () => {
    if (!title.trim()) {
      setError('Title is required.');
      return;
    }
    await onSubmit(title.trim(), description.trim(), status);
  }, [description, onSubmit, status, title]);

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
      label="New Issue"
      title={<h2 className="text-xl font-semibold tracking-tight">Create New Issue</h2>}
      meta={
        <p className="text-muted-foreground text-xs">
          Capture context and pick an initial workflow status.
        </p>
      }
      onClose={onClose}
      className="max-w-[1400px]"
      footer={
        <div className="flex flex-wrap items-center justify-end gap-2">
          <span className="text-muted-foreground mr-auto text-xs">Cmd/Ctrl+Enter to create</span>
          <Button type="button" variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" form="new-issue-form">
            Create Issue
          </Button>
        </div>
      }
    >
      <form id="new-issue-form" onSubmit={handleSubmit} className="space-y-4">
        {error && (
          <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {error}
          </div>
        )}

        <div className="grid gap-4 md:grid-cols-[1fr_220px]">
          <div className="space-y-2">
            <Label htmlFor="issue-title">
              Title <span className="text-destructive">*</span>
            </Label>
            <Input
              id="issue-title"
              autoFocus
              value={title}
              placeholder="Short, descriptive title"
              onChange={(event) => {
                setTitle(event.target.value);
                setError(null);
              }}
            />
          </div>
          <div className="space-y-2">
            <Label>Initial Status</Label>
            <Select value={status} onValueChange={(value) => value && setStatus(value)}>
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {statuses.map((entry) => (
                  <SelectItem key={entry.id} value={entry.id}>
                    {entry.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <MarkdownEditor
          label="Description"
          value={description}
          onChange={setDescription}
          placeholder="Steps to reproduce, context, links..."
          rows={22}
          defaultMode="doc"
          mcpEnabled={!!title.trim()}
          sampleLabel="Generate Draft"
          onMcpSample={async () => {
            try {
              return await generateIssueDescriptionCmd(title.trim());
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
