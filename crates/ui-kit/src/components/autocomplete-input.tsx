import { KeyboardEvent, useMemo, useState } from 'react';
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput as UiComboboxInput,
  ComboboxItem,
  ComboboxList,
} from './combobox';

export interface AutocompleteOption {
  value: string;
  label?: string;
  keywords?: string[];
}

interface AutocompleteInputProps {
  id?: string;
  value: string;
  options: AutocompleteOption[];
  placeholder?: string;
  autoFocus?: boolean;
  disabled?: boolean;
  className?: string;
  noResultsText?: string;
  maxResults?: number;
  allowCustom?: boolean;
  onValueChange: (value: string) => void;
  onCommit?: (value: string) => void;
}

function normalize(text: string | null | undefined): string {
  return (text || '').trim().toLowerCase();
}

export function AutocompleteInput({
  id,
  value,
  options,
  placeholder,
  autoFocus = false,
  disabled = false,
  className,
  noResultsText = 'No matches found.',
  maxResults = 8,
  allowCustom = true,
  onValueChange,
  onCommit,
}: AutocompleteInputProps) {
  const [open, setOpen] = useState(false);

  const filtered = useMemo(() => {
    const query = normalize(value);
    const seen = new Set<string>();
    const pool = options.filter((option) => {
      if (!option.value.trim()) return false;
      const key = option.value.trim();
      if (seen.has(key)) return false;
      seen.add(key);
      if (!query) return true;
      const haystacks = [option.value, option.label ?? '', ...(option.keywords ?? [])].map(normalize);
      return haystacks.some((haystack) => haystack.includes(query));
    });
    return pool.slice(0, maxResults);
  }, [maxResults, options, value]);

  const exactMatch = useMemo(() => {
    const query = normalize(value);
    if (!query) return null;
    return options.find((option) => normalize(option.value) === query) ?? null;
  }, [options, value]);

  const selectOption = (option: AutocompleteOption) => {
    onValueChange(option.value);
    onCommit?.(option.value);
    setOpen(false);
  };

  const commitCurrentValue = () => {
    const next = value.trim();
    if (!next) return;
    if (!allowCustom && !exactMatch) return;
    onValueChange(next);
    onCommit?.(next);
    setOpen(false);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      if (!onCommit) return;
      if (open && filtered.length > 0) return;
      event.preventDefault();
      commitCurrentValue();
    }
  };

  return (
    <Combobox<AutocompleteOption>
      items={options}
      filteredItems={filtered}
      inputValue={value}
      open={open}
      autoHighlight
      onOpenChange={setOpen}
      onInputValueChange={(nextValue) => {
        onValueChange(nextValue);
      }}
      itemToStringLabel={(option) => option.value}
      itemToStringValue={(option) => option.value}
    >
      <UiComboboxInput
        id={id}
        placeholder={placeholder}
        autoFocus={autoFocus}
        disabled={disabled}
        className={className}
        showTrigger={false}
        onFocus={() => setOpen(true)}
        onKeyDown={onKeyDown}
      />
      <ComboboxContent>
        <ComboboxEmpty>{noResultsText}</ComboboxEmpty>
        <ComboboxList>
          {(option: AutocompleteOption) => (
            <ComboboxItem
              key={option.value}
              value={option}
              onClick={() => selectOption(option)}
            >
              <span className="truncate">{option.label ?? option.value}</span>
              {option.label && option.label !== option.value && (
                <span className="text-muted-foreground ml-2 truncate text-[11px]">{option.value}</span>
              )}
            </ComboboxItem>
          )}
        </ComboboxList>
      </ComboboxContent>
    </Combobox>
  );
}
