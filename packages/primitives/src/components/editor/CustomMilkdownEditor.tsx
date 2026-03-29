import { useEffect, useRef, useState, useCallback } from 'react';
import { Crepe, CrepeFeature } from '@milkdown/crepe';
import { editorViewCtx } from '@milkdown/kit/core';
import { replaceAll } from '@milkdown/kit/utils';
import { cn } from '@/lib/utils';
import './editor.css';

const COMMENT_ICON = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z"/></svg>';
const SPARKLES_ICON = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9.937 15.5A2 2 0 0 0 8.5 14.063l-6.135-1.582a.5.5 0 0 1 0-.962L8.5 9.936A2 2 0 0 0 9.937 8.5l1.582-6.135a.5.5 0 0 1 .963 0L14.063 8.5A2 2 0 0 0 15.5 9.937l6.135 1.581a.5.5 0 0 1 0 .964L15.5 14.063a2 2 0 0 0-1.437 1.437l-1.582 6.135a.5.5 0 0 1-.963 0z"/><path d="M20 3v4"/><path d="M22 5h-4"/></svg>';

export interface CustomMilkdownEditorProps {
    value: string;
    onChange: (value: string) => void;
    placeholder?: string;
    fillHeight?: boolean;
    minHeightPx?: number;
    className?: string;
    /** Called when user highlights text and submits a comment via the native toolbar */
    onComment?: (selectedText: string, comment: string) => void;
    /** Called when user highlights text and requests AI generation */
    onGenerate?: (selectedText: string) => void;
}

