import { ReactNode } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@ship/primitives';
import { cn } from '@/lib/utils';

type HubMetricTone = 'default' | 'primary' | 'success' | 'warning';

const toneStyles: Record<HubMetricTone, string> = {
  default: '',
  primary: 'border-primary/20 bg-gradient-to-br from-primary/8 to-card',
  success: '',
  warning: '',
};

interface HubMetricCardProps {
  title: string;
  value: ReactNode;
  description: string;
  icon?: ReactNode;
  tone?: HubMetricTone;
}

export default function HubMetricCard({
  title,
  value,
  description,
  icon,
  tone = 'default',
}: HubMetricCardProps) {
  return (
    <Card size="sm" className={cn(toneStyles[tone])}>
      <CardHeader className="pb-2">
        <CardDescription className="text-[10px] uppercase tracking-wider">{title}</CardDescription>
        <CardTitle className="text-lg flex items-center gap-2">
          {icon}
          {value}
        </CardTitle>
      </CardHeader>
      <CardContent className="pt-0">
        <p className="text-muted-foreground text-xs">{description}</p>
      </CardContent>
    </Card>
  );
}
