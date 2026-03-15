import { cn } from '@/lib/utils';

interface ShipMarkProps {
  className?: string;
  spinning?: boolean;
  alt?: string;
}

export default function ShipMark({
  className,
  spinning = false,
  alt = 'Ship',
}: ShipMarkProps) {
  const classes = cn(
    className,
    spinning && 'animate-[spin_1.8s_linear_infinite]'
  );

  return (
    <>
      <img src="/ship_logo.svg" alt={alt} className={cn('dark:hidden', classes)} />
      <img
        src="/ship_logo_secondary.svg"
        alt={alt}
        className={cn('hidden dark:block', classes)}
      />
    </>
  );
}

