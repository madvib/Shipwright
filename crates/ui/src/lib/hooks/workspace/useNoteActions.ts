import { Dispatch, SetStateAction, useCallback, useMemo } from 'react';
import { NoteInfo as NoteEntry } from '@/bindings';
import {
    createNoteCmd,
    deleteNoteCmd,
    getNoteCmd,
    updateNoteCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseNoteActionsParams {
    setNotes: Dispatch<SetStateAction<NoteEntry[]>>;
    setSelectedNote: Dispatch<SetStateAction<NoteEntry | null>>;
    setError: Dispatch<SetStateAction<string | null>>;
    refreshActivity: () => Promise<void>;
}

export function useNoteActions({
    setNotes,
    setSelectedNote,
    setError,
    refreshActivity,
}: UseNoteActionsParams) {
    const handleSelectNote = useCallback(async (entry: NoteEntry) => {
        if (!isTauriRuntime()) {
            setSelectedNote(entry);
            return;
        }

        try {
            const latest = await getNoteCmd(entry.id);
            setSelectedNote({ id: latest.id, title: latest.title, updated: latest.updated });
        } catch (error) {
            setError(String(error));
        }
    }, [setSelectedNote, setError]);

    const handleCreateNote = useCallback(async (title: string, content: string) => {
        if (!isTauriRuntime()) {
            setError('Note creation is only available in Tauri runtime.');
            return;
        }

        try {
            const created = await createNoteCmd(title, content);
            const entry = { id: created.id, title: created.title, updated: created.updated };
            setNotes((prev) => [entry, ...prev.filter((e) => e.id !== created.id)]);
            setSelectedNote(entry);
            await refreshActivity();
            return created;
        } catch (error) {
            setError(String(error));
            throw error;
        }
    }, [setNotes, setSelectedNote, setError, refreshActivity]);

    const handleSaveNote = useCallback(async (id: string, content: string) => {
        if (!isTauriRuntime()) {
            setError('Saving notes is only available in Tauri runtime.');
            return;
        }

        try {
            const updated = await updateNoteCmd(id, content);
            const entry = { id: updated.id, title: updated.title, updated: updated.updated };
            setNotes((prev) =>
                prev.map((e) =>
                    e.id === updated.id ? entry : e
                )
            );
            setSelectedNote(entry);
            await refreshActivity();
            return updated;
        } catch (error) {
            setError(String(error));
            throw error;
        }
    }, [setNotes, setSelectedNote, setError, refreshActivity]);

    const handleDeleteNote = useCallback(async (id: string) => {
        if (!isTauriRuntime()) {
            setError('Deleting notes is only available in Tauri runtime.');
            return;
        }

        try {
            await deleteNoteCmd(id);
            setNotes((prev) => prev.filter((entry) => entry.id !== id));
            setSelectedNote(null);
            await refreshActivity();
        } catch (error) {
            setError(String(error));
            throw error;
        }
    }, [setNotes, setSelectedNote, setError, refreshActivity]);

    return useMemo(() => ({
        handleSelectNote,
        handleCreateNote,
        handleSaveNote,
        handleDeleteNote,
    }), [handleSelectNote, handleCreateNote, handleSaveNote, handleDeleteNote]);
}
