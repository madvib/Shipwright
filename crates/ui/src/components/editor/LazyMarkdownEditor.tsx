import { Suspense, lazy } from 'react';
import type { MarkdownEditorProps } from '@ship/ui';

const MarkdownEditorModule = lazy(() =>
  import('@ship/ui').then((m) => ({ default: m.MarkdownEditor }))
);

export type { MarkdownEditorProps } from '@ship/ui';

export default function MarkdownEditor(props: MarkdownEditorProps) {
  const fallbackClass = props.fillHeight
    ? 'h-full min-h-0 rounded-md border bg-card/70'
    : 'h-52 rounded-md border bg-card/70';

  return (
    <Suspense fallback={<div className={fallbackClass} />}>
      <MarkdownEditorModule {...props} />
    </Suspense>
  );
}
