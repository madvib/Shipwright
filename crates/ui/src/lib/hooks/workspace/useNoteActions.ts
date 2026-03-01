import { Dispatch, SetStateAction } from 'react';
import { NoteDocument, NoteInfo as NoteEntry } from '@/bindings';
import {
    createNoteCmd,
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
            const latest = await getNoteCmd(entry.file_name);
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
            setNotes((prev) => [
                { file_name: created.file_name, title: created.title, path: created.path, updated: created.updated },
                ...prev,
            ]);
            setSelectedNote(created);
            await refreshActivity();
            return created;
        } catch (error) {
            setError(String(error));
            throw error;
        }
    };

    const handleSaveNote = async (fileName: string, content: string) => {
        if (!isTauriRuntime()) {
            setError('Saving notes is only available in Tauri runtime.');
            return;
        }

        try {
            const updated = await updateNoteCmd(fileName, content);
            setNotes((prev) =>
                prev.map((entry) =>
                    entry.file_name === updated.file_name
                        ? { file_name: updated.file_name, title: updated.title, path: updated.path, updated: updated.updated }
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

    return {
        handleSelectNote,
        handleCreateNote,
        handleSaveNote,
    };
}
