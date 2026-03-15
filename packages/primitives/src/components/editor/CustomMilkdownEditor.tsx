import { useEffect, useRef } from 'react';
import { Crepe, CrepeFeature } from '@milkdown/crepe';
import { replaceAll } from '@milkdown/kit/utils';
import { cn } from '@/lib/utils';
import './editor.css';

export interface CustomMilkdownEditorProps {
    value: string;
    onChange: (value: string) => void;
    placeholder?: string;
    fillHeight?: boolean;
    minHeightPx?: number;
    className?: string;
}

export default function CustomMilkdownEditor({
    value,
    onChange,
    placeholder,
    fillHeight = false,
    minHeightPx = 320,
    className,
}: CustomMilkdownEditorProps) {
    const rootRef = useRef<HTMLDivElement | null>(null);
    const crepeRef = useRef<Crepe | null>(null);
    const tooltipObserverRef = useRef<MutationObserver | null>(null);
    const externalValueRef = useRef(value);
    const onChangeRef = useRef(onChange);
    // Tracks every markdown string emitted by this editor instance.
    // When React re-renders with one of these values it was echoed from the editor —
    // skip replaceAll so we never reset the cursor during normal typing.
    const editorEmissionsRef = useRef(new Set<string>());

    useEffect(() => {
        onChangeRef.current = onChange;
    }, [onChange]);

    useEffect(() => {
        externalValueRef.current = value;

        if (editorEmissionsRef.current.has(value)) {
            // Value originated from this editor and was echoed back via onChange.
            // Calling replaceAll here would reset the cursor — skip it.
            editorEmissionsRef.current.delete(value);
            return;
        }

        // Not in the emissions set → genuine external update (AI sample, undo, etc.)
        const crepe = crepeRef.current;
        if (!crepe) return;
        if (crepe.getMarkdown() === value) return;

        // Discard any stale emissions — they're no longer relevant after an external reset.
        editorEmissionsRef.current.clear();
        crepe.editor.action(replaceAll(value, true));
    }, [value]);

    useEffect(() => {
        const root = rootRef.current;
        if (!root) return;

        let disposed = false;
        editorEmissionsRef.current.clear();

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
            featureConfigs: {
                [CrepeFeature.Placeholder]: {
                    text: placeholder?.trim() || '',
                    mode: 'doc',
                },
            },
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

                const toolbarLabels = ['Bold', 'Italic', 'Strike', 'Code', 'Link', 'Math'];

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

    return (
        <div
            className={cn('rounded-md border bg-card', fillHeight && 'h-full min-h-0', className)}
            style={fillHeight ? undefined : { height: `${minHeightPx}px` }}
        >
            <div ref={rootRef} className="ship-milkdown-shell h-full w-full" />
        </div>
    );
}
