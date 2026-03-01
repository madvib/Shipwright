import { memo, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Plus, StickyNote } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { PageFrame, PageHeader } from '@/components/app/PageFrame';
import MarkdownEditor from '@/components/editor';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';
import { useWorkspace } from '@/lib/hooks/workspace/WorkspaceContext';
import { relativeDate } from '@/lib/date';

// Memoized individual note item to prevent full list re-renders
const NoteListItem = memo(({
    note,
    isActive,
    onClick
}: {
    note: { file_name: string; title: string; updated: string };
    isActive: boolean;
    onClick: () => void
}) => (
    <button
        type="button"
        className={cn(
            'w-full rounded-md border px-2.5 py-2 text-left transition-all duration-200 outline-none',
            isActive
                ? 'border-primary/40 bg-primary/10 shadow-sm'
                : 'hover:bg-muted/50 border-transparent hover:border-border/50'
        )}
        onClick={onClick}
    >
        <p className="truncate text-sm font-semibold">{note.title || 'Untitled'}</p>
        <p className="text-muted-foreground text-[10px] mt-0.5">{relativeDate(note.updated)}</p>
    </button>
));
NoteListItem.displayName = 'NoteListItem';

export default function NotesPage() {
    const {
        notes,
        selectedNote,
        loading: isLoading,
        handleSelectNote,
        handleCreateNote,
        handleSaveNote,
        setSelectedNote,
    } = useWorkspace();

    const [saveIndicator, setSaveIndicator] = useState<'idle' | 'saving' | 'saved'>('idle');
    const autoSaveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
    const titleInputRef = useRef<HTMLInputElement>(null);

    // localNote only tracks the note being edited to decouple from workspace updates
    const [localNote, setLocalNote] = useState<{ title: string; content: string; file_name?: string } | null>(null);

    // Track the active file name to avoid redundant syncs
    const activeFileName = useRef<string | null>(null);

    // Sync local state ONLY when selection changes, not on every content update
    useEffect(() => {
        if (selectedNote) {
            // Only update if it's a DIFFERENT note
            if (activeFileName.current !== selectedNote.file_name) {
                activeFileName.current = selectedNote.file_name;
                setLocalNote({
                    title: selectedNote.title,
                    content: selectedNote.content,
                    file_name: selectedNote.file_name,
                });
                setSaveIndicator('idle');
            }
        } else if (localNote?.file_name) {
            // Note was deleted or deselected
            activeFileName.current = null;
            setLocalNote(null);
        }
    }, [selectedNote, localNote?.file_name]);

    const handleNewNote = useCallback(() => {
        const stub = {
            title: '',
            content: '',
        };
        activeFileName.current = '';
        setSelectedNote(null);
        setLocalNote(stub);
        setSaveIndicator('idle');
        setTimeout(() => titleInputRef.current?.focus(), 50);
    }, [setSelectedNote]);

    const scheduleAutoSave = useCallback((title: string, content: string, fileName?: string) => {
        if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
        autoSaveTimer.current = setTimeout(async () => {
            setSaveIndicator('saving');
            try {
                if (!fileName) {
                    if (!title.trim() && !content.trim()) {
                        setSaveIndicator('idle');
                        return;
                    }
                    await handleCreateNote(title || 'Untitled', content);
                } else {
                    await handleSaveNote(fileName, content);
                }
                setSaveIndicator('saved');
                setTimeout(() => setSaveIndicator('idle'), 2000);
            } catch (error) {
                console.error('Failed to auto-save note', error);
                setSaveIndicator('idle');
            }
        }, 1500);
    }, [handleCreateNote, handleSaveNote]);

    const handleTitleChange = (title: string) => {
        if (!localNote) return;
        const next = { ...localNote, title };
        setLocalNote(next);
        scheduleAutoSave(next.title, next.content, next.file_name);
    };

    const handleContentChange = (content: string) => {
        if (!localNote) return;
        const next = { ...localNote, content };
        setLocalNote(next);
        scheduleAutoSave(next.title, next.content, next.file_name);
    };

    const handleTitleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Tab') {
            e.preventDefault();
            const editorEl = document.querySelector<HTMLElement>('.ProseMirror');
            editorEl?.focus();
        }
    };

    useEffect(() => {
        return () => {
            if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
        };
    }, []);

    const sortedNotes = useMemo(() =>
        [...notes].sort((a, b) => new Date(b.updated).getTime() - new Date(a.updated).getTime()),
        [notes]
    );

    const isCreating = localNote && !localNote.file_name;

    return (
        <PageFrame className="h-screen overflow-hidden flex flex-col md:p-8">
            <div className="flex-none">
                <PageHeader
                    title="Notes"
                    description="Capture freeform thoughts and project context."
                    badge={<Badge variant="outline">Notes</Badge>}
                    actions={
                        <Button size="sm" onClick={handleNewNote}>
                            <Plus className="size-3.5" />
                            New Note
                        </Button>
                    }
                />
            </div>

            <div className="flex-1 min-h-0 flex flex-col lg:flex-row gap-4 pb-4">
                {/* Left: Note List */}
                <div className="lg:w-72 flex flex-col gap-2 rounded-lg border bg-card/30 p-2 overflow-hidden shadow-sm">
                    <div className="flex-1 overflow-y-auto pr-1">
                        {isLoading ? (
                            <p className="text-muted-foreground px-2 py-4 text-center text-sm">Loading…</p>
                        ) : sortedNotes.length === 0 && !isCreating ? (
                            <div className="flex flex-col items-center gap-2 px-3 py-8 text-center">
                                <StickyNote className="text-muted-foreground size-8 opacity-50" />
                                <p className="text-muted-foreground text-sm">No notes yet.</p>
                                <p className="text-muted-foreground text-xs">Capture a thought.</p>
                            </div>
                        ) : (
                            <div className="space-y-1 pr-1">
                                {isCreating && (
                                    <div className="border-primary/40 bg-primary/10 rounded-md border px-2.5 py-2 animate-in fade-in slide-in-from-top-2">
                                        <p className="truncate text-sm font-medium">{localNote.title || 'Untitled'}</p>
                                        <Badge variant="secondary" className="text-[9px] h-4 mt-1">Unsaved</Badge>
                                    </div>
                                )}
                                {sortedNotes.map((note) => (
                                    <NoteListItem
                                        key={note.file_name}
                                        note={note}
                                        isActive={note.file_name === selectedNote?.file_name}
                                        onClick={() => void handleSelectNote(note)}
                                    />
                                ))}
                            </div>
                        )}
                    </div>
                </div>

                {/* Right: Editor */}
                <div className="flex-1 min-h-0 flex flex-col rounded-lg border bg-card/30 overflow-hidden shadow-sm">
                    {!localNote ? (
                        <div className="flex flex-1 flex-col items-center justify-center gap-2 text-center p-8">
                            <StickyNote className="text-muted-foreground size-12 opacity-30" />
                            <p className="text-muted-foreground text-sm">Select a note or create a new one.</p>
                        </div>
                    ) : (
                        <div className="flex flex-1 flex-col min-h-0 p-4 gap-3">
                            <div className="flex items-center gap-3 flex-none">
                                <Input
                                    ref={titleInputRef}
                                    value={localNote.title}
                                    onChange={(e) => handleTitleChange(e.target.value)}
                                    onKeyDown={handleTitleKeyDown}
                                    placeholder="Note title…"
                                    className="text-lg font-bold border-none bg-transparent shadow-none focus-visible:ring-0 px-0 h-10 transition-all placeholder:opacity-50"
                                />
                                <div
                                    className={cn(
                                        'shrink-0 text-[10px] font-medium transition-all px-2 py-1 rounded-full border',
                                        saveIndicator === 'idle' ? 'opacity-0 scale-95' : 'opacity-100 scale-100',
                                        saveIndicator === 'saving' ? 'text-muted-foreground bg-muted/30 border-border/50' : 'text-emerald-500 bg-emerald-500/10 border-emerald-500/20'
                                    )}
                                >
                                    {saveIndicator === 'saving' ? 'Saving…' : 'Saved ✓'}
                                </div>
                            </div>
                            <div className="flex-1 min-h-0 rounded-lg border border-border/40 bg-background/50 overflow-hidden shadow-inner ring-1 ring-inset ring-white/5">
                                <MarkdownEditor
                                    value={localNote.content}
                                    onChange={handleContentChange}
                                    placeholder="Start writing your thoughts…"
                                    showFrontmatter={false}
                                    showStats={false}
                                    fillHeight
                                />
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </PageFrame>
    );
}
