import { useEffect, useState } from 'react';
import { Issue, IssueEntry, IssueLink, StatusConfig } from '@/bindings';
import DetailSheet from './DetailSheet';
import MarkdownEditor from '@/components/editor';
import { Button } from '@ship/ui';
import { Card, CardContent } from '@ship/ui';
import { AutocompleteInput } from '@ship/ui';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ship/ui';
import { IssueHeaderMetadata } from './IssueHeaderMetadata';
import { EntityLink } from '@/lib/links';

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
    if (target) return { type, target };
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
  const source = issue as Record<string, unknown>;
  return {
    id: typeof source.id === 'string' ? source.id : '',
    title: typeof source.title === 'string' ? source.title : '',
    created: typeof source.created === 'string' ? source.created : '',
    updated: typeof source.updated === 'string' ? source.updated : '',
    assignee: typeof source.assignee === 'string' ? source.assignee : null,
    priority: null,
    tags: Array.isArray(source.tags) ? source.tags.filter((tag): tag is string => typeof tag === 'string') : [],
    spec_id: typeof source.spec_id === 'string' ? source.spec_id : null,
    feature_id: typeof source.feature_id === 'string' ? source.feature_id : null,
    links: Array.isArray(source.links)
      ? (source.links as unknown[]).map(normalizeLink).filter((link): link is IssueLink => link !== null)
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
  const [showAddLink, setShowAddLink] = useState(false);
  const [newLinkType, setNewLinkType] = useState<LinkType>('relates-to');
  const [newLinkTarget, setNewLinkTarget] = useState('');

  useEffect(() => {
    setDraft(normalizeIssue(entry.issue));
    setDirty(false);
    setShowAddLink(false);
    setNewLinkType('relates-to');
    setNewLinkTarget('');
  }, [entry]);

  const displayTitle = draft.title.trim() || deriveTitleFromFileName(entry.file_name);

  const linkRows = (draft.links ?? []).map((link) => ({
    type: LINK_TYPES.includes(link.type as LinkType) ? (link.type as LinkType) : 'relates-to',
    target: link.target,
  }));

  const markDirty = () => setDirty(true);

  const saveIssue = () => {
    onSave(entry.path, draft);
    setDirty(false);
  };

  // Auto-save after 2s of inactivity
  useEffect(() => {
    if (!dirty) return;
    const timer = window.setTimeout(() => saveIssue(), 2000);
    return () => window.clearTimeout(timer);
  }, [dirty, draft]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') { event.preventDefault(); onClose(); return; }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
        event.preventDefault(); saveIssue();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [draft, onClose]);

  const addLink = (targetOverride?: string) => {
    const target = (targetOverride ?? newLinkTarget).trim();
    if (!target) return;
    setDraft((current) => ({ ...current, links: [...(current.links ?? []), { type: newLinkType, target }] }));
    setNewLinkTarget('');
    setNewLinkType('relates-to');
    setShowAddLink(false);
    markDirty();
  };

  const removeLink = (index: number) => {
    setDraft((current) => ({ ...current, links: (current.links ?? []).filter((_, i) => i !== index) }));
    markDirty();
  };

  const activeStatus = statuses.find((s) => s.id === entry.status) ?? statuses[0];

  const toolbarActions = (
    <>
      <Button size="xs" className="h-7 px-2 text-xs" onClick={saveIssue} disabled={!dirty}>
        Save Issue
      </Button>
      {!confirmDelete ? (
        <Button size="xs" variant="outline" className="h-7 border-destructive/40 px-2 text-destructive hover:bg-destructive/10" onClick={() => setConfirmDelete(true)}>
          Delete
        </Button>
      ) : (
        <Card size="sm" className="border-destructive/30">
          <CardContent className="flex items-center gap-2 py-1.5 px-2">
            <span className="text-xs">Delete?</span>
            <Button variant="destructive" size="xs" onClick={() => onDelete(entry.path)}>Yes</Button>
            <Button variant="outline" size="xs" onClick={() => setConfirmDelete(false)}>Cancel</Button>
          </CardContent>
        </Card>
      )}
    </>
  );

  return (
    <DetailSheet
      label={activeStatus?.name ?? entry.status}
      title={<h2 className="truncate text-lg font-semibold tracking-tight">{displayTitle}</h2>}
      meta={
        <IssueHeaderMetadata
          status={entry.status}
          statuses={statuses}
          assignee={draft.assignee ?? null}
          specId={draft.spec_id ?? null}
          tags={draft.tags ?? []}
          tagSuggestions={tagSuggestions}
          specSuggestions={specSuggestions}
          onStatusChange={(nextStatus) => {
            if (nextStatus !== entry.status) {
              onStatusChange(entry.file_name, entry.status, nextStatus);
            }
          }}
          onAssigneeChange={(v) => { setDraft((c) => ({ ...c, assignee: v })); markDirty(); }}
          onSpecIdChange={(v) => { setDraft((c) => ({ ...c, spec_id: v })); markDirty(); }}
          onTagsChange={(v) => { setDraft((c) => ({ ...c, tags: v })); markDirty(); }}
        />
      }
      onClose={onClose}
      className="max-w-[1400px]"
      bodyScrollable={false}
      bodyClassName="overflow-hidden p-0"
      inlineHeader
    >
      <div className="flex h-full min-h-0 flex-col">
        <div className="flex min-h-0 flex-1">
          {/* Editor */}
          <div className="min-w-0 flex-1 p-1.5">
            <MarkdownEditor
              label={undefined}
              toolbarStart={toolbarActions}
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
            />
          </div>
        </div>

        {/* Links — collapsible footer strip */}
        <div className="border-t bg-muted/10 px-4 py-2">
          <div className="flex items-center gap-2">
            <p className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">
              Links {linkRows.length > 0 && `(${linkRows.length})`}
            </p>
            <Button variant="ghost" size="xs" className="ml-auto h-6 text-xs" onClick={() => setShowAddLink((v) => !v)}>
              + Add Link
            </Button>
          </div>
          {(linkRows.length > 0 || showAddLink) && (
            <div className="mt-2 space-y-2">
              {linkRows.map((link, index) => (
                <EntityLink
                  key={`${link.type}-${link.target}-${index}`}
                  link={link}
                  onRemove={() => removeLink(index)}
                />
              ))}
              {showAddLink && (
                <div className="grid gap-2 md:grid-cols-[10rem_1fr_auto_auto]">
                  <Select value={newLinkType} onValueChange={(value) => setNewLinkType(value as LinkType)}>
                    <SelectTrigger className="w-full"><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {LINK_TYPES.map((type) => (<SelectItem key={type} value={type}>{type}</SelectItem>))}
                    </SelectContent>
                  </Select>
                  <AutocompleteInput
                    value={newLinkTarget}
                    options={issueSuggestions.filter((issue) => issue !== entry.file_name).map((issue) => ({ value: issue }))}
                    placeholder="target issue filename"
                    noResultsText="No issue matches."
                    onCommit={(value) => addLink(value)}
                    onValueChange={(value) => setNewLinkTarget(value)}
                  />
                  <Button size="xs" onClick={() => addLink()}>Add</Button>
                  <Button size="xs" variant="ghost" onClick={() => setShowAddLink(false)}>Cancel</Button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </DetailSheet>
  );
}
