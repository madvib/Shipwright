import { ReactNode } from 'react';
import { CardDescription, CardTitle } from '@ship/ui';

interface HubSectionHeaderProps {
  title?: string;
  description?: ReactNode;
  controls: ReactNode;
}

export default function HubSectionHeader({
  title,
  description,
  controls,
}: HubSectionHeaderProps) {
  const hasHeader = !!title || !!description;

  if (!hasHeader) {
    return <div className="flex justify-end">{controls}</div>;
  }

  return (
    <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-start gap-3">
      <div />
      <div className="min-w-0 text-center">
        {title && <CardTitle className="text-sm">{title}</CardTitle>}
        {description && <CardDescription className="text-xs">{description}</CardDescription>}
      </div>
      <div className="flex min-w-0 justify-end">{controls}</div>
    </div>
  );
}
