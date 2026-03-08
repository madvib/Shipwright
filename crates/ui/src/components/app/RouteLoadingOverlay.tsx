import { cn } from '@/lib/utils';
import ShipMark from '@/components/app/ShipMark';

interface RouteLoadingOverlayProps {
  visible: boolean;
  label?: string;
}

export default function RouteLoadingOverlay({
  visible,
  label = 'Compiling context...',
}: RouteLoadingOverlayProps) {
  return (
    <div
      className={cn(
        'pointer-events-none absolute inset-0 z-40 flex items-center justify-center bg-background/65 backdrop-blur-[2px] transition-opacity duration-200',
        visible ? 'opacity-100' : 'opacity-0'
      )}
      aria-hidden={!visible}
    >
      <div className="relative flex flex-col items-center gap-3">
        <div className="relative flex h-16 w-16 items-center justify-center">
          <span className="absolute inset-0 rounded-full border border-primary/35 animate-ping" />
          <span className="absolute inset-1 rounded-full border border-primary/45" />
          <ShipMark className="relative h-9 w-9" spinning />
        </div>
        <p className="text-[11px] font-medium tracking-wide text-muted-foreground">{label}</p>
      </div>
    </div>
  );
}
