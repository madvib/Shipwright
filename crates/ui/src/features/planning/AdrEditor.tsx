import { ReactNode } from 'react';
import { FilePlus2 } from 'lucide-react';
import { ADR } from '@/bindings';
import MarkdownEditor from '@/components/editor';
import AdrFrontmatterPanel from '@/components/editor/AdrFrontmatterPanel';
import { Button } from '@/components/ui/button';
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
        <div className="p-4">
          <AdrFrontmatterPanel
            adr={adr}
            specSuggestions={specSuggestions}
            tagSuggestions={tagSuggestions}
            adrSuggestions={adrSuggestions}
            onChange={onChange}
          />
        </div>
      }
      value={adr.body}
      onChange={(body) => {
        const docTitle = deriveAdrDocTitle(body);
        onChange({
          ...adr,
          body,
          metadata: {
            ...adr.metadata,
            title: docTitle || adr.metadata.title,
          },
        });
      }}
      placeholder={placeholder}
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
