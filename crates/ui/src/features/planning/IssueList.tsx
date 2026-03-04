import { useMemo, useState } from 'react';
import {
  DndContext,
  DragEndEvent,
  DragOverlay,
  DragStartEvent,
  PointerSensor,
  useDraggable,
  useDroppable,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import { ChevronDown, ChevronUp, GripVertical, Plus } from 'lucide-react';
import { IssueEntry, StatusConfig } from '@/bindings';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Card, CardContent, CardHeader, CardTitle } from '@ship/ui';
import { getStatusStyles } from '@/lib/workspace-ui';
import { cn } from '@/lib/utils';

interface IssueListProps {
  issues: IssueEntry[];
  statuses: StatusConfig[];
  onSelect: (entry: IssueEntry) => void;
  onMove: (fileName: string, from: string, to: string) => Promise<void> | void;
  onNewIssue: () => void;
}

const ISSUE_DND_PREFIX = 'issue:';
const COLUMN_DND_PREFIX = 'column:';

function deriveTitle(entry: IssueEntry): string {
  const providedTitle = entry.issue?.title;
  if (typeof providedTitle === 'string' && providedTitle.trim()) {
    return providedTitle.trim();
  }

  const stem = entry.file_name?.replace(/\.md$/i, '') ?? 'untitled-issue';
  return stem
    .split(/[-_]+/)
    .filter(Boolean)
    .map((chunk) => chunk.charAt(0).toUpperCase() + chunk.slice(1))
    .join(' ');
}

