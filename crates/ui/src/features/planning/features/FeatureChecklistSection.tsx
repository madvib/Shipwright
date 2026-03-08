import { CheckCircle2, Circle } from 'lucide-react';
import { cn } from '@/lib/utils';

interface ChecklistItem {
  id: string;
  text: string;
  done: boolean;
}

interface FeatureChecklistSectionProps {
  title: string;
  items: ChecklistItem[];
  emptyLabel: string;
}

export function FeatureChecklistSection({
  title,
  items,
  emptyLabel,
}: FeatureChecklistSectionProps) {
  return (
    <section className="space-y-2 rounded-md border bg-card px-3 py-2">
      <div className="flex items-center justify-between">
        <h4 className="text-sm font-medium">{title}</h4>
        <span className="text-muted-foreground text-xs">
          {items.filter((item) => item.done).length}/{items.length}
        </span>
      </div>
      {items.length === 0 ? (
        <p className="text-muted-foreground text-xs italic">{emptyLabel}</p>
      ) : (
        <ul className="space-y-1.5">
          {items.map((item) => (
            <li key={item.id} className="flex items-start gap-2 text-sm">
              {item.done ? (
                <CheckCircle2 className="mt-0.5 size-4 text-emerald-600" />
              ) : (
                <Circle className="text-muted-foreground mt-0.5 size-4" />
              )}
              <span
                className={cn(
                  'leading-5',
                  item.done ? 'text-muted-foreground line-through' : 'text-foreground'
                )}
              >
                {item.text}
              </span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
