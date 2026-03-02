import { useCallback, useEffect, useState } from 'react';
import { Trash2 } from 'lucide-react';
import { ADR, AdrEntry } from '@/bindings';
import AdrEditor from './AdrEditor';
import DetailSheet from './DetailSheet';
import { Button } from '@/components/ui/button';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { deriveAdrHeaderTitle } from './adrTitle';

interface AdrDetailProps {
  entry: AdrEntry;
  specSuggestions: string[];
  tagSuggestions: string[];
  adrSuggestions?: string[];
  mcpEnabled?: boolean;
  onClose: () => void;
  onSave: (id: string, adr: ADR) => void;
  onDelete: (id: string) => void;
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
  adrSuggestions = [],
  mcpEnabled = false,
  onClose,
  onSave,
  onDelete,
}: AdrDetailProps) {
  const [draft, setDraft] = useState<ADR>(normalizeAdr(entry.adr));
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    setDraft(normalizeAdr(entry.adr));
    setDirty(false);
  }, [entry]);

  const saveAdr = useCallback(() => {
    onSave(entry.id, draft);
    setDirty(false);
  }, [draft, entry.id, onSave]);

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

  const actionButtons = (
    <>
      <AlertDialog>
        <AlertDialogTrigger
          render={
            <Button
              size="xs"
              variant="outline"
              className="h-7 border-destructive/40 px-2 text-destructive hover:bg-destructive/10"
              title="Delete ADR"
            />
          }
        >
          <Trash2 className="size-3.5" />
        </AlertDialogTrigger>
        <AlertDialogContent size="sm">
          <AlertDialogHeader>
            <AlertDialogTitle>Delete this ADR?</AlertDialogTitle>
            <AlertDialogDescription>
              This will permanently remove the decision document.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
            <AlertDialogAction size="sm" variant="destructive" onClick={() => onDelete(entry.id)}>
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Button size="xs" className="h-7 px-2 text-xs" onClick={saveAdr} disabled={!dirty}>
        Save
      </Button>
    </>
  );

  return (
    <DetailSheet
      title={<h2 className="truncate text-lg font-semibold tracking-tight">{deriveAdrHeaderTitle(draft, entry.file_name)}</h2>}
      meta={null}
      onClose={onClose}
      className="max-w-[1800px]"
      bodyScrollable={false}
      bodyClassName="overflow-hidden p-0"
      inlineHeader
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
          extraActions={actionButtons}
          adrSuggestions={adrSuggestions}
          mcpEnabled={mcpEnabled}
        />
      </div>
    </DetailSheet>
  );
}
