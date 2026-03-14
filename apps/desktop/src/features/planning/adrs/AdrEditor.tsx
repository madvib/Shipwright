import { ReactNode } from 'react';
import { FilePlus2 } from 'lucide-react';
import { ADR } from '@/bindings';
import MarkdownEditor from '@/components/editor';
import { Button } from '@ship/primitives';
import { deriveAdrDocTitle } from './adrTitle';

interface AdrEditorProps {
  adr: ADR;
  onChange: (next: ADR) => void;
  tagSuggestions: string[];
  adrSuggestions?: { id: string; title: string }[];
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
