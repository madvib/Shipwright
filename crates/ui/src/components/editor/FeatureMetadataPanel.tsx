import { useMemo, useState } from 'react';
import { Check, Plus, X } from 'lucide-react';
import { Badge } from '@ship/ui';
import { Button } from '@ship/ui';
import { Input } from '@ship/ui';
import { AutocompleteInput } from '@ship/ui';
import {
  FrontmatterDelimiter,
  readFrontmatterStringField,
  readFrontmatterStringListField,
  setFrontmatterStringField,
  setFrontmatterStringListField,
} from '@ship/ui';

const FEATURE_STATUSES = ['active', 'paused', 'complete', 'archived'];

interface FeatureMetadataPanelProps {
  frontmatter: string | null;
  delimiter: FrontmatterDelimiter | null;
  defaultTitle: string;
  defaultStatus?: string;
  releaseSuggestions?: string[];
  adrSuggestions?: string[];
  tagSuggestions?: string[];
  onChange: (frontmatter: string | null, delimiter: FrontmatterDelimiter) => void;
}

function createStarterMetadata(
  delimiter: FrontmatterDelimiter,
  title: string,
  status: string
): string {
  if (delimiter === '---') {
    return `title: "${title}"\nstatus: "${status}"\nrelease: ""\nadrs: []\ntags: []`;
  }
  return `title = "${title}"\nstatus = "${status}"\nrelease = ""\nadrs = []\ntags = []`;
}

