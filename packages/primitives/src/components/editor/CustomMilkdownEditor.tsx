import { useEffect, useRef, useState, useCallback } from 'react';
import { Crepe, CrepeFeature } from '@milkdown/crepe';
import { editorViewCtx } from '@milkdown/kit/core';
import { replaceAll } from '@milkdown/kit/utils';
import { cn } from '@/lib/utils';
import './editor.css';

// Icon node data from lucide-react v0.575.0 (message-square-plus, sparkles).
// Structured as lucide IconNode arrays for verifiability against the source.
type IconNode = [string, Record<string, string>][];

function iconNodeToSvg(nodes: IconNode, size = 24): string {
    const children = nodes
        .map(([tag, attrs]) => {
            const pairs = Object.entries(attrs)
                .filter(([k]) => k !== 'key')
                .map(([k, v]) => `${k}="${v}"`)
                .join(' ');
            return `<${tag} ${pairs}/>`;
        })
        .join('');
    return (
        `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}"` +
        ` viewBox="0 0 24 24" fill="none" stroke="currentColor"` +
        ` stroke-width="2" stroke-linecap="round" stroke-linejoin="round">` +
        children +
        `</svg>`
    );
}

// Source: lucide-react/dist/esm/icons/message-square-plus.js
const messageSquarePlusNode: IconNode = [
    ['path', { d: 'M22 17a2 2 0 0 1-2 2H6.828a2 2 0 0 0-1.414.586l-2.202 2.202A.71.71 0 0 1 2 21.286V5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2z' }],
    ['path', { d: 'M12 8v6' }],
    ['path', { d: 'M9 11h6' }],
];

// Source: lucide-react/dist/esm/icons/sparkles.js
const sparklesNode: IconNode = [
    ['path', { d: 'M11.017 2.814a1 1 0 0 1 1.966 0l1.051 5.558a2 2 0 0 0 1.594 1.594l5.558 1.051a1 1 0 0 1 0 1.966l-5.558 1.051a2 2 0 0 0-1.594 1.594l-1.051 5.558a1 1 0 0 1-1.966 0l-1.051-5.558a2 2 0 0 0-1.594-1.594l-5.558-1.051a1 1 0 0 1 0-1.966l5.558-1.051a2 2 0 0 0 1.594-1.594z' }],
    ['path', { d: 'M20 2v4' }],
    ['path', { d: 'M22 4h-4' }],
    ['circle', { cx: '4', cy: '20', r: '2' }],
];

const COMMENT_ICON = iconNodeToSvg(messageSquarePlusNode);
const SPARKLES_ICON = iconNodeToSvg(sparklesNode);

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
                // Disable spellcheck on ProseMirror contenteditable elements.
                // ProseMirror may recreate DOM elements, so we re-apply on every mutation.
                for (const el of root.querySelectorAll('.ProseMirror')) {
                    if (el.getAttribute('spellcheck') !== 'false') {
                        el.setAttribute('spellcheck', 'false');
                    }
                }

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
