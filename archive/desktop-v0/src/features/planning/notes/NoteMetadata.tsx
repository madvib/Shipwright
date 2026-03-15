import { Calendar as CalendarIcon, Tag } from 'lucide-react';
import {
    Badge,
    FacetedFilter,
    readFrontmatterSummary,
} from '@ship/primitives';
import { formatDate } from '@/lib/date';
import { BaseMetadataHeader } from '../common/BaseMetadataHeader';
import { MetadataPopover } from '../common/MetadataPopover';

interface NoteMetadataProps {
    frontmatter: string | null;
    updated: string;
    tagSuggestions: string[];
    isEditing: boolean;
    onChange: (tags: string[]) => void;
    onNavigate?: (id: string) => void;
}

export function NoteMetadata({
    frontmatter,
    updated,
    tagSuggestions,
    isEditing,
    onChange,
}: NoteMetadataProps) {
    const summary = readFrontmatterSummary(frontmatter);
    const { tags = [] } = summary;

    const handleTagChange = (nextTags: string[]) => {
        onChange(nextTags);
    };

    return (
        <BaseMetadataHeader>
            <div className="flex items-center gap-1.5 shrink-0">
                <CalendarIcon className="size-3.5" />
                <span>{formatDate(updated)}</span>
            </div>



            {/* Tags Popover - Hide if empty unless editing */}
            {(tags.length > 0 || isEditing) && (
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
            )}
        </BaseMetadataHeader>
    );
}
