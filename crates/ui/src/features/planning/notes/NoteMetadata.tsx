import { Calendar as CalendarIcon, Shapes, Tag } from 'lucide-react';
import {
    Badge,
    FacetedFilter,
    readFrontmatterSummary,
} from '@ship/ui';
import { formatDate } from '@/lib/date';
import { BaseMetadataHeader } from '../common/BaseMetadataHeader';
import { MetadataPopover } from '../common/MetadataPopover';

interface NoteMetadataProps {
    frontmatter: string | null;
    updated: string;
    specSuggestions: { id: string; title: string }[];
    tagSuggestions: string[];
    isEditing: boolean;
    onChange: (specs: string[], tags: string[]) => void;
    onNavigate?: (id: string) => void;
}

export function NoteMetadata({
    frontmatter,
    updated,
    specSuggestions,
    tagSuggestions,
    isEditing,
    onChange,
    onNavigate,
}: NoteMetadataProps) {
    const summary = readFrontmatterSummary(frontmatter);
    const { specs = [], tags = [] } = summary;


    const handleSpecChange = (nextSpecs: string[]) => {
        onChange(nextSpecs, tags);
    };

    const handleTagChange = (nextTags: string[]) => {
        onChange(specs, nextTags);
    };

    return (
        <BaseMetadataHeader>
            <div className="flex items-center gap-1.5 shrink-0">
                <CalendarIcon className="size-3.5" />
                <span>{formatDate(updated)}</span>
            </div>

            {/* Specs Popover */}
            <MetadataPopover
                icon={Shapes}
                label={specs.length > 0
                    ? `${specs.length} Spec${specs.length === 1 ? '' : 's'}`
                    : 'No linked specs'}
                title="Linked Specs"
                triggerClassName="max-w-[300px]"
            >
                <div className="flex flex-wrap gap-1.5 min-h-[20px]">
                    {specs.map((id) => {
                        const title = specSuggestions.find(s => s.id === id)?.title || id;
                        return (
                            <Badge
                                key={id}
                                variant="secondary"
                                className="h-5 px-1.5 text-[10px] font-normal cursor-pointer hover:bg-secondary/80"
                                onClick={() => onNavigate?.(id)}
                                title={`View ${title}`}
                            >
                                {title}
                            </Badge>
                        );
                    })}
                    {specs.length === 0 && (
                        <span className="text-xs text-muted-foreground italic">No linked specs</span>
                    )}
                </div>

                {isEditing && (
                    <FacetedFilter
                        title="Link Specs"
                        options={specSuggestions.map(s => ({ value: s.id, label: s.title }))}
                        selectedValues={specs}
                        onSelectionChange={handleSpecChange}
                    />
                )}
            </MetadataPopover>

            {/* Tags Popover */}
            <MetadataPopover
                icon={Tag}
                label={tags.length > 0 ? `${tags.length} Tag${tags.length === 1 ? '' : 's'}` : 'No tags'}
                title="Tags"
                triggerClassName="shrink-0"
                contentClassName="w-64 p-3"
            >
                <div className="flex flex-wrap gap-1.5">
                    {tags.map((tag) => (
                        <Badge key={tag} variant="secondary" className="h-5 px-1.5 text-[10px] font-normal uppercase">
                            {tag}
                        </Badge>
                    ))}
                    {tags.length === 0 && (
                        <span className="text-xs text-muted-foreground italic">No tags</span>
                    )}
                </div>
                {isEditing && (
                    <FacetedFilter
                        title="Edit Tags"
                        options={tagSuggestions.map(t => ({ value: t, label: t }))}
                        selectedValues={tags}
                        onSelectionChange={handleTagChange}
                        allowNew
                        onAddNew={(tag) => handleTagChange([...tags, tag])}
                    />
                )}
            </MetadataPopover>
        </BaseMetadataHeader>
    );
}
