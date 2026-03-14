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
  disabled?: boolean;
  onToggleItem?: (itemIndex: number) => void;
}

export function FeatureChecklistSection({
  title,
  items,
  emptyLabel,
  disabled = false,
  onToggleItem,
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
          {items.map((item, index) => {
            const row = (
              <>
                {item.done ? (
                  <CheckCircle2 className="mt-0.5 size-4 text-emerald-600" />
                ) : (
                  <Circle className="text-muted-foreground mt-0.5 size-4" />
                )}
                <span
                  className={cn(
                    'text-left leading-5',
                    item.done ? 'text-muted-foreground line-through' : 'text-foreground'
                  )}
                >
                  {item.text}
                </span>
              </>
            );

            return (
              <li key={item.id} className="text-sm">
                {onToggleItem ? (
                  <button
                    type="button"
                    className="hover:bg-accent/40 focus-visible:ring-ring/50 flex w-full items-start gap-2 rounded-sm px-1 py-0.5 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 disabled:cursor-not-allowed disabled:opacity-50"
                    onClick={() => onToggleItem(index)}
                    disabled={disabled}
                    aria-label={`${item.done ? 'Mark incomplete' : 'Mark complete'}: ${item.text}`}
                  >
                    {row}
                  </button>
                ) : (
                  <div className="flex items-start gap-2">{row}</div>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}
