import { Plus, Trash2 } from 'lucide-react';
import { Button } from '@ship/primitives';
import { AutocompleteInput } from '@ship/primitives';

// ── PatternListEditor ────────────────────────────────────────────────────────

export interface PatternListEditorProps {
  patterns: string[];
  options: Array<{ value: string; label?: string; keywords?: string[] }>;
  addLabel: string;
  addValue?: string;
  noResultsText: string;
  onChange: (updater: (current: string[]) => string[]) => void;
}

export function PatternListEditor({
  patterns,
  options,
  addLabel,
  addValue = '',
  noResultsText,
  onChange,
}: PatternListEditorProps) {
  return (
    <div className="space-y-2">
      {patterns.map((pattern, idx) => (
        <div key={idx} className="flex items-center gap-2">
          <AutocompleteInput
            value={pattern || ''}
            options={options}
            noResultsText={noResultsText}
            onValueChange={(value) =>
              onChange((current) => current.map((item, itemIndex) => (itemIndex === idx ? value : item)))
            }
            className="font-mono text-xs"
            autoCapitalize="none"
            autoCorrect="off"
            spellCheck={false}
          />
          <Button
            variant="ghost"
            size="xs"
            onClick={() => onChange((current) => current.filter((_, index) => index !== idx))}
          >
            <Trash2 className="size-3.5" />
          </Button>
        </div>
      ))}
      <Button
        variant="outline"
        size="xs"
        className="w-full border-dashed"
        onClick={() => onChange((current) => [...current, addValue])}
      >
        <Plus className="mr-1 size-3.5" /> {addLabel}
      </Button>
    </div>
  );
}
