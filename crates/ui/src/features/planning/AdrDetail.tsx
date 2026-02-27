import { useCallback, useEffect, useState } from 'react';
import { ADR, AdrEntry } from '@/bindings';
import AdrEditor from './AdrEditor';
import DetailSheet from './DetailSheet';
import { loadProjectTemplate } from '@/components/editor/templateLoader';
import { Button } from '@/components/ui/button';

interface AdrDetailProps {
  entry: AdrEntry;
  specSuggestions: string[];
  tagSuggestions: string[];
  mcpEnabled?: boolean;
  onClose: () => void;
  onSave: (fileName: string, adr: ADR) => void;
  onDelete: (fileName: string) => void;
}

function normalizeAdr(adr: ADR): ADR {
  return {
    ...adr,
    metadata: {
      ...adr.metadata,
      tags: adr.metadata.tags ?? [],
    },
  };
}

export default function AdrDetail({
  entry,
  specSuggestions,
  tagSuggestions,
  mcpEnabled = false,
  onClose,
  onSave,
  onDelete,
}: AdrDetailProps) {
  const [draft, setDraft] = useState<ADR>(normalizeAdr(entry.adr));
  const [dirty, setDirty] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    setDraft(normalizeAdr(entry.adr));
    setDirty(false);
    setConfirmDelete(false);
  }, [entry]);

  const saveAdr = useCallback(() => {
    onSave(entry.file_name, draft);
    setDirty(false);
  }, [draft, entry.file_name, onSave]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
        event.preventDefault();
        saveAdr();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onClose, saveAdr]);

  const insertTemplate = useCallback(async () => {
    const template = await loadProjectTemplate('adr', { bodyOnly: true });
    const snippet = template?.trim();
    if (!snippet) return;
    setDraft((current) => {
      const nextBody = current.body.trimEnd() ? `${current.body.trimEnd()}\n\n${snippet}` : snippet;
      return {
        ...current,
        body: nextBody,
      };
    });
    setDirty(true);
  }, []);

  const actionButtons = (
    <>
      {!confirmDelete ? (
        <Button size="xs" variant="ghost" className="h-7 px-2 text-xs" onClick={() => setConfirmDelete(true)}>
          Delete
        </Button>
      ) : (
        <>
          <Button
            size="xs"
            variant="destructive"
            className="h-7 px-2 text-xs"
            onClick={() => onDelete(entry.file_name)}
          >
            Confirm
          </Button>
          <Button
            size="xs"
            variant="outline"
            className="h-7 px-2 text-xs"
            onClick={() => setConfirmDelete(false)}
          >
            Cancel
          </Button>
        </>
      )}

      <Button size="xs" variant="outline" className="h-7 px-2 text-xs" onClick={onClose}>
        Close
      </Button>
      <Button size="xs" className="h-7 px-2 text-xs" onClick={saveAdr} disabled={!dirty}>
        Save
      </Button>
    </>
  );

  return (
    <DetailSheet
      title={null}
      onClose={onClose}
      className="max-w-[1800px]"
      bodyScrollable={false}
      bodyClassName="overflow-hidden p-0"
      showHeader={false}
    >
      <div className="h-full min-h-0 p-1.5">
        <AdrEditor
          adr={draft}
          onChange={(next) => {
            setDraft(next);
            setDirty(true);
          }}
          specSuggestions={specSuggestions}
          tagSuggestions={tagSuggestions}
          placeholder="Describe this decision, context, and consequences..."
          onInsertTemplate={insertTemplate}
          extraActions={actionButtons}
          mcpEnabled={mcpEnabled}
        />
      </div>
    </DetailSheet>
  );
}