export default function FeatureMetadataPanel({
  frontmatter,
  delimiter,
  defaultTitle,
  defaultStatus = 'active',
  releaseSuggestions = [],
  adrSuggestions = [],
  tagSuggestions = [],
  onChange,
}: FeatureMetadataPanelProps) {
  const [tagInput, setTagInput] = useState('');
  const [tagInputOpen, setTagInputOpen] = useState(false);
  const [adrInput, setAdrInput] = useState('');
  const [adrInputOpen, setAdrInputOpen] = useState(false);
  const currentDelimiter: FrontmatterDelimiter = delimiter ?? '+++';

  const effectiveFrontmatter = frontmatter ?? createStarterMetadata(currentDelimiter, defaultTitle, defaultStatus);
  const title = readFrontmatterStringField(effectiveFrontmatter, 'title') || defaultTitle;
  const status = readFrontmatterStringField(effectiveFrontmatter, 'status') || defaultStatus;
  const owner = readFrontmatterStringField(effectiveFrontmatter, 'owner');
  const branch = readFrontmatterStringField(effectiveFrontmatter, 'branch');
  const releaseId = readFrontmatterStringField(effectiveFrontmatter, 'release_id') || readFrontmatterStringField(effectiveFrontmatter, 'release');
  const adrs = readFrontmatterStringListField(effectiveFrontmatter, 'adrs');
  const tags = readFrontmatterStringListField(effectiveFrontmatter, 'tags');

  const statusOptions = useMemo(() => {
    if (!status || FEATURE_STATUSES.includes(status)) return FEATURE_STATUSES;
    return [status, ...FEATURE_STATUSES];
  }, [status]);

  const availableTagOptions = tagSuggestions
    .filter((tag) => !tags.includes(tag))
    .map((value) => ({ value }));
  const availableAdrOptions = adrSuggestions
    .filter((adr) => !adrs.includes(adr))
    .map((value) => ({ value }));

  const commit = (nextFrontmatter: string | null) => onChange(nextFrontmatter, currentDelimiter);

  const updateField = (key: string, value: string) => {
    commit(setFrontmatterStringField(effectiveFrontmatter, key, value, currentDelimiter));
  };

  const updateTags = (nextTags: string[]) => {
    commit(setFrontmatterStringListField(effectiveFrontmatter, 'tags', nextTags, currentDelimiter));
  };

  const updateAdrs = (nextAdrs: string[]) => {
    commit(setFrontmatterStringListField(effectiveFrontmatter, 'adrs', nextAdrs, currentDelimiter));
  };

  const addTag = (valueOverride?: string) => {
    const clean = (valueOverride ?? tagInput).trim();
    if (!clean || tags.includes(clean)) return;
    updateTags([...tags, clean]);
    setTagInput('');
  };

  const addAdr = (valueOverride?: string) => {
    const clean = (valueOverride ?? adrInput).trim();
    if (!clean || adrs.includes(clean)) return;
    updateAdrs([...adrs, clean]);
    setAdrInput('');
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
        {/* Basic Info */}
        <div className="space-y-3.5">
          <div className="space-y-1">
            <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">Title</label>
            <Input
              value={title}
              className="h-8 border-border/40 bg-background/30 text-xs transition-colors hover:bg-background/50 focus:bg-background"
              placeholder="Feature title"
              onChange={(event) => updateField('title', event.target.value)}
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
              onValueChange={(value) => updateField('status', value)}
            />
          </div>
        </div>

        {/* Relationships */}
        <div className="space-y-3.5">
          <div className="space-y-1">
            <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">Release</label>
            <AutocompleteInput
              value={releaseId || ''}
              options={releaseSuggestions.map((value) => ({ value }))}
              className="h-8 border-border/40 bg-background/30 text-xs transition-colors hover:bg-background/50"
              placeholder="Unassigned"
              noResultsText="No releases found."
              onValueChange={(value) => updateField('release_id', value)}
            />
          </div>

        </div>

        {/* Ownership & Branch */}
        <div className="space-y-3.5">
          <div className="space-y-1">
            <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">Owner</label>
            <Input
              value={owner}
              className="h-8 border-border/40 bg-background/30 text-xs transition-colors hover:bg-background/50 focus:bg-background"
              placeholder="e.g. @username"
              onChange={(event) => updateField('owner', event.target.value)}
            />
          </div>

          <div className="space-y-1">
            <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">Branch</label>
            <Input
              value={branch}
              className="h-8 border-border/40 bg-background/30 text-xs transition-colors hover:bg-background/50 focus:bg-background"
              placeholder="feature/..."
              onChange={(event) => updateField('branch', event.target.value)}
            />
          </div>
        </div>

        {/* ADRs Section */}
        <div className="space-y-1 md:col-span-2 lg:col-span-1">
          <label className="text-muted-foreground/60 text-[9px] font-bold uppercase tracking-widest">ADRs</label>
          <div className="flex min-h-[72px] flex-wrap content-start gap-1.5 rounded-md border border-border/30 bg-background/20 p-2 shadow-inner transition-colors focus-within:bg-background/40">
            {adrs.map((adr) => (
              <Badge key={adr} variant="secondary" className="h-5 gap-1 rounded-sm bg-primary/10 px-1.5 text-[10px] font-medium text-primary hover:bg-primary/20">
                {adr}
                <button
                  type="button"
                  className="text-primary/40 hover:text-primary"
                  onClick={() => updateAdrs(adrs.filter((value) => value !== adr))}
                >
                  <X className="size-2.5" />
                </button>
              </Badge>
            ))}

            {adrInputOpen ? (
              <div className="flex w-full items-center gap-1">
                <AutocompleteInput
                  value={adrInput}
                  options={availableAdrOptions}
                  className="h-6 min-w-[80px] flex-1 border-none bg-transparent text-[11px] shadow-none focus-visible:ring-0"
                  autoFocus
                  placeholder="..."
                  onCommit={(value) => {
                    addAdr(value);
                    setAdrInputOpen(false);
                  }}
                  onValueChange={setAdrInput}
                />
                <Button variant="ghost" size="icon-xs" className="size-5 shrink-0" onClick={() => { addAdr(); setAdrInputOpen(false); }}>
                  <Check className="size-3" />
                </Button>
              </div>
            ) : (
              <Button
                variant="ghost"
                size="xs"
                className="h-5 px-1.5 text-[10px] text-muted-foreground hover:bg-background/50"
                onClick={() => setAdrInputOpen(true)}
              >
                <Plus className="mr-1 size-2.5" />
                Add
              </Button>
            )}
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
