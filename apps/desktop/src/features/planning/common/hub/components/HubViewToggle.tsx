import { Button } from '@ship/primitives';

interface HubViewToggleProps<TValue extends string> {
  value: TValue;
  options: ReadonlyArray<TValue>;
  onChange: (value: TValue) => void;
}

export default function HubViewToggle<TValue extends string>({
  value,
  options,
  onChange,
}: HubViewToggleProps<TValue>) {
  return (
    <div className="flex items-center rounded-md border bg-background/40 p-0.5">
      {options.map((option) => (
        <Button
          key={option}
          size="xs"
          variant={value === option ? 'secondary' : 'ghost'}
          className="h-7 px-2 capitalize"
          onClick={() => onChange(option)}
        >
          {option}
        </Button>
      ))}
    </div>
  );
}