export default function CustomMilkdownEditor({
    value,
    onChange,
    placeholder,
    fillHeight = false,
    minHeightPx = 320,
    className,
    onComment,
    onGenerate,
}: CustomMilkdownEditorProps) {
    const rootRef = useRef<HTMLDivElement | null>(null);
    const crepeRef = useRef<Crepe | null>(null);
    const tooltipObserverRef = useRef<MutationObserver | null>(null);
    const externalValueRef = useRef(value);
    const onChangeRef = useRef(onChange);
    const onCommentRef = useRef(onComment);
    const onGenerateRef = useRef(onGenerate);
    const editorEmissionsRef = useRef(new Set<string>());

    // Comment input state — triggered by milkdown toolbar button
    const [pendingComment, setPendingComment] = useState<{ text: string } | null>(null);
    const [commentValue, setCommentValue] = useState('');
    const commentInputRef = useRef<HTMLTextAreaElement>(null);

    useEffect(() => { onChangeRef.current = onChange; }, [onChange]);
    useEffect(() => { onCommentRef.current = onComment; }, [onComment]);
    useEffect(() => { onGenerateRef.current = onGenerate; }, [onGenerate]);

    useEffect(() => {
        externalValueRef.current = value;
        if (editorEmissionsRef.current.has(value)) {
            editorEmissionsRef.current.delete(value);
            return;
        }
        const crepe = crepeRef.current;
        if (!crepe) return;
        if (crepe.getMarkdown() === value) return;
        editorEmissionsRef.current.clear();
        crepe.editor.action(replaceAll(value, true));
    }, [value]);

    useEffect(() => {
        const root = rootRef.current;
        if (!root) return;

        let disposed = false;
        editorEmissionsRef.current.clear();

        const featureConfigs: Record<string, unknown> = {
            [CrepeFeature.Placeholder]: {
                text: placeholder?.trim() || '',
                mode: 'doc',
            },
        };

        // Add Comment + Generate buttons to milkdown's native selection toolbar
        if (onCommentRef.current || onGenerateRef.current) {
            type ToolbarGroup = { addItem: (key: string, item: { icon: string; active: () => boolean; onRun: (ctx: { get: (key: unknown) => unknown }) => void }) => ToolbarGroup };
            type ToolbarBuilder = { getGroup: (key: string) => ToolbarGroup };

            const getSelectedText = (ctx: { get: (key: unknown) => unknown }): string => {
                try {
                    const view = ctx.get(editorViewCtx) as { state: { selection: { from: number; to: number }; doc: { textBetween: (from: number, to: number) => string } } };
                    const { from, to } = view.state.selection;
                    return view.state.doc.textBetween(from, to).trim();
                } catch { return ''; }
            };

            featureConfigs[CrepeFeature.Toolbar] = {
                buildToolbar: (builder: ToolbarBuilder) => {
                    try {
                        const group = builder.getGroup('function');
                        if (onCommentRef.current) {
                            group.addItem('comment', {
                                icon: COMMENT_ICON,
                                active: () => false,
                                onRun: (ctx) => {
                                    const text = getSelectedText(ctx);
                                    if (text) {
                                        setPendingComment({ text });
                                        setCommentValue('');
                                    }
                                },
                            });
                        }
                        if (onGenerateRef.current) {
                            group.addItem('generate', {
                                icon: SPARKLES_ICON,
                                active: () => false,
                                onRun: (ctx) => {
                                    const text = getSelectedText(ctx);
                                    if (text) onGenerateRef.current?.(text);
                                },
                            });
                        }
                    } catch { /* Group may not exist */ }
                },
            };
        }

        const crepe = new Crepe({
            root,
            defaultValue: externalValueRef.current,
            features: {
                [CrepeFeature.Toolbar]: true,
                [CrepeFeature.BlockEdit]: false,
                [CrepeFeature.LinkTooltip]: true,
                [CrepeFeature.Placeholder]: true,
                [CrepeFeature.Table]: true,
                [CrepeFeature.ListItem]: true,
                [CrepeFeature.ImageBlock]: true,
                [CrepeFeature.CodeMirror]: true,
                [CrepeFeature.Latex]: true,
            },
            featureConfigs,
        });

        crepe.on((listener) => {
            listener.markdownUpdated((_ctx, markdown) => {
                if (markdown === externalValueRef.current) return;
                editorEmissionsRef.current.add(markdown);
                externalValueRef.current = markdown;
                onChangeRef.current(markdown);
            });
        });

        void crepe.create().then(() => {
            if (disposed) {
                void crepe.destroy();
                return;
            }

            crepeRef.current = crepe;
            const liveValue = crepe.getMarkdown();
            if (liveValue !== externalValueRef.current) {
                crepe.editor.action(replaceAll(externalValueRef.current, true));
            }

            const annotateTooltips = () => {
                const slashCandidates = root.querySelectorAll('.milkdown-slash-menu li');
                const toolbarCandidates = root.querySelectorAll('.milkdown-toolbar .toolbar-item');
                const toolbarLabels = ['Bold', 'Italic', 'Strike', 'Code', 'Link', 'Math', 'Comment'];

                Array.from(toolbarCandidates).forEach((element, index) => {
                    const el = element as HTMLElement;
                    if (el.title) return;
                    el.title = toolbarLabels[index] ?? 'Format';
                });

                for (const element of slashCandidates) {
                    const el = element as HTMLElement;
                    if (el.title) continue;
                    const label = el.textContent?.replace(/\s+/g, ' ').trim();
                    if (label) el.title = label;
                }
            };

            annotateTooltips();
            const observer = new MutationObserver(() => annotateTooltips());
            observer.observe(root, { childList: true, subtree: true });
            tooltipObserverRef.current = observer;
        });

        return () => {
            disposed = true;
            tooltipObserverRef.current?.disconnect();
            tooltipObserverRef.current = null;
            if (crepeRef.current === crepe) crepeRef.current = null;
            void crepe.destroy();
        };
    }, [placeholder]);

    // Focus comment input when shown
    useEffect(() => {
        if (pendingComment) commentInputRef.current?.focus();
    }, [pendingComment]);

    const handleCommentSubmit = useCallback(() => {
        if (!pendingComment || !commentValue.trim()) return;
        onCommentRef.current?.(pendingComment.text, commentValue.trim());
        setPendingComment(null);
        setCommentValue('');
    }, [pendingComment, commentValue]);

    const handleCommentKeyDown = useCallback((e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
            e.preventDefault();
            handleCommentSubmit();
        }
        if (e.key === 'Escape') {
            setPendingComment(null);
            setCommentValue('');
        }
    }, [handleCommentSubmit]);

    return (
        <div
            className={cn('rounded-md border bg-card relative', fillHeight && 'h-full min-h-0', className)}
            style={fillHeight ? undefined : { height: `${minHeightPx}px` }}
        >
            <div ref={rootRef} className="ship-milkdown-shell h-full w-full" />

            {/* Comment input dialog — triggered from milkdown toolbar */}
            {pendingComment && (
                <div className="absolute inset-0 z-50 flex items-start justify-center pt-16" onClick={() => setPendingComment(null)}>
                    <div className="ship-comment-tooltip-expanded" onClick={(e) => e.stopPropagation()}>
                        <div className="ship-comment-tooltip-selected">
                            &ldquo;{pendingComment.text.slice(0, 120)}{pendingComment.text.length > 120 ? '...' : ''}&rdquo;
                        </div>
                        <textarea
                            ref={commentInputRef}
                            value={commentValue}
                            onChange={(e) => setCommentValue(e.target.value)}
                            onKeyDown={handleCommentKeyDown}
                            placeholder="Add your comment..."
                            className="ship-comment-tooltip-input"
                            rows={2}
                        />
                        <div className="ship-comment-tooltip-actions">
                            <button onClick={() => setPendingComment(null)} className="ship-comment-tooltip-cancel">Cancel</button>
                            <button onClick={handleCommentSubmit} disabled={!commentValue.trim()} className="ship-comment-tooltip-submit">
                                Add comment <kbd>&#x2318;&#x21B5;</kbd>
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
