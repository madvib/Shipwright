import { ArrowRight, Link2 } from 'lucide-react';
import { FeatureInfo as FeatureEntry, ReleaseInfo as ReleaseEntry } from '@/bindings';
import { Badge } from '@ship/primitives';
import { Progress } from '@ship/primitives';
import { cn } from '@/lib/utils';
import { FeatureChecklistMetrics, formatStatusLabel } from '@/features/planning/common/hub/utils/featureMetrics';

interface FeatureHubRowProps {
  feature: FeatureEntry;
  release: ReleaseEntry | null;
  metrics?: FeatureChecklistMetrics;
  readiness: number;
  isBlocking: boolean;
  onSelect: (feature: FeatureEntry) => void;
}

export default function FeatureHubRow({
  feature,
  release,
  metrics,
  readiness,
  isBlocking,
  onSelect,
}: FeatureHubRowProps) {
  return (
    <button
      type="button"
      className="grid w-full gap-2 rounded-md border border-border p-3 text-left transition-colors hover:border-primary/45 hover:bg-muted/35 hover:shadow-sm md:grid-cols-[1fr_auto] md:items-start"
      title={feature.path}
      onClick={() => onSelect(feature)}
    >
      <div className="min-w-0 space-y-2">
        <div className="flex flex-wrap items-center gap-2">
          <p className="truncate text-sm font-semibold">{feature.title}</p>
          <div className="flex flex-wrap items-center gap-1.5 min-w-0">
            <Badge variant="outline" className="h-5 px-1.5 text-[10px] uppercase font-bold tracking-tighter shrink-0">{formatStatusLabel(feature.status)}</Badge>
            {feature.docs_status && (
              <Badge
                variant="secondary"
                className={cn(
                  "h-5 px-1.5 text-[10px] shrink-0",
                  feature.docs_status.toLowerCase().includes('not-started') && "bg-muted/50 text-muted-foreground border-transparent font-medium"
                )}
              >
                Docs: {feature.docs_status}
              </Badge>
            )}
            {isBlocking && <Badge variant="secondary" className="h-5 px-1.5 text-[10px] shrink-0">Blocking</Badge>}
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
          <Badge
            variant={feature.release_id ? "secondary" : "outline"}
            className={cn(
              "h-5 px-1.5 text-[10px] font-bold tracking-tight",
              !feature.release_id && "border-dashed opacity-70"
            )}
          >
            {release?.version ?? feature.release_id ?? 'No release'}
          </Badge>

          {feature.branch && (
            <span className="inline-flex items-center gap-1 rounded-sm border px-1.5 py-0.5 text-[10px] font-mono opacity-80">
              <Link2 className="size-3" />
              {feature.branch}
            </span>
          )}
        </div>

        <div className="space-y-1">
          <div className="flex items-center justify-between text-[11px]">
            <span className="text-muted-foreground">Readiness</span>
            <span className="font-medium">{readiness}%</span>
          </div>
          <Progress
            value={readiness}
            indicatorClassName={cn(isBlocking ? 'bg-amber-500' : 'bg-emerald-500')}
          />
          <div className="flex flex-wrap items-center gap-3 text-[11px] text-muted-foreground">
            <span>
              Todos: {metrics?.todos.done ?? 0}/{metrics?.todos.total ?? 0}
            </span>
            <span>
              Acceptance: {metrics?.acceptance.done ?? 0}/{metrics?.acceptance.total ?? 0}
            </span>
          </div>
        </div>
      </div>

      <div className="flex items-center gap-2 text-xs text-muted-foreground md:justify-end">
        <ArrowRight className="size-4" />
      </div>
    </button>
  );
}
