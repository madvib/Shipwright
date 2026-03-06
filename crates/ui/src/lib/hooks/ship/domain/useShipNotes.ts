import { Dispatch, SetStateAction, useState, useMemo } from 'react';
import { NoteDocument, NoteInfo as NoteEntry } from '@/bindings';
import { useNoteActions } from '../../workspace/useNoteActions';

interface UseShipNotesParams {
    setError: Dispatch<SetStateAction<string | null>>;
    refreshActivity: () => Promise<void>;
}

export function useShipNotes({
    setError,
    refreshActivity,
}: UseShipNotesParams) {
    const [notes, setNotes] = useState<NoteEntry[]>([]);
    const [selectedNote, setSelectedNote] = useState<NoteDocument | null>(null);

    const actions = useNoteActions({
        setNotes,
        setSelectedNote,
        setError,
        refreshActivity,
    });

    return useMemo(() => ({
        notes,
        setNotes,
        selectedNote,
        setSelectedNote,
        ...actions,
    }), [notes, selectedNote, actions]);
}