function summarizeMarkdown(markdown: string, maxLength = 120): string {
  const compact = markdown
    .replace(/^---[\s\S]*?---\s*/m, ' ')
    .replace(/^\+\+\+[\s\S]*?\+\+\+\s*/m, ' ')
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/`([^`]+)`/g, '$1')
    .replace(/!\[[^\]]*]\([^)]*\)/g, ' ')
    .replace(/\[([^\]]+)]\([^)]*\)/g, '$1')
    .replace(/<[^>]+>/g, ' ')
    .replace(/^#{1,6}\s+/gm, '')
    .replace(/^[-*+]\s+/gm, '')
    .replace(/^\d+\.\s+/gm, '')
    .replace(/^\|/gm, '')
    .replace(/\|$/gm, '')
    .replace(/^>\s?/gm, '')
    .replace(/\s+/g, ' ')
    .trim();

  if (compact.length <= maxLength) {
    return compact;
  }

  return `${compact.slice(0, maxLength - 1).trimEnd()}…`;
}

function issueDndId(path: string): string {
  return `${ISSUE_DND_PREFIX}${path}`;
}

function columnDndId(statusId: string): string {
  return `${COLUMN_DND_PREFIX}${statusId}`;
}

function statusFromDndId(
  id: string | number | null | undefined,
  issuesByPath: Map<string, IssueEntry>
): string | null {
  if (id === null || id === undefined) {
    return null;
  }

  const value = String(id);
  if (value.startsWith(COLUMN_DND_PREFIX)) {
    return value.slice(COLUMN_DND_PREFIX.length);
  }

  if (value.startsWith(ISSUE_DND_PREFIX)) {
    const path = value.slice(ISSUE_DND_PREFIX.length);
    return issuesByPath.get(path)?.status ?? null;
  }

  return null;
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    if (Number.isNaN(date.getTime())) return '';
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  } catch {
    return '';
  }
}

interface IssueCardProps {
  entry: IssueEntry;
  isMoving: boolean;
  onSelect: (entry: IssueEntry) => void;
}

function IssueCard({ entry, isMoving, onSelect }: IssueCardProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    isDragging,
  } = useDraggable({
    id: issueDndId(entry.path),
  });

  const title = deriveTitle(entry);
  const description = typeof entry.issue?.description === 'string' ? entry.issue.description : '';
  const summary = summarizeMarkdown(description, 76);
  const created = typeof entry.issue?.created === 'string' ? entry.issue.created : '';
  const tags = Array.isArray(entry.issue?.tags)
    ? entry.issue.tags.filter((tag): tag is string => Boolean(tag)).slice(0, 2)
    : [];
  const translate = transform
    ? `translate3d(${Math.round(transform.x)}px, ${Math.round(transform.y)}px, 0)`
    : undefined;

  return (
    <article
      ref={setNodeRef}
      style={{
        transform: translate,
        zIndex: isDragging ? 20 : undefined,
      }}
      className={cn(
        'group relative min-w-0 select-none rounded-md border bg-card/90 p-2 shadow-sm transition-colors',
        'hover:border-accent/40 hover:bg-card',
        isDragging && 'opacity-0',
        isMoving && 'animate-pulse'
      )}
    >
      <div className="mb-1 flex items-start justify-between gap-1.5">
        <span className="line-clamp-2 break-words text-[13px] font-semibold leading-4.5">{title}</span>
        <button
          type="button"
          className="text-muted-foreground inline-flex cursor-grab touch-none items-center rounded p-0 transition-opacity hover:opacity-100 active:cursor-grabbing"
          aria-label={`Drag ${title}`}
          onClick={(event) => event.preventDefault()}
          {...listeners}
          {...attributes}
        >
          <GripVertical className="size-3 shrink-0 opacity-70" />
        </button>
      </div>

      <button type="button" className="w-full min-w-0 text-left" onClick={() => onSelect(entry)}>
        {summary ? (
          <p className="text-muted-foreground line-clamp-2 break-words text-[10px] leading-3.5">
            {summary}
          </p>
        ) : (
          <p className="text-muted-foreground text-[10px] italic">No description yet.</p>
        )}

        <div className="mt-1.5 flex flex-wrap items-center gap-1">
          {tags.map((tag) => (
            <Badge key={tag} variant="outline" className="h-4.5 px-1.5 text-[10px]">
              {tag}
            </Badge>
          ))}
        </div>

        {created && <span className="text-muted-foreground mt-1.5 block text-[10px]">{formatDate(created)}</span>}
      </button>
    </article>
  );
}

interface IssueColumnProps {
  status: StatusConfig;
  issues: IssueEntry[];
  isDropTarget: boolean;
  isMovingPath: string | null;
  showNewIssueButton: boolean;
  collapsed: boolean;
  onToggleCollapse: () => void;
  onSelect: (entry: IssueEntry) => void;
  onNewIssue: () => void;
}

function IssueColumn({
  status,
  issues,
  isDropTarget,
  isMovingPath,
  showNewIssueButton,
  collapsed,
  onToggleCollapse,
  onSelect,
  onNewIssue,
}: IssueColumnProps) {
  const style = getStatusStyles(status);
  const { setNodeRef } = useDroppable({
    id: columnDndId(status.id),
  });

  if (collapsed) {
    return (
      <div
        ref={setNodeRef}
        className={cn(
          'flex w-11 shrink-0 cursor-pointer flex-col items-center rounded-lg border py-3 transition-all hover:w-14',
          style.border,
          style.bg,
          isDropTarget && 'ring-2 ring-accent/60'
        )}
        onClick={onToggleCollapse}
        title={`Expand ${style.label}`}
      >
        {/* Count badge */}
        <span className={cn('mb-3 text-[11px] font-bold', style.color)}>
          {issues.length}
        </span>
        {/* Rotated label */}
        <span
          className="flex-1 text-[11px] font-semibold tracking-wider text-muted-foreground"
          style={{ writingMode: 'vertical-rl', transform: 'rotate(180deg)' }}
        >
          {style.label}
        </span>
        {/* Expand indicator */}
        <ChevronDown className="mt-3 size-3 text-muted-foreground opacity-60" />
      </div>
    );
  }

  return (
    <div
      ref={setNodeRef}
      className={cn(
        'flex min-w-[200px] flex-1 flex-col',
        isDropTarget && 'ring-2 ring-inset ring-accent/60 rounded-lg'
      )}
    >
      <Card
        size="sm"
        className={cn(
          'flex h-full flex-col border transition-colors min-h-[8rem] max-h-[calc(100vh-14.5rem)]',
          style.border,
          style.bg,
        )}
      >
        <CardHeader className="pb-1.5">
          <CardTitle className="flex items-center justify-between gap-2 text-sm">
            <span className="inline-flex items-center gap-2">
              <span className={cn('text-xs', style.color)}>●</span>
              {style.label}
            </span>
            <div className="flex items-center gap-1.5">
              <Badge variant="outline">{issues.length}</Badge>
              <Button
                type="button"
                variant="ghost"
                size="icon-xs"
                className="size-6 opacity-50 hover:opacity-100"
                onClick={onToggleCollapse}
                title="Collapse column"
              >
                <ChevronUp className="size-3.5" />
              </Button>
            </div>
          </CardTitle>
        </CardHeader>
        <CardContent className="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto overflow-x-visible pr-1">
          {issues.map((entry) => (
            <IssueCard
              key={entry.path}
              entry={entry}
              isMoving={isMovingPath === entry.path}
              onSelect={onSelect}
            />
          ))}

          {issues.length === 0 && (
            <div className="text-muted-foreground rounded-md border border-dashed px-2.5 py-3 text-center text-xs">
              Drop an issue here
            </div>
          )}

          {showNewIssueButton && (
            <Button
              variant="secondary"
              className="h-auto w-full justify-start border border-dashed border-accent/40 bg-accent/10 px-2.5 py-1.5 text-accent-foreground"
              onClick={onNewIssue}
            >
              <Plus className="size-4" />
              New Issue
            </Button>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function IssueDragPreview({ entry }: { entry: IssueEntry }) {
  const title = deriveTitle(entry);
  const description = typeof entry.issue?.description === 'string' ? entry.issue.description : '';
  const summary = summarizeMarkdown(description, 100);

  return (
    <div className="w-[240px] rounded-md border bg-card p-2.5 shadow-xl">
      <p className="line-clamp-2 break-words text-[13px] font-semibold leading-4.5">{title}</p>
      {summary ? (
        <p className="text-muted-foreground mt-1 line-clamp-3 break-words text-[11px] leading-4">{summary}</p>
      ) : (
        <p className="text-muted-foreground mt-1 text-[11px] italic">No description yet.</p>
      )}
    </div>
  );
}

export default function IssueList({ issues, statuses, onSelect, onMove, onNewIssue }: IssueListProps) {
  const [activePath, setActivePath] = useState<string | null>(null);
  const [dropTargetStatus, setDropTargetStatus] = useState<string | null>(null);
  const [movingPath, setMovingPath] = useState<string | null>(null);
  const [suppressClickUntil, setSuppressClickUntil] = useState(0);

  // Collapsible state keyed by status id. Default: collapse columns that have 0 issues.
  const [collapsedColumns, setCollapsedColumns] = useState<Record<string, boolean>>(() => {
    const initial: Record<string, boolean> = {};
    // We'll update this after we know issue counts, but default all to false
    return initial;
  });

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 6 },
    })
  );

  const issuesByStatus = useMemo(() => {
    const map = new Map<string, IssueEntry[]>();
    for (const status of statuses) {
      map.set(status.id, []);
    }
    for (const issue of issues) {
      if (!map.has(issue.status)) {
        map.set(issue.status, []);
      }
      map.get(issue.status)?.push(issue);
    }
    return map;
  }, [issues, statuses]);

  const issuesByPath = useMemo(() => {
    const map = new Map<string, IssueEntry>();
    for (const issue of issues) {
      map.set(issue.path, issue);
    }
    return map;
  }, [issues]);
  const activeEntry = activePath ? issuesByPath.get(activePath) ?? null : null;

  const isColumnCollapsed = (statusId: string): boolean => {
    // Explicitly set value wins; otherwise auto-collapse empty columns
    if (statusId in collapsedColumns) return collapsedColumns[statusId];
    return (issuesByStatus.get(statusId)?.length ?? 0) === 0;
  };

  const toggleColumn = (statusId: string) => {
    setCollapsedColumns((prev) => ({
      ...prev,
      [statusId]: !isColumnCollapsed(statusId),
    }));
  };

  const handleDragStart = (event: DragStartEvent) => {
    const id = String(event.active.id);
    if (!id.startsWith(ISSUE_DND_PREFIX)) {
      setDropTargetStatus(null);
      setActivePath(null);
      return;
    }
    setActivePath(id.slice(ISSUE_DND_PREFIX.length));
    setSuppressClickUntil(Date.now() + 250);
  };

  const handleDragOver = (targetId: string | number | null | undefined) => {
    const targetStatus = statusFromDndId(targetId, issuesByPath);
    setDropTargetStatus(targetStatus);
  };

  const handleDragEnd = async (event: DragEndEvent) => {
    setDropTargetStatus(null);
    setSuppressClickUntil(Date.now() + 350);

    const activeId = String(event.active.id);
    setActivePath(null);
    if (!activeId.startsWith(ISSUE_DND_PREFIX)) {
      return;
    }
    const currentPath = activeId.slice(ISSUE_DND_PREFIX.length);

    if (!currentPath) {
      return;
    }

    const source = issuesByPath.get(currentPath);
    if (!source) {
      return;
    }

    const targetStatus = statusFromDndId(event.over?.id, issuesByPath);
    if (!targetStatus || targetStatus === source.status) {
      return;
    }

    setMovingPath(currentPath);
    try {
      await onMove(source.file_name, source.status, targetStatus);
    } finally {
      setMovingPath(null);
    }
  };

  return (
    <DndContext
      sensors={sensors}
      onDragStart={handleDragStart}
      onDragOver={(event) => handleDragOver(event.over?.id)}
      onDragEnd={(event) => void handleDragEnd(event)}
      onDragCancel={() => {
        setActivePath(null);
        setDropTargetStatus(null);
        setSuppressClickUntil(Date.now() + 250);
      }}
    >
      {/* Flex row — columns never wrap, collapsed ones shrink to narrow strips */}
      <div className="flex min-h-[400px] gap-2 overflow-x-auto pb-2" style={{ alignItems: 'stretch' }}>
        {statuses.map((status) => (
          <IssueColumn
            key={status.id}
            status={status}
            issues={issuesByStatus.get(status.id) ?? []}
            isDropTarget={dropTargetStatus === status.id}
            isMovingPath={movingPath}
            showNewIssueButton={status.id === statuses[0]?.id}
            collapsed={isColumnCollapsed(status.id)}
            onToggleCollapse={() => toggleColumn(status.id)}
            onSelect={(entry) => {
              if (Date.now() < suppressClickUntil) {
                return;
              }
              onSelect(entry);
            }}
            onNewIssue={onNewIssue}
          />
        ))}
      </div>
      <DragOverlay dropAnimation={null} adjustScale={false}>
        {activeEntry ? <IssueDragPreview entry={activeEntry} /> : null}
      </DragOverlay>
    </DndContext>
  );
}
