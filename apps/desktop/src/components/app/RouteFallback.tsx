import ShipMark from '@/components/app/ShipMark';

interface RouteFallbackProps {
  label?: string;
}

export default function RouteFallback({ label = 'Loading view...' }: RouteFallbackProps) {
  return (
    <div className="flex h-full min-h-[260px] items-center justify-center p-8">
      <div className="flex flex-col items-center gap-3">
        <div className="relative flex h-14 w-14 items-center justify-center">
          <span className="absolute inset-0 rounded-full border border-primary/30 animate-ping" />
          <span className="absolute inset-1 rounded-full border border-primary/40" />
          <ShipMark className="relative h-8 w-8 animate-[spin_1.6s_linear_infinite]" />
        </div>
        <span className="text-xs text-muted-foreground">{label}</span>
      </div>
    </div>
  );
}
