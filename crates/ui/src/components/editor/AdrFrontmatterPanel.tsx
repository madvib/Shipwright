import { useState } from 'react';
import { Check, Plus, X } from 'lucide-react';
import { ADR } from '@/bindings';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { DatePicker } from '@ship/ui';
import { AutocompleteInput } from '@ship/ui';

interface AdrFrontmatterPanelProps {
  adr: ADR;
  specSuggestions: string[];
  tagSuggestions: string[];
  adrSuggestions: string[];
  onChange: (next: ADR) => void;
}

export default function AdrFrontmatterPanel({
  adr,
  specSuggestions,
  tagSuggestions,
  adrSuggestions,
  onChange,
}: AdrFrontmatterPanelProps) {
  const [tagInput, setTagInput] = useState('');
  const [tagInputOpen, setTagInputOpen] = useState(false);

  const updateMetadata = (patch: Partial<ADR['metadata']>) => {
    onChange({
      ...adr,
      metadata: {
        ...adr.metadata,
        ...patch,
      },
    });
  };

  const addTag = (valueOverride?: string) => {
    const cleaned = (valueOverride ?? tagInput).trim();
    if (!cleaned || (adr.metadata.tags ?? []).includes(cleaned)) return;
    updateMetadata({ tags: [...(adr.metadata.tags ?? []), cleaned] });
    setTagInput('');
  };

  const removeTag = (tag: string) => {
    updateMetadata({ tags: (adr.metadata.tags ?? []).filter((value) => value !== tag) });
  };

  const availableTagOptions = tagSuggestions
    .filter((tag) => !(adr.metadata.tags ?? []).includes(tag))
    .map((tag) => ({ value: tag }));

  return (
    <section className="rounded-lg border border-border/40 bg-muted/20 px-4 py-3 shadow-none">
      <div className="mb-3 flex items-center justify-between border-b border-border/30 pb-2">
        <div className="flex items-center gap-2">
          <div className="size-1.5 rounded-full bg-primary/60" />
          <h3 className="text-[11px] font-bold uppercase tracking-wider text-muted-foreground/80">
            Properties
          </h3>
        </div>
        <Badge
          variant="outline"
          className="h-5 bg-background/50 px-1.5 font-mono text-[9px] font-medium text-muted-foreground/70"
        >
          {adr.metadata.id || 'pending'}
        </Badge>
      </div>

      <div className="grid gap-x-8 gap-y-4 md:grid-cols-2 lg:grid-cols-3">
        <div className="space-y-1">
          <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">
            Date
          </label>
          <DatePicker
            value={adr.metadata.date}
            className="h-8 w-full border-border/40 bg-background/30 text-xs transition-colors hover:bg-background/50"
            onValueChange={(date) => updateMetadata({ date })}
          />
        </div>

        <div className="space-y-1">
          <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">
            Reference Spec
          </label>
          <AutocompleteInput
            value={adr.metadata.spec_id || ''}
            options={specSuggestions.map((spec) => ({ value: spec }))}
            className="h-8 border-border/40 bg-background/30 text-xs transition-colors hover:bg-background/50"
            placeholder="None"
            noResultsText="No specs found."
            onValueChange={(spec) => updateMetadata({ spec_id: (spec || '').trim() || null })}
          />
        </div>

        <div className="space-y-1">
          <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">
            Supersedes
          </label>
          <AutocompleteInput
            value={adr.metadata.supersedes_id || ''}
            options={adrSuggestions.map((id) => ({ value: id }))}
            className="h-8 border-border/40 bg-background/30 text-xs transition-colors hover:bg-background/50"
            placeholder="None"
            noResultsText="No ADRs found."
            onValueChange={(id) => updateMetadata({ supersedes_id: (id || '').trim() || null })}
          />
        </div>

        <div className="space-y-1 md:col-span-2 lg:col-span-3">
          <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">
            Tags
          </label>
          <div className="flex min-h-[72px] flex-wrap content-start gap-1.5 rounded-md border border-border/30 bg-background/20 p-2 shadow-inner transition-colors focus-within:bg-background/40">
            {(adr.metadata.tags ?? []).map((tag: string) => (
              <Badge
                key={tag}
                variant="secondary"
                className="h-5 gap-1 rounded-sm bg-primary/10 px-1.5 text-[10px] font-medium text-primary hover:bg-primary/20"
              >
                {tag}
                <button
                  type="button"
                  className="text-primary/40 hover:text-primary"
                  onClick={() => removeTag(tag)}
                >
                  <X className="size-2.5" />
                </button>
              </Badge>
            ))}

            {tagInputOpen ? (
              <div className="flex w-full items-center gap-1">
                <AutocompleteInput
                  value={tagInput}
                  options={availableTagOptions}
                  className="h-6 min-w-[80px] flex-1 border-none bg-transparent text-[11px] shadow-none focus-visible:ring-0"
                  autoFocus
                  placeholder="..."
                  onCommit={(value) => {
                    addTag(value);
                    setTagInputOpen(false);
                  }}
                  onValueChange={setTagInput}
                />
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="size-5 shrink-0"
                  onClick={() => {
                    addTag();
                    setTagInputOpen(false);
                  }}
                >
                  <Check className="size-3" />
                </Button>
              </div>
            ) : (
              <Button
                variant="ghost"
                size="xs"
                className="h-5 px-1.5 text-[10px] text-muted-foreground hover:bg-background/50"
                onClick={() => setTagInputOpen(true)}
              >
                <Plus className="mr-1 size-2.5" />
                Add
              </Button>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}
