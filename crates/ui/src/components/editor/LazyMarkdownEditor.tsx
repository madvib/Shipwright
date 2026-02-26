import { Suspense, lazy } from 'react';
import type { MarkdownEditorProps } from './MarkdownEditor';

const MarkdownEditorModule = lazy(() => import('./MarkdownEditor'));

export type { MarkdownEditorProps } from './MarkdownEditor';

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
