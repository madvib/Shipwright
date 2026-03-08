import { ArrowRight, Link2 } from 'lucide-react';
import { FeatureInfo as FeatureEntry, ReleaseInfo as ReleaseEntry } from '@/bindings';
import { SpecInfo as SpecEntry } from '@/lib/types/spec';
import { Badge } from '@ship/ui';
import { Progress } from '@ship/ui';
import { cn } from '@/lib/utils';
import { FeatureChecklistMetrics, formatStatusLabel } from '@/features/planning/common/hub/utils/featureMetrics';

interface FeatureHubRowProps {
  feature: FeatureEntry;
  release: ReleaseEntry | null;
  spec: SpecEntry | null;
  metrics?: FeatureChecklistMetrics;
  readiness: number;
  isBlocking: boolean;
  onSelect: (feature: FeatureEntry) => void;
}

export default function FeatureHubRow({
  feature,
  release,
  spec,
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
          <Badge variant="outline">{formatStatusLabel(feature.status)}</Badge>
          {feature.docs_status && (
            <Badge variant="outline">Docs: {feature.docs_status}</Badge>
          )}
          {isBlocking && <Badge variant="secondary">Blocking</Badge>}
        </div>
        <div className="flex flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
          <Badge variant="secondary">{release?.version ?? feature.release_id ?? 'No release'}</Badge>
          {spec && <Badge variant="secondary">{spec.spec.metadata.title}</Badge>}
          {feature.branch && (
            <span className="inline-flex items-center gap-1 rounded-sm border px-1.5 py-0.5">
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
