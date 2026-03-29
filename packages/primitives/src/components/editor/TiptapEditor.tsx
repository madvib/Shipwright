import { useEffect, useRef, useState, useCallback } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { Table, TableRow, TableCell, TableHeader } from '@tiptap/extension-table';
import Link from '@tiptap/extension-link';
import Image from '@tiptap/extension-image';
import TaskList from '@tiptap/extension-task-list';
import TaskItem from '@tiptap/extension-task-item';
import Typography from '@tiptap/extension-typography';
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight';
import { Markdown } from 'tiptap-markdown';
import { common, createLowlight } from 'lowlight';
import { MessageSquarePlus, Sparkles } from 'lucide-react';
import { cn } from '@/lib/utils';
import './editor.css';

const lowlight = createLowlight(common);

export interface TiptapEditorProps {
    value: string;
    onChange: (value: string) => void;
    placeholder?: string;
    fillHeight?: boolean;
    minHeightPx?: number;
    className?: string;
    onComment?: (selectedText: string, comment: string) => void;
    onGenerate?: (selectedText: string) => void;
}

export default function TiptapEditor({
    value,
    onChange,
    placeholder,
    fillHeight = false,
    minHeightPx = 320,
    className,
    onComment,
    onGenerate,
}: TiptapEditorProps) {
    const onChangeRef = useRef(onChange);
    const onCommentRef = useRef(onComment);
    const onGenerateRef = useRef(onGenerate);
    const externalValueRef = useRef(value);
    const suppressNextUpdate = useRef(false);

    const [pendingComment, setPendingComment] = useState<{ text: string } | null>(null);
    const [commentValue, setCommentValue] = useState('');
    const commentInputRef = useRef<HTMLTextAreaElement>(null);

    // Selection toolbar positioning
    const [toolbarPos, setToolbarPos] = useState<{ top: number; left: number } | null>(null);
    const shellRef = useRef<HTMLDivElement>(null);

    useEffect(() => { onChangeRef.current = onChange; }, [onChange]);
    useEffect(() => { onCommentRef.current = onComment; }, [onComment]);
    useEffect(() => { onGenerateRef.current = onGenerate; }, [onGenerate]);

    const editor = useEditor({
        extensions: [
            StarterKit.configure({ codeBlock: false }),
            Placeholder.configure({ placeholder: placeholder ?? '' }),
            Table.configure({ resizable: false }),
            TableRow, TableCell, TableHeader,
            Link.configure({ openOnClick: false, HTMLAttributes: { class: 'ship-tiptap-link' } }),
            Image,
            TaskList,
            TaskItem.configure({ nested: true }),
            Typography,
            CodeBlockLowlight.configure({ lowlight }),
            Markdown.configure({ html: true, transformPastedText: true, transformCopiedText: true }),
        ],
        content: value,
        editorProps: {
            attributes: { class: 'ship-tiptap-editor', spellcheck: 'false' },
        },
        onUpdate: ({ editor: ed }) => {
            if (suppressNextUpdate.current) { suppressNextUpdate.current = false; return; }
            const md = ((ed.storage as unknown) as Record<string, { getMarkdown: () => string }>).markdown.getMarkdown();
            if (md === externalValueRef.current) return;
            externalValueRef.current = md;
            onChangeRef.current(md);
        },
        onSelectionUpdate: ({ editor: ed }) => {
            if (ed.state.selection.empty || !(onComment || onGenerate)) {
                setToolbarPos(null);
                return;
            }
            const { from } = ed.state.selection;
            const coords = ed.view.coordsAtPos(from);
            const shell = shellRef.current;
            if (!shell) return;
            const shellRect = shell.getBoundingClientRect();
            setToolbarPos({
                top: coords.top - shellRect.top - 40,
                left: coords.left - shellRect.left,
            });
        },
    });

    // Sync external value changes
    useEffect(() => {
        if (!editor || editor.isDestroyed) return;
        externalValueRef.current = value;
        const md = ((editor.storage as unknown) as Record<string, { getMarkdown: () => string }>).markdown.getMarkdown();
        if (md === value) return;
        suppressNextUpdate.current = true;
        editor.commands.setContent(value);
    }, [value, editor]);

    // Hide toolbar on blur
    useEffect(() => {
        if (!editor) return;
        const onBlur = () => setToolbarPos(null);
        editor.on('blur', onBlur);
        return () => { editor.off('blur', onBlur); };
    }, [editor]);

    useEffect(() => {
        if (pendingComment) commentInputRef.current?.focus();
    }, [pendingComment]);

    const getSelectedText = useCallback((): string => {
        if (!editor) return '';
        const { from, to } = editor.state.selection;
        return editor.state.doc.textBetween(from, to, ' ').trim();
    }, [editor]);

    const handleCommentSubmit = useCallback(() => {
        if (!pendingComment || !commentValue.trim()) return;
        onCommentRef.current?.(pendingComment.text, commentValue.trim());
        setPendingComment(null);
        setCommentValue('');
    }, [pendingComment, commentValue]);

    const handleCommentKeyDown = useCallback((e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) { e.preventDefault(); handleCommentSubmit(); }
        if (e.key === 'Escape') { setPendingComment(null); setCommentValue(''); }
    }, [handleCommentSubmit]);

    return (
        <div
            ref={shellRef}
            className={cn('ship-tiptap-shell rounded-md border bg-card relative', fillHeight && 'h-full min-h-0', className)}
            style={fillHeight ? undefined : { height: `${minHeightPx}px` }}
        >
            {/* Floating selection toolbar */}
            {toolbarPos && editor && !editor.state.selection.empty && (
                <div
                    className="ship-tiptap-bubble"
                    style={{ position: 'absolute', top: toolbarPos.top, left: toolbarPos.left, zIndex: 40 }}
                    onMouseDown={(e) => e.preventDefault()}
                >
                    <button onClick={() => editor.chain().focus().toggleBold().run()} className={editor.isActive('bold') ? 'is-active' : ''} title="Bold"><strong>B</strong></button>
                    <button onClick={() => editor.chain().focus().toggleItalic().run()} className={editor.isActive('italic') ? 'is-active' : ''} title="Italic"><em>I</em></button>
                    <button onClick={() => editor.chain().focus().toggleStrike().run()} className={editor.isActive('strike') ? 'is-active' : ''} title="Strike"><s>S</s></button>
                    <button onClick={() => editor.chain().focus().toggleCode().run()} className={editor.isActive('code') ? 'is-active' : ''} title="Code">{'</>'}</button>
                    {(onComment || onGenerate) && <span className="ship-tiptap-bubble-divider" />}
                    {onComment && (
                        <button onClick={() => { const t = getSelectedText(); if (t) { setPendingComment({ text: t }); setCommentValue(''); } }} title="Comment">
                            <MessageSquarePlus className="size-4" />
                        </button>
                    )}
                    {onGenerate && (
                        <button onClick={() => { const t = getSelectedText(); if (t) onGenerateRef.current?.(t); }} title="Generate">
                            <Sparkles className="size-4" />
                        </button>
                    )}
                </div>
            )}

            <EditorContent editor={editor} className="h-full overflow-auto" />

            {/* Comment input dialog */}
            {pendingComment && (
                <div className="absolute inset-0 z-50 flex items-start justify-center pt-16" onClick={() => setPendingComment(null)}>
                    <div className="ship-comment-tooltip-expanded" onClick={(e) => e.stopPropagation()}>
                        <div className="ship-comment-tooltip-selected">
                            &ldquo;{pendingComment.text.slice(0, 120)}{pendingComment.text.length > 120 ? '...' : ''}&rdquo;
                        </div>
                        <textarea ref={commentInputRef} value={commentValue} onChange={(e) => setCommentValue(e.target.value)} onKeyDown={handleCommentKeyDown} placeholder="Add your comment..." className="ship-comment-tooltip-input" rows={2} />
                        <div className="ship-comment-tooltip-actions">
                            <button onClick={() => setPendingComment(null)} className="ship-comment-tooltip-cancel">Cancel</button>
                            <button onClick={handleCommentSubmit} disabled={!commentValue.trim()} className="ship-comment-tooltip-submit">Add comment <kbd>&#x2318;&#x21B5;</kbd></button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
