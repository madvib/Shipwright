import { ReactNode } from 'react';
import { AlertTriangle, CheckCircle2, Gauge, Link2, Rocket } from 'lucide-react';
import { ReleaseInfo as ReleaseEntry } from '@/bindings';
import { Badge, Tooltip, TooltipContent, TooltipTrigger } from '@ship/ui';
import { cn } from '@/lib/utils';

interface ReleaseHubStatsProps {
  activeRelease: ReleaseEntry | null;
  activeBlockers: number;
  shippedCount: number;
  totalReleases: number;
  activeTargetFeatureCount: number;
  activeTargetReleaseCount: number;
  avgProgress: number;
}

interface StatChipProps {
  label: string;
  value: string;
  hint?: string;
  icon?: ReactNode;
  tone?: 'default' | 'warning' | 'success';
  tooltip?: string;
}

function StatChip({ label, value, hint, icon, tone = 'default', tooltip }: StatChipProps) {
  const chip = (
    <div
      className={cn(
        'inline-flex items-center gap-1.5 rounded-md border px-2 py-1 text-xs',
        tone === 'warning' && 'border-orange-500/30 bg-orange-500/5',
        tone === 'success' && 'border-emerald-500/30 bg-emerald-500/5'
      )}
    >
      {icon}
      <span className="text-muted-foreground">{label}</span>
      <span className="font-semibold">{value}</span>
      {hint && <span className="text-muted-foreground">· {hint}</span>}
    </div>
  );

  if (!tooltip) return chip;
  return (
    <Tooltip>
      <TooltipTrigger asChild>{chip}</TooltipTrigger>
      <TooltipContent side="top">{tooltip}</TooltipContent>
    </Tooltip>
  );
}

export default function ReleaseHubStats({
  activeRelease,
  activeBlockers,
  shippedCount,
  totalReleases,
  activeTargetFeatureCount,
  activeTargetReleaseCount,
  avgProgress,
}: ReleaseHubStatsProps) {
  const activeReleaseLabel = activeRelease?.version ?? 'None';

  return (
    <div className="mb-2 flex flex-wrap items-center gap-2 rounded-lg border bg-card/45 px-2.5 py-2">
      <Badge variant="outline" className="h-6 px-2 text-[10px] uppercase tracking-wider">
        Release Health
      </Badge>
      <StatChip
        label="Active"
        value={activeReleaseLabel}
        hint={activeRelease ? `${activeBlockers} blockers` : 'No active release'}
        tooltip="Current active release and blocker count across linked features."
        icon={
          <AlertTriangle
            className={cn(
              'size-3.5',
              activeBlockers > 0 ? 'text-orange-500/80' : 'text-muted-foreground'
            )}
          />
        }
        tone={activeBlockers > 0 ? 'warning' : 'default'}
      />
      <StatChip
        label="Shipped"
        value={`${shippedCount}`}
        hint={`${totalReleases} total`}
        tooltip="How many release records are marked shipped."
        icon={<CheckCircle2 className="size-4 text-emerald-500" />}
        tone={shippedCount > 0 ? 'success' : 'default'}
      />
      <StatChip
        label="Active Targets"
        value={`${activeTargetFeatureCount}`}
        tooltip="Total features currently targeted to a release."
        icon={<Rocket className="size-4 text-sky-500" />}
      />
      <StatChip
        label="Avg Progress"
        value={`${avgProgress}%`}
        tooltip="Average launch readiness across release records."
        icon={<Gauge className="size-4 text-emerald-500/70" />}
      />
      <StatChip
        label="Coverage"
        value={`${activeTargetReleaseCount}/${Math.max(totalReleases, 1)}`}
        hint="targets covered"
        tooltip="Releases that have at least one active target."
        icon={<Link2 className="size-3.5 text-primary" />}
      />
    </div>
  );
}
