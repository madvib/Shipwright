import { ReactNode } from 'react';
import { AlertTriangle, CheckCircle2, Link2, Sparkles } from 'lucide-react';
import { Badge } from '@ship/ui';
import { cn } from '@/lib/utils';

export interface FeatureHubSummaryMetrics {
  total: number;
  implemented: number;
  blocking: number;
  unlinked: number;
  avgReadiness: number;
}

interface FeatureHubStatsProps {
  metrics: FeatureHubSummaryMetrics;
}

interface MetricChipProps {
  label: string;
  value: string;
  hint?: string;
  icon?: ReactNode;
  tone?: 'default' | 'warning' | 'success';
}

function MetricChip({ label, value, hint, icon, tone = 'default' }: MetricChipProps) {
  return (
    <div
      className={cn(
        'inline-flex items-center gap-1.5 rounded-md border px-2 py-1 text-xs',
        tone === 'warning' && 'border-amber-500/40 bg-amber-500/5',
        tone === 'success' && 'border-emerald-500/35 bg-emerald-500/5'
      )}
    >
      {icon}
      <span className="text-muted-foreground">{label}</span>
      <span className="font-semibold">{value}</span>
      {hint && <span className="text-muted-foreground">· {hint}</span>}
    </div>
  );
}

export default function FeatureHubStats({ metrics }: FeatureHubStatsProps) {
  const implementedPercent =
    metrics.total > 0 ? Math.round((metrics.implemented / metrics.total) * 100) : 0;

  return (
    <div className="mb-2 flex flex-wrap items-center gap-2 rounded-lg border bg-card/45 px-2.5 py-2">
      <Badge variant="outline" className="h-6 px-2 text-[10px] uppercase tracking-wider">
        Feature Health
      </Badge>
      <MetricChip
        label="Coverage"
        value={`${metrics.total}`}
        hint={`${metrics.unlinked} unlinked`}
      />
      <MetricChip
        label="Implemented"
        value={`${metrics.implemented}/${metrics.total}`}
        hint={`${implementedPercent}%`}
        icon={<CheckCircle2 className="size-3.5 text-emerald-500" />}
        tone={implementedPercent >= 80 ? 'success' : 'default'}
      />
      <MetricChip
        label="Blockers"
        value={`${metrics.blocking}`}
        icon={
          <AlertTriangle
            className={cn(
              'size-3.5',
              metrics.blocking > 0 ? 'text-amber-500' : 'text-emerald-500'
            )}
          />
        }
        tone={metrics.blocking > 0 ? 'warning' : 'success'}
      />
      <MetricChip
        label="Readiness"
        value={`${metrics.avgReadiness}%`}
        icon={<Sparkles className="size-3.5 text-sky-500" />}
      />
      <MetricChip
        label="Links"
        value={`${metrics.total - metrics.unlinked}/${metrics.total}`}
        icon={<Link2 className="size-3.5 text-primary" />}
      />
    </div>
  );
}
