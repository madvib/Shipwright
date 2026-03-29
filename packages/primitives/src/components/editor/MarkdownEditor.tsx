import { type ReactNode, useEffect, useMemo, useRef, useState } from 'react';
import { Maximize2, Minimize2, Sparkles } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { cn } from '@/lib/utils';
import { Button } from '../button';
import { Label } from '../label';
import { Tabs, TabsList, TabsTrigger } from '../tabs';
import TiptapEditor from './TiptapEditor';
import { AiActionsMenu, INSTRUCTION_MAP } from './AiActionsMenu';
import type { AiAction } from './AiActionsMenu';
import { stripAllFrontmatter, splitFrontmatterDocument, composeFrontmatterDocument, parseFrontmatterEntries } from './frontmatter';
import type { FrontmatterEntry } from './frontmatter';

type EditorMode = 'edit' | 'read';
type LegacyEditorMode = 'doc' | 'raw' | 'preview' | 'split' | 'edit' | 'read';

export interface MarkdownEditorProps {
    label?: string;
    toolbarStart?: ReactNode;
    value: string;
    onChange: (value: string) => void;
    className?: string;
    placeholder?: string;
    rows?: number;
    defaultMode?: EditorMode | LegacyEditorMode;
    mcpEnabled?: boolean;
    onMcpSample?: () => Promise<string | null | undefined> | string | null | undefined;
    sampleLabel?: string;
    sampleRequiresMcp?: boolean;
    showStats?: boolean;
    fillHeight?: boolean;
    fullWidth?: boolean;
    editorClassName?: string;
    showAiActions?: boolean;
    onTransformText?: (instruction: string, text: string) => Promise<string>;
    /** Called when user highlights text and submits a comment via the selection menu */
    onComment?: (selectedText: string, comment: string) => void;
    /** Hide the wrapper chrome (label, Edit/Read tabs, stats). Just render the editor. */
    hideChrome?: boolean;
    /** Called when user highlights text and requests AI generation */
    onGenerate?: (selectedText: string) => void;
    /** Called with parsed frontmatter when the document has frontmatter */
    onFrontmatterParsed?: (entries: FrontmatterEntry[], raw: string | null) => void;
}

function normalizeMode(defaultMode?: EditorMode | LegacyEditorMode): EditorMode {
    if (defaultMode === 'raw' || defaultMode === 'preview' || defaultMode === 'split' || defaultMode === 'read') {
        return 'read';
    }
    return 'edit';
}

