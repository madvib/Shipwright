import { KeyboardEvent, useEffect, useMemo, useState } from 'react';
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
  syncOnInput?: boolean;
  autoCapitalize?: string;
  autoCorrect?: string;
  spellCheck?: boolean;
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
  maxResults = 150,
  allowCustom = true,
  syncOnInput = false,
  autoCapitalize,
  autoCorrect,
  spellCheck,
  onValueChange,
  onCommit,
}: AutocompleteInputProps) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState(value);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  const filtered = useMemo(() => {
    const query = normalize(draft);
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
  }, [draft, maxResults, options]);

  const exactMatch = useMemo(() => {
    const query = normalize(draft);
    if (!query) return null;
    return options.find((option) => normalize(option.value) === query) ?? null;
  }, [draft, options]);

  const selectOption = (option: AutocompleteOption) => {
    setDraft(option.value);
    onValueChange(option.value);
    onCommit?.(option.value);
    setOpen(false);
  };

  const commitCurrentValue = (rawDraft?: string) => {
    const next = (rawDraft ?? draft).trim();
    if (!next) {
      setDraft(value);
      setOpen(false);
      return;
    }
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
      inputValue={draft}
      open={open}
      autoHighlight
      onOpenChange={setOpen}
      onInputValueChange={(nextValue) => {
        setDraft(nextValue);
        if (syncOnInput) onValueChange(nextValue);
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
        autoCapitalize={autoCapitalize}
        autoCorrect={autoCorrect}
        spellCheck={spellCheck}
        showTrigger={false}
        onFocus={() => setOpen(true)}
        onBlur={(event) => {
          if (syncOnInput) {
            setOpen(false);
            return;
          }
          commitCurrentValue(event.currentTarget.value);
        }}
        onKeyDown={onKeyDown}
      />
      <ComboboxContent className="w-[min(48rem,calc(100vw-2rem))] min-w-[26rem] max-w-[calc(100vw-2rem)]">
        <ComboboxEmpty>{noResultsText}</ComboboxEmpty>
        <ComboboxList>
          {(option: AutocompleteOption) => (
            <ComboboxItem
              key={option.value}
              value={option}
              onClick={() => selectOption(option)}
              className="items-start"
            >
              <span className="block break-words">{option.label ?? option.value}</span>
              {option.label && option.label !== option.value && (
                <span className="text-muted-foreground ml-2 break-all text-[11px]">{option.value}</span>
              )}
            </ComboboxItem>
          )}
        </ComboboxList>
      </ComboboxContent>
    </Combobox>
  );
}
