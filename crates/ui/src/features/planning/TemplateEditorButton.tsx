import { useState } from 'react';
import { FilePenLine } from 'lucide-react';
import { Button } from '@ship/ui';
import { TemplateKind } from '@/lib/platform/tauri/commands';
import TemplateEditorModal from './TemplateEditorModal';

interface TemplateEditorButtonProps {
  kind: TemplateKind;
  label?: string;
  title?: string;
}

export default function TemplateEditorButton({
  kind,
  label = 'Template',
  title,
}: TemplateEditorButtonProps) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button variant="outline" onClick={() => setOpen(true)}>
        <FilePenLine className="size-4" />
        {label}
      </Button>
      {open && (
        <TemplateEditorModal
          kind={kind}
          title={title}
          onClose={() => setOpen(false)}
        />
      )}
    </>
  );
}
