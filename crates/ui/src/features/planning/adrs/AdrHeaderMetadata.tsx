import { GitBranch, Tag } from 'lucide-react';
import {
    DatePicker,
    AutocompleteInput,
    Badge,
    Button,
} from '@ship/ui';
import { useState } from 'react';
import { ADR } from '@/bindings';
import { BaseMetadataHeader } from '../common/BaseMetadataHeader';
import { MetadataPopover } from '../common/MetadataPopover';

interface AdrHeaderMetadataProps {
    adr: ADR;
    onChange: (next: ADR) => void;
    adrSuggestions: { id: string; title: string }[];
    tagSuggestions: string[];
    onNavigate?: (type: 'adr', id: string) => void;
}

export function AdrHeaderMetadata({
    adr,
    onChange,
    adrSuggestions,
    tagSuggestions,
    onNavigate,
}: AdrHeaderMetadataProps) {
    const { date, supersedes_id, tags = [] } = adr.metadata;

    const adrTitle = adrSuggestions.find(a => a.id === supersedes_id)?.title || supersedes_id;
    const [lineageInput, setLineageInput] = useState(supersedes_id || '');
    const [tagInput, setTagInput] = useState('');

    const updateMetadata = (updates: Partial<ADR['metadata']>) => {
        onChange({
            ...adr,
            metadata: {
                ...adr.metadata,
                ...updates,
            },
        });
    };

    return (
        <BaseMetadataHeader>
            {/* Date Picker */}
            <DatePicker
                value={date || ''}
                onValueChange={(next: string) => updateMetadata({ date: next })}
                className="h-7 w-auto border-none bg-transparent p-0 hover:bg-accent/50 text-muted-foreground transition-colors"
            />


            {/* Lineage Popover */}
            {(supersedes_id || lineageInput) && (
                <MetadataPopover
                    icon={GitBranch}
                    label={supersedes_id ? `Supersedes: ${adrTitle}` : 'No lineage link'}
                    title="Supersedes"
                    action={supersedes_id && onNavigate && (
                        <Button
                            variant="link"
                            size="xs"
                            className="h-auto p-0 text-[10px]"
                            onClick={() => onNavigate('adr', supersedes_id)}
                        >
                            View ADR
                        </Button>
                    )}
                >
                    <AutocompleteInput
                        value={lineageInput}
                        onValueChange={setLineageInput}
                        options={adrSuggestions.map((a) => ({ value: a.id, label: a.title }))}
                        placeholder="Search ADRs..."
                        onCommit={(val: string) => updateMetadata({ supersedes_id: val || null })}
                    />
                </MetadataPopover>
            )}

            {/* Tags Popover */}
            <MetadataPopover
                icon={Tag}
                label={tags.length > 0 ? `${tags.length} Tag${tags.length === 1 ? '' : 's'}` : 'Add Tags'}
                title="Tags"
                triggerClassName="shrink-0"
                contentClassName="w-64 p-3"
            >
                <div className="flex flex-wrap gap-1.5">
                    {tags.map((tag) => (
                        <Badge key={tag} variant="secondary" className="h-5 px-1.5 text-[10px] font-normal uppercase">
                            {tag}
                            <button
                                className="ml-1 hover:text-destructive"
                                onClick={() => updateMetadata({ tags: tags.filter((t) => t !== tag) })}
                            >
                                ×
                            </button>
                        </Badge>
                    ))}
                    {tags.length === 0 && (
                        <span className="text-xs text-muted-foreground italic">No tags</span>
                    )}
                </div>
                <AutocompleteInput
                    value={tagInput}
                    onValueChange={setTagInput}
                    options={tagSuggestions.filter(t => !tags.includes(t)).map((t) => ({ value: t }))}
                    placeholder="Add tag..."
                    onCommit={(val: string) => {
                        if (val && !tags.includes(val)) {
                            updateMetadata({ tags: [...tags, val] });
                            setTagInput('');
                        }
                    }}
                />
            </MetadataPopover>
        </BaseMetadataHeader>
    );
}
