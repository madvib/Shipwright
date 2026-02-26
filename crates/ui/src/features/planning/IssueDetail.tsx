import { useEffect, useMemo, useState } from 'react';
import { Issue, IssueEntry, IssueLink, StatusConfig } from '@/bindings';
import DetailSheet from './DetailSheet';
import MarkdownEditor from '@/components/editor';
import { loadProjectTemplate } from '@/components/editor/templateLoader';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { getStatusStyles } from '@/lib/workspace-ui';

interface IssueDetailProps {
  entry: IssueEntry;
  statuses: StatusConfig[];
  tagSuggestions: string[];
  mcpEnabled?: boolean;
  onClose: () => void;
  onStatusChange: (file_name: string, from: string, to: string) => void;
  onDelete: (path: string) => void;
  onSave: (path: string, issue: Issue) => void;
}

type LinkType = 'blocks' | 'blocked-by' | 'relates-to';

const LINK_TYPES: LinkType[] = ['blocks', 'blocked-by', 'relates-to'];

function deriveTitleFromFileName(fileName: string): string {
  const stem = fileName.replace(/\.md$/i, '');
  const words = stem.split(/[-_]+/).filter(Boolean);
  if (words.length === 0) return 'Untitled Issue';
  return words.map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join(' ');
}

function normalizeLink(link: unknown): IssueLink | null {
  if (link && typeof link === 'object') {
    const value = link as { type?: unknown; target?: unknown };
    const type = typeof value.type === 'string' ? value.type : 'relates-to';
    const target = typeof value.target === 'string' ? value.target.trim() : '';
    if (target) {
      return { type, target };
    }
  }

  if (typeof link === 'string') {
    const raw = link.trim();
    if (!raw) return null;
    const idx = raw.indexOf(':');
    if (idx > 0) {
      const maybeType = raw.slice(0, idx);
      const target = raw.slice(idx + 1).trim();
      if (target) {
        return {
          type: LINK_TYPES.includes(maybeType as LinkType) ? maybeType : 'relates-to',
          target,
        };
      }
    }
    return { type: 'relates-to', target: raw };
  }

  return null;
}

function normalizeIssue(issue: Issue): Issue {
  const source = issue as Partial<Issue>;
  return {
    id: typeof source.id === 'string' ? source.id : '',
    title: typeof source.title === 'string' ? source.title : '',
    created: typeof source.created === 'string' ? source.created : '',
    updated: typeof source.updated === 'string' ? source.updated : '',
    assignee: typeof source.assignee === 'string' ? source.assignee : null,
    tags: Array.isArray(source.tags) ? source.tags.filter((tag): tag is string => typeof tag === 'string') : [],
    spec: typeof source.spec === 'string' ? source.spec : null,
    links: Array.isArray(source.links)
      ? source.links
          .map(normalizeLink)
          .filter((link): link is IssueLink => link !== null)
      : [],
    description: typeof source.description === 'string' ? source.description : '',
  };
}

