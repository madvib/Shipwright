import { useMemo } from 'react';
import { FacetedFilter } from '@ship/primitives';
import { FieldLabel } from '@ship/primitives';
import { Input } from '@ship/primitives';
import {
  FrontmatterDelimiter,
  readFrontmatterStringField,
  readFrontmatterStringListField,
  setFrontmatterStringField,
  setFrontmatterStringListField,
} from '@ship/primitives';

const SPEC_STATUSES = ['draft', 'active', 'archived'];

interface SpecMetadataPanelProps {
  frontmatter: string | null;
  delimiter: FrontmatterDelimiter | null;
  defaultTitle: string;
  defaultStatus?: string;
  /** @deprecated tagSuggestions is no longer used - tags are managed via FacetedFilter */
  tagSuggestions?: string[];
  onChange: (frontmatter: string | null, delimiter: FrontmatterDelimiter) => void;
}

function createStarterMetadata(
  delimiter: FrontmatterDelimiter,
  title: string,
  status: string
): string {
  if (delimiter === '---') {
    return `title: "${title}"\nstatus: "${status}"\nauthor: ""\ntags: []`;
  }
  return `title = "${title}"\nstatus = "${status}"\nauthor = ""\ntags = []`;
}

export default function SpecMetadataPanel({
  frontmatter,
  delimiter,
  defaultTitle,
  defaultStatus = 'draft',
  tagSuggestions,
  onChange,
}: SpecMetadataPanelProps) {
  const currentDelimiter: FrontmatterDelimiter = delimiter ?? '+++';

  const effectiveFrontmatter = frontmatter ?? createStarterMetadata(currentDelimiter, defaultTitle, defaultStatus);
  const title = readFrontmatterStringField(effectiveFrontmatter, 'title') || defaultTitle;
  const status = readFrontmatterStringField(effectiveFrontmatter, 'status') || defaultStatus;
  const author = readFrontmatterStringField(effectiveFrontmatter, 'author');
  const tags = readFrontmatterStringListField(effectiveFrontmatter, 'tags');

  const statusOptions = useMemo(() => {
    if (!status || SPEC_STATUSES.includes(status)) return SPEC_STATUSES;
    return [status, ...SPEC_STATUSES];
  }, [status]);

  const commit = (nextFrontmatter: string | null) => onChange(nextFrontmatter, currentDelimiter);

  const updateField = (key: string, value: string) => {
    commit(setFrontmatterStringField(effectiveFrontmatter, key, value, currentDelimiter));
  };

  const updateTags = (nextTags: string[]) => {
    commit(setFrontmatterStringListField(effectiveFrontmatter, 'tags', nextTags, currentDelimiter));
  };

  return (
    <aside className="flex h-full min-h-0 flex-col overflow-y-auto border-l bg-muted/20">
      {/* Properties header */}
      <div className="flex items-center gap-2 border-b bg-card/50 px-4 py-3">
        <p className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground">Properties</p>
      </div>

      <div className="flex flex-col gap-4 p-4">
        {/* Title */}
        <div className="space-y-1.5">
          <FieldLabel>Title</FieldLabel>
          <Input
            value={title}
            className="h-8"
            placeholder="Spec title"
            onChange={(event) => updateField('title', event.target.value)}
          />
        </div>

        {/* Status */}
        <div className="space-y-1.5">
          <FieldLabel>Status</FieldLabel>
          <div className="flex flex-wrap gap-1.5">
            {statusOptions.map((s) => (
              <button
                key={s}
                type="button"
                onClick={() => updateField('status', s)}
                className={`rounded-md border px-2.5 py-1 text-xs font-medium transition-all ${status === s
                  ? 'border-primary/40 bg-primary/10 text-primary shadow-sm'
                  : 'border-border/50 text-muted-foreground hover:bg-muted/80'
                  }`}
              >
                {s}
              </button>
            ))}
          </div>
        </div>

        {/* Author */}
        <div className="space-y-1.5">
          <FieldLabel>Author</FieldLabel>
          <Input
            value={author}
            className="h-8"
            placeholder="Author"
            onChange={(event) => updateField('author', event.target.value)}
          />
        </div>

        {/* Tags */}
        <div className="space-y-1.5">
          <FieldLabel>Tags {tags.length ? `(${tags.length})` : ''}</FieldLabel>
          <FacetedFilter
            title="Add tag"
            options={tagSuggestions?.map((tag) => ({ label: tag, value: tag })) ?? []}
            selectedValues={tags}
            onSelectionChange={updateTags}
            allowNew
            onAddNew={(tag) => updateTags([...tags, tag])}
          />
        </div>
      </div>
    </aside>
  );
}
