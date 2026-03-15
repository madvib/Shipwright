import { ReactNode, useEffect, useMemo, useRef, useState } from 'react';
import { Maximize2, Minimize2, Sparkles, Wand2, Type, AlignLeft, CheckCircle } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { cn } from '@/lib/utils';
import { Button } from '../button';
import { Label } from '../label';
import { Tabs, TabsList, TabsTrigger } from '../tabs';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuGroup,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '../dropdown-menu';
import { Tooltip, TooltipContent, TooltipTrigger } from '../tooltip';
import CustomMilkdownEditor from './CustomMilkdownEditor';
import { stripAllFrontmatter } from './frontmatter';

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
    sampleInline?: boolean;
    showStats?: boolean;
    fillHeight?: boolean;
    fullWidth?: boolean;
    editorClassName?: string;
    /** @deprecated No-op — frontmatter display is YAGNI until rich field components exist */
    showFrontmatter?: boolean;
    /** @deprecated No-op — metadata panels should manage frontmatter directly */
    frontmatterPanel?: ReactNode | unknown;
    showAiActions?: boolean;
    onTransformText?: (instruction: string, text: string) => Promise<string>;
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

    const wordCount = useMemo(() => {
        const trimmed = stripAllFrontmatter(internalMarkdown).trim();
        return trimmed ? trimmed.split(/\s+/).length : 0;
    }, [internalMarkdown]);

    const resolvedSampleLabel = sampleLabel ?? (sampleRequiresMcp ? 'Generate Draft' : 'Insert Template');
    const sampleDisabled = sampling || !onMcpSample || (sampleRequiresMcp && !mcpEnabled);

    const handleEditorChange = (next: string) => {
        if (next === internalMarkdownRef.current) return;
        internalMarkdownRef.current = next;
        setInternalMarkdown(next);
        onChangeRef.current(next);
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

    const handleAiAction = async (action: 'polish' | 'shorten' | 'expand' | 'fix_grammar') => {
        if (!internalMarkdown.trim() || !onTransformText) return;
        setAiActionError(null);
        setSampling(true);
        try {
            const instructionMap = {
                polish: 'Polish the writing to be more professional and clear',
                shorten: 'Make the text more concise and remove jargon',
                expand: 'Add more relevant details and context',
                fix_grammar: 'Fix any grammar or spelling issues',
            };
            const res = await onTransformText(instructionMap[action], internalMarkdown);
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
                fillHeight ? 'flex h-full min-h-0 flex-col gap-1' : 'space-y-1',
                fullWidth && 'w-full',
                expanded && 'fixed inset-0 z-[120] bg-background p-1 shadow-2xl md:p-2',
                className
            )}
        >
            {/* Toolbar */}
            <div className="flex items-center gap-1 overflow-x-auto">
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
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <DropdownMenu>
                                    <DropdownMenuTrigger render={
                                        <Button variant="outline" size="xs" disabled={sampling}>
                                            <Wand2 className="size-3.5" />
                                            Create with AI
                                        </Button>
                                    } />
                                    <DropdownMenuContent align="end" className="w-56 p-1.5 shadow-xl">
                                        <DropdownMenuGroup>
                                            <DropdownMenuLabel className="px-2 pb-2 opacity-50 uppercase text-[9px] tracking-[0.2em] font-black">
                                                Transform Text
                                            </DropdownMenuLabel>
                                        </DropdownMenuGroup>
                                        <DropdownMenuSeparator className="opacity-50" />
                                        <div className="space-y-0.5">
                                            <DropdownMenuItem onClick={() => handleAiAction('polish')} className="flex items-center gap-2 rounded-md">
                                                <Sparkles className="size-3.5 text-amber-500" />
                                                <span className="text-sm">Polish Writing</span>
                                            </DropdownMenuItem>
                                            <DropdownMenuItem onClick={() => handleAiAction('shorten')} className="flex items-center gap-2 rounded-md">
                                                <AlignLeft className="size-3.5 text-blue-500" />
                                                <span className="text-sm">Make Concise</span>
                                            </DropdownMenuItem>
                                            <DropdownMenuItem onClick={() => handleAiAction('expand')} className="flex items-center gap-2 rounded-md">
                                                <Type className="size-3.5 text-indigo-500" />
                                                <span className="text-sm">Expand Details</span>
                                            </DropdownMenuItem>
                                            <DropdownMenuSeparator className="opacity-50" />
                                            <DropdownMenuItem onClick={() => handleAiAction('fix_grammar')} className="flex items-center gap-2 rounded-md">
                                                <CheckCircle className="size-3.5 text-emerald-500" />
                                                <span className="text-sm">Fix Grammar</span>
                                            </DropdownMenuItem>
                                        </div>
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            </TooltipTrigger>
                            <TooltipContent>Refine, polish, or transform your text using AI.</TooltipContent>
                        </Tooltip>
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
            </div>

            {aiActionError && (
                <p className="text-[11px] text-destructive">AI action failed: {aiActionError}</p>
            )}

            {/* Editor / Reader */}
            <div className={cn(fillHeight && 'min-h-0 flex-1')}>
                {mode === 'edit' && (
                    <div className={cn(fillHeight ? 'flex h-full min-h-0 flex-col' : 'space-y-1')}>
                        <div className={cn(fillHeight ? 'min-h-0 flex-1' : 'h-full')}>
                            <CustomMilkdownEditor
                                value={internalMarkdown}
                                onChange={handleEditorChange}
                                placeholder={placeholder}
                                fillHeight={fillHeight}
                                minHeightPx={minHeightPx}
                                className={editorClassName}
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