export default function MarkdownEditor({
    label,
    toolbarStart,
    value,
    onChange,
    className,
    placeholder,
    rows = 12,
    defaultMode = 'edit',
    mcpEnabled = false,
    onMcpSample,
    sampleLabel,
    sampleRequiresMcp = true,
    showStats = true,
    fillHeight = false,
    fullWidth = true,
    editorClassName,
    showAiActions = true,
    onTransformText,
    onComment,
    hideChrome = false,
    onGenerate,
    onFrontmatterParsed,
}: MarkdownEditorProps) {
    const onChangeRef = useRef(onChange);
    const internalMarkdownRef = useRef(value);

    const [mode, setMode] = useState<EditorMode>(normalizeMode(defaultMode));
    const [sampling, setSampling] = useState(false);
    const [expanded, setExpanded] = useState(false);
    const [sampleUndoState, setSampleUndoState] = useState<{ before: string; after: string } | null>(null);
    const [internalMarkdown, setInternalMarkdown] = useState(value);
    const [aiActionError, setAiActionError] = useState<string | null>(null);

    const minHeightPx = Math.max(rows, 8) * 24;

    useEffect(() => {
        onChangeRef.current = onChange;
    }, [onChange]);

    useEffect(() => {
        setInternalMarkdown((current) => (current === value ? current : value));
        internalMarkdownRef.current = value;
    }, [value]);

    // Split frontmatter from body — tiptap only edits the body
    const fmDoc = useMemo(() => splitFrontmatterDocument(internalMarkdown), [internalMarkdown]);
    const fmEntries = useMemo(() => parseFrontmatterEntries(fmDoc.frontmatter), [fmDoc.frontmatter]);

    // Notify parent of frontmatter entries
    const onFrontmatterParsedRef = useRef(onFrontmatterParsed);
    useEffect(() => { onFrontmatterParsedRef.current = onFrontmatterParsed; }, [onFrontmatterParsed]);
    useEffect(() => {
        onFrontmatterParsedRef.current?.(fmEntries, fmDoc.frontmatter);
    }, [fmEntries, fmDoc.frontmatter]);

    const wordCount = useMemo(() => {
        const trimmed = fmDoc.body.trim();
        return trimmed ? trimmed.split(/\s+/).length : 0;
    }, [fmDoc.body]);

    const resolvedSampleLabel = sampleLabel ?? (sampleRequiresMcp ? 'Generate Draft' : 'Insert Template');
    const sampleDisabled = sampling || !onMcpSample || (sampleRequiresMcp && !mcpEnabled);

    const handleEditorChange = (next: string) => {
        // Recompose frontmatter + edited body
        const full = fmDoc.frontmatter
            ? composeFrontmatterDocument(fmDoc.frontmatter, next, fmDoc.delimiter ?? '---')
            : next;
        if (full === internalMarkdownRef.current) return;
        internalMarkdownRef.current = full;
        setInternalMarkdown(full);
        onChangeRef.current(full);
    };

    const triggerSample = async () => {
        if (!onMcpSample || sampling) return;
        try {
            setSampling(true);
            const sample = await onMcpSample();
            if (!sample?.trim()) return;
            const combined = internalMarkdown.trimEnd()
                ? `${internalMarkdown.trimEnd()}\n\n${sample.trim()}`
                : sample.trim();
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

    const handleAiAction = async (action: AiAction) => {
        if (!internalMarkdown.trim() || !onTransformText) return;
        setAiActionError(null);
        setSampling(true);
        try {
            const res = await onTransformText(INSTRUCTION_MAP[action], internalMarkdown);
            handleEditorChange(res);
        } catch (err) {
            console.error(`AI Action failed: ${err}`);
            setAiActionError(String(err));
        } finally {
            setSampling(false);
        }
    };

    return (
        <div
            className={cn(
                fillHeight ? 'flex h-full min-h-0 flex-col' : 'space-y-1',
                !hideChrome && 'gap-1',
                fullWidth && 'w-full',
                expanded && 'fixed inset-0 z-[120] bg-background p-1 shadow-2xl md:p-2',
                className
            )}
        >
            {/* Toolbar — not rendered when hideChrome is true */}
            {!hideChrome && <div className="flex items-center gap-1 overflow-x-auto">
                {(label || toolbarStart) && (
                    <div className="flex shrink-0 items-center gap-1">
                        {label && <Label>{label}</Label>}
                        {toolbarStart}
                    </div>
                )}

                <div className="ml-auto flex shrink-0 items-center gap-1">
                    {onMcpSample && mode === 'edit' && (
                        <>
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
                        </>
                    )}

                    {showAiActions && mode === 'edit' && onTransformText && (
                        <AiActionsMenu disabled={sampling} onAction={handleAiAction} />
                    )}

                    {showStats && mode === 'edit' && (
                        <span className="text-xs text-muted-foreground">
                            {wordCount} words · {internalMarkdown.length} chars
                        </span>
                    )}

                    <Tabs value={mode} onValueChange={(next) => next && setMode(next as EditorMode)} className="gap-0">
                        <TabsList>
                            <TabsTrigger value="edit">Edit</TabsTrigger>
                            <TabsTrigger value="read">Read</TabsTrigger>
                        </TabsList>
                    </Tabs>

                    <Button type="button" variant="ghost" size="icon-sm" onClick={() => setExpanded((current) => !current)}>
                        {expanded ? <Minimize2 className="size-3.5" /> : <Maximize2 className="size-3.5" />}
                    </Button>
                </div>
            </div>}

            {aiActionError && (
                <p className="text-[11px] text-destructive">AI action failed: {aiActionError}</p>
            )}

            {/* Editor / Reader */}
            <div className={cn(fillHeight && 'min-h-0 flex-1')}>
                {mode === 'edit' && (
                    <div className={cn(fillHeight ? 'flex h-full min-h-0 flex-col' : 'space-y-1')}>
                        <div className={cn(fillHeight ? 'min-h-0 flex-1' : 'h-full')}>
                            <TiptapEditor
                                value={fmDoc.body}
                                onChange={handleEditorChange}
                                placeholder={placeholder}
                                fillHeight={fillHeight}
                                minHeightPx={minHeightPx}
                                className={editorClassName}
                                onComment={onComment}
                                onGenerate={onGenerate}
                            />
                        </div>
                    </div>
                )}

                {mode === 'read' && (
                    <div
                        className={cn(
                            'ship-markdown-preview rounded-md border bg-background px-4 py-3',
                            fillHeight ? 'h-full min-h-0 overflow-y-auto' : 'overflow-y-auto'
                        )}
                        style={fillHeight ? undefined : { minHeight: `${minHeightPx}px`, maxHeight: '600px' }}
                    >
                        {stripAllFrontmatter(internalMarkdown).trim() ? (
                            <ReactMarkdown remarkPlugins={[remarkGfm]}>
                                {stripAllFrontmatter(internalMarkdown)}
                            </ReactMarkdown>
                        ) : (
                            <p className="text-muted-foreground text-sm">Nothing to preview yet.</p>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
