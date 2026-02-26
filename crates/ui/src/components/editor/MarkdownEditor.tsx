import { useEffect, useMemo, useRef, useState } from 'react';
import { Maximize2, Minimize2, Sparkles } from 'lucide-react';
import { Crepe, CrepeFeature } from '@milkdown/crepe';
import { replaceAll } from '@milkdown/kit/utils';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';

type EditorMode = 'doc' | 'raw';
type LegacyEditorMode = 'edit' | 'preview' | 'split';

export interface MarkdownEditorProps {
  label?: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  rows?: number;
  defaultMode?: EditorMode | LegacyEditorMode;
  mcpEnabled?: boolean;
  onMcpSample?: () => Promise<string | null | undefined> | string | null | undefined;
  sampleLabel?: string;
  sampleRequiresMcp?: boolean;
  fillHeight?: boolean;
}

function splitFrontmatter(markdown: string): { frontmatter: string | null; body: string } {
  const match = markdown.match(/^\uFEFF?(?:[ \t]*\r?\n)*---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/);
  if (!match) return { frontmatter: null, body: markdown };
  return {
    frontmatter: match[1].trim(),
    body: markdown.slice(match[0].length),
  };
}

function composeMarkdown(frontmatter: string | null, body: string): string {
  const cleanedFrontmatter = frontmatter?.trimEnd();
  if (!cleanedFrontmatter) return body;

  const cleanedBody = body.replace(/^\n+/, '');
  return `---\n${cleanedFrontmatter}\n---${cleanedBody ? `\n\n${cleanedBody}` : '\n'}`;
}

function parseFrontmatterEntries(frontmatter: string | null): Array<{ key: string; value: string }> {
  if (!frontmatter) return [];
  return frontmatter
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const match = line.match(/^([A-Za-z0-9_.-]+)\s*:\s*(.*)$/);
      if (!match) return null;
      return { key: match[1], value: match[2] };
    })
    .filter((entry): entry is { key: string; value: string } => entry !== null);
}

function normalizeMode(defaultMode?: EditorMode | LegacyEditorMode): EditorMode {
  if (defaultMode === 'raw' || defaultMode === 'preview') return 'raw';
  return 'doc';
}

