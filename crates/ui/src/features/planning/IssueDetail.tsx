import { useEffect, useMemo, useState } from 'react';
import { Issue, IssueEntry, IssueLink, StatusConfig } from '@/bindings';
import DetailSheet from './DetailSheet';
import MarkdownEditor from '@/components/editor';
import { loadProjectTemplate } from '@/components/editor/templateLoader';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import AutocompleteInput from '@/components/ui/autocomplete-input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { getStatusStyles } from '@/lib/workspace-ui';

interface IssueDetailProps {
  entry: IssueEntry;
  statuses: StatusConfig[];
  tagSuggestions: string[];
  specSuggestions: string[];
  issueSuggestions: string[];
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
  specSuggestions,
  issueSuggestions,
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
  const [showAddLink, setShowAddLink] = useState(false);
  const [newLinkType, setNewLinkType] = useState<LinkType>('relates-to');
  const [newLinkTarget, setNewLinkTarget] = useState('');

  useEffect(() => {
    setDraft(normalizeIssue(entry.issue));
    setDirty(false);
    setTagInput('');
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

  const addLink = (targetOverride?: string) => {
    const target = (targetOverride ?? newLinkTarget).trim();
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
      className="max-w-[1800px]"
      bodyScrollable={false}
      bodyClassName="overflow-hidden p-0"
      footerClassName="px-3 py-2 md:px-4 md:py-2.5"
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
      <div className="flex h-full min-h-0 flex-col gap-2 p-2">
        <div className="flex flex-wrap items-center gap-2">
          <Input
            id="issue-title"
            value={draft.title}
            className="h-8 min-w-[260px] flex-1"
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
          <div className="flex items-center gap-1.5">
            {assigneeInitials && (
              <Badge variant="secondary" className="h-7 min-w-7 justify-center rounded-full px-2 text-xs">
                {assigneeInitials}
              </Badge>
            )}
            <Input
              id="issue-assignee"
              value={draft.assignee ?? ''}
              className="h-8 w-[160px]"
              placeholder="Assignee"
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
          <AutocompleteInput
            id="issue-spec"
            value={draft.spec ?? ''}
            options={specSuggestions.map((spec) => ({ value: spec }))}
            className="h-8 w-[180px] text-xs"
            placeholder="Spec"
            onValueChange={(spec) => {
              setDraft((current) => ({
                ...current,
                spec: spec.trim() ? spec : null,
              }));
              markDirty();
            }}
          />
        </div>

        <div className="flex flex-wrap items-center gap-1.5">
          {statuses.map((status) => {
            const statusStyles = getStatusStyles(status);
            const active = entry.status === status.id;
            return (
              <Button
                key={status.id}
                variant={active ? 'secondary' : 'outline'}
                size="xs"
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
        </div>

        <div className="flex flex-wrap items-center gap-1.5">
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
          <AutocompleteInput
            value={tagInput}
            options={tagSuggestions
              .filter((tag) => !(draft.tags ?? []).includes(tag))
              .map((tag) => ({ value: tag }))}
            className="h-8 w-[220px] text-xs"
            placeholder="Add tag"
            noResultsText="No tag suggestions."
            onCommit={(value) => addTag(value)}
            onValueChange={(value) => setTagInput(value)}
          />
        </div>

        <details className="rounded-md border bg-card/35 px-2 py-1">
          <summary className="text-muted-foreground cursor-pointer select-none text-xs font-medium">
            Links ({linkRows.length})
          </summary>
          <div className="mt-2 space-y-2">
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
                    <span className="text-xs">{link.target}</span>
                    <Button className="ml-auto" size="xs" variant="ghost" onClick={() => removeLink(index)}>
                      Remove
                    </Button>
                  </div>
                ))}
              </div>
            )}
            {!showAddLink ? (
              <Button variant="outline" size="xs" onClick={() => setShowAddLink(true)}>
                Add Link
              </Button>
            ) : (
              <div className="grid gap-2 md:grid-cols-[10rem_1fr_auto_auto]">
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
                <AutocompleteInput
                  value={newLinkTarget}
                  options={issueSuggestions
                    .filter((issue) => issue !== entry.file_name)
                    .map((issue) => ({ value: issue }))}
                  placeholder="target issue filename"
                  noResultsText="No issue matches."
                  onCommit={(value) => addLink(value)}
                  onValueChange={(value) => setNewLinkTarget(value)}
                />
                <Button size="xs" onClick={() => addLink()}>
                  Add
                </Button>
                <Button size="xs" variant="ghost" onClick={() => setShowAddLink(false)}>
                  Cancel
                </Button>
              </div>
            )}
          </div>
        </details>

        <div className="min-h-0 flex-1">
        <MarkdownEditor
          label={undefined}
          value={draft.description}
          onChange={(description) => {
            setDraft((current) => ({ ...current, description }));
            markDirty();
          }}
          placeholder="Describe this issue..."
          rows={16}
          defaultMode="doc"
          fillHeight
          showStats={false}
          mcpEnabled={mcpEnabled}
          sampleInline
          sampleLabel="Insert Template"
          sampleRequiresMcp={false}
          onMcpSample={() => loadProjectTemplate('issue', { bodyOnly: true })}
        />
        </div>
      </div>
    </DetailSheet>
  );
}
