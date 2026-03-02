import { ReactNode } from 'react';
import { FilePlus2 } from 'lucide-react';
import { ADR } from '@/bindings';
import MarkdownEditor from '@/components/editor';
import AdrFrontmatterPanel from '@/components/editor/AdrFrontmatterPanel';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { deriveAdrDocTitle } from './adrTitle';

interface AdrEditorProps {
  adr: ADR;
  onChange: (next: ADR) => void;
  specSuggestions: string[];
  tagSuggestions: string[];
  adrSuggestions?: string[];
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
  adrSuggestions = [],
  placeholder,
  onInsertTemplate,
  onMcpSample,
  sampleLabel,
  sampleRequiresMcp,
  mcpEnabled,
  extraActions,
}: AdrEditorProps) {
  const toolbarControls = (
    <>
      {onInsertTemplate && (
        <Button size="xs" variant="outline" className="h-7 px-2 text-xs" onClick={() => void onInsertTemplate()}>
          <FilePlus2 className="size-3.5" />
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
      showFrontmatter={false}
      frontmatterPanel={
        <div className="space-y-3 p-4">
          <section className="rounded-lg border border-border/40 bg-muted/20 px-4 py-3">
            <div className="mb-2 flex items-center gap-2">
              <div className="size-1.5 rounded-full bg-primary/60" />
              <h3 className="text-[11px] font-bold uppercase tracking-wider text-muted-foreground/80">
                Context
              </h3>
            </div>
            <Textarea
              value={adr.context}
              onChange={(event) => onChange({ ...adr, context: event.target.value })}
              placeholder="Capture constraints, drivers, and background for this decision."
              className="min-h-24 bg-background/60 text-sm"
            />
          </section>
          <AdrFrontmatterPanel
            adr={adr}
            specSuggestions={specSuggestions}
            tagSuggestions={tagSuggestions}
            adrSuggestions={adrSuggestions}
            onChange={onChange}
          />
        </div>
      }
      value={adr.decision}
      onChange={(decision) => {
        const docTitle = deriveAdrDocTitle(decision) || deriveAdrDocTitle(adr.context);
        onChange({
          ...adr,
          decision,
          metadata: {
            ...adr.metadata,
            title: docTitle || adr.metadata.title,
          },
        });
      }}
      placeholder={placeholder ?? 'Write the decision narrative, alternatives, and consequences.'}
      rows={16}
      defaultMode="doc"
      fillHeight
      mcpEnabled={mcpEnabled}
      sampleLabel={sampleLabel}
      sampleRequiresMcp={sampleRequiresMcp}
      onMcpSample={onMcpSample}
    />
  );
}
