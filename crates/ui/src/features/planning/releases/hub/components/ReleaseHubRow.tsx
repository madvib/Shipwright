import { AlertTriangle, ArrowRight, CheckCircle2 } from 'lucide-react';
import { FeatureInfo as FeatureEntry, ReleaseInfo as ReleaseEntry } from '@/bindings';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Progress } from '@ship/ui';
import { cn } from '@/lib/utils';
import { formatStatusLabel } from '@/features/planning/common/hub/utils/featureMetrics';

interface ReleaseHubLinkedFeature {
  feature: FeatureEntry;
  readiness: number;
  blocking: boolean;
}

interface ReleaseHubRowProps {
  release: ReleaseEntry;
  linked: ReleaseHubLinkedFeature[];
  progress: number;
  blockers: number;
  todosDone: number;
  todosTotal: number;
  acceptanceDone: number;
  acceptanceTotal: number;
  onOpen: (release: ReleaseEntry) => void;
}

export default function ReleaseHubRow({
  release,
  linked,
  progress,
  blockers,
  todosDone,
  todosTotal,
  acceptanceDone,
  acceptanceTotal,
  onOpen,
}: ReleaseHubRowProps) {
  return (
    <div
      className={cn(
        'grid gap-3 rounded-md border p-3 transition-colors',
        blockers > 0 ? 'border-orange-500/25 bg-orange-500/[0.03] shadow-sm' : 'hover:bg-muted/35'
      )}
      title={release.path}
    >
      <div className="grid gap-2 md:grid-cols-[1fr_auto] md:items-start">
        <div className="min-w-0 space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <p className="truncate text-sm font-semibold">{release.version}</p>
            <Badge variant="outline">{formatStatusLabel(release.status)}</Badge>
            <Badge variant="secondary">{linked.length} linked features</Badge>
            {blockers > 0 && <Badge variant="secondary">{blockers} blockers</Badge>}
          </div>
          <div className="space-y-1">
            <div className="flex items-center justify-between text-[11px]">
              <span className="text-muted-foreground">Launch readiness</span>
              <span className="font-medium">{progress}%</span>
            </div>
            <Progress
              value={progress}
              indicatorClassName={cn(blockers > 0 ? 'bg-amber-500' : 'bg-emerald-500')}
            />
            <div className="flex flex-wrap items-center gap-3 text-[11px] text-muted-foreground">
              <span>
                Todos: {todosDone}/{todosTotal}
              </span>
              <span>
                Acceptance: {acceptanceDone}/{acceptanceTotal}
              </span>
            </div>
          </div>
        </div>
        <Button variant="outline" size="sm" onClick={() => onOpen(release)}>
          Open
          <ArrowRight className="size-3.5" />
        </Button>
      </div>

      <div className="grid gap-1.5">
        {linked.slice(0, 4).map((entry) => (
          <div
            key={entry.feature.file_name}
            className="flex items-center justify-between rounded-sm border bg-background/60 px-2.5 py-1.5 text-[11px]"
          >
            <div className="min-w-0">
              <p className="truncate font-medium">{entry.feature.title}</p>
              <p className="text-muted-foreground">
                {formatStatusLabel(entry.feature.status)} · {entry.readiness}% ready
              </p>
            </div>
            {entry.blocking ? (
              <AlertTriangle className="size-3.5 text-amber-500 shrink-0" />
            ) : (
              <CheckCircle2 className="size-3.5 text-emerald-500 shrink-0" />
            )}
          </div>
        ))}
        {linked.length === 0 && (
          <p className="text-muted-foreground rounded-sm border border-dashed px-2.5 py-2 text-[11px] italic">
            No linked features yet.
          </p>
        )}
        {linked.length > 4 && (
          <p className="text-muted-foreground text-[11px]">
            +{linked.length - 4} more linked features
          </p>
        )}
      </div>
    </div>
  );
}
