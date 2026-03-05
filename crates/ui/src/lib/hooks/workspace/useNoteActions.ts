import { Dispatch, SetStateAction } from 'react';
import { NoteDocument, NoteInfo as NoteEntry } from '@/bindings';
import {
    createNoteCmd,
    deleteNoteCmd,
    getNoteCmd,
    updateNoteCmd,
} from '../../platform/tauri/commands';
import { isTauriRuntime } from '../../platform/tauri/runtime';

interface UseNoteActionsParams {
    setNotes: Dispatch<SetStateAction<NoteEntry[]>>;
    setSelectedNote: Dispatch<SetStateAction<NoteDocument | null>>;
    setError: Dispatch<SetStateAction<string | null>>;
    refreshActivity: () => Promise<void>;
}

export function useNoteActions({
    setNotes,
    setSelectedNote,
    setError,
    refreshActivity,
}: UseNoteActionsParams) {
    const handleSelectNote = async (entry: NoteEntry) => {
        if (!isTauriRuntime()) {
            setSelectedNote({
                ...entry,
                content: '',
            });
            return;
        }

        try {
            const latest = await getNoteCmd(entry.id);
            setSelectedNote(latest);
        } catch (error) {
            setError(String(error));
        }
    };

    const handleCreateNote = async (title: string, content: string) => {
        if (!isTauriRuntime()) {
            setError('Note creation is only available in Tauri runtime.');
            return;
        }

        try {
            const created = await createNoteCmd(title, content);
            setNotes((prev) => [{ id: created.id, title: created.title, updated: created.updated }, ...prev.filter((entry) => entry.id !== created.id)]);
            setSelectedNote(created);
            await refreshActivity();
            return created;
        } catch (error) {
            setError(String(error));
            throw error;
        }
    };

    const handleSaveNote = async (id: string, content: string) => {
        if (!isTauriRuntime()) {
            setError('Saving notes is only available in Tauri runtime.');
            return;
        }

        try {
            const updated = await updateNoteCmd(id, content);
            setNotes((prev) =>
                prev.map((entry) =>
                    entry.id === updated.id
                        ? { id: updated.id, title: updated.title, updated: updated.updated }
                        : entry
                )
            );
            setSelectedNote(updated);
            await refreshActivity();
            return updated;
        } catch (error) {
            setError(String(error));
            throw error;
        }
    };

    const handleDeleteNote = async (id: string) => {
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
    };

    return {
        handleSelectNote,
        handleCreateNote,
        handleSaveNote,
        handleDeleteNote,
    };
}
