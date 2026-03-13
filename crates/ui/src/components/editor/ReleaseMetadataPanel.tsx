import { useMemo, useState } from 'react';
import { Check, Plus, X } from 'lucide-react';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { DatePicker } from '@ship/ui';
import { Input } from '@ship/ui';
import { Switch } from '@ship/ui';
import { AutocompleteInput } from '@ship/ui';
import {
  FrontmatterDelimiter,
  readFrontmatterBooleanField,
  readFrontmatterStringField,
  readFrontmatterStringListField,
  setFrontmatterBooleanField,
  setFrontmatterStringField,
  setFrontmatterStringListField,
} from '@ship/ui';

const RELEASE_STATUSES = ['planned', 'active', 'shipped', 'archived'];

interface ReleaseMetadataPanelProps {
  frontmatter: string | null;
  delimiter: FrontmatterDelimiter | null;
  defaultVersion: string;
  defaultStatus?: string;
  tagSuggestions?: string[];
  onChange: (frontmatter: string | null, delimiter: FrontmatterDelimiter) => void;
}

function createStarterMetadata(
  delimiter: FrontmatterDelimiter,
  version: string,
  status: string
): string {
  if (delimiter === '---') {
    return `version: "${version}"\nstatus: "${status}"\nsupported: false\ntags: []`;
  }
  return `version = "${version}"\nstatus = "${status}"\nsupported = false\ntags = []`;
}

export default function ReleaseMetadataPanel({
  frontmatter,
  delimiter,
  defaultVersion,
  defaultStatus = 'planned',
  tagSuggestions = [],
  onChange,
}: ReleaseMetadataPanelProps) {
  const [tagInput, setTagInput] = useState('');
  const [tagInputOpen, setTagInputOpen] = useState(false);
  const currentDelimiter: FrontmatterDelimiter = delimiter ?? '+++';

  const effectiveFrontmatter = frontmatter ?? createStarterMetadata(currentDelimiter, defaultVersion, defaultStatus);
  const version = readFrontmatterStringField(effectiveFrontmatter, 'version') || defaultVersion;
  const status = readFrontmatterStringField(effectiveFrontmatter, 'status') || defaultStatus;
  const supported = readFrontmatterBooleanField(effectiveFrontmatter, 'supported') ?? false;
  const targetDate = readFrontmatterStringField(effectiveFrontmatter, 'target_date');
  const tags = readFrontmatterStringListField(effectiveFrontmatter, 'tags');

  const statusOptions = useMemo(() => {
    if (!status || RELEASE_STATUSES.includes(status)) return RELEASE_STATUSES;
    return [status, ...RELEASE_STATUSES];
  }, [status]);

  const availableTagOptions = tagSuggestions
    .filter((tag) => !tags.includes(tag))
    .map((tag) => ({ value: tag }));

  const commit = (nextFrontmatter: string | null) => onChange(nextFrontmatter, currentDelimiter);

  const updateVersion = (next: string) => {
    commit(setFrontmatterStringField(effectiveFrontmatter, 'version', next, currentDelimiter));
  };

  const updateStatus = (next: string) => {
    commit(setFrontmatterStringField(effectiveFrontmatter, 'status', next, currentDelimiter));
  };

  const updateSupported = (next: boolean) => {
    commit(setFrontmatterBooleanField(effectiveFrontmatter, 'supported', next, currentDelimiter));
  };

  const updateTargetDate = (next: string) => {
    commit(setFrontmatterStringField(effectiveFrontmatter, 'target_date', next, currentDelimiter));
  };

  const updateTags = (nextTags: string[]) => {
    commit(setFrontmatterStringListField(effectiveFrontmatter, 'tags', nextTags, currentDelimiter));
  };

  const addTag = (valueOverride?: string) => {
    const clean = (valueOverride ?? tagInput).trim();
    if (!clean || tags.includes(clean)) return;
    updateTags([...tags, clean]);
    setTagInput('');
  };

  return (
    <section className="rounded-lg border border-border/40 bg-muted/20 px-4 py-3 shadow-none">
      <div className="mb-3 flex items-center justify-between border-b border-border/30 pb-2">
        <div className="flex items-center gap-2">
          <div className="size-1.5 rounded-full bg-primary/60" />
          <h3 className="text-[11px] font-bold uppercase tracking-wider text-muted-foreground/80">Properties</h3>
        </div>
      </div>

      <div className="grid gap-x-8 gap-y-4 md:grid-cols-2 lg:grid-cols-3">
        {/* Release Details */}
        <div className="space-y-3.5">
          <div className="space-y-1">
            <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">Version</label>
            <Input
              value={version}
              className="h-8 border-border/40 bg-background/30 text-xs transition-colors hover:bg-background/50 focus:bg-background"
              placeholder="v0.1.1-alpha"
              onChange={(event) => updateVersion(event.target.value)}
            />
          </div>

          <div className="space-y-1">
            <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">Status</label>
            <AutocompleteInput
              value={status}
              options={statusOptions.map((value) => ({ value }))}
              placeholder="Select status"
              className="h-8 border-border/40 bg-background/30 text-xs transition-colors hover:bg-background/50 focus:bg-background"
              noResultsText="No matching status."
              onValueChange={updateStatus}
            />
          </div>
        </div>

        {/* Schedule & Support */}
        <div className="space-y-3.5">
          <div className="space-y-1">
            <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">Target Date</label>
            <DatePicker
              value={targetDate}
              className="h-8 w-full border-border/40 bg-background/30 text-xs transition-colors hover:bg-background/50"
              onValueChange={updateTargetDate}
            />
          </div>

          <div className="space-y-1">
            <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">Maintenance</label>
            <div className="flex h-8 items-center justify-between rounded-md border border-border/40 bg-background/30 px-3 transition-colors hover:bg-background/50">
              <span className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">Active Support</span>
              <Switch checked={supported} onCheckedChange={updateSupported} />
            </div>
          </div>
        </div>

        {/* Tags Section */}
        <div className="space-y-1 md:col-span-2 lg:col-span-1">
          <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">Tags</label>
          <div className="flex min-h-[72px] flex-wrap content-start gap-1.5 rounded-md border border-border/30 bg-background/20 p-2 shadow-inner transition-colors focus-within:bg-background/40">
            {tags.map((tag) => (
              <Badge key={tag} variant="secondary" className="h-5 gap-1 rounded-sm bg-primary/10 px-1.5 text-[10px] font-medium text-primary hover:bg-primary/20">
                {tag}
                <button
                  type="button"
                  className="text-primary/40 hover:text-primary"
                  onClick={() => updateTags(tags.filter((value) => value !== tag))}
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
                <Button variant="ghost" size="icon-xs" className="size-5 shrink-0" onClick={() => { addTag(); setTagInputOpen(false); }}>
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
