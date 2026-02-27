import { ReactNode, useMemo, useState } from 'react';
import { Sparkles } from 'lucide-react';
import { ADR } from '@/bindings';
import MarkdownEditor from '@/components/editor';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import DatePicker from '@/components/ui/date-picker';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import AutocompleteInput from '@/components/ui/autocomplete-input';

const ADR_STATUSES = ['proposed', 'accepted', 'rejected', 'superseded', 'deprecated'];

interface AdrEditorProps {
  adr: ADR;
  onChange: (next: ADR) => void;
  specSuggestions: string[];
  tagSuggestions: string[];
  placeholder?: string;
  onInsertTemplate?: () => void | Promise<void>;
  onMcpSample?: () => Promise<string | null | undefined> | string | null | undefined;
  sampleLabel?: string;
  sampleRequiresMcp?: boolean;
  mcpEnabled?: boolean;
  extraActions?: ReactNode;
}

export default function AdrEditor({
  adr,
  onChange,
  specSuggestions,
  tagSuggestions,
  placeholder = 'Describe this decision, trade-offs, and consequences...',
  onInsertTemplate,
  onMcpSample,
  sampleLabel,
  sampleRequiresMcp,
  mcpEnabled,
  extraActions,
}: AdrEditorProps) {
  const [tagInput, setTagInput] = useState('');

  const statusOptions = useMemo(() => {
    const current = adr.metadata.status;
    if (!current || ADR_STATUSES.includes(current)) return ADR_STATUSES;
    return [current, ...ADR_STATUSES];
  }, [adr.metadata.status]);

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

  const toolbarControls = (
    <>
      <Input
        value={adr.metadata.title}
        onChange={(event) => updateMetadata({ title: event.target.value })}
        className="h-7 w-[220px] shrink-0 text-xs"
        placeholder="Title"
      />

      <div className="w-[108px] shrink-0">
        <Select
          value={adr.metadata.status}
          onValueChange={(status) => {
            if (!status || status === adr.metadata.status) return;
            updateMetadata({ status });
          }}
        >
          <SelectTrigger className="h-7 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {statusOptions.map((status) => (
              <SelectItem key={status} value={status}>
                {status}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <DatePicker value={adr.metadata.date} onValueChange={(date) => updateMetadata({ date })} />

      <AutocompleteInput
        value={adr.metadata.spec ?? ''}
        options={specSuggestions.map((spec) => ({ value: spec }))}
        className="h-7 w-[140px] shrink-0 text-xs"
        placeholder="Spec"
        onValueChange={(spec) => updateMetadata({ spec: spec.trim() ? spec.trim() : null })}
      />

      <DropdownMenu>
        <DropdownMenuTrigger
          render={<Button size="xs" variant="outline" className="h-7 px-2 text-xs" />}
        >
          Tags {(adr.metadata.tags ?? []).length ? `(${(adr.metadata.tags ?? []).length})` : ''}
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-72 p-2">
          <div className="space-y-2">
            <div className="flex flex-wrap gap-1.5">
              {(adr.metadata.tags ?? []).length === 0 ? (
                <span className="text-muted-foreground text-xs">No tags yet.</span>
              ) : (
                (adr.metadata.tags ?? []).map((tag) => (
                  <button
                    key={tag}
                    type="button"
                    className="bg-muted hover:bg-muted/80 rounded px-2 py-1 text-xs"
                    onClick={() => removeTag(tag)}
                  >
                    {tag} ×
                  </button>
                ))
              )}
            </div>
            <AutocompleteInput
              value={tagInput}
              options={tagSuggestions
                .filter((tag) => !(adr.metadata.tags ?? []).includes(tag))
                .map((tag) => ({ value: tag }))}
              className="h-8 text-xs"
              placeholder="Add tag"
              noResultsText="No tag suggestions."
              onCommit={(value) => addTag(value)}
              onValueChange={setTagInput}
            />
          </div>
        </DropdownMenuContent>
      </DropdownMenu>

      {onInsertTemplate && (
        <Button size="xs" variant="outline" className="h-7 px-2 text-xs" onClick={() => void onInsertTemplate()}>
          <Sparkles className="size-3.5" />
          Insert Template
        </Button>
      )}

      {extraActions}
    </>
  );

  return (
    <MarkdownEditor
      label={undefined}
      toolbarStart={toolbarControls}
      showStats={false}
      value={adr.body}
      onChange={(body) => onChange({ ...adr, body })}
      placeholder={placeholder}
      rows={16}
      defaultMode="doc"
      fillHeight
      mcpEnabled={mcpEnabled}
      sampleInline={!!onMcpSample}
      sampleLabel={sampleLabel}
      sampleRequiresMcp={sampleRequiresMcp}
      onMcpSample={onMcpSample}
    />
  );
}
