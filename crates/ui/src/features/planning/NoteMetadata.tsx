import { useState } from 'react';
import { Calendar as CalendarIcon, Shapes, Tag } from 'lucide-react';
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
    Button,
    Badge,
    FacetedFilter,
    readFrontmatterSummary,
} from '@ship/ui';
import { formatDate } from '@/lib/date';
import { cn } from '@/lib/utils';

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

    const [tagInput, setTagInput] = useState('');

    const handleSpecChange = (nextSpecs: string[]) => {
        onChange(nextSpecs, tags);
    };

    const handleTagChange = (nextTags: string[]) => {
        onChange(specs, nextTags);
    };

    return (
        <div className="flex flex-nowrap items-center gap-4 text-xs text-muted-foreground overflow-hidden">
            <div className="flex items-center gap-1.5 shrink-0">
                <CalendarIcon className="size-3.5" />
                <span>{formatDate(updated)}</span>
            </div>

            <span className="text-muted-foreground/30 px-1">|</span>

            {/* Specs Popover */}
            <Popover>
                <PopoverTrigger
                    render={
                        <Button
                            variant="ghost"
                            size="xs"
                            className="h-7 gap-1.5 px-2 text-xs font-normal text-muted-foreground hover:bg-accent/50 max-w-[300px]"
                        />
                    }
                >
                    <Shapes className="size-3.5 shrink-0" />
                    <span className="truncate">
                        {specs.length > 0
                            ? `${specs.length} Spec${specs.length === 1 ? '' : 's'}`
                            : 'No linked specs'}
                    </span>
                </PopoverTrigger>
                <PopoverContent align="start" className="w-80 p-2 text-left">
                    <div className="space-y-3">
                        <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground">Linked Specs</p>

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
                                className="w-full"
                            />
                        )}
                    </div>
                </PopoverContent>
            </Popover>

            <span className="text-muted-foreground/30 px-1">|</span>

            {/* Tags Popover */}
            <Popover>
                <PopoverTrigger
                    render={
                        <Button
                            variant="ghost"
                            size="xs"
                            className="h-7 gap-1.5 px-2 text-xs font-normal text-muted-foreground hover:bg-accent/50 shrink-0"
                        />
                    }
                >
                    <Tag className="size-3.5" />
                    {tags.length > 0 ? `${tags.length} Tag${tags.length === 1 ? '' : 's'}` : 'No tags'}
                </PopoverTrigger>
                <PopoverContent align="start" className="w-64 p-3 text-left">
                    <div className="space-y-3">
                        <p className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground">Tags</p>
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
                                className="w-full"
                            />
                        )}
                    </div>
                </PopoverContent>
            </Popover>
        </div>
    );
}
