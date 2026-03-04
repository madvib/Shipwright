import { FormEvent, useCallback, useEffect, useState } from 'react';
import { StatusConfig } from '@/bindings';
import { generateIssueDescriptionCmd } from '@/lib/platform/tauri/commands';
import DetailSheet from './DetailSheet';
import { Alert, AlertDescription } from '@ship/ui';
import { Button } from '@ship/ui';
import { Input } from '@ship/ui';
import AutocompleteInput from '@ship/ui';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@ship/ui';
import MarkdownEditor from '@/components/editor';

interface NewIssueModalProps {
  onClose: () => void;
  statuses: StatusConfig[];
  tagSuggestions: string[];
  specSuggestions: string[];
  onSubmit: (
    title: string,
    description: string,
    status: string,
    options?: {
      assignee?: string | null;
      tags?: string[];
      spec?: string | null;
    }
  ) => void | Promise<void>;
  defaultStatus?: string;
}

export default function NewIssueModal({
  onClose,
  statuses,
  tagSuggestions,
  specSuggestions,
  onSubmit,
  defaultStatus,
}: NewIssueModalProps) {
  const initialStatus = defaultStatus ?? statuses[0]?.id ?? 'backlog';
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [status, setStatus] = useState<string>(initialStatus);
  const [assignee, setAssignee] = useState('');
  const [spec, setSpec] = useState('');
  const [tagInput, setTagInput] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const submit = useCallback(async () => {
    if (!title.trim()) {
      setError('Title is required.');
      return;
    }
    await onSubmit(title.trim(), description.trim(), status, {
      assignee: assignee.trim() ? assignee.trim() : null,
      spec: spec.trim() ? spec.trim() : null,
      tags,
    });
  }, [assignee, description, onSubmit, spec, status, tags, title]);

  const addTag = (value: string) => {
    const clean = value.trim();
    if (!clean || tags.includes(clean)) return;
    setTags((current) => [...current, clean]);
    setTagInput('');
  };

  const removeTag = (value: string) => {
    setTags((current) => current.filter((tag) => tag !== value));
  };

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
      bodyScrollable={false}
      bodyClassName="overflow-hidden p-0"
      footerClassName="px-3 py-2 md:px-4 md:py-2.5"
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
      <form id="new-issue-form" onSubmit={handleSubmit} className="flex h-full min-h-0 flex-col gap-2 p-3">
        {error && (
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        <div className="grid gap-2 md:grid-cols-[1fr_220px]">
          <Input
            id="issue-title"
            autoFocus
            value={title}
            className="h-8"
            placeholder="Title *"
            onChange={(event) => {
              setTitle(event.target.value);
              setError(null);
            }}
          />
          <Select value={status} onValueChange={(value) => value && setStatus(value)}>
            <SelectTrigger className="h-8 w-full">
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
        <div className="grid gap-2 md:grid-cols-[220px_1fr]">
          <Input
            id="issue-assignee"
            value={assignee}
            className="h-8"
            placeholder="Assignee"
            onChange={(event) => {
              setAssignee(event.target.value);
              setError(null);
            }}
          />
          <AutocompleteInput
            id="issue-spec"
            value={spec}
            options={specSuggestions.map((value) => ({ value }))}
            className="h-8"
            placeholder="Spec"
            noResultsText="No specs found."
            onValueChange={(value) => {
              setSpec(value);
              setError(null);
            }}
          />
        </div>
        <div className="flex flex-wrap items-center gap-1.5">
          {tags.map((tag) => (
            <Button key={tag} type="button" variant="outline" size="xs" className="h-7 px-2 text-xs" onClick={() => removeTag(tag)}>
              {tag} ×
            </Button>
          ))}
          <AutocompleteInput
            value={tagInput}
            options={tagSuggestions
              .filter((tag) => !tags.includes(tag))
              .map((value) => ({ value }))}
            className="h-8 w-[220px] text-xs"
            placeholder="Add tag"
            noResultsText="No tag suggestions."
            onCommit={(value) => addTag(value)}
            onValueChange={(value) => setTagInput(value)}
          />
        </div>

        <div className="min-h-0 flex-1">
          <MarkdownEditor
            label={undefined}
            value={description}
            onChange={setDescription}
            placeholder="Steps to reproduce, context, links..."
            rows={22}
            defaultMode="doc"
            fillHeight
            showStats={false}
            sampleInline
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
        </div>
      </form>
    </DetailSheet>
  );
}
