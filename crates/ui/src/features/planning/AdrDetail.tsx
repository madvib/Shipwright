import { useEffect, useState } from 'react';
import { ADR, AdrEntry } from '@/bindings';
import DetailSheet from './DetailSheet';
import MarkdownEditor from '@/components/editor';
import { loadProjectTemplate } from '@/components/editor/templateLoader';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

interface AdrDetailProps {
  entry: AdrEntry;
  mcpEnabled?: boolean;
  onClose: () => void;
  onSave: (fileName: string, adr: ADR) => void;
  onDelete: (fileName: string) => void;
}

const ADR_STATUSES = ['proposed', 'accepted', 'rejected', 'superseded', 'deprecated'];

function normalizeAdr(adr: ADR): ADR {
  return {
    ...adr,
    metadata: {
      ...adr.metadata,
      tags: adr.metadata.tags ?? [],
    },
  };
}

export default function AdrDetail({
  entry,
  mcpEnabled = false,
  onClose,
  onSave,
  onDelete,
}: AdrDetailProps) {
  const [draft, setDraft] = useState<ADR>(normalizeAdr(entry.adr));
  const [dirty, setDirty] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [tagInput, setTagInput] = useState('');

  useEffect(() => {
    setDraft(normalizeAdr(entry.adr));
    setDirty(false);
    setTagInput('');
    setConfirmDelete(false);
  }, [entry]);

  const saveAdr = () => {
    onSave(entry.file_name, draft);
    setDirty(false);
  };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
        event.preventDefault();
        saveAdr();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [draft, onClose]);

  const addTag = () => {
    const cleaned = tagInput.trim();
    if (!cleaned || (draft.metadata.tags ?? []).includes(cleaned)) return;
    setDraft((current) => ({
      ...current,
      metadata: {
        ...current.metadata,
        tags: [...(current.metadata.tags ?? []), cleaned],
      },
    }));
    setTagInput('');
    setDirty(true);
  };

  const removeTag = (tag: string) => {
    setDraft((current) => ({
      ...current,
      metadata: {
        ...current.metadata,
        tags: (current.metadata.tags ?? []).filter((value) => value !== tag),
      },
    }));
    setDirty(true);
  };

  return (
    <DetailSheet
      label="ADR"
      title={
        <Input
          value={draft.metadata.title}
          onChange={(event) => {
            const title = event.target.value;
            setDraft((current) => ({
              ...current,
              metadata: { ...current.metadata, title },
            }));
            setDirty(true);
          }}
          className="h-11 text-base font-semibold"
        />
      }
      meta={<p className="text-muted-foreground text-xs">{entry.file_name}</p>}
      onClose={onClose}
      footer={
        <div className="flex flex-wrap items-center justify-end gap-2">
          <Button variant="outline" onClick={onClose}>
            Close
          </Button>
          <Button onClick={saveAdr} disabled={!dirty}>
            Save Decision
          </Button>
          {!confirmDelete ? (
            <Button variant="destructive" onClick={() => setConfirmDelete(true)}>
              Delete
            </Button>
          ) : (
            <Card size="sm" className="w-full border-destructive/30 md:w-auto">
              <CardContent className="flex items-center gap-2 py-2">
                <span className="text-sm">Delete this ADR?</span>
                <Button variant="destructive" size="xs" onClick={() => onDelete(entry.file_name)}>
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
      <div className="grid gap-3 md:grid-cols-2">
        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Status</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-wrap gap-2">
            {ADR_STATUSES.map((status) => {
              const active = draft.metadata.status === status;
              return (
                <Button
                  key={status}
                  variant={active ? 'secondary' : 'outline'}
                  size="sm"
                  onClick={() => {
                    if (status === draft.metadata.status) return;
                    setDraft((current) => ({
                      ...current,
                      metadata: { ...current.metadata, status },
                    }));
                    setDirty(true);
                  }}
                >
                  {status}
                </Button>
              );
            })}
          </CardContent>
        </Card>

        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Metadata</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="space-y-1">
              <Label htmlFor="adr-date">Date</Label>
              <Input
                id="adr-date"
                value={draft.metadata.date}
                onChange={(event) => {
                  const date = event.target.value;
                  setDraft((current) => ({
                    ...current,
                    metadata: { ...current.metadata, date },
                  }));
                  setDirty(true);
                }}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="adr-spec">Spec Reference</Label>
              <Input
                id="adr-spec"
                value={draft.metadata.spec ?? ''}
                placeholder="alpha-spec.md"
                onChange={(event) => {
                  const spec = event.target.value;
                  setDraft((current) => ({
                    ...current,
                    metadata: {
                      ...current.metadata,
                      spec: spec.trim() ? spec : null,
                    },
                  }));
                  setDirty(true);
                }}
              />
            </div>
          </CardContent>
        </Card>
      </div>

      <Card size="sm" className="mt-3">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">Tags</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex flex-wrap gap-2">
            {(draft.metadata.tags ?? []).length === 0 ? (
              <span className="text-muted-foreground text-xs">No tags yet.</span>
            ) : (
              (draft.metadata.tags ?? []).map((tag) => (
                <Badge key={tag} variant="secondary" className="gap-1">
                  {tag}
                  <button
                    type="button"
                    className="text-muted-foreground hover:text-foreground"
                    onClick={() => removeTag(tag)}
                  >
                    ✕
                  </button>
                </Badge>
              ))
            )}
          </div>
          <div className="flex gap-2">
            <Input
              value={tagInput}
              placeholder="Add a tag"
              onChange={(event) => setTagInput(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter' || event.key === ',') {
                  event.preventDefault();
                  addTag();
                }
              }}
            />
            <Button variant="outline" onClick={addTag}>
              Add
            </Button>
          </div>
        </CardContent>
      </Card>

      <div className="mt-3">
        <MarkdownEditor
          label="Decision Body"
          value={draft.body}
          onChange={(body) => {
            setDraft((current) => ({ ...current, body }));
            setDirty(true);
          }}
          placeholder="Describe this decision..."
          rows={14}
          defaultMode="doc"
          mcpEnabled={mcpEnabled}
          sampleLabel="Insert Template"
          sampleRequiresMcp={false}
          onMcpSample={() => loadProjectTemplate('adr', { bodyOnly: true })}
        />
      </div>
    </DetailSheet>
  );
}