export default function IssueDetail({
  entry,
  statuses,
  tagSuggestions,
  mcpEnabled = false,
  onClose,
  onStatusChange,
  onDelete,
  onSave,
}: IssueDetailProps) {
  const [draft, setDraft] = useState<Issue>(() => normalizeIssue(entry.issue));
  const [dirty, setDirty] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [tagInput, setTagInput] = useState('');
  const [showTagSuggestions, setShowTagSuggestions] = useState(false);
  const [showAddLink, setShowAddLink] = useState(false);
  const [newLinkType, setNewLinkType] = useState<LinkType>('relates-to');
  const [newLinkTarget, setNewLinkTarget] = useState('');

  useEffect(() => {
    setDraft(normalizeIssue(entry.issue));
    setDirty(false);
    setTagInput('');
    setShowTagSuggestions(false);
    setShowAddLink(false);
    setNewLinkType('relates-to');
    setNewLinkTarget('');
  }, [entry]);

  const activeStatus = statuses.find((status) => status.id === entry.status) ?? statuses[0];
  const cfg = activeStatus
    ? getStatusStyles(activeStatus)
    : { label: entry.status, color: 'text-zinc-400', bg: 'bg-zinc-800', border: 'border-zinc-700' };

  const createdDate = (() => {
    const date = new Date(draft.created);
    if (Number.isNaN(date.getTime())) return 'Unknown';
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  })();
  const displayTitle = draft.title.trim() || deriveTitleFromFileName(entry.file_name);

  const assigneeInitials = useMemo(() => {
    const name = (draft.assignee ?? '').trim();
    if (!name) return '';
    const parts = name.split(/\s+/).filter(Boolean);
    return parts.slice(0, 2).map((p) => p[0]?.toUpperCase() ?? '').join('');
  }, [draft.assignee]);

  const linkRows = (draft.links ?? []).map((link) => ({
    type: LINK_TYPES.includes(link.type as LinkType) ? (link.type as LinkType) : 'relates-to',
    target: link.target,
  }));

  const filteredSuggestions = tagSuggestions
    .filter((tag) => !(draft.tags ?? []).includes(tag))
    .filter((tag) => !tagInput.trim() || tag.toLowerCase().includes(tagInput.trim().toLowerCase()))
    .slice(0, 6);

  const markDirty = () => setDirty(true);

  const saveIssue = () => {
    onSave(entry.path, draft);
    setDirty(false);
  };

  useEffect(() => {
    if (!dirty) return;
    const timer = window.setTimeout(() => {
      saveIssue();
    }, 2000);
    return () => window.clearTimeout(timer);
  }, [dirty, draft]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
        event.preventDefault();
        saveIssue();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [draft, onClose]);

  const addTag = (value: string) => {
    const cleaned = value.trim();
    if (!cleaned || (draft.tags ?? []).includes(cleaned)) return;
    setDraft((current) => ({
      ...current,
      tags: [...(current.tags ?? []), cleaned],
    }));
    setTagInput('');
    markDirty();
  };

  const removeTag = (tag: string) => {
    setDraft((current) => ({
      ...current,
      tags: (current.tags ?? []).filter((t) => t !== tag),
    }));
    markDirty();
  };

  const addLink = () => {
    const target = newLinkTarget.trim();
    if (!target) return;
    setDraft((current) => ({
      ...current,
      links: [...(current.links ?? []), { type: newLinkType, target }],
    }));
    setNewLinkTarget('');
    setNewLinkType('relates-to');
    setShowAddLink(false);
    markDirty();
  };

  const removeLink = (index: number) => {
    setDraft((current) => ({
      ...current,
      links: (current.links ?? []).filter((_, i) => i !== index),
    }));
    markDirty();
  };

  return (
    <DetailSheet
      label={cfg.label}
      title={<h2 className="text-xl font-semibold tracking-tight">{displayTitle}</h2>}
      meta={
        <p className="text-muted-foreground text-xs">
          Created {createdDate} · {entry.file_name}
        </p>
      }
      onClose={onClose}
      footer={
        <div className="flex flex-wrap items-center justify-end gap-2">
          <Button variant="outline" onClick={onClose}>
            Close
          </Button>
          {dirty && <Button onClick={saveIssue}>Save Changes</Button>}
          {!confirmDelete ? (
            <Button variant="destructive" onClick={() => setConfirmDelete(true)}>
              Delete
            </Button>
          ) : (
            <Card size="sm" className="w-full border-destructive/30 md:w-auto">
              <CardContent className="flex items-center gap-2 py-2">
                <span className="text-sm">Delete this issue?</span>
                <Button variant="destructive" size="xs" onClick={() => onDelete(entry.path)}>
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
            <CardTitle className="text-sm">Issue Metadata</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="space-y-1">
              <Label htmlFor="issue-title">Title</Label>
              <Input
                id="issue-title"
                value={draft.title}
                placeholder={deriveTitleFromFileName(entry.file_name)}
                onChange={(event) => {
                  const title = event.target.value;
                  setDraft((current) => ({
                    ...current,
                    title,
                  }));
                  markDirty();
                }}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="issue-assignee">Assignee</Label>
              <div className="flex items-center gap-2">
                {assigneeInitials && (
                  <Badge variant="secondary" className="h-8 min-w-8 justify-center rounded-full px-2">
                    {assigneeInitials}
                  </Badge>
                )}
                <Input
                  id="issue-assignee"
                  value={draft.assignee ?? ''}
                  placeholder="Assign to..."
                  onChange={(event) => {
                    const assignee = event.target.value;
                    setDraft((current) => ({
                      ...current,
                      assignee: assignee.trim() ? assignee : null,
                    }));
                    markDirty();
                  }}
                />
              </div>
            </div>
            <div className="space-y-1">
              <Label htmlFor="issue-spec">Spec Reference</Label>
              <Input
                id="issue-spec"
                value={draft.spec ?? ''}
                placeholder="alpha-spec.md"
                onChange={(event) => {
                  const spec = event.target.value;
                  setDraft((current) => ({
                    ...current,
                    spec: spec.trim() ? spec : null,
                  }));
                  markDirty();
                }}
              />
            </div>
          </CardContent>
        </Card>

        <Card size="sm">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Status</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-wrap gap-2">
            {statuses.map((status) => {
              const statusStyles = getStatusStyles(status);
              const active = entry.status === status.id;
              return (
                <Button
                  key={status.id}
                  variant={active ? 'secondary' : 'outline'}
                  size="sm"
                  className={active ? `${statusStyles.bg} ${statusStyles.color} ${statusStyles.border}` : ''}
                  onClick={() => {
                    if (status.id !== entry.status) {
                      onStatusChange(entry.file_name, entry.status, status.id);
                    }
                  }}
                >
                  {statusStyles.label}
                </Button>
              );
            })}
          </CardContent>
        </Card>
      </div>

      <Card size="sm" className="mt-3">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">Tags</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex flex-wrap gap-2">
            {(draft.tags ?? []).length === 0 ? (
              <span className="text-muted-foreground text-xs">No tags yet.</span>
            ) : (
              (draft.tags ?? []).map((tag) => (
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
          <div className="relative">
            <Input
              value={tagInput}
              placeholder="Add tag and press Enter"
              onFocus={() => setShowTagSuggestions(true)}
              onBlur={() => window.setTimeout(() => setShowTagSuggestions(false), 100)}
              onChange={(event) => setTagInput(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter' || event.key === ',') {
                  event.preventDefault();
                  addTag(tagInput);
                }
              }}
            />
            {showTagSuggestions && filteredSuggestions.length > 0 && (
              <div className="bg-popover absolute left-0 right-0 top-[calc(100%+0.35rem)] z-10 flex flex-wrap gap-1 rounded-md border p-2">
                {filteredSuggestions.map((tag) => (
                  <Button key={tag} variant="ghost" size="xs" onClick={() => addTag(tag)}>
                    {tag}
                  </Button>
                ))}
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <Card size="sm" className="mt-3">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">Links</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {linkRows.length === 0 ? (
            <p className="text-muted-foreground text-xs">No linked issues yet.</p>
          ) : (
            <div className="space-y-2">
              {linkRows.map((link, index) => (
                <div
                  key={`${link.type}-${link.target}-${index}`}
                  className="bg-muted/40 flex items-center gap-2 rounded-md border p-2"
                >
                  <Badge variant="outline">{link.type}</Badge>
                  <span className="text-sm">{link.target}</span>
                  <Button className="ml-auto" size="xs" variant="ghost" onClick={() => removeLink(index)}>
                    Remove
                  </Button>
                </div>
              ))}
            </div>
          )}
          {!showAddLink ? (
            <Button variant="outline" size="sm" onClick={() => setShowAddLink(true)}>
              Add Link
            </Button>
          ) : (
            <div className="grid gap-2 md:grid-cols-[12rem_1fr_auto_auto]">
              <Select value={newLinkType} onValueChange={(value) => setNewLinkType(value as LinkType)}>
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {LINK_TYPES.map((type) => (
                    <SelectItem key={type} value={type}>
                      {type}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Input
                value={newLinkTarget}
                placeholder="target issue filename"
                onChange={(event) => setNewLinkTarget(event.target.value)}
              />
              <Button onClick={addLink}>Add</Button>
              <Button variant="ghost" onClick={() => setShowAddLink(false)}>
                Cancel
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      <div className="mt-3">
        <MarkdownEditor
          label="Description"
          value={draft.description}
          onChange={(description) => {
            setDraft((current) => ({ ...current, description }));
            markDirty();
          }}
          placeholder="Describe this issue..."
          rows={12}
          defaultMode="doc"
          mcpEnabled={mcpEnabled}
          sampleLabel="Insert Template"
          sampleRequiresMcp={false}
          onMcpSample={() => loadProjectTemplate('issue', { bodyOnly: true })}
        />
      </div>
    </DetailSheet>
  );
}
