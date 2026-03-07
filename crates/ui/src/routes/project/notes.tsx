import { createFileRoute } from '@tanstack/react-router';
import NotesPage from '@/features/planning/notes/NotesPage';

export const Route = createFileRoute('/project/notes')({
    component: NotesPage,
});