export default function MarkdownEditor({
  label,
  value,
  onChange,
  placeholder,
  rows = 12,
  defaultMode = 'doc',
  mcpEnabled = false,
  onMcpSample,
  sampleLabel,
  sampleRequiresMcp = true,
  fillHeight = false,
}: MarkdownEditorProps) {
  const editorRootRef = useRef<HTMLDivElement | null>(null);
  const crepeRef = useRef<Crepe | null>(null);
  const onChangeRef = useRef(onChange);
  const internalRef = useRef(value);
  const editorMarkdownRef = useRef(value);

  const [mode, setMode] = useState<EditorMode>(normalizeMode(defaultMode));
  const [sampling, setSampling] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [sampleUndoState, setSampleUndoState] = useState<{ before: string; after: string } | null>(null);
  const [internalMarkdown, setInternalMarkdown] = useState(value);
  const minHeightPx = Math.max(rows, 8) * 24;

  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

  useEffect(() => {
    if (value === internalRef.current) return;
    setInternalMarkdown(value);
  }, [value]);

  useEffect(() => {
    internalRef.current = internalMarkdown;
    const crepe = crepeRef.current;
    if (!crepe) return;
    if (editorMarkdownRef.current === internalMarkdown) return;
    crepe.editor.action(replaceAll(internalMarkdown, true));
    editorMarkdownRef.current = internalMarkdown;
  }, [internalMarkdown]);

  useEffect(() => {
    if (mode !== 'doc') return;
    const root = editorRootRef.current;
    if (!root) return;

    let cancelled = false;
    const crepe = new Crepe({
      root,
      defaultValue: internalRef.current,
      features: {
        [CrepeFeature.Toolbar]: false,
      },
      featureConfigs: {
        [CrepeFeature.Placeholder]: {
          text: placeholder?.trim() || 'Write markdown...',
          mode: 'doc',
        },
      },
    });

    crepe.on((listener) => {
      listener.markdownUpdated((_ctx, markdown) => {
        editorMarkdownRef.current = markdown;
        if (markdown === internalRef.current) return;
        setInternalMarkdown(markdown);
        onChangeRef.current(markdown);
      });
    });

    void crepe.create().then(() => {
      if (cancelled) {
        void crepe.destroy();
        return;
      }
      crepeRef.current = crepe;
      editorMarkdownRef.current = crepe.getMarkdown();
      if (editorMarkdownRef.current !== internalRef.current) {
        crepe.editor.action(replaceAll(internalRef.current, true));
        editorMarkdownRef.current = internalRef.current;
      }
    });

    return () => {
      cancelled = true;
      if (crepeRef.current === crepe) crepeRef.current = null;
      void crepe.destroy();
    };
  }, [mode, placeholder]);

  const wordCount = useMemo(() => {
    const trimmed = internalMarkdown.trim();
    return trimmed ? trimmed.split(/\s+/).length : 0;
  }, [internalMarkdown]);
  const previewModel = useMemo(() => splitFrontmatter(internalMarkdown), [internalMarkdown]);
  const frontmatterEntries = useMemo(
    () => parseFrontmatterEntries(previewModel.frontmatter),
    [previewModel.frontmatter]
  );
  const frontmatterRows = Math.min(
    Math.max((previewModel.frontmatter?.split(/\r?\n/).length ?? 3) + 1, 4),
    12
  );
  const resolvedSampleLabel =
    sampleLabel ?? (sampleRequiresMcp ? 'Generate Draft' : 'Insert Template');
  const sampleDisabled = sampling || !onMcpSample || (sampleRequiresMcp && !mcpEnabled);

  const handleEditorChange = (next: string) => {
    setInternalMarkdown(next);
    onChangeRef.current(next);
  };

  const triggerSample = async () => {
    if (!onMcpSample || sampling) return;
    try {
      setSampling(true);
      const next = await onMcpSample();
      if (!next || !next.trim()) return;
      const scaffold = next.trim();
      const current = internalMarkdown.trimEnd();
      const combined = current ? `${current}\n\n${scaffold}` : scaffold;
      setSampleUndoState({ before: internalMarkdown, after: combined });
      handleEditorChange(combined);
    } finally {
      setSampling(false);
    }
  };

  const undoSample = () => {
    if (!sampleUndoState) return;
    handleEditorChange(sampleUndoState.before);
    setSampleUndoState(null);
  };

  const handleRawFrontmatterChange = (frontmatter: string) => {
    handleEditorChange(composeMarkdown(frontmatter, previewModel.body));
  };

  const handleRawBodyChange = (body: string) => {
    handleEditorChange(composeMarkdown(previewModel.frontmatter, body));
  };

  const addFrontmatter = () => {
    if (previewModel.frontmatter) return;
    handleEditorChange(composeMarkdown('title: \nstatus: draft', previewModel.body));
  };

  const removeFrontmatter = () => {
    if (!previewModel.frontmatter) return;
    handleEditorChange(previewModel.body);
  };

  const panelStyle = fillHeight ? undefined : { height: `${minHeightPx}px` };

  return (
    <div
      className={cn(
        fillHeight ? 'flex h-full min-h-0 flex-col gap-2' : 'space-y-2',
        expanded &&
          'fixed inset-3 z-50 rounded-xl border bg-background p-3 shadow-2xl md:inset-6 md:p-4'
      )}
    >
      <div className={cn('flex flex-wrap items-center gap-2', label ? 'justify-between' : 'justify-end')}>
        {label && <Label>{label}</Label>}
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">
            {wordCount} words · {internalMarkdown.length} chars
          </span>
          <Tabs value={mode} onValueChange={(next) => next && setMode(next as EditorMode)} className="gap-0">
            <TabsList>
              <TabsTrigger value="doc">Doc</TabsTrigger>
              <TabsTrigger value="raw">Raw</TabsTrigger>
            </TabsList>
          </Tabs>
          <Button type="button" variant="ghost" size="icon-sm" onClick={() => setExpanded((current) => !current)}>
            {expanded ? <Minimize2 className="size-3.5" /> : <Maximize2 className="size-3.5" />}
          </Button>
        </div>
      </div>

      {onMcpSample && (
        <div className="flex items-center justify-end gap-1 rounded-md border bg-muted/30 px-2 py-1.5">
          <Button
            type="button"
            variant="outline"
            size="xs"
            disabled={sampleDisabled}
            onClick={triggerSample}
            title={sampleRequiresMcp ? 'Use AI to generate draft content' : 'Insert a starter template'}
          >
            <Sparkles className="size-3.5" />
            {sampling ? 'Working…' : resolvedSampleLabel}
          </Button>
          {sampleUndoState && internalMarkdown === sampleUndoState.after && (
            <Button type="button" variant="outline" size="xs" onClick={undoSample}>
              Undo
            </Button>
          )}
        </div>
      )}

      <div className={cn(fillHeight && 'min-h-0 flex-1', mode === 'raw' && 'grid gap-2 lg:grid-cols-2')}>
        {mode === 'doc' && (
          <div className={cn('rounded-md border bg-card', fillHeight && 'h-full min-h-0')} style={panelStyle}>
            <div ref={editorRootRef} className="ship-milkdown-shell h-full w-full" />
          </div>
        )}

        {mode === 'raw' && (
          <div
            className={cn('rounded-md border bg-card', fillHeight && 'h-full min-h-0')}
            style={panelStyle}
          >
            <div className="flex h-full min-h-0 flex-col overflow-hidden">
              <div className="border-b px-2 py-1.5">
                {previewModel.frontmatter ? (
                  <div className="space-y-2">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <span className="text-muted-foreground text-xs font-medium uppercase tracking-wide">
                        Frontmatter
                      </span>
                      <Button type="button" variant="ghost" size="xs" onClick={removeFrontmatter}>
                        Remove
                      </Button>
                    </div>
                    <Textarea
                      rows={frontmatterRows}
                      value={previewModel.frontmatter}
                      onChange={(event) => handleRawFrontmatterChange(event.target.value)}
                      className="font-mono text-xs leading-5"
                    />
                  </div>
                ) : (
                  <Button type="button" variant="outline" size="xs" onClick={addFrontmatter}>
                    Add Frontmatter
                  </Button>
                )}
              </div>
              <div className="min-h-0 flex-1 p-2">
                <Textarea
                  value={previewModel.body}
                  onChange={(event) => handleRawBodyChange(event.target.value)}
                  className="h-full min-h-0 resize-none font-mono text-sm leading-6"
                />
              </div>
            </div>
          </div>
        )}

        {mode === 'raw' && (
          <div
            className={cn(
              'ship-markdown-preview rounded-md border bg-background',
              fillHeight && 'h-full min-h-0'
            )}
            style={panelStyle}
          >
            {previewModel.frontmatter && (
              <section className="ship-markdown-frontmatter">
                <div className="text-muted-foreground border-b px-3 py-2 text-[11px] font-medium uppercase tracking-wide">
                  Frontmatter
                </div>
                {frontmatterEntries.length > 0 ? (
                  <dl className="grid gap-x-3 gap-y-1 px-3 py-2 text-xs md:grid-cols-[9rem_1fr]">
                    {frontmatterEntries.map((entry) => (
                      <div key={`${entry.key}-${entry.value}`} className="contents">
                        <dt className="text-muted-foreground font-medium">{entry.key}</dt>
                        <dd className="font-mono break-words">{entry.value || '""'}</dd>
                      </div>
                    ))}
                  </dl>
                ) : (
                  <pre className="whitespace-pre-wrap break-words px-3 py-2 text-xs">
                    <code>{previewModel.frontmatter}</code>
                  </pre>
                )}
                <details>
                  <summary>Raw YAML</summary>
                  <pre>
                    <code>{previewModel.frontmatter}</code>
                  </pre>
                </details>
              </section>
            )}

            {previewModel.body.trim() ? (
              <ReactMarkdown remarkPlugins={[remarkGfm]}>{previewModel.body}</ReactMarkdown>
            ) : previewModel.frontmatter ? (
              <p className="text-muted-foreground text-sm">No body content yet.</p>
            ) : (
              <p className="text-muted-foreground text-sm">Nothing to preview yet.</p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
