import { memo, useCallback, useEffect, useMemo, useRef, useState } from 'react';
// useNavigate removed – no longer needed after SPECS_ROUTE removal
import { Plus, StickyNote, Trash2 } from 'lucide-react';
import { Button } from '@ship/ui';
import { Input } from '@ship/ui';
import { PageFrame, PageHeader } from '@ship/ui';
import MarkdownEditor from '@/components/editor';
import { cn } from '@/lib/utils';
import { Badge } from '@ship/ui';
import { useWorkspace, useShip } from '@/lib/hooks/workspace/WorkspaceContext';
import { relativeDate } from '@/lib/date';
import { NoteDocument, NoteInfo as NoteEntry } from '@/bindings';
import { createNoteCmd, deleteNoteCmd, getNoteCmd, listNotes, updateNoteCmd } from '@/lib/platform/tauri/commands';
import { isTauriRuntime } from '@/lib/platform/tauri/runtime';
import { NoteMetadata } from './NoteMetadata';
import {
    splitFrontmatterDocument,
    composeFrontmatterDocument,
    setFrontmatterStringListField,
} from '@ship/ui';
// Specs are no longer a top-level route

type EditableNote = {
    title: string;
    content: string;
    id?: string;
};

const NoteListItem = memo(({
    note,
    isActive,
    onClick
}: {
    note: NoteEntry;
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
        loading: projectLoading,
        notesScope,
        setError,
        refreshActivity,
    } = useWorkspace();

    const {
        notes: projectNotes,
        selectedNote: projectSelectedNote,
        handleSelectNote: handleSelectProjectNote,
        handleCreateNote: handleCreateProjectNote,
        handleSaveNote: handleSaveProjectNote,
        handleDeleteNote: handleDeleteProjectNote,
        setSelectedNote: setProjectSelectedNote,
        specSuggestions,
        tagSuggestions,
    } = useShip();

    const isGlobalScope = notesScope === 'global';
    const [globalNotes, setGlobalNotes] = useState<NoteEntry[]>([]);
    const [globalSelectedNote, setGlobalSelectedNote] = useState<NoteDocument | null>(null);
    const [globalLoading, setGlobalLoading] = useState(false);

    const notes = isGlobalScope ? globalNotes : projectNotes;
    const selectedNote = isGlobalScope ? globalSelectedNote : projectSelectedNote;
    const isLoading = isGlobalScope ? globalLoading : projectLoading;

    const [saveIndicator, setSaveIndicator] = useState<'idle' | 'saving' | 'saved'>('idle');
    const autoSaveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
    const titleInputRef = useRef<HTMLInputElement>(null);
    const [localNote, setLocalNote] = useState<EditableNote | null>(null);
    const activeNoteId = useRef<string | null>(null);

    useEffect(() => {
        if (!isGlobalScope) return;

        let cancelled = false;

        const loadGlobalNotes = async () => {
            if (!isTauriRuntime()) {
                setGlobalNotes([]);
                setGlobalSelectedNote(null);
                setGlobalLoading(false);
                return;
            }

            setGlobalLoading(true);
            try {
                const entries = await listNotes('global');
                if (cancelled) return;
                setGlobalNotes(entries);
                setGlobalSelectedNote((current) =>
                    current && entries.some((entry) => entry.id === current.id) ? current : null
                );
            } catch (error) {
                if (!cancelled) {
                    setError(String(error));
                }
            } finally {
                if (!cancelled) {
                    setGlobalLoading(false);
                }
            }
        };

        void loadGlobalNotes();

        return () => {
            cancelled = true;
        };
    }, [isGlobalScope, setError]);

    useEffect(() => {
        activeNoteId.current = null;
        setLocalNote(null);
        setSaveIndicator('idle');
    }, [isGlobalScope]);

    useEffect(() => {
        if (selectedNote) {
            if (activeNoteId.current !== selectedNote.id) {
                activeNoteId.current = selectedNote.id;
                setLocalNote({
                    title: selectedNote.title,
                    content: selectedNote.content,
                    id: selectedNote.id,
                });
                setSaveIndicator('idle');
            }
        } else if (localNote?.id) {
            activeNoteId.current = null;
            setLocalNote(null);
        }
    }, [selectedNote, localNote?.id]);

    const clearSelectedNote = useCallback(() => {
        if (isGlobalScope) {
            setGlobalSelectedNote(null);
            return;
        }
        setProjectSelectedNote(null);
    }, [isGlobalScope, setProjectSelectedNote]);

    const handleSelectScopedNote = useCallback(async (entry: NoteEntry) => {
        if (!isGlobalScope) {
            await handleSelectProjectNote(entry);
            return;
        }

        if (!isTauriRuntime()) {
            setGlobalSelectedNote({
                ...entry,
                content: '',
            });
            return;
        }

        try {
            const latest = await getNoteCmd(entry.id, 'global');
            setGlobalSelectedNote(latest);
        } catch (error) {
            setError(String(error));
        }
    }, [isGlobalScope, handleSelectProjectNote, setError]);

    const handleCreateScopedNote = useCallback(async (title: string, content: string) => {
        if (!isGlobalScope) {
            return handleCreateProjectNote(title, content);
        }

        if (!isTauriRuntime()) {
            setError('Note creation is only available in Tauri runtime.');
            return;
        }

        try {
            const created = await createNoteCmd(title, content, 'global');
            setGlobalNotes((prev) => [{ id: created.id, title: created.title, updated: created.updated }, ...prev.filter((entry) => entry.id !== created.id)]);
            setGlobalSelectedNote(created);
            await refreshActivity();
            return created;
        } catch (error) {
            setError(String(error));
            throw error;
        }
    }, [isGlobalScope, handleCreateProjectNote, refreshActivity, setError]);

    const handleSaveScopedNote = useCallback(async (noteId: string, content: string) => {
        if (!isGlobalScope) {
            return handleSaveProjectNote(noteId, content);
        }

        if (!isTauriRuntime()) {
            setError('Saving notes is only available in Tauri runtime.');
            return;
        }

        try {
            const updated = await updateNoteCmd(noteId, content, 'global');
            setGlobalNotes((prev) =>
                prev.map((entry) =>
                    entry.id === updated.id
                        ? { id: updated.id, title: updated.title, updated: updated.updated }
                        : entry
                )
            );
            setGlobalSelectedNote(updated);
            await refreshActivity();
            return updated;
        } catch (error) {
            setError(String(error));
            throw error;
        }
    }, [isGlobalScope, handleSaveProjectNote, refreshActivity, setError]);

    const handleDeleteNote = useCallback(async () => {
        if (!localNote?.id) return;

        if (!confirm('Are you sure you want to delete this note?')) return;

        try {
            if (isGlobalScope) {
                await deleteNoteCmd(localNote.id, 'global');
                setGlobalNotes(prev => prev.filter(n => n.id !== localNote.id));
                setGlobalSelectedNote(null);
            } else {
                await handleDeleteProjectNote(localNote.id);
            }
            setLocalNote(null);
            await refreshActivity();
        } catch (error) {
            setError(String(error));
        }
    }, [isGlobalScope, localNote?.id, refreshActivity, setError, handleDeleteProjectNote]);

    const handleNewNote = useCallback(() => {
        const stub: EditableNote = {
            title: '',
            content: '',
        };
        activeNoteId.current = '';
        clearSelectedNote();
        setLocalNote(stub);
        setSaveIndicator('idle');
        setTimeout(() => titleInputRef.current?.focus(), 50);
    }, [clearSelectedNote]);

    const scheduleAutoSave = useCallback((title: string, content: string, noteId?: string) => {
        if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
        autoSaveTimer.current = setTimeout(async () => {
            setSaveIndicator('saving');
            try {
                if (!noteId) {
                    if (!title.trim() && !content.trim()) {
                        setSaveIndicator('idle');
                        return;
                    }
                    await handleCreateScopedNote(title || 'Untitled', content);
                } else {
                    await handleSaveScopedNote(noteId, content);
                }
                setSaveIndicator('saved');
                setTimeout(() => setSaveIndicator('idle'), 2000);
            } catch (error) {
                console.error('Failed to auto-save note', error);
                setSaveIndicator('idle');
            }
        }, 1500);
    }, [handleCreateScopedNote, handleSaveScopedNote]);

    const handleTitleChange = (title: string) => {
        if (!localNote) return;
        const next = { ...localNote, title };
        setLocalNote(next);
        scheduleAutoSave(next.title, next.content, next.id);
    };

    const handleContentChange = (content: string) => {
        if (!localNote) return;
        const next = { ...localNote, content };
        setLocalNote(next);
        scheduleAutoSave(next.title, next.content, next.id);
    };

    const handleMetadataChange = (nextSpecs: string[], nextTags: string[]) => {
        if (!localNote) return;
        const { frontmatter, body, delimiter } = splitFrontmatterDocument(localNote.content);
        let nextFrontmatter = setFrontmatterStringListField(
            frontmatter,
            'specs',
            nextSpecs,
            delimiter ?? '+++'
        );
        nextFrontmatter = setFrontmatterStringListField(
            nextFrontmatter,
            'tags',
            nextTags,
            delimiter ?? '+++'
        );
        const nextContent = composeFrontmatterDocument(nextFrontmatter, body, delimiter ?? '+++');
        handleContentChange(nextContent);
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

    const isCreating = Boolean(localNote && !localNote.id);

    return (
        <PageFrame className="h-screen overflow-hidden flex flex-col md:p-8">
            <div className="flex-none">
                <PageHeader
                    title={isGlobalScope ? 'Global Notes' : 'Notes'}
                    badge={<Badge variant="outline">{isGlobalScope ? 'Global' : 'Project'}</Badge>}
                    actions={
                        <Button onClick={handleNewNote} className="gap-2">
                            <Plus className="size-4" />
                            New Note
                        </Button>
                    }
                />
            </div>

            <div className="flex-1 min-h-0 flex flex-col lg:flex-row gap-4 pb-4">
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
                                        <p className="truncate text-sm font-medium">{localNote?.title || 'Untitled'}</p>
                                        <Badge variant="secondary" className="text-[9px] h-4 mt-1">Unsaved</Badge>
                                    </div>
                                )}
                                {sortedNotes.map((note) => (
                                    <NoteListItem
                                        key={note.id}
                                        note={note}
                                        isActive={note.id === selectedNote?.id}
                                        onClick={() => void handleSelectScopedNote(note)}
                                    />
                                ))}
                            </div>
                        )}
                    </div>
                </div>

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
                                <Button
                                    variant="ghost"
                                    size="xs"
                                    onClick={handleDeleteNote}
                                    className="text-muted-foreground hover:text-destructive transition-colors"
                                >
                                    <Trash2 className="size-4" />
                                </Button>
                            </div>

                            <NoteMetadata
                                frontmatter={splitFrontmatterDocument(localNote.content).frontmatter}
                                updated={selectedNote?.updated || new Date().toISOString()}
                                specSuggestions={specSuggestions}
                                tagSuggestions={tagSuggestions}
                                isEditing={true}
                                onChange={handleMetadataChange}
                                onNavigate={() => { }}
                            />

                            <div className="flex-1 min-h-0 rounded-lg border border-border/40 bg-background/50 overflow-hidden shadow-inner ring-1 ring-inset ring-white/5">
                                <MarkdownEditor
                                    key={localNote?.id || 'new'}
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
