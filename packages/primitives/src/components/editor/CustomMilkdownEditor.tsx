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
    const lastEmittedValueRef = useRef(value);
    const onChangeRef = useRef(onChange);

    useEffect(() => {
        onChangeRef.current = onChange;
    }, [onChange]);

    useEffect(() => {
        externalValueRef.current = value;

        // Only treat value changes as external when they differ from the last value we emitted.
        // This avoids replaceAll loops that reset selection/caret while typing.
        if (value === lastEmittedValueRef.current) return;

        const crepe = crepeRef.current;
        if (!crepe) return;

        const editorValue = crepe.getMarkdown();
        if (editorValue === value) return;
        lastEmittedValueRef.current = value;
        crepe.editor.action(replaceAll(value, true));
    }, [value]);

    useEffect(() => {
        const root = rootRef.current;
        if (!root) return;

        let disposed = false;
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
                if (markdown === lastEmittedValueRef.current) return;
                lastEmittedValueRef.current = markdown;
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

                const candidates = slashCandidates;
                for (const element of candidates) {
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
