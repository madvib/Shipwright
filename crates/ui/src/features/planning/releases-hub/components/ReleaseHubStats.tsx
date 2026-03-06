import { ReactNode } from 'react';
import { AlertTriangle, CheckCircle2, Gauge, Link2, Rocket } from 'lucide-react';
import { ReleaseInfo as ReleaseEntry } from '@/bindings';
import { Badge } from '@ship/ui';
import { cn } from '@/lib/utils';

interface ReleaseHubStatsProps {
  activeRelease: ReleaseEntry | null;
  activeBlockers: number;
  shippedCount: number;
  totalReleases: number;
  linkedFeatureCount: number;
  avgProgress: number;
}

interface StatChipProps {
  label: string;
  value: string;
  hint?: string;
  icon?: ReactNode;
  tone?: 'default' | 'warning' | 'success';
}

function StatChip({ label, value, hint, icon, tone = 'default' }: StatChipProps) {
  return (
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
}

export default function ReleaseHubStats({
  activeRelease,
  activeBlockers,
  shippedCount,
  totalReleases,
  linkedFeatureCount,
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
        icon={<CheckCircle2 className="size-4 text-emerald-500" />}
        tone={shippedCount > 0 ? 'success' : 'default'}
      />
      <StatChip
        label="Linked Features"
        value={`${linkedFeatureCount}`}
        icon={<Rocket className="size-4 text-sky-500" />}
      />
      <StatChip
        label="Avg Progress"
        value={`${avgProgress}%`}
        icon={<Gauge className="size-4 text-emerald-500/70" />}
      />
      <StatChip
        label="Coverage"
        value={`${linkedFeatureCount}/${Math.max(totalReleases, 1)}`}
        hint="feature links/release count"
        icon={<Link2 className="size-3.5 text-primary" />}
      />
    </div>
  );
}
